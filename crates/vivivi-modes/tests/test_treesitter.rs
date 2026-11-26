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

#[test]
fn test_treesitter_highlight_syntax_tokens_preserves_alignment_and_comments() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Rust));

    let rust_code = concat!(
        "#[test]\n",
        "fn test_count_and_motion() {\n",
        "    let mut parser = CommandParser::new();\n",
        "\n",
        "    // 3j -> Move(Down(3)) 日本語コメント\n",
        "    assert_eq!(parser.feed_char('3', ViMode::Normal), None);\n",
        "    assert_eq!(\n",
        "        parser.feed_char('j', ViMode::Normal),\n",
        "        Some(ViCommand::Move(Motion::Down(3)))\n",
        "    ); // inline comment\n",
        "    /* block comment */\n",
        "}\n",
    );
    assert!(engine.parse(rust_code));

    let lines: Vec<&str> = rust_code.lines().collect();

    // 各行で連結テキストが元の行と完全一致し、位置ズレが一切起きないこと
    for (idx, line) in lines.iter().enumerate() {
        let tokens = engine.highlight_syntax_tokens(idx, line);
        if !tokens.is_empty() {
            let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
            assert_eq!(
                reconstructed, *line,
                "行 {idx} のトークン連結結果が元の行と一致すること"
            );
        }
    }

    // 4行目: 単一行コメント ("    // 3j -> Move(Down(3)) 日本語コメント")
    let comment_tokens = engine.highlight_syntax_tokens(4, lines[4]);
    assert!(
        comment_tokens
            .iter()
            .any(|t| t.token_type == TokenType::Comment && t.text.starts_with("// 3j")),
        "単一行コメントが Comment トークンとして認識されること: {comment_tokens:?}"
    );

    // 5行目: "    assert_eq!(parser.feed_char('3', ViMode::Normal), None);"
    // '3' の文字リテラル部分が正確に StringLiteral で、feed_char はズレていないこと
    let line5_tokens = engine.highlight_syntax_tokens(5, lines[5]);
    assert!(
        line5_tokens
            .iter()
            .any(|t| t.token_type == TokenType::StringLiteral && t.text == "'3'"),
        "'3' が StringLiteral として位置ズレなく抽出されること: {line5_tokens:?}"
    );
    assert!(
        !line5_tokens
            .iter()
            .any(|t| t.token_type == TokenType::StringLiteral && t.text.contains("har")),
        "feed_char の文字が StringLiteral に誤色付けされないこと: {line5_tokens:?}"
    );

    // 9行目: "    ); // inline comment"
    let inline_comment_tokens = engine.highlight_syntax_tokens(9, lines[9]);
    assert!(
        inline_comment_tokens
            .iter()
            .any(|t| t.token_type == TokenType::Comment && t.text == "// inline comment"),
        "コードと同一行のインラインコメントも Comment になること: {inline_comment_tokens:?}"
    );

    // 10行目: "    /* block comment */"
    let block_comment_tokens = engine.highlight_syntax_tokens(10, lines[10]);
    assert!(
        block_comment_tokens
            .iter()
            .any(|t| t.token_type == TokenType::Comment && t.text == "/* block comment */"),
        "ブロックコメントも Comment になること: {block_comment_tokens:?}"
    );
}

#[test]
fn test_treesitter_python_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Python));

    let code = "# Python comment\ndef greet(name):\n    msg = \"hello\"\n    return 42\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Comment));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "def" && t.token_type == TokenType::Keyword)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text.contains("hello") && t.token_type == TokenType::StringLiteral)
    );

    let t3 = engine.highlight_syntax_tokens(3, lines[3]);
    assert!(
        t3.iter()
            .any(|t| t.text == "return" && t.token_type == TokenType::Keyword)
    );
    assert!(
        t3.iter()
            .any(|t| t.text == "42" && t.token_type == TokenType::Number)
    );
}

