//! `ViEngine` の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、`ViEngine` によるモーション移動、
//! 行削除・ヤンク・貼り付け、単語間移動、アンドゥ/リドゥ、Exコマンド実行等の振る舞いを検証します。

use vivivi_core::{TextBuffer, TextCursor};
use vivivi_vi::ViEngine;
use vivivi_vi::command::{Motion, Operator, ViCommand};

#[test]
fn test_motions_and_clamp() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("first line\nsecond line\nthird line");
    let mut cur = TextCursor::new(0, 0);

    // j (下へ)
    vi.execute(&ViCommand::Move(Motion::Down(1)), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 1);
    assert_eq!(cur.pos.col, 0);

    // $ (行末へ)
    vi.execute(&ViCommand::Move(Motion::EndOfLine), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 10); // "second line" length is 11, col is 10

    // 0 (行頭へ)
    vi.execute(&ViCommand::Move(Motion::StartOfLine), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 0);

    // G (文書末尾へ)
    vi.execute(
        &ViCommand::Move(Motion::DocumentEnd(None)),
        &mut buf,
        &mut cur,
    );
    assert_eq!(cur.pos.line, 2);

    // gg (文書先頭へ)
    vi.execute(&ViCommand::Move(Motion::DocumentStart), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 0);
}

#[test]
fn test_delete_and_yank_and_paste() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("apple\nbanana\ncherry");
    let mut cur = TextCursor::new(0, 0);

    // dd (1行削除) -> "banana\ncherry"
    vi.execute(
        &ViCommand::OperateLine {
            op: Operator::Delete,
            count: 1,
            register: None,
        },
        &mut buf,
        &mut cur,
    );
    assert_eq!(buf.to_string(), "banana\ncherry");
    assert_eq!(vi.registers().get(None).unwrap().as_str(), "apple\n");

    // p (下行に貼り付け) -> "banana\napple\ncherry"
    vi.execute(
        &ViCommand::PasteAfter {
            count: 1,
            register: None,
        },
        &mut buf,
        &mut cur,
    );
    assert_eq!(buf.to_string(), "banana\napple\ncherry");

    // u (Undo) -> "banana\ncherry"
    vi.execute(&ViCommand::Undo, &mut buf, &mut cur);
    assert_eq!(buf.to_string(), "banana\ncherry");
}

#[test]
fn test_word_motions() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("hello world again");
    let mut cur = TextCursor::new(0, 0);

    // w -> "world" の 'w' (pos 6)
    vi.execute(&ViCommand::Move(Motion::NextWord(1)), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 6);

    // w -> "again" の 'a' (pos 12)
    vi.execute(&ViCommand::Move(Motion::NextWord(1)), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 12);

    // b -> "world" の 'w' (pos 6)
    vi.execute(&ViCommand::Move(Motion::PrevWord(1)), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 6);
}

#[test]
fn test_ex_command_write() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("sample content");
    let mut cur = TextCursor::new(0, 0);

    let res = vi.execute(&ViCommand::ExCommand("w".to_string()), &mut buf, &mut cur);
    assert_eq!(res.message, Some("ファイルを保存しました".to_string()));
}
