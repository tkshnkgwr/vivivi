use super::*;
use vivivi_core::TextBuffer;

#[test]
fn test_keisen_table_lookups() {
    // 水平線・垂直線
    assert_eq!(
        state_to_keisen_char(KT_LEFT_THIN | KT_RIGHT_THIN),
        Some('─')
    );
    assert_eq!(
        state_to_keisen_char(KT_LEFT_THICK | KT_RIGHT_THICK),
        Some('━')
    );
    assert_eq!(state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN), Some('│'));
    assert_eq!(state_to_keisen_char(KT_UP_THICK | KT_DOWN_THICK), Some('┃'));

    // コーナー
    assert_eq!(
        state_to_keisen_char(KT_DOWN_THIN | KT_RIGHT_THIN),
        Some('┌')
    );
    assert_eq!(
        state_to_keisen_char(KT_DOWN_THICK | KT_RIGHT_THICK),
        Some('┏')
    );
    assert_eq!(state_to_keisen_char(KT_LEFT_THIN | KT_DOWN_THIN), Some('┐'));
    assert_eq!(state_to_keisen_char(KT_UP_THIN | KT_RIGHT_THIN), Some('└'));
    assert_eq!(state_to_keisen_char(KT_LEFT_THIN | KT_UP_THIN), Some('┘'));

    // T字路・十字路
    assert_eq!(
        state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN | KT_RIGHT_THIN),
        Some('├')
    );
    assert_eq!(
        state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THIN),
        Some('┤')
    );
    assert_eq!(
        state_to_keisen_char(KT_DOWN_THIN | KT_LEFT_THIN | KT_RIGHT_THIN),
        Some('┬')
    );
    assert_eq!(
        state_to_keisen_char(KT_UP_THIN | KT_LEFT_THIN | KT_RIGHT_THIN),
        Some('┴')
    );
    assert_eq!(
        state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THIN | KT_RIGHT_THIN),
        Some('┼')
    );
    assert_eq!(
        state_to_keisen_char(KT_UP_THICK | KT_DOWN_THICK | KT_LEFT_THICK | KT_RIGHT_THICK),
        Some('╋')
    );
}

#[test]
fn test_keisen_mode_toggle() {
    let mut km = KeisenMode::new();
    assert!(!km.is_active);
    km.toggle();
    assert!(km.is_active);
    km.toggle();
    assert!(!km.is_active);
}

#[test]
fn test_keisen_draw_and_erase() {
    let mut buffer = TextBuffer::new();
    let mut cursor = CursorPosition::new(0, 0);
    let km = KeisenMode::new();

    // 右へ2回描画: ─ ─
    km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
    km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
    assert_eq!(cursor, CursorPosition::new(0, 2));
    assert_eq!(buffer.char_at(0), Some('─'));
    assert_eq!(buffer.char_at(1), Some('─'));

    // 下へ1回描画: 水平線の端点 (0, 2) から下へ曲がったため右上角「┐」に結合
    km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    assert_eq!(cursor, CursorPosition::new(1, 2));
    assert_eq!(buffer.char_at(2), Some('┐'));

    // 消去テスト: 上へ戻りながら消去
    km.step(&mut buffer, &mut cursor, KeisenDirection::Up, true);
    assert_eq!(cursor, CursorPosition::new(0, 2));
    // 下成分が消えて水平線 ─ に綺麗に戻る
    assert_eq!(buffer.char_at(2), Some('─'));
}

#[test]
fn test_keisen_draw_box_clean_corners() {
    let mut buffer = TextBuffer::new();
    let mut cursor = CursorPosition::new(0, 0);
    let km = KeisenMode::new();

    // 1. (0, 0) から右へ 2回描画: ─ ─
    km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
    km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
    assert_eq!(cursor, CursorPosition::new(0, 2));

    // 2. (0, 2) から下へ 2回描画:
    // (0, 2) は右上角「┐」となり、右へ突き出た「┬」にはならない！
    km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    assert_eq!(buffer.char_at(2), Some('┐'));

    km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    assert_eq!(cursor, CursorPosition::new(2, 2));

    // 3. (2, 2) から左へ 2回描画:
    // (2, 2) は右下角「┘」となり、下へ突き出た「┤」にはならない！
    km.step(&mut buffer, &mut cursor, KeisenDirection::Left, false);
    assert_eq!(cursor, CursorPosition::new(2, 1));
    let pos_2_2 = buffer.cursor_to_position(CursorPosition::new(2, 2));
    assert_eq!(buffer.char_at(pos_2_2), Some('┘'));

    km.step(&mut buffer, &mut cursor, KeisenDirection::Left, false);
    assert_eq!(cursor, CursorPosition::new(2, 0));

    // 4. (2, 0) から上へ 2回描画して四角を閉じる:
    // (2, 0) は左下角「└」となり、左へ突き出た「┴」にはならない！
    km.step(&mut buffer, &mut cursor, KeisenDirection::Up, false);
    let pos_2_0 = buffer.cursor_to_position(CursorPosition::new(2, 0));
    assert_eq!(buffer.char_at(pos_2_0), Some('└'));

    km.step(&mut buffer, &mut cursor, KeisenDirection::Up, false);
    assert_eq!(cursor, CursorPosition::new(0, 0));
    // (0, 0) は左上角「┌」となり、上へ突き出た「├」にはならない！
    assert_eq!(buffer.char_at(0), Some('┌'));

    // 四角形全体の完成形状を確認（糸のほつれが一切ないこと）
    assert_eq!(buffer.get_line(0), Some("┌─┐".to_string()));
    assert_eq!(buffer.get_line(1), Some("│ │".to_string()));
    assert_eq!(buffer.get_line(2), Some("└─┘".to_string()));
}

