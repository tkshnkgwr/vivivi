//! Tree-sitter 構文木解析・ハイライト・スマート選択 統合テスト (tests/test_treesitter.rs)

use vivivi_modes::syntax::{SyntaxLang, TokenType};
use vivivi_modes::treesitter::TreeSitterEngine;

#[test]
fn test_treesitter_rust_parse_and_ast() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Rust));
    assert_eq!(engine.current_language(), Some(SyntaxLang::Rust));

    let rust_code = r#"
pub fn add(a: i32, b: i32) -> i32 {
    let result = a + b;
    result
}
"#;

    assert!(engine.parse(rust_code));
    assert!(engine.has_tree());

    let root = engine.root_node().expect("root node should exist");
    assert_eq!(root.kind(), "source_file");
    assert!(root.child_count() > 0);
}

#[test]
fn test_treesitter_highlight_line() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Rust));

    let rust_code = "pub fn greet() {\n    let msg = \"hello\";\n}\n";
    assert!(engine.parse(rust_code));

    // 0行目: "pub fn greet() {"
    let tokens_line0 = engine.highlight_line(0, "pub fn greet() {");
    assert!(!tokens_line0.is_empty());

    // "pub" と "fn" が Keyword として認識されているか
    let has_pub = tokens_line0
        .iter()
        .any(|t| t.text == "pub" && t.token_type == TokenType::Keyword);
    let has_fn = tokens_line0
        .iter()
        .any(|t| t.text == "fn" && t.token_type == TokenType::Keyword);
    assert!(has_pub);
    assert!(has_fn);

    // 1行目: "    let msg = \"hello\";"
    let tokens_line1 = engine.highlight_line(1, "    let msg = \"hello\";");
    assert!(!tokens_line1.is_empty());
    let has_let = tokens_line1
        .iter()
        .any(|t| t.text == "let" && t.token_type == TokenType::Keyword);
    let has_str = tokens_line1
        .iter()
        .any(|t| t.text.contains("hello") && t.token_type == TokenType::StringLiteral);
    assert!(has_let);
    assert!(has_str);
}

#[test]
fn test_treesitter_smart_expand_and_shrink_selection() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Rust));

    let rust_code = "fn main() {\n    let count = 42;\n}\n";
    assert!(engine.parse(rust_code));

    // 1行目・列8 ("count" の位置) でスマート選択
    // 1回目: 最小ノード (identifier: "count")
    let range1 = engine
        .smart_expand_selection(1, 8, None)
        .expect("step 1 selection");
    assert_eq!(range1.start_line, 1);
    assert!(range1.start_col <= 8 && range1.end_col >= 13);

    // 2回目: 親ノードへ拡大 (let_declaration または binary expression)
    let range2 = engine
        .smart_expand_selection(1, 8, Some(range1))
        .expect("step 2 selection");
    assert!(
        range2.start_byte < range1.start_byte || range2.end_byte > range1.end_byte,
        "selection should expand"
    );

    // 3回目: さらに拡大 (block または function全体)
    let range3 = engine
        .smart_expand_selection(1, 8, Some(range2))
        .expect("step 3 selection");
    assert!(range3.start_byte < range2.start_byte || range3.end_byte > range2.end_byte);

    // 縮小 (Shrink) -> range2 に戻る
    let shrunk1 = engine.smart_shrink_selection().expect("shrink 1");
    assert_eq!(shrunk1, range2);

    // 縮小 (Shrink) -> range1 に戻る
    let shrunk2 = engine.smart_shrink_selection().expect("shrink 2");
    assert_eq!(shrunk2, range1);
}

#[test]
fn test_treesitter_incremental_edit() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Rust));

    let initial_code = "fn foo() {}";
    assert!(engine.parse(initial_code));

    // インクリメンタル変更通知: "foo" を "foobar" に変更 (+3文字)
    engine.edit(3, 6, 9, (0, 3), (0, 6), (0, 9));

    let updated_code = "fn foobar() {}";
    assert!(engine.parse(updated_code));
    assert!(engine.has_tree());
}
