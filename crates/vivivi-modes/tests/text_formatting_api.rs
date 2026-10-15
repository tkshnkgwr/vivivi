//! テキスト整形・変換・編集機能の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、`align`（垂直整列）、`calc`（BOX数値集計）、
//! `conv`（文字種変換）、`csv`（CSV/TSV整形）、`keisen`（罫線描画・保護）、`line_edit`（行頭行末挿入）、
//! `renumber`（連番再計算）、`zen_coding`（HTML展開）、`expr_subst`（式置換）の公開振る舞いを検証します。

use vivivi_core::{CursorPosition, TextBuffer};
use vivivi_modes::align::align_lines;
use vivivi_modes::calc::{calculate_box_lines, calculate_box_selection};
use vivivi_modes::conv::{ConversionType, convert_text};
use vivivi_modes::csv::{CsvMode, CsvOptions, Delimiter};
use vivivi_modes::expr_subst::{ExprContext, ExprParser, ExprValue, execute_expr_substitute};
use vivivi_modes::keisen::{
    KT_DOWN_THIN, KT_LEFT_THIN, KT_RIGHT_THIN, KT_UP_THIN, KeisenDirection, KeisenMode,
    char_to_keisen_state, is_horizontal_keisen, is_vertical_keisen, state_to_keisen_char,
};
use vivivi_modes::line_edit::insert_beg_end_lines;
use vivivi_modes::renumber::renumber_lines;
use vivivi_modes::zen_coding::expand_zen_coding;

// ==========================================
// 1. Align (垂直整列)
// ==========================================

#[test]
fn test_align_equal_sign() {
    let lines = vec![
        "let a = 1;",
        "let long_variable_name = 200;",
        "let count = 3;",
    ];
    let aligned = align_lines(&lines, "=");
    assert_eq!(
        aligned,
        vec![
            "let a                  = 1;",
            "let long_variable_name = 200;",
            "let count              = 3;",
        ]
    );
}

#[test]
fn test_align_colon_with_zenkaku() {
    let lines = vec!["名前: 田中", "生年月日: 1990/01/01", "住所: 東京都"];
    let aligned = align_lines(&lines, ":");
    assert_eq!(
        aligned,
        vec![
            "名前     : 田中",
            "生年月日 : 1990/01/01",
            "住所     : 東京都",
        ]
    );
}

#[test]
fn test_align_skip_unmatched_lines() {
    let lines = vec!["// コメント行", "val x = 10", "", "val yyyyy = 20"];
    let aligned = align_lines(&lines, "=");
    assert_eq!(
        aligned,
        vec!["// コメント行", "val x     = 10", "", "val yyyyy = 20",]
    );
}

// ==========================================
// 2. Calc (BOX数値集計)
// ==========================================

#[test]
fn test_box_calculation_basic() {
    let lines = vec![
        "  10  ".to_string(),
        "  20  ".to_string(),
        "  30  ".to_string(),
    ];
    let result = calculate_box_lines(&lines).expect("should calculate");
    assert_eq!(result.count, 3);
    assert_eq!(result.sum, 60.0);
    assert_eq!(result.average, 20.0);
    assert!((result.std_dev - 8.1649).abs() < 1e-4);
    assert_eq!(result.min, 10.0);
    assert_eq!(result.max, 30.0);
}

#[test]
fn test_box_calculation_with_commas_and_floats() {
    let lines = vec![
        "1,234.50".to_string(),
        "2,000.00".to_string(),
        "-234.50".to_string(),
    ];
    let result = calculate_box_lines(&lines).expect("should calculate");
    assert_eq!(result.count, 3);
    assert_eq!(result.sum, 3000.0);
    assert_eq!(result.average, 1000.0);
    assert_eq!(result.min, -234.50);
    assert_eq!(result.max, 2000.0);
}

#[test]
fn test_calculate_box_selection_from_buffer() {
    let text = "Item A:   100 yen\nItem B:   250 yen\nItem C:   150 yen";
    let buffer = TextBuffer::from_text(text);
    let result = calculate_box_selection(&buffer, 0, 2, 10, 13).expect("should find numbers");
    assert_eq!(result.count, 3);
    assert_eq!(result.sum, 500.0);
    assert_eq!(result.average, 166.66666666666666);
}

// ==========================================
// 3. Conv (文字種変換)
// ==========================================

