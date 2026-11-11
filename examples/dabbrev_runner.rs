//! 動的単語補完 (dabbrev) ランナー (examples/dabbrev_runner.rs)
//!
//! `vivivi-modes` の dabbrev (Dynamic Abbreviation) エンジンを使用し、
//! ドキュメント内の現在位置から前方・後方へ単語を走査し、
//! 入力プレフィックスに一致する補完候補を探索・順位付けするデモです。

use vivivi_core::TextBuffer;
use vivivi_modes::dabbrev::{DabbrevSession, find_candidates};

fn main() {
    println!("=== vivivi: 動的単語補完 (dabbrev) ランナー ===\n");

    let source_code = "\
use std::collections::HashMap;
use std::sync::Arc;

pub struct ApplicationContext {
    pub active_buffers: Vec<String>,
    pub configuration_map: HashMap<String, String>,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self {
            active_buffers: Vec::new(),
            configuration_map: HashMap::new(),
        }
    }

    pub fn apply_configuration(&mut self, app_key: &str) {
        println!(\"Applying configuration...\");
    }
}
";

    println!("--- サンプルテキスト ---");
    println!("{}\n", source_code);

    let buffer = TextBuffer::from_text(source_code);

    let prefix = "app";
    println!("検索プレフィックス: \"{}\"", prefix);

    // カーソル位置を行・列で指定 (17行目: "    pub fn apply_configuration...", col: 14 で "app" 直後)
    let cursor_line = 16;
    let cursor_col = 14;

    let candidates = find_candidates(&buffer, cursor_line, cursor_col, prefix);

    println!("見つかった候補数: {}", candidates.len());
    for (i, cand) in candidates.iter().enumerate() {
        println!("  [{}] {}", i + 1, cand);
    }

    println!("\n--- インタラクティブセッションのシミュレーション ---");
    let mut session = DabbrevSession::new();
    let first = session
        .start(&buffer, cursor_line, cursor_col)
        .map(|s| s.to_string());

    println!("初期状態: active = {}", session.is_active());
    if let Some(cand) = first {
        println!("第1候補 (Alt+/ 1回目): \"{}\"", cand);
    }
    if let Some(cand) = session.next_candidate() {
        println!("第2候補 (Alt+/ 2回目): \"{}\"", cand);
    }
    if let Some(cand) = session.prev_candidate() {
        println!("直前候補 (Alt+Shift+/): \"{}\"", cand);
    }

    session.reset();
    println!("リセット後: active = {}", session.is_active());

    println!("\ndabbrev 単体動作確認が完了しました。");
}
