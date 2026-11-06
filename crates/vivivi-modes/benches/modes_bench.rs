//! vivivi-modes 性能測定ベンチマークスイート。
//!
//! ViVi 3 / ViVi 4 独自のキラー編集機能（インラインCSV整列、Migemoローマ字検索、
//! Unicode罫線自動結合、行末空白トリミング）の高速性を統計的に測定します。

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vivivi_modes::csv::{CsvMode, CsvOptions};
use vivivi_modes::keisen::{
    KeisenDirection, KeisenType, char_to_keisen_state, state_to_keisen_char,
};
use vivivi_modes::migemo::MigemoEngine;
use vivivi_modes::trim::{TrimKind, trim_lines};

/// 1,000 行のインライン CSV テキストに対する列幅自動計算と垂直整列のスループット。
fn bench_csv_alignment(c: &mut Criterion) {
    let raw_csv_line = "101,田中 太郎,\"東京都千代田区1-1-1\",95000,開発部\n";
    let sample_csv = raw_csv_line.repeat(1000);
    let lines: Vec<&str> = sample_csv.lines().collect();

    c.bench_function("csv_align_1000_lines", |b| {
        b.iter(|| {
            let mut csv_mode = CsvMode::new(CsvOptions::default());
            csv_mode.calculate_widths(lines.iter().copied());
            let mut formatted_chars = 0;
            for line in &lines {
                let formatted = csv_mode.format_line(line, false);
                formatted_chars += formatted.len();
            }
            black_box(formatted_chars);
        })
    });
}

/// 日本語長文テキストに対する Migemo ローマ字クエリ展開およびマッチングレイテンシ。
fn bench_migemo_search(c: &mut Criterion) {
    let long_japanese_text = "吾輩は猫である。名前はまだ無い。どこで生れたかとんと見当がつかぬ。\
何でも薄暗いじめじめした所でニャーニャー泣いていた事だけは記憶している。\
吾輩はここで始めて人間というものを見た。しかもあとで聞くとそれは書生という人間中で一番獰悪な種族であったそうだ。\
吾輩は検索機能をテストしている。\n"
        .repeat(200);

    let migemo = MigemoEngine::new();

    c.bench_function("migemo_query_and_match_wagahai", |b| {
        b.iter(|| {
            // "kensaku" (検索) に対するローマ字インクリメンタル正規表現をコンパイル
            let re = migemo.compile_query(black_box("kensaku")).unwrap();
            let matches: Vec<_> = re.find_iter(black_box(&long_japanese_text)).collect();
            black_box(matches.len());
        })
    });
}

/// Unicode 罫線の 10,000 回の十字自動結合判定スループット。
fn bench_keisen_combine(c: &mut Criterion) {
    c.bench_function("keisen_combine_10000_crosses", |b| {
        b.iter(|| {
            let mut result_char = '│';
            for _ in 0..10000 {
                let state = char_to_keisen_state(result_char).unwrap_or(0);
                let right_state = state | KeisenDirection::Right.flag(KeisenType::Thin);
                result_char = state_to_keisen_char(right_state).unwrap_or('┼');
            }
            black_box(result_char);
        })
    });
}

/// 1,000 行の混在テキストに対する末尾空白一括削除 (`:trim`) のスループット。
fn bench_trim_trailing_whitespace(c: &mut Criterion) {
    let sample_lines: Vec<String> = vec![
        "通常テキスト   \t\n".to_string(),
        "全角スペース末尾　　\n".to_string(),
        "末尾スペースなし行\n".to_string(),
        "空白のみの行    \n".to_string(),
    ]
    .into_iter()
    .cycle()
    .take(1000)
    .collect();

    c.bench_function("trim_1000_lines_all_kinds", |b| {
        b.iter(|| {
            let modified = trim_lines(black_box(&sample_lines), 0, 999, TrimKind::All);
            black_box(modified.len());
        })
    });
}

criterion_group!(
    benches,
    bench_csv_alignment,
    bench_migemo_search,
    bench_keisen_combine,
    bench_trim_trailing_whitespace
);
criterion_main!(benches);
