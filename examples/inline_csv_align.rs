//! インライン CSV/TSV 整列ツール (examples/inline_csv_align.rs)
//!
//! `vivivi-modes` の CSV モードロジックを使用し、不揃いなカンマ・タブ区切りテキストを
//! 各列の最大幅を自動計算して垂直整列した表組みテキストとして標準出力します。

use vivivi_modes::csv::{CsvMode, CsvOptions, Delimiter};

fn main() {
    println!("=== vivivi: インライン CSV 即時整列ツール ===\n");

    let sample_csv = "\
id,name,role,department,salary
1,Alice,Software Engineer,Core Platform,95000
2,Bob,Senior Architect,Infrastructure,120000
103,Charlie Brown,Engineering Manager,Product Development,135000
4,Dave,Designer,UX Team,82000
5005,Eva Rostova,Director of Technology,Executive Board,180000";

    println!("--- 入力生データ (Raw CSV) ---");
    println!("{}\n", sample_csv);

    let options = CsvOptions {
        delimiter: Delimiter::Comma,
        min_col_width: 6,
        ..Default::default()
    };

    let mut mode = CsvMode::new(options);

    let lines: Vec<&str> = sample_csv.lines().collect();

    // 全行をスキャンして各列の最大文字表示幅を計算
    mode.calculate_widths(lines.iter().copied());

    println!("--- 整列結果 (Aligned Grid View) ---");
    for (idx, line) in lines.iter().enumerate() {
        let aligned = mode.format_line(line, false);
        println!("{:>3} | {}", idx + 1, aligned);
    }

    println!("\n--- TSV (タブ区切り) の整列デモ ---");
    let sample_tsv = "\
コマンド\tキー\t説明
:w\t<Enter>\tファイルを保存する
:q!\t<Enter>\t保存せずに強制終了
:hex\t<Enter>\tバイナリ(Hex)エディタモードへ切り替え
:lsp start\t<Enter>\tLanguage Server を起動";

    let tsv_options = CsvOptions {
        delimiter: Delimiter::Tab,
        min_col_width: 8,
        ..Default::default()
    };
    let mut tsv_mode = CsvMode::new(tsv_options);
    let tsv_lines: Vec<&str> = sample_tsv.lines().collect();
    tsv_mode.calculate_widths(tsv_lines.iter().copied());

    for (idx, line) in tsv_lines.iter().enumerate() {
        let aligned = tsv_mode.format_line(line, false);
        println!("{:>2} | {}", idx + 1, aligned);
    }

    println!("\nCSV/TSV インライン整列が正常に完了しました。");
}
