//! `GapBuffer` の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、`GapBuffer` の文字挿入、
//! スライス挿入、削除、範囲削除、文字列変換などの公開振る舞いを検証します。

use vivivi_core::GapBuffer;

#[test]
fn test_gap_buffer_basic() {
    let mut gb: GapBuffer<char> = GapBuffer::new();
    assert!(gb.is_empty());
    assert_eq!(gb.len(), 0);

    gb.insert(0, 'a');
    gb.insert(1, 'b');
    gb.insert(2, 'd');
    assert_eq!(gb.to_string(), "abd");

    gb.insert(2, 'c');
    assert_eq!(gb.to_string(), "abcd");

    let removed = gb.remove(1);
    assert_eq!(removed, Some('b'));
    assert_eq!(gb.to_string(), "acd");

    gb.remove_range(1, 3);
    assert_eq!(gb.to_string(), "a");
}

#[test]
fn test_gap_buffer_slice_insertion_and_growth() {
    let mut gb = GapBuffer::from_text("Hello World!");
    assert_eq!(gb.to_string(), "Hello World!");

    let insert_text: Vec<char> = " beautiful".chars().collect();
    gb.insert_slice(5, &insert_text);
    assert_eq!(gb.to_string(), "Hello beautiful World!");

    // ギャップを先頭へ移動して挿入
    gb.insert(0, '>');
    assert_eq!(gb.to_string(), ">Hello beautiful World!");

    // ギャップを末尾へ移動して挿入
    let end = gb.len();
    gb.insert(end, '<');
    assert_eq!(gb.to_string(), ">Hello beautiful World!<");
}