#[test]
fn test_keisen_draw_down_extends_lines() {
    // 最終行（初期は1行）から下へ罫線を引いた場合に行が自動増加することを確認
    let mut buffer = TextBuffer::new();
    let mut cursor = CursorPosition::new(0, 0);
    let km = KeisenMode::new();

    assert_eq!(buffer.line_count(), 1);
    km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    assert_eq!(cursor, CursorPosition::new(1, 0));
    assert_eq!(buffer.line_count(), 2);
    assert_eq!(buffer.get_line(0), Some("│".to_string()));
    assert_eq!(buffer.get_line(1), Some("│".to_string()));

    km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    assert_eq!(cursor, CursorPosition::new(2, 0));
    assert_eq!(buffer.line_count(), 3);
    assert_eq!(buffer.get_line(2), Some("│".to_string()));

    // さらに右へ引いた場合は下へ突き抜けず綺麗な左下角「└─」として結合
    km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
    assert_eq!(cursor, CursorPosition::new(2, 1));
    assert_eq!(buffer.get_line(2), Some("└─".to_string()));
}

#[test]
fn test_align_box_text() {
    let mut buffer = TextBuffer::from_text("│ hello     │\n│    world  │");

    // 1行目を右揃えに整列
    assert!(KeisenMode::align_box_text(
        &mut buffer,
        0,
        BoxAlignment::Right
    ));
    assert_eq!(buffer.get_line(0), Some("│      hello│".to_string()));

    // 1行目を中央揃えに整列
    assert!(KeisenMode::align_box_text(
        &mut buffer,
        0,
        BoxAlignment::Center
    ));
    assert_eq!(buffer.get_line(0), Some("│   hello   │".to_string()));

    // 1行目を左揃えに整列
    assert!(KeisenMode::align_box_text(
        &mut buffer,
        0,
        BoxAlignment::Left
    ));
    assert_eq!(buffer.get_line(0), Some("│hello      │".to_string()));
}

#[test]
fn test_keisen_protected_edit() {
    // "│ hello     │" (全長13文字: │, 空白, h, e, l, l, o, 空白x5, │)
    let mut buffer = TextBuffer::from_text("│ hello     │");
    let mut cursor = CursorPosition::new(0, 7); // 'o'の直後

    // '!' を挿入 -> 末尾の空白が1つ削られ、右枠線 '│' の位置は維持される
    assert!(KeisenMode::insert_char_protected(
        &mut buffer,
        &mut cursor,
        '!'
    ));
    assert_eq!(buffer.get_line(0), Some("│ hello!    │".to_string()));
    assert_eq!(cursor, CursorPosition::new(0, 8));

    // BackSpace で '!' を削除 -> 末尾に空白が補填され、右枠線の位置は維持される
    assert!(KeisenMode::delete_char_protected(
        &mut buffer,
        &mut cursor,
        true
    ));
    assert_eq!(buffer.get_line(0), Some("│ hello     │".to_string()));
    assert_eq!(cursor, CursorPosition::new(0, 7));

    // 全角文字の挿入テスト: 全角「あ」（幅2）を挿入 -> 末尾の空白が2つ削られ、右枠線の位置と全体の表示幅は維持される
    assert!(KeisenMode::insert_char_protected(
        &mut buffer,
        &mut cursor,
        'あ'
    ));
    assert_eq!(buffer.get_line(0), Some("│ helloあ   │".to_string()));
    assert_eq!(cursor, CursorPosition::new(0, 8));

    // BackSpace で全角「あ」を削除 -> 末尾に空白が2つ補填され、右枠線の位置と表示幅が維持される
    assert!(KeisenMode::delete_char_protected(
        &mut buffer,
        &mut cursor,
        true
    ));
    assert_eq!(buffer.get_line(0), Some("│ hello     │".to_string()));
    assert_eq!(cursor, CursorPosition::new(0, 7));

    // 空白に余裕がないボックスでの保護テスト
    let mut full_box_buffer = TextBuffer::from_text("│12345│");
    let mut full_cursor = CursorPosition::new(0, 3);
    // 空白がないため、枠線を壊さないよう挿入拒否される
    assert!(!KeisenMode::insert_char_protected(
        &mut full_box_buffer,
        &mut full_cursor,
        'x'
    ));
    assert_eq!(full_box_buffer.get_line(0), Some("│12345│".to_string()));

    // 余白が1文字分（半角スペース1個）しかない場合、全角文字（幅2）は挿入拒否される
    let mut tight_box_buffer = TextBuffer::from_text("│1234 │");
    let mut tight_cursor = CursorPosition::new(0, 3);
    assert!(!KeisenMode::insert_char_protected(
        &mut tight_box_buffer,
        &mut tight_cursor,
        'あ'
    ));
    assert_eq!(tight_box_buffer.get_line(0), Some("│1234 │".to_string()));
}