#[test]
fn test_treesitter_bash_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Bash));

    let code = "#!/bin/bash\nif [ \"$USER\" = \"root\" ]; then\n    echo \"hi\"\nfi\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Comment));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "if" && t.token_type == TokenType::Keyword)
    );
    assert!(
        t1.iter()
            .any(|t| t.text == "then" && t.token_type == TokenType::Keyword)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text.contains("hi") && t.token_type == TokenType::StringLiteral)
    );
}

#[test]
fn test_treesitter_sql_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Sql));

    let code =
        "-- fetch users\nSELECT id, name FROM users WHERE age >= 20 AND status = 'active';\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Comment));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "SELECT" && t.token_type == TokenType::Keyword)
    );
    assert!(
        t1.iter()
            .any(|t| t.text == "FROM" && t.token_type == TokenType::Keyword)
    );
    assert!(
        t1.iter()
            .any(|t| t.text == "20" && t.token_type == TokenType::Number)
    );
    assert!(
        t1.iter()
            .any(|t| t.text == "'active'" && t.token_type == TokenType::StringLiteral)
    );
}

#[test]
fn test_treesitter_markdown_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Markdown));

    let code = "# Heading\n\n- item 1\n\n```rust\nlet x = 1;\n```\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(
        t0.iter()
            .any(|t| t.text.contains("Heading") && t.token_type == TokenType::Keyword)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(t2.iter().any(|t| t.token_type == TokenType::Punctuation));

    let t5 = engine.highlight_syntax_tokens(5, lines[5]);
    assert!(
        t5.iter()
            .any(|t| t.text.contains("let x = 1;") && t.token_type == TokenType::StringLiteral)
    );
}

#[test]
fn test_treesitter_powershell_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::PowerShell));

    let code = "# PS comment\nfunction Get-Greeting {\n    $msg = \"hello\"\n    if ($x -eq 10) { return $msg }\n}\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Comment));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "function" && t.token_type == TokenType::Keyword)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text == "$msg" && t.token_type == TokenType::Type)
    );
    assert!(
        t2.iter()
            .any(|t| t.text.contains("hello") && t.token_type == TokenType::StringLiteral)
    );

    let t3 = engine.highlight_syntax_tokens(3, lines[3]);
    assert!(
        t3.iter()
            .any(|t| t.text == "if" && t.token_type == TokenType::Keyword)
    );
    assert!(
        t3.iter()
            .any(|t| t.text == "10" && t.token_type == TokenType::Number)
    );
}

#[test]
fn test_treesitter_batch_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Batch));

    let code = "@echo off\nREM batch comment\nset NAME=vivivi\n:start\necho \"%NAME%\"\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Keyword));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(t1.iter().any(|t| t.token_type == TokenType::Comment));

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text.eq_ignore_ascii_case("set") && t.token_type == TokenType::Keyword)
    );

    let t3 = engine.highlight_syntax_tokens(3, lines[3]);
    assert!(
        t3.iter()
            .any(|t| t.text.contains("start") && t.token_type == TokenType::Type)
    );

    let t4 = engine.highlight_syntax_tokens(4, lines[4]);
    assert!(t4.iter().any(|t| t.token_type == TokenType::StringLiteral));
}

#[test]
fn test_treesitter_toml_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Toml));

    let code = "# config\n[package]\nname = \"vivivi\"\nport = 8080\nenabled = true\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Comment));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "package" && t.token_type == TokenType::Type)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text == "name" && t.token_type == TokenType::Type)
    );
    assert!(
        t2.iter()
            .any(|t| t.text == "\"vivivi\"" && t.token_type == TokenType::StringLiteral)
    );

    let t3 = engine.highlight_syntax_tokens(3, lines[3]);
    assert!(
        t3.iter()
            .any(|t| t.text == "8080" && t.token_type == TokenType::Number)
    );

    let t4 = engine.highlight_syntax_tokens(4, lines[4]);
    assert!(
        t4.iter()
            .any(|t| t.text == "true" && t.token_type == TokenType::Keyword)
    );
}

