//! vivivi: OpenViVi を現代的な Rust で再構築した高機能・日本語対応テキストエディタ。
//!
//! Vi/Vim の操作体系を基盤とし、全角文字を崩さない罫線描画、インラインCSV/TSV整列、
//! Migemo ローマ字日本語検索、アウトライン表示、画面分割、永続Undoなどをターミナル上で提供します。

use vivivi_app::tui;

use std::env;
use std::path::PathBuf;

use tui::App;
use vivivi_core::{CharEncoding, CursorPosition, NewlineType, TextBuffer};
use vivivi_modes::{BoxAlignment, CsvMode, KeisenDirection, KeisenMode};
use vivivi_vi::ViEngine;

/// アプリケーションのエントリポイント。
///
/// コマンドライン引数をパースし、指定されたファイル（または新規バッファ）を開いて
/// TUI セッションを開始します。`--demo` フラグが指定された場合は簡易コンソールデモを実行します。
///
/// # Errors
/// ターミナルの初期化失敗やファイル読み込みエラーが発生した場合に [`anyhow::Error`] を返します。
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    // --demo フラグが渡された場合はコンソールデモを実行
    if args.iter().any(|a| a == "--demo" || a == "-d") {
        run_console_demo();
        return Ok(());
    }

    // 引数から開くファイルパスを取得
    let file_arg = args.get(1).filter(|a| !a.starts_with('-'));
    let (initial_text, file_path, encoding, newline) = if let Some(path_str) = file_arg {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let bytes = std::fs::read(&path)?;
            let (content, enc, nl) = vivivi_core::decode_bytes(&bytes);
            (content, Some(path), enc, nl)
        } else {
            (
                String::new(),
                Some(path),
                CharEncoding::Utf8,
                NewlineType::default(),
            )
        }
    } else {
        let welcome = "=== vivivi: OpenViVi reconstructed in Rust ===\n\n\
            * Vi基本操作: h,j,k,l (移動), i (挿入), Esc (ノーマル), :w (保存), :q (終了)\n\
            * 対応括弧ジャンプ: % (カーソル位置の括弧へ即座にジャンプ)\n\
            * 罫線モード: Shift+F5 で切替 (Ctrl+矢印で描画、Shift+矢印で消去)\n\
            * CSVモード: csv/tsvファイルを開くと自動整列ビューが有効化されます\n\
            * 文字コード/改行: UTF-8, Shift_JIS, EUC-JP / CRLF, LF 自動判別\n\n\
            編集を開始するには 'i' を押してください。";
        (
            welcome.to_string(),
            None,
            CharEncoding::Utf8,
            NewlineType::Lf,
        )
    };

    let app = App::new(&initial_text, file_path, encoding, newline);
    tui::run(app)
}

/// 非対話型の動作確認コンソールデモ
fn run_console_demo() {
    println!("=== vivivi: OpenViVi reconstructed in Rust (Console Demo) ===");
    let mut buf = TextBuffer::from_text("Hello, vivivi!\nWelcome to OpenViVi reimplementation.");
    let mut vi = ViEngine::new();
    let csv = CsvMode::default();
    let keisen = KeisenMode::default();

    println!("Lines: {}", buf.line_count());
    println!("Initial text:\n{}", buf);

    // Undo/Redo と GapBuffer の統合動作テスト
    buf.open_undo_block();
    let pos = buf.cursor_to_position(CursorPosition::new(0, 14));
    buf.insert_text(pos, " [Rust Edition]");
    buf.close_undo_block();

    println!("\nAfter insertion:\n{}", buf);
    println!("Is modified: {}", buf.is_modified());

    buf.undo();
    println!("\nAfter undo:\n{}", buf);
    println!("Is modified: {}", buf.is_modified());

    buf.redo();
    println!("\nAfter redo:\n{}", buf);

    println!("\nVi Mode: {}", vi.mode().name());
    println!("CSV Mode options: {:?}", csv.options);
    println!("Keisen Mode active: {}", keisen.is_active);

    vi.feed_char('i');
    println!("After 'i' key, Mode: {}", vi.mode().name());
    vi.feed_char('\x1b');
    println!("After Escape key, Mode: {}", vi.mode().name());

    // --- フェーズ 2: 罫線モードのデモ ---
    println!("\n=== Phase 2 Demo: Keisen Mode ===");
    let mut keisen_buf = TextBuffer::new();
    let mut keisen_cur = CursorPosition::new(0, 0);
    keisen.step(
        &mut keisen_buf,
        &mut keisen_cur,
        KeisenDirection::Right,
        false,
    );
    keisen.step(
        &mut keisen_buf,
        &mut keisen_cur,
        KeisenDirection::Right,
        false,
    );
    keisen.step(
        &mut keisen_buf,
        &mut keisen_cur,
        KeisenDirection::Down,
        false,
    );
    println!("Drawn Keisen buffer:\n{}", keisen_buf);

    let mut box_buf = TextBuffer::from_text("│ hello     │");
    KeisenMode::align_box_text(&mut box_buf, 0, BoxAlignment::Center);
    println!("Centered box text: {}", box_buf);

    // --- フェーズ 2: インラインCSVモードのデモ ---
    println!("\n=== Phase 2 Demo: Inline CSV Mode ===");
    let mut csv_mgr = CsvMode::default();
    let sample_csv = ["id,name,score", "1,\"Alice Smith\",95", "2,\"Bob\",80"];
    csv_mgr.calculate_widths(sample_csv.iter().copied());
    println!("Column widths: {:?}", csv_mgr.column_widths);
    for (i, line) in sample_csv.iter().enumerate() {
        println!("Row {}: {}", i, csv_mgr.format_line(line, false));
    }
}
