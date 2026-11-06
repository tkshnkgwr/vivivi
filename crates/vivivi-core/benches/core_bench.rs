//! vivivi-core 性能測定ベンチマークスイート。
//!
//! OpenViVi 由来のデータ構造（`GapBuffer`, `UndoMgr`）の極限性能と、
//! 標準データ構造（`String` / `Vec`）に対する局所編集 $O(1)$ の優位性を統計的に測定・実証します。

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use vivivi_core::gap_buffer::GapBuffer;
use vivivi_core::undo::UndoMgr;
use vivivi_core::{CharEncoding, detect_encoding};

/// 10万文字の文章中央における局所連続タイピング（1,000文字連続挿入）の性能測定。
///
/// GapBuffer は Gap 移動後の編集が $O(1)$ であるのに対し、
/// 標準 String は1文字挿入ごとに後続の約5万バイトを memmove ($O(N)$) するため、
/// 圧倒的なスループット差が生まれることを統計的に計測します。
fn bench_gap_buffer_vs_string_typing(c: &mut Criterion) {
    let mut group = c.benchmark_group("typing_at_center_100k");

    // 10万文字のダミーテキストを用意
    let base_text: String = "あいうえおABCDE12345\n".repeat(4762); // 約10万文字
    let insert_chars: Vec<char> = "テストタイピング文字列"
        .chars()
        .cycle()
        .take(1000)
        .collect();

    // 1. GapBuffer による局所連続挿入
    group.bench_function(BenchmarkId::new("data_structure", "GapBuffer"), |b| {
        b.iter(|| {
            let mut gb = GapBuffer::from_text(black_box(&base_text));
            let center = gb.len() / 2;
            for (offset, &ch) in insert_chars.iter().enumerate() {
                gb.insert(center + offset, ch);
            }
            black_box(gb.len());
        })
    });

    // 2. 標準 String (Vec memmove) による局所連続挿入
    group.bench_function(BenchmarkId::new("data_structure", "StandardString"), |b| {
        b.iter(|| {
            let mut s = base_text.clone();
            let mut byte_idx = s
                .char_indices()
                .nth(black_box(s.chars().count() / 2))
                .map(|(i, _)| i)
                .unwrap_or(0);
            for &ch in &insert_chars {
                s.insert(byte_idx, ch);
                byte_idx += ch.len_utf8();
            }
            black_box(s.len());
        })
    });

    group.finish();
}

/// UndoMgr による 1,000 回の編集記録、自動タイピングマージ、および一括 Undo/Redo のスループット。
fn bench_undo_manager_lifecycle(c: &mut Criterion) {
    c.bench_function("undo_mgr_1000_edits_and_undo_redo", |b| {
        b.iter(|| {
            let mut mgr = UndoMgr::new();
            mgr.open_block();

            // 1,000回の文字挿入トランザクション
            for i in 0..1000 {
                mgr.push_insert(i, "a");
            }
            mgr.close_block();

            // 一括 Undo
            let mut undo_actions = Vec::new();
            while let Some(action) = mgr.pop_undo() {
                undo_actions.push(action);
            }

            // 一括 Redo
            let mut redo_actions = Vec::new();
            while let Some(action) = mgr.pop_redo() {
                redo_actions.push(action);
            }

            black_box((undo_actions.len(), redo_actions.len()));
        })
    });
}

/// 大容量テキストに対する文字コード・BOM早期自動判定のスループット。
fn bench_encoding_detection(c: &mut Criterion) {
    let sample_utf8 =
        "吾輩は猫である。名前はまだ無い。\nどこで生れたかとんと見当がつかぬ。\n".repeat(500);
    let sample_bytes = sample_utf8.as_bytes();

    c.bench_function("detect_encoding_utf8_10k_chars", |b| {
        b.iter(|| {
            let enc = detect_encoding(black_box(sample_bytes));
            black_box(enc.0 == CharEncoding::Utf8);
        })
    });
}

criterion_group!(
    benches,
    bench_gap_buffer_vs_string_typing,
    bench_undo_manager_lifecycle,
    bench_encoding_detection
);
criterion_main!(benches);