#[test]
fn test_treesitter_json_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Json));

    let code = "{\n  \"name\": \"vivivi\",\n  \"count\": 42,\n  \"active\": true\n}\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "\"name\"" && t.token_type == TokenType::Type)
    );
    assert!(
        t1.iter()
            .any(|t| t.text == "\"vivivi\"" && t.token_type == TokenType::StringLiteral)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text == "42" && t.token_type == TokenType::Number)
    );

    let t3 = engine.highlight_syntax_tokens(3, lines[3]);
    assert!(
        t3.iter()
            .any(|t| t.text == "true" && t.token_type == TokenType::Keyword)
    );
}

#[test]
fn test_treesitter_yaml_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::Yaml));

    let code = "# yaml config\nname: vivivi\nport: 8080\nenabled: true\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Comment));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "name" && t.token_type == TokenType::Type)
    );
    assert!(
        t1.iter()
            .any(|t| t.text == "vivivi" && t.token_type == TokenType::StringLiteral)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text == "8080" && t.token_type == TokenType::Number)
    );

    let t3 = engine.highlight_syntax_tokens(3, lines[3]);
    assert!(
        t3.iter()
            .any(|t| t.text == "true" && t.token_type == TokenType::Keyword)
    );
}

#[test]
fn test_treesitter_javascript_highlight() {
    let mut engine = TreeSitterEngine::new();
    assert!(engine.set_language(SyntaxLang::JavaScript));

    let code =
        "// js comment\nconst greet = (name) => {\n  return \"hello\";\n};\nconst n = 123;\n";
    assert!(engine.parse(code));
    assert!(engine.has_tree());

    let lines: Vec<&str> = code.lines().collect();
    let t0 = engine.highlight_syntax_tokens(0, lines[0]);
    assert!(t0.iter().any(|t| t.token_type == TokenType::Comment));

    let t1 = engine.highlight_syntax_tokens(1, lines[1]);
    assert!(
        t1.iter()
            .any(|t| t.text == "const" && t.token_type == TokenType::Keyword)
    );

    let t2 = engine.highlight_syntax_tokens(2, lines[2]);
    assert!(
        t2.iter()
            .any(|t| t.text == "return" && t.token_type == TokenType::Keyword)
    );
    assert!(
        t2.iter()
            .any(|t| t.text == "\"hello\"" && t.token_type == TokenType::StringLiteral)
    );

    let t4 = engine.highlight_syntax_tokens(4, lines[4]);
    assert!(
        t4.iter()
            .any(|t| t.text == "123" && t.token_type == TokenType::Number)
    );
}

