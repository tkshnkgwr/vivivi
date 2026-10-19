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

    // 行頭 (col 0) から % を押した場合、行内の最初の括弧 '(' (col 7) の対応 ')' (col 8) へジャンプ
    cur.pos.col = 0;
    vi.execute(&ViCommand::Move(Motion::MatchingParen), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 8);

    // 全角括弧「」のジャンプテスト
    let mut zen_buf = TextBuffer::from_text("吾輩は「猫」である");
    let mut zen_cur = TextCursor::new(0, 0); // 先頭「吾」
    // 行頭から % を押すと、行内の「 (col 3) の対応 」 (col 5) へジャンプ
    vi.execute(
        &ViCommand::Move(Motion::MatchingParen),
        &mut zen_buf,
        &mut zen_cur,
    );
    assert_eq!(zen_cur.pos.col, 5);

    // 再度 % で対応する「 (col 3) へジャンプ
    vi.execute(
        &ViCommand::Move(Motion::MatchingParen),
        &mut zen_buf,
        &mut zen_cur,
    );
    assert_eq!(zen_cur.pos.col, 3);
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

#[test]
fn test_visual_block_change_and_replace() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("apple\nbanana\ncherry\n");
    let mut cur = TextCursor::new(0, 0);

    // 1. VisualBlock で (0,0) から (2,1) まで矩形選択（先頭2文字 'ap', 'ba', 'ch'）
    vi.execute(
        &ViCommand::EnterMode(ViMode::VisualBlock),
        &mut buf,
        &mut cur,
    );
    cur.anchor = Some(cur.pos);
    cur.pos = CursorPosition::new(2, 1);

    // 2. r で一括文字置換 'X' -> 'XXple', 'XXnana', 'XXerry'
    vi.execute(&ViCommand::VisualReplaceChar('X'), &mut buf, &mut cur);
    assert_eq!(vi.mode(), ViMode::Normal);

    let lines: Vec<String> = (0..buf.line_count())
        .filter_map(|i| buf.line_text(i))
        .collect();
    assert_eq!(lines[0], "XXple");
    assert_eq!(lines[1], "XXnana");
    assert_eq!(lines[2], "XXerry");

    // 3. 次に VisualBlock で 'XX' を選択し、c で一括置換
    cur.pos = CursorPosition::new(0, 0);
    vi.execute(
        &ViCommand::EnterMode(ViMode::VisualBlock),
        &mut buf,
        &mut cur,
    );
    cur.anchor = Some(cur.pos);
    cur.pos = CursorPosition::new(2, 1);

    vi.execute(
        &ViCommand::VisualOperate {
            op: Operator::Change,
            register: None,
        },
        &mut buf,
        &mut cur,
    );
    assert_eq!(vi.mode(), ViMode::Insert);

    // 'NEW_' を入力
    for c in "NEW_".chars() {
        vi.record_inserted_char(c);
        let pos = cur.pos.col;
        buf.insert_text(pos, &c.to_string());
        cur.pos.col += 1;
    }

    // Esc で Normal に戻ると全行に 'NEW_' が反映される
    vi.execute(&ViCommand::EnterMode(ViMode::Normal), &mut buf, &mut cur);

    let lines2: Vec<String> = (0..buf.line_count())
        .filter_map(|i| buf.line_text(i))
        .collect();
    assert_eq!(lines2[0], "NEW_ple");
    assert_eq!(lines2[1], "NEW_nana");
    assert_eq!(lines2[2], "NEW_erry");
}

#[test]
fn test_macro_record_and_playback() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("line1\nline2\nline3\n");
    let mut cur = TextCursor::new(0, 0);

    // 1. qa でレジスタ a にマクロ記録開始
    let cmd = vi.feed_char('q');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('a');
    assert_eq!(cmd, Some(ViCommand::RecordMacro('a')));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.recording_register, Some('a'));

    // 2. 行末に "!" を追加して次行へ移動する操作（A!Esc j）
    let cmd = vi.feed_char('A');
    assert_eq!(cmd, Some(ViCommand::InsertLineEnd));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.mode(), ViMode::Insert);

    vi.record_inserted_char('!');
    let pos = buf.cursor_to_position(cur.pos);
    buf.insert_char(pos, '!');
    cur.pos.col += 1;

    let cmd = vi.feed_char('\x1b');
    assert_eq!(cmd, Some(ViCommand::EnterMode(ViMode::Normal)));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.mode(), ViMode::Normal);

    let cmd = vi.feed_char('j');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);

    // 3. q でマクロ記録終了
    let cmd = vi.feed_char('q');
    assert_eq!(cmd, Some(ViCommand::StopRecordMacro));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert!(vi.recording_register.is_none());
    assert!(vi.macro_storage.contains_key(&'a'));

    // 4. @a でマクロ再生（2行目 line2 に適用される）
    let cmd = vi.feed_char('@');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('a');
    assert_eq!(cmd, Some(ViCommand::ExecuteMacro('a')));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);

    // 5. @@ で直前マクロ再実行（3行目 line3 に適用される）
    let cmd = vi.feed_char('@');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('@');
    assert_eq!(cmd, Some(ViCommand::ExecuteMacro('@')));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);

    let lines: Vec<String> = (0..buf.line_count())
        .filter_map(|i| buf.line_text(i))
        .collect();
    assert_eq!(lines[0], "line1!");
    assert_eq!(lines[1], "line2!");
    assert_eq!(lines[2], "line3!");
}

