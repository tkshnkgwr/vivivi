//! 画面表示・構文解析・移動機能の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、`diff`（差分計算）、`difflog`（差分ログ）、
//! `folding`（コード折り畳み）、`gutter`（行変更追跡）、`invisible`（不可視文字可視化）、
//! `outline`（アウトライン抽出）、`print`（印刷レイアウト）、`syntax`（ハイライト）、`fftt`（文字検索）を検証します。

use vivivi_modes::diff::{
    DiffType, apply_unified_patch, create_side_by_side, create_unified_diff, diff_lines,
};
use vivivi_modes::difflog::{DiffLog, DiffLogEntry};
use vivivi_modes::fftt::{char_matches_fftt, find_char_backward, find_char_forward};
use vivivi_modes::folding::FoldManager;
use vivivi_modes::gutter::{ChangeTracker, LineChangeState};
use vivivi_modes::invisible::{ListCharsConfig, format_line_invisibles};
use vivivi_modes::outline::parse_outline;
use vivivi_modes::print::{PrintSettings, format_preview_text, paginate_text};
use vivivi_modes::syntax::{SyntaxLang, SyntaxToken, TokenType, tokenize_line};

// ==========================================
// 1. Diff (差分計算)
// ==========================================

#[test]
fn test_diff_lines_and_unified() {
    let old = "apple\nbanana\ncherry\n";
    let new = "apple\nblueberry\ncherry\ndate\n";

    let lines = diff_lines(old, new);
    assert_eq!(lines.len(), 5);

    let patch = create_unified_diff(old, new, "old.txt", "new.txt");
    assert!(patch.contains("--- old.txt"));
    assert!(patch.contains("+++ new.txt"));
    assert!(patch.contains("-banana"));
    assert!(patch.contains("+blueberry"));
    assert!(patch.contains("+date"));

    let restored = apply_unified_patch(old, &patch).expect("apply patch");
    assert_eq!(restored.trim(), new.trim());
}

#[test]
fn test_side_by_side() {
    let old = "foo\nbar\nbaz\n";
    let new = "foo\nBAR\nbaz\nqux\n";

    let sbs = create_side_by_side(old, new);
    assert_eq!(sbs[0].left_text.as_deref(), Some("foo"));
    assert_eq!(sbs[0].right_text.as_deref(), Some("foo"));
    assert_eq!(sbs[1].left_text.as_deref(), Some("bar"));
    assert_eq!(sbs[1].right_text.as_deref(), Some("BAR"));
    assert_eq!(sbs[1].left_type, DiffType::Delete);
    assert_eq!(sbs[1].right_type, DiffType::Insert);
}

// ==========================================
// 2. DiffLog (差分ログ・復元)
// ==========================================

#[test]
fn test_difflog_record_and_restore() {
    let v1 = "Line 1\nLine 2\nLine 3\n";
    let v2 = "Line 1\nLine 2 (edited)\nLine 3\n";
    let v3 = "Line 1\nLine 2 (edited)\nLine 3\nLine 4 (new)\n";

    let mut log = DiffLog::new();

    let patch1 = create_unified_diff(v1, v2, "v1", "v2");
    log.entries.push(DiffLogEntry {
        generation: 1,
        timestamp: "2026-10-14 10:00:00".into(),
        comment: "edit line 2".into(),
        patch: patch1,
    });

    let patch2 = create_unified_diff(v2, v3, "v2", "v3");
    log.entries.push(DiffLogEntry {
        generation: 2,
        timestamp: "2026-10-14 11:00:00".into(),
        comment: "add line 4".into(),
        patch: patch2,
    });

    let serialized = log.serialize();
    let parsed = DiffLog::parse(&serialized).expect("parse difflog");
    assert_eq!(parsed.entries.len(), 2);
}

// ==========================================
// 3. Folding (コード折り畳み)
// ==========================================

#[test]
fn test_compute_indent_folds() {
    let code = "fn main() {\n    let a = 1;\n    let b = 2;\n    println!(\"{}\", a + b);\n}\n";
    let mut fm = FoldManager::new();
    fm.compute_indent_folds(code, 4);

    assert!(!fm.folds.is_empty());
    let top_fold = &fm.folds[0];
    assert_eq!(top_fold.start_line, 1);
    assert_eq!(top_fold.end_line, 4);

    assert!(fm.toggle_fold(2));
    assert!(!fm.folds[0].is_open);
    assert!(fm.is_line_hidden(2));
    assert!(fm.is_line_hidden(3));
    assert!(!fm.is_line_hidden(1));
}

// ==========================================
// 4. Gutter (行変更追跡)
// ==========================================