#[test]
fn test_treesitter_cpp_java_go_ruby_html_xml_plsql_and_incremental() {
    let mut engine = TreeSitterEngine::new();

    // C / C++
    assert!(engine.set_language(SyntaxLang::Cpp));
    let cpp = "// cpp\nint add(int a, int b) {\n    return a + 1;\n}\n";
    assert!(engine.parse(cpp));
    let cpp_t1 = engine.highlight_syntax_tokens(1, "int add(int a, int b) {");
    assert!(
        cpp_t1
            .iter()
            .any(|t| t.text == "int" && t.token_type == TokenType::Type)
    );
    let cpp_t2 = engine.highlight_syntax_tokens(2, "    return a + 1;");
    assert!(
        cpp_t2
            .iter()
            .any(|t| t.text == "return" && t.token_type == TokenType::Keyword)
    );
    assert!(
        cpp_t2
            .iter()
            .any(|t| t.text == "1" && t.token_type == TokenType::Number)
    );

    // Java
    assert!(engine.set_language(SyntaxLang::Java));
    let java = "public class Main {\n    String s = \"ok\";\n}\n";
    assert!(engine.parse(java));
    let java_t0 = engine.highlight_syntax_tokens(0, "public class Main {");
    assert!(
        java_t0
            .iter()
            .any(|t| t.text == "public" && t.token_type == TokenType::Keyword)
    );
    assert!(
        java_t0
            .iter()
            .any(|t| t.text == "Main" && t.token_type == TokenType::Type)
    );
    let java_t1 = engine.highlight_syntax_tokens(1, "    String s = \"ok\";");
    assert!(
        java_t1
            .iter()
            .any(|t| t.text == "\"ok\"" && t.token_type == TokenType::StringLiteral)
    );

    // Go
    assert!(engine.set_language(SyntaxLang::Go));
    let go = "package main\nfunc run() int {\n    return 7\n}\n";
    assert!(engine.parse(go));
    let go_t1 = engine.highlight_syntax_tokens(1, "func run() int {");
    assert!(
        go_t1
            .iter()
            .any(|t| t.text == "func" && t.token_type == TokenType::Keyword)
    );
    assert!(
        go_t1
            .iter()
            .any(|t| t.text == "int" && t.token_type == TokenType::Type)
    );

    // Ruby
    assert!(engine.set_language(SyntaxLang::Ruby));
    let rb = "# rb\ndef greet\n  \"hi\"\nend\n";
    assert!(engine.parse(rb));
    let rb_t1 = engine.highlight_syntax_tokens(1, "def greet");
    assert!(
        rb_t1
            .iter()
            .any(|t| t.text == "def" && t.token_type == TokenType::Keyword)
    );
    let rb_t2 = engine.highlight_syntax_tokens(2, "  \"hi\"");
    assert!(
        rb_t2
            .iter()
            .any(|t| t.text == "\"hi\"" && t.token_type == TokenType::StringLiteral)
    );

    // HTML
    assert!(engine.set_language(SyntaxLang::Html));
    let html = "<div class=\"box\">hello</div>";
    assert!(engine.parse(html));
    let html_t0 = engine.highlight_syntax_tokens(0, html);
    assert!(
        html_t0
            .iter()
            .any(|t| t.text == "div" && t.token_type == TokenType::Keyword)
    );
    assert!(
        html_t0
            .iter()
            .any(|t| t.text == "class" && t.token_type == TokenType::Type)
    );
    assert!(
        html_t0
            .iter()
            .any(|t| t.text == "\"box\"" && t.token_type == TokenType::StringLiteral)
    );

    // XML
    assert!(engine.set_language(SyntaxLang::Xml));
    let xml = "<config id=\"1\"><!-- note --></config>";
    assert!(engine.parse(xml));
    let xml_t0 = engine.highlight_syntax_tokens(0, xml);
    assert!(
        xml_t0
            .iter()
            .any(|t| t.text == "config" && t.token_type == TokenType::Keyword)
    );
    assert!(
        xml_t0
            .iter()
            .any(|t| t.text == "<!-- note -->" && t.token_type == TokenType::Comment)
    );

    // PL/SQL
    assert!(engine.set_language(SyntaxLang::PlSql));
    let plsql = "SELECT id FROM users;";
    assert!(engine.parse(plsql));
    let plsql_t0 = engine.highlight_syntax_tokens(0, plsql);
    assert!(
        plsql_t0
            .iter()
            .any(|t| t.text == "SELECT" && t.token_type == TokenType::Keyword)
    );

    // parse_incremental 日本語含む差分更新
    assert!(engine.set_language(SyntaxLang::Rust));
    assert!(engine.parse("fn main() {\n    let msg = \"こんにちは\";\n}\n"));
    assert!(
        engine.parse_incremental(
            "fn main() {\n    let msg = \"こんにちは世界\";\n    let n = 99;\n}\n"
        )
    );
    let inc_t2 = engine.highlight_syntax_tokens(2, "    let n = 99;");
    assert!(
        inc_t2
            .iter()
            .any(|t| t.text == "let" && t.token_type == TokenType::Keyword)
    );
    assert!(
        inc_t2
            .iter()
            .any(|t| t.text == "99" && t.token_type == TokenType::Number)
    );
}
