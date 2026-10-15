//! 括弧マッチング機能の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、半角および全角括弧の対応探索、
//! 文字列・コメント内部の括弧スキップ、カーソル行内の括弧探索の振る舞いを検証します。

use vivivi_core::{CursorPosition, TextBuffer, find_matching_paren, find_matching_paren_in_line};

#[test]
fn test_matching_paren_simple() {
    let buffer = TextBuffer::from_text("fn test() { let x = (1 + 2); }");

    // '(' at 7 -> ')' at 8
    assert_eq!(find_matching_paren(&buffer, 7), Some(8));
    assert_eq!(find_matching_paren(&buffer, 8), Some(7));

    // '{' at 10 -> '}' at 29
    assert_eq!(find_matching_paren(&buffer, 10), Some(29));
    assert_eq!(find_matching_paren(&buffer, 29), Some(10));

    // '(' at 20 -> ')' at 26
    assert_eq!(find_matching_paren(&buffer, 20), Some(26));
}

#[test]
fn test_matching_paren_with_string_and_comments() {
    let buffer = TextBuffer::from_text("let a = ( \"quoted ( paren )\" + /* comment ( */ 10 );");
    // '(' at 8 -> ')' at 50
    assert_eq!(find_matching_paren(&buffer, 8), Some(50));
    assert_eq!(find_matching_paren(&buffer, 50), Some(8));
}

#[test]
fn test_matching_paren_zenkaku() {
    let buffer = TextBuffer::from_text("こんにちは「世界」と【Rust】");
    // 「 at 5 -> 」 at 8
    assert_eq!(find_matching_paren(&buffer, 5), Some(8));
    assert_eq!(find_matching_paren(&buffer, 8), Some(5));

    // 【 at 10 -> 】 at 15
    assert_eq!(find_matching_paren(&buffer, 10), Some(15));
    assert_eq!(find_matching_paren(&buffer, 15), Some(10));
}

#[test]
fn test_find_matching_paren_in_line() {
    let buffer = TextBuffer::from_text("fn main() {\n    let msg = 「こんにちは」;\n}");

    // 1行目: カーソルが 'f'(col 0) にあるとき、行内の最初の括弧 '(' (col 7) の対応 ')' (col 8) へ
    let match1 = find_matching_paren_in_line(&buffer, CursorPosition::new(0, 0));
    assert_eq!(match1, Some(8));

    // 2行目: カーソルがインデント空白(col 0) にあるとき、全角括弧 '「' (col 14) の対応 '」' (col 20) へ
    let match2 = find_matching_paren_in_line(&buffer, CursorPosition::new(1, 0));
    // 「 is at 12 + 14 = 26. こんにちは is 5 chars. 」 is at 26 + 1 + 5 = 32.
    assert_eq!(match2, Some(32));

    // カーソルが '」' の上にあるとき、対応する '「' (pos 26) へ
    let match3 = find_matching_paren_in_line(&buffer, CursorPosition::new(1, 20));
    assert_eq!(match3, Some(26));
}
