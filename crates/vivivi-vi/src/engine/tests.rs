use super::*;
use crate::command::{Motion, Operator, ViCommand};
use crate::mode::ViMode;
use vivivi_core::{CursorPosition, TextBuffer, TextCursor};

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
fn test_word_motion() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("hello world test");
    let mut cur = TextCursor::new(0, 0);

    // w (次の単語へ: world)
    vi.execute(&ViCommand::Move(Motion::NextWord(1)), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 6);

    // w (次の単語へ: test)
    vi.execute(&ViCommand::Move(Motion::NextWord(1)), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 12);

    // b (前の単語へ: world)
    vi.execute(&ViCommand::Move(Motion::PrevWord(1)), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 6);
}

#[test]
fn test_search_and_substitute() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("alpha beta gamma\nalpha delta");
    let mut cur = TextCursor::new(0, 0);

    // /delta で検索
    let res = vi.execute(
        &ViCommand::SearchForward("delta".to_string()),
        &mut buf,
        &mut cur,
    );
    assert!(res.need_redraw);
    assert_eq!(cur.pos.line, 1);
    assert_eq!(cur.pos.col, 6); // "alpha " is 6

    // :%s/alpha/ALPHA/g で一括置換
    vi.execute(
        &ViCommand::ExCommand("%s/alpha/ALPHA/g".to_string()),
        &mut buf,
        &mut cur,
    );
    assert_eq!(buf.to_string(), "ALPHA beta gamma\nALPHA delta");

    // :w で保存要求
    let res = vi.execute(&ViCommand::ExCommand("w".to_string()), &mut buf, &mut cur);
    assert!(res.save_requested);
}

#[test]
fn test_matching_paren_jump() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("fn main() { println!(\"hello\"); }");
    let mut cur = TextCursor::new(0, 10); // '{' at 10

    // % で対応する '}' (31) へジャンプ
    vi.execute(&ViCommand::Move(Motion::MatchingParen), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 31);

    // 再度 % で元の '{' (10) へ戻る
    vi.execute(&ViCommand::Move(Motion::MatchingParen), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 10);
}

#[test]
fn test_visual_block_yank_and_paste() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("apple 100\nbanana 200\ncherry 300");
    let mut cur = TextCursor::new(0, 0);

    // Ctrl-V で VisualBlock モードに入る
    vi.execute(
        &ViCommand::EnterMode(ViMode::VisualBlock),
        &mut buf,
        &mut cur,
    );
    assert_eq!(cur.anchor, Some(CursorPosition::new(0, 0)));

    // 2行下、4文字右へ移動して "apple", "banan", "cherr" の頭5文字を選択
    cur.pos = CursorPosition::new(2, 4);

    // y で矩形ヤンク
    vi.execute(
        &ViCommand::VisualOperate {
            op: Operator::Yank,
            register: None,
        },
        &mut buf,
        &mut cur,
    );
    assert_eq!(vi.mode(), ViMode::Normal);
    assert!(cur.anchor.is_none());

    // 行0、列12（行末の後ろ）に矩形貼り付け
    cur.pos = CursorPosition::new(0, 12);
    vi.execute(
        &ViCommand::PasteBefore {
            count: 1,
            register: None,
        },
        &mut buf,
        &mut cur,
    );

    let lines: Vec<String> = (0..buf.line_count())
        .filter_map(|i| buf.line_text(i))
        .collect();
    assert_eq!(lines[0], "apple 100   apple");
    assert_eq!(lines[1], "banana 200  banan");
    assert_eq!(lines[2], "cherry 300  cherr");
}

#[test]
fn test_visual_block_insert() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("item1\nitem2\nitem3");
    let mut cur = TextCursor::new(0, 0);

    // VisualBlock で全行の先頭を選択
    vi.execute(
        &ViCommand::EnterMode(ViMode::VisualBlock),
        &mut buf,
        &mut cur,
    );
    cur.pos = CursorPosition::new(2, 0);

    // I で一括挿入モードへ
    vi.execute(
        &ViCommand::VisualBlockInsert { at_end: false },
        &mut buf,
        &mut cur,
    );
    assert_eq!(vi.mode(), ViMode::Insert);

    // 1行目に "> " を入力
    vi.record_inserted_char('>');
    buf.insert_text(0, ">");
    vi.record_inserted_char(' ');
    buf.insert_text(1, " ");

    // Esc で Normal モードに戻ると、2行目・3行目にも "> " が一括挿入される
    vi.execute(&ViCommand::EnterMode(ViMode::Normal), &mut buf, &mut cur);

    let lines: Vec<String> = (0..buf.line_count())
        .filter_map(|i| buf.line_text(i))
        .collect();
    assert_eq!(lines[0], "> item1");
    assert_eq!(lines[1], "> item2");
    assert_eq!(lines[2], "> item3");
}