#[test]
fn test_expression_substitute_integration() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("val: 10\nval: 20\nval: 30");
    let mut cur = TextCursor::new(0, 0);

    // %s/([0-9]+)/$1 * 2/ge
    let res = vi.execute(
        &ViCommand::ExCommand("%s/([0-9]+)/$1 * 2/ge".to_string()),
        &mut buf,
        &mut cur,
    );
    assert_eq!(res.message, Some("3 箇所の式置換を行いました".to_string()));
    let lines: Vec<String> = (0..buf.line_count())
        .filter_map(|i| buf.line_text(i))
        .collect();
    assert_eq!(lines[0], "val: 20");
    assert_eq!(lines[1], "val: 40");
    assert_eq!(lines[2], "val: 60");

    // 単一行での式連動リナンバ :s/val: [0-9]+/\"item_\" + $i/e
    cur.pos.line = 0;
    let res = vi.execute(
        &ViCommand::ExCommand(r#"s/val: [0-9]+/"item_" + $i/e"#.to_string()),
        &mut buf,
        &mut cur,
    );
    assert_eq!(res.message, Some("1 箇所の式置換を行いました".to_string()));
    assert_eq!(buf.line_text(0).unwrap(), "item_1");
}

#[test]
fn test_fftt_fullwidth_matching_and_operator() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("Hello (世界) １２３ 「テスト」");
    let mut cur = TextCursor::new(0, 0);

    // 1. 'f' '(' で半角 '(' に移動 (col 6)
    let cmd = vi.feed_char('f');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('(');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 6);

    // 2. ';' で次の '(' (全角 '「' col 15) へジャンプ
    let cmd = vi.feed_char(';');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 15);

    // 3. ',' で逆方向（半角 '(' col 6）へ復帰
    let cmd = vi.feed_char(',');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.col, 6);

    // 4. 'df)' で全角 '）' まで削除 -> "Hello  １２３ 「テスト」"
    let cmd = vi.feed_char('d');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('f');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char(')');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.to_string(), "Hello  １２３ 「テスト」");
}

#[test]
fn test_word_search_variants() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("cat concat catalog concatenate");
    let mut cur = TextCursor::new(0, 0); // "cat"

    // * (完全単語一致)
    let cmd = vi.feed_char('*');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    // 次の単語境界 cat は存在しないので自身へラップアラウンド
    assert_eq!(vi.last_search_pattern, Some(r"\bcat\b".to_string()));

    // g* (部分一致)
    let cmd = vi.feed_char('g');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('*');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.last_search_pattern, Some("cat".to_string()));
    // 次の部分一致 "concat" 内の "cat" (col 7) へジャンプ
    assert_eq!(cur.pos.col, 7);

    // g< (前方一致)
    cur.pos.col = 0;
    let cmd = vi.feed_char('g');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('<');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.last_search_pattern, Some(r"\bcat".to_string()));
    // "catalog" の先頭 (col 11) へジャンプ
    assert_eq!(cur.pos.col, 11);
}

#[test]
fn test_increment_decrement_and_boolean_toggle() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("count = 10; flag = true; letter = a;");
    let mut cur = TextCursor::new(0, 0);

    // 1. #+ で 10 -> 11
    let cmd = vi.feed_char('#');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('+');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert!(buf.to_string().starts_with("count = 11;"));

    // 2. 5#- で 11 -> 6
    let cmd = vi.feed_char('5');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('#');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('-');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert!(buf.to_string().starts_with("count = 6;"));

    // 3. #! で true -> false
    cur.pos.col = 19; // "true"
    let cmd = vi.feed_char('#');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('!');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert!(buf.to_string().contains("flag = false;"));

    // 4. アルファベットのインクリメント a -> b
    cur.pos.col = buf.to_string().rfind('a').unwrap();
    let cmd = vi.feed_char('#');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('#'); // ## もインクリメント
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert!(buf.to_string().ends_with("letter = b;"));
}