#[test]
fn test_change_tracker() {
    let lines = vec!["line 1".to_string(), "line 2".to_string()];
    let tracker = ChangeTracker::new(&lines);

    let modified_lines = vec![
        "line 1".to_string(),
        "line 2 modified".to_string(),
        "line 3 added".to_string(),
    ];
    let markers = tracker.compute_markers(&modified_lines);
    assert_eq!(
        markers,
        vec![
            LineChangeState::Unchanged,
            LineChangeState::Modified,
            LineChangeState::Added
        ]
    );
}

// ==========================================
// 5. Invisible (不可視文字可視化)
// ==========================================

#[test]
fn test_format_zenkaku_space_and_eol() {
    let config = ListCharsConfig {
        enabled: true,
        ..Default::default()
    };

    let chars = format_line_invisibles("テスト　コード", 4, &config);
    let s: String = chars.iter().map(|c| c.ch).collect();
    assert_eq!(s, "テスト　コード$");

    // カスタム指定時は指定記号
    let custom_config = ListCharsConfig {
        enabled: true,
        zenkaku_space: "□",
        ..Default::default()
    };
    let custom_chars = format_line_invisibles("テスト　コード", 4, &custom_config);
    let custom_s: String = custom_chars.iter().map(|c| c.ch).collect();
    assert_eq!(custom_s, "テスト□コード$");
}

// ==========================================
// 6. Outline (アウトライン抽出)
// ==========================================

#[test]
fn test_markdown_outline() {
    let lines = vec![
        "# Heading 1".to_string(),
        "Some text".to_string(),
        "## Subheading 1.1".to_string(),
    ];
    let items = parse_outline(&lines, SyntaxLang::Markdown);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "Heading 1");
    assert_eq!(items[0].level, 0);
    assert_eq!(items[1].title, "Subheading 1.1");
    assert_eq!(items[1].level, 1);
}

#[test]
fn test_rust_outline() {
    let lines = vec![
        "pub struct MyStruct {".to_string(),
        "}".to_string(),
        "impl MyStruct {".to_string(),
        "    pub fn new() -> Self {".to_string(),
        "    }".to_string(),
        "}".to_string(),
    ];
    let items = parse_outline(&lines, SyntaxLang::Rust);
    assert!(items.iter().any(|it| it.title.contains("struct MyStruct")));
    assert!(items.iter().any(|it| it.title.contains("impl MyStruct")));
    assert!(items.iter().any(|it| it.title.contains("fn new")));
}

// ==========================================
// 7. Print (印刷・ページ分割レイアウト)
// ==========================================

#[test]
fn test_paginate_and_preview() {
    let text = "Line 1\nLine 2\nLine 3\n";
    let settings = PrintSettings {
        lines_per_page: 10,
        columns_per_line: 40,
        show_line_numbers: true,
        header_format: "TestDoc".into(),
        footer_format: "%Page%".into(),
    };

    let pages = paginate_text(text, "test.txt", "2026-10-14", &settings);
    assert_eq!(pages.len(), 1);
    let preview = format_preview_text(&pages);
    assert!(preview.contains("TestDoc"));
    assert!(preview.contains("Line 1"));
}

// ==========================================
// 8. Syntax (シンタックスハイライト)
// ==========================================

#[test]
fn test_rust_tokenization() {
    let line = "pub fn add(a: i32) -> i32 { // comment\n";
    let tokens = tokenize_line(line, SyntaxLang::Rust);

    assert_eq!(
        tokens[0],
        SyntaxToken {
            text: "pub".to_string(),
            token_type: TokenType::Keyword
        }
    );
    assert_eq!(
        tokens[2],
        SyntaxToken {
            text: "fn".to_string(),
            token_type: TokenType::Keyword
        }
    );
}

// ==========================================
// 9. FFTT (文字検索)
// ==========================================

#[test]
fn test_char_matches_fftt() {
    assert!(char_matches_fftt('a', 'ａ'));
    assert!(char_matches_fftt('1', '１'));
    assert!(char_matches_fftt('Z', 'Ｚ'));
    assert!(char_matches_fftt(' ', '　'));
    assert!(char_matches_fftt('(', '（'));
    assert!(char_matches_fftt('(', '「'));
    assert!(char_matches_fftt(')', '）'));
}

#[test]
fn test_find_forward_and_backward() {
    let line = "Hello (世界) 123 １２３ 「テスト」";
    assert_eq!(find_char_forward(line, 0, '1', false, 1), Some(11));
    assert_eq!(find_char_forward(line, 0, '1', false, 2), Some(15));
    assert_eq!(find_char_forward(line, 0, '(', false, 1), Some(6));
    assert_eq!(find_char_forward(line, 0, '(', false, 2), Some(19));
    assert_eq!(find_char_backward(line, 22, '(', false, 1), Some(19));
    assert_eq!(find_char_backward(line, 19, '(', false, 1), Some(6));
}
