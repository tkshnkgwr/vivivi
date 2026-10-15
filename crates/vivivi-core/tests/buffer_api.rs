//! `TextBuffer` の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、`TextBuffer` のライフサイクル、
//! テキスト挿入・削除、アンドゥ/リドゥ、カーソル・位置変換等の振る舞いを検証します。

use vivivi_core::{CursorPosition, TextBuffer};

#[test]
fn test_text_buffer_insert_delete_undo() {
    let mut buf = TextBuffer::from_text("Hello");
    assert_eq!(buf.to_string(), "Hello");
    assert!(!buf.is_modified());

    buf.insert_text(5, " World");
    assert_eq!(buf.to_string(), "Hello World");
    assert!(buf.is_modified());

    assert!(buf.undo());
    assert_eq!(buf.to_string(), "Hello");
    assert!(!buf.is_modified());

    assert!(buf.redo());
    assert_eq!(buf.to_string(), "Hello World");
    assert!(buf.is_modified());
}

#[test]
fn test_undo_block() {
    let mut buf = TextBuffer::from_text("12345");
    buf.open_undo_block();
    buf.delete_text(0, 2, false); // "12" を削除 -> "345"
    buf.insert_text(0, "ABC"); // "ABC" を挿入 -> "ABC345"
    buf.close_undo_block();

    assert_eq!(buf.to_string(), "ABC345");

    // 1回のUndoでブロック内の両操作が一括アンドゥされる
    assert!(buf.undo());
    assert_eq!(buf.to_string(), "12345");

    assert!(buf.redo());
    assert_eq!(buf.to_string(), "ABC345");
}

#[test]
fn test_lines_and_positions() {
    let buf = TextBuffer::from_text("alpha\nbeta\ngamma");
    assert_eq!(buf.line_count(), 3);
    assert_eq!(buf.line_text(0), Some("alpha".to_string()));
    assert_eq!(buf.line_text(1), Some("beta".to_string()));
    assert_eq!(buf.line_text(2), Some("gamma".to_string()));

    assert_eq!(buf.line_start_position(0), 0);
    assert_eq!(buf.line_start_position(1), 6); // "alpha\n" は 6 文字
    assert_eq!(buf.line_start_position(2), 11); // "alpha\nbeta\n" は 11 文字

    assert_eq!(buf.position_to_line(0), 0);
    assert_eq!(buf.position_to_line(5), 0);
    assert_eq!(buf.position_to_line(6), 1);
    assert_eq!(buf.position_to_line(12), 2);
}

#[test]
fn test_prohibit_merge_and_newlines() {
    let mut buf = TextBuffer::new();
    // 'a', 'b', 'c' を入力
    buf.insert_text(0, "a");
    buf.insert_text(1, "b");
    buf.insert_text(2, "c");

    // カーソル移動やモード変更によりマージ禁止を設定
    buf.prohibit_merge_undo();

    // 'd', 'e' を入力
    buf.insert_text(3, "d");
    buf.insert_text(4, "e");

    assert_eq!(buf.to_string(), "abcde");

    // 最初のUndoは "de" のみをアンドゥ
    assert!(buf.undo());
    assert_eq!(buf.to_string(), "abc");

    // 2回目のUndoで "abc" をアンドゥ
    assert!(buf.undo());
    assert_eq!(buf.to_string(), "");

    // Redoで "abc"、続いて "abcde" が復元される
    assert!(buf.redo());
    assert_eq!(buf.to_string(), "abc");
    assert!(buf.redo());
    assert_eq!(buf.to_string(), "abcde");
}

#[test]
fn test_cursor_and_position_conversion() {
    let buf = TextBuffer::from_text("hello\nworld");
    let cur = buf.position_to_cursor(7); // "world" の 'o' (行 1, 列 1)
    assert_eq!(cur.line, 1);
    assert_eq!(cur.col, 1);

    let pos = buf.cursor_to_position(CursorPosition::new(1, 3));
    assert_eq!(pos, 9); // "world" の 'l'
    assert_eq!(buf.char_at(pos), Some('l'));
}