#[test]
fn test_marks_and_last_edit_jump() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("first line\nsecond line\nthird line");
    let mut cur = TextCursor::new(0, 0);

    // 1. mA でマーク A を設定 (0行0列)
    let cmd = vi.feed_char('m');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('A');
    assert_eq!(cmd, Some(ViCommand::SetMark('A')));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert!(vi.marks.contains_key(&'A'));

    // 2. 移動して編集操作（x で文字削除）
    cur.pos.line = 2;
    cur.pos.col = 4;
    let cmd = vi.feed_char('x');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.last_edit_pos, Some(CursorPosition::new(2, 4)));

    // 3. 'A でマーク A (0行目) へジャンプ
    let cmd = vi.feed_char('\'');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('A');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 0);

    // 4. '@ で直前編集位置 (2行目) へジャンプ
    let cmd = vi.feed_char('\'');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('@');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 2);

    // 5. :marks コマンドで一覧メッセージ取得
    let res = vi.execute(
        &ViCommand::ExCommand("marks".to_string()),
        &mut buf,
        &mut cur,
    );
    assert!(res.message.unwrap().contains("'A': 1行"));
}

#[test]
fn test_copy_from_above_line() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("    function calculateTotal() {\n    ");
    let mut cur = TextCursor::new(1, 4);

    // = で直前行の 'f' 以降から ')' までの文字列を複製
    let cmd = vi.feed_char('=');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char(')');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(1).unwrap(), "    function calculateTotal()");
}

#[test]
fn test_tag_jump_and_pop() {
    let mut vi = ViEngine::new();
    let code = "fn main() {\n    foo();\n}\n\nfn foo() {\n    println!(\"hi\");\n}\n";
    let mut buf = TextBuffer::from_text(code);
    vi.current_file_path = "main.rs".to_string();

    // 2行目の "foo" 上にカーソルを置く
    let mut cur = TextCursor::new(1, 4);

    // Ctrl-] (\x1d) でタグジャンプ
    let cmd = vi.feed_char('\x1d');
    assert!(cmd.is_some());
    let res = vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 4); // "fn foo()" 行へジャンプ
    assert!(res.message.unwrap().contains("foo"));

    // Ctrl-t (\x14) で元の行へ復帰
    let cmd = vi.feed_char('\x14');
    assert!(cmd.is_some());
    let res = vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 1); // 2行目へ復帰
    assert!(res.message.unwrap().contains("復帰"));
}

#[test]
fn test_basic_editing_commands() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("hello world\nsecond line\nthird line");
    let mut cur = TextCursor::new(0, 0);

    // 1. 'r' による文字置換: 'h' -> 'H'
    let cmd = vi.feed_char('r');
    assert_eq!(cmd, None);
    let cmd = vi.feed_char('H');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "Hello world");

    // 2. '~' によるケース反転: 'H' -> 'h', カーソルが1文字進む
    let cmd = vi.feed_char('~');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "hello world");
    assert_eq!(cur.pos.col, 1);

    // 3. 'J' による行結合: 0行目と1行目を連結
    let cmd = vi.feed_char('J');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "hello world second line");
    assert_eq!(buf.line_count(), 2);

    // 4. 's' による文字置換挿入: カーソル位置 'e' を削除して Insert モードへ
    cur.pos.col = 1; // 'e' の位置へ
    let cmd = vi.feed_char('s');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(buf.line_text(0).unwrap(), "hllo world second line");

    // 5. 'o' (OpenLineBelow) の動作確認
    vi.mode = ViMode::Normal;
    cur.pos.line = 0;
    cur.pos.col = 0;
    let cmd = vi.feed_char('o');
    assert!(cmd.is_some());
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(cur.pos.line, 1);
    assert_eq!(cur.pos.col, 0);
    assert_eq!(buf.line_text(0).unwrap(), "hllo world second line");
    assert_eq!(buf.line_text(1).unwrap(), "");
    assert_eq!(buf.line_text(2).unwrap(), "third line");
}