#[test]
fn test_zenkaku_hankaku_alphanumeric() {
    let text = "Hello, World! 123";
    let zen = convert_text(text, ConversionType::ToZenkakuAlphanumeric);
    assert_eq!(zen, "Ｈｅｌｌｏ，　Ｗｏｒｌｄ！　１２３");

    let han = convert_text(&zen, ConversionType::ToHankakuAlphanumeric);
    assert_eq!(han, "Hello, World! 123");
}

#[test]
fn test_hiragana_katakana() {
    let text = "あいうえお かきくけこ ゔ";
    let kata = convert_text(text, ConversionType::ToKatakana);
    assert_eq!(kata, "アイウエオ カキクケコ ヴ");

    let hira = convert_text(&kata, ConversionType::ToHiragana);
    assert_eq!(hira, "あいうえお かきくけこ ゔ");
}

#[test]
fn test_zenkaku_hankaku_katakana() {
    let text = "アイウエオ ガギグゲゴ パピプペポ ヴー";
    let han = convert_text(text, ConversionType::ToHankakuKatakana);
    assert_eq!(han, "ｱｲｳｴｵ ｶﾞｷﾞｸﾞｹﾞｺﾞ ﾊﾟﾋﾟﾌﾟﾍﾟﾎﾟ ｳﾞｰ");

    let zen = convert_text(&han, ConversionType::ToZenkakuKatakana);
    assert_eq!(zen, "アイウエオ ガギグゲゴ パピプペポ ヴー");
}

#[test]
fn test_case_conversions() {
    let text = "Hello World!";
    assert_eq!(
        convert_text(text, ConversionType::ToUppercase),
        "HELLO WORLD!"
    );
    assert_eq!(
        convert_text(text, ConversionType::ToLowercase),
        "hello world!"
    );
}

// ==========================================
// 4. CSV/TSV 整形
// ==========================================

#[test]
fn test_csv_column_widths_calculation() {
    let mut csv = CsvMode::default();
    let lines = vec!["id,name,score", "1,\"Alice\",100", "20,\"Bob (ボブ)\",85"];

    csv.calculate_widths(lines);

    assert_eq!(csv.column_widths[0], 3);
    assert_eq!(csv.column_widths[1], 10);
    assert_eq!(csv.column_widths[2], 5);
}

#[test]
fn test_tsv_delimiter() {
    let mut tsv = CsvMode::new(CsvOptions {
        delimiter: Delimiter::Tab,
        ..Default::default()
    });

    let lines = vec!["ColA\tColB", "LongerData\t1"];
    tsv.calculate_widths(lines);

    assert_eq!(tsv.column_widths[0], 10);
    assert_eq!(tsv.column_widths[1], 4);
}

#[test]
fn test_csv_format_line_and_quote_hiding() {
    let mut csv = CsvMode::default();
    let lines = vec!["id,name", "1,\"Alice\""];
    csv.calculate_widths(lines);

    let active = csv.format_line("1,\"Alice\"", true);
    assert_eq!(active, "1,\"Alice\"");

    let formatted = csv.format_line("1,\"Alice\"", false);
    assert!(formatted.contains("Alice"));
}

// ==========================================
// 5. Zen-Coding (HTML展開)
// ==========================================

#[test]
fn test_single_tag() {
    let res = expand_zen_coding("div").unwrap();
    assert_eq!(res, "<div></div>");
}

#[test]
fn test_id_and_classes() {
    let res = expand_zen_coding("div#header.nav.dark").unwrap();
    assert_eq!(res, "<div id=\"header\" class=\"nav dark\"></div>");
}

#[test]
fn test_children_and_multiplication() {
    let res = expand_zen_coding("ul>li*2").unwrap();
    assert_eq!(res, "<ul>\n  <li></li>\n  <li></li>\n</ul>");
}

#[test]
fn test_siblings() {
    let res = expand_zen_coding("header+footer").unwrap();
    assert_eq!(res, "<header></header>\n<footer></footer>");
}

// ==========================================
// 6. Renumber (連番再計算)
// ==========================================

#[test]
fn test_renumber_dot_list() {
    let lines = vec!["1. apple", "1. banana", "1. cherry"];
    let result = renumber_lines(&lines, 1, 1);
    assert_eq!(result, vec!["1. apple", "2. banana", "3. cherry"]);
}