#[test]
fn test_indent_operators() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("first\nsecond\nthird");
    let mut cur = TextCursor::new(0, 0);

    // 1. '>>' で 0行目をインデント
    let cmd = vi.feed_char('>');
    assert!(cmd.is_none());
    let cmd = vi.feed_char('>');
    assert_eq!(
        cmd,
        Some(ViCommand::OperateLine {
            op: Operator::ShiftRight,
            count: 1,
            register: None,
        })
    );
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "    first");

    // 2. '<<' で 0行目をアンインデント
    let cmd = vi.feed_char('<');
    assert!(cmd.is_none());
    let cmd = vi.feed_char('<');
    assert_eq!(
        cmd,
        Some(ViCommand::OperateLine {
            op: Operator::ShiftLeft,
            count: 1,
            register: None,
        })
    );
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "first");

    // 3. '2>>' で 2行分一括インデント
    vi.feed_char('2');
    vi.feed_char('>');
    let cmd = vi.feed_char('>');
    assert_eq!(
        cmd,
        Some(ViCommand::OperateLine {
            op: Operator::ShiftRight,
            count: 2,
            register: None,
        })
    );
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "    first");
    assert_eq!(buf.line_text(1).unwrap(), "    second");
    assert_eq!(buf.line_text(2).unwrap(), "third");

    // 4. ビジュアルモード（V: VisualLine）での '>' と '<'
    vi.set_mode(ViMode::VisualLine);
    cur.pos.line = 1;
    cur.anchor = Some(CursorPosition::new(2, 0));
    let cmd = vi.feed_char('>');
    assert_eq!(
        cmd,
        Some(ViCommand::VisualOperate {
            op: Operator::ShiftRight,
            register: None,
        })
    );
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.mode(), ViMode::Normal);
    assert_eq!(buf.line_text(1).unwrap(), "        second");
    assert_eq!(buf.line_text(2).unwrap(), "    third");
}

#[test]
fn test_paragraph_motion() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("line1\nline2\n\nline3\nline4\n\nline5");
    let mut cur = TextCursor::new(0, 0);

    // 1. '}' で次の空行（2行目）へ移動
    let cmd = vi.feed_char('}');
    assert_eq!(cmd, Some(ViCommand::Move(Motion::ParagraphForward(1))));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 2);

    // 2. 再度 '}' でその次の空行（5行目）へ移動
    let cmd = vi.feed_char('}');
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 5);

    // 3. '{' で前の空行（2行目）へ移動
    let cmd = vi.feed_char('{');
    assert_eq!(cmd, Some(ViCommand::Move(Motion::ParagraphBackward(1))));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 2);

    // 4. '{' で先頭（0行目）へ移動
    let cmd = vi.feed_char('{');
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(cur.pos.line, 0);
}

#[test]
fn test_scroll_adjust() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("line1\nline2\nline3");
    let mut cur = TextCursor::new(1, 0);

    // 'zz' (Center)
    vi.feed_char('z');
    let cmd = vi.feed_char('z');
    assert_eq!(cmd, Some(ViCommand::ScrollAdjust(ScrollTarget::Center)));
    let res = vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(res.scroll_adjust, Some(ScrollTarget::Center));

    // 'zt' (Top)
    vi.feed_char('z');
    let cmd = vi.feed_char('t');
    assert_eq!(cmd, Some(ViCommand::ScrollAdjust(ScrollTarget::Top)));
    let res = vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(res.scroll_adjust, Some(ScrollTarget::Top));

    // 'zb' (Bottom)
    vi.feed_char('z');
    let cmd = vi.feed_char('b');
    assert_eq!(cmd, Some(ViCommand::ScrollAdjust(ScrollTarget::Bottom)));
    let res = vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(res.scroll_adjust, Some(ScrollTarget::Bottom));
}

#[test]
fn test_dot_repeat_commands() {
    let mut vi = ViEngine::new();
    let mut buf = TextBuffer::from_text("abcdef\n123456\nxyz");
    let mut cur = TextCursor::new(0, 0);

    // 1. 'x' で文字削除 -> 'b'が先頭に
    let cmd = vi.feed_char('x');
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "bcdef");

    // 2. '.' で直前コマンド 'x' を繰り返し実行 -> 'c'が先頭に
    let cmd = vi.feed_char('.');
    assert_eq!(cmd, Some(ViCommand::RepeatLastChange));
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(0).unwrap(), "cdef");

    // 3. 挿入モード 'i' で "NEW" を挿入し Esc で抜ける
    let cmd = vi.feed_char('i');
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    vi.record_inserted_char('N');
    vi.record_inserted_char('E');
    vi.record_inserted_char('W');
    let cmd = vi.feed_char('\x1b');
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(vi.mode(), ViMode::Normal);

    // 4. 1行目へ移動して '.' を押すと "NEW" が反復挿入される
    cur.pos.line = 1;
    cur.pos.col = 0;
    let cmd = vi.feed_char('.');
    vi.execute(&cmd.unwrap(), &mut buf, &mut cur);
    assert_eq!(buf.line_text(1).unwrap(), "NEW123456");
}