#[test]
fn test_renumber_with_indent_and_step() {
    let lines = vec!["  1. item A", "  5. item B", "  9. item C"];
    let result = renumber_lines(&lines, 10, 5);
    assert_eq!(result, vec!["  10. item A", "  15. item B", "  20. item C"]);
}

#[test]
fn test_renumber_bracket_list() {
    let lines = vec!["[1] first", "[2] second", "[3] third"];
    let result = renumber_lines(&lines, 100, 10);
    assert_eq!(result, vec!["[100] first", "[110] second", "[120] third"]);
}

// ==========================================
// 7. Line Edit (行頭行末一括挿入)
// ==========================================

#[test]
fn test_insert_beg_end_comment_out() {
    let lines = vec!["let a = 1;", "    let b = 2;", "", "let c = 3;"];
    let result = insert_beg_end_lines(&lines, "// ", "", true, true);
    assert_eq!(
        result,
        vec!["// let a = 1;", "    // let b = 2;", "", "// let c = 3;",]
    );
}

#[test]
fn test_insert_beg_end_html_tag() {
    let lines = vec!["item 1", "item 2"];
    let result = insert_beg_end_lines(&lines, "<li>", "</li>", false, false);
    assert_eq!(result, vec!["<li>item 1</li>", "<li>item 2</li>"]);
}

// ==========================================
// 8. Keisen (罫線描画・保護)
// ==========================================

#[test]
fn test_keisen_table_lookups() {
    assert_eq!(
        state_to_keisen_char(KT_LEFT_THIN | KT_RIGHT_THIN),
        Some('─')
    );
    assert_eq!(state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN), Some('│'));
    assert_eq!(
        state_to_keisen_char(KT_DOWN_THIN | KT_RIGHT_THIN),
        Some('┌')
    );
    assert_eq!(state_to_keisen_char(KT_LEFT_THIN | KT_UP_THIN), Some('┘'));

    assert_eq!(char_to_keisen_state('│'), Some(KT_UP_THIN | KT_DOWN_THIN));
    assert_eq!(
        char_to_keisen_state('─'),
        Some(KT_LEFT_THIN | KT_RIGHT_THIN)
    );
    assert_eq!(
        char_to_keisen_state('┌'),
        Some(KT_DOWN_THIN | KT_RIGHT_THIN)
    );

    assert!(is_vertical_keisen('│'));
    assert!(is_vertical_keisen('┃'));
    assert!(is_horizontal_keisen('─'));
    assert!(is_horizontal_keisen('━'));
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

    km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
    km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
    assert_eq!(cursor, CursorPosition::new(0, 2));
    assert_eq!(buffer.char_at(0), Some('─'));
    assert_eq!(buffer.char_at(1), Some('─'));

    km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    assert_eq!(cursor, CursorPosition::new(1, 2));
    assert_eq!(buffer.char_at(2), Some('┐'));

    km.step(&mut buffer, &mut cursor, KeisenDirection::Up, true);
    assert_eq!(cursor, CursorPosition::new(0, 2));
    assert_eq!(buffer.char_at(2), Some('─'));
}

// ==========================================
// 9. Expr Substitute (式置換)
// ==========================================

#[test]
fn test_arithmetic_expression() {
    let mut ctx = ExprContext::new();
    ctx.set_var("1", ExprValue::String("25".to_string()));
    ctx.set_var("i", ExprValue::Number(3.0));

    let mut parser = ExprParser::new("$1 * 2 + $i", &mut ctx).unwrap();
    let val = parser.parse_expression().unwrap();
    assert_eq!(val, ExprValue::Number(53.0));
    assert_eq!(val.to_display_string(), "53");
}

#[test]
fn test_renumbering_substitute() {
    let input = "item\nitem\nitem";
    let (output, count) =
        execute_expr_substitute(input, "item", "\"item_\" + $i", true, false, 1).unwrap();
    assert_eq!(count, 3);
    assert_eq!(output, "item_1\nitem_2\nitem_3");
}

#[test]
fn test_numeric_multiplication_substitute() {
    let input = "width: 10px;\nheight: 20px;";
    let (output, count) =
        execute_expr_substitute(input, r"(\d+)px", r#""px_" + ($1 * 2)"#, true, false, 1).unwrap();
    assert_eq!(count, 2);
    assert_eq!(output, "width: px_20;\nheight: px_40;");
}
