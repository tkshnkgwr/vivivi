use super::*;

#[test]
fn test_rust_tokenization() {
    let line = "pub fn add(a: i32) -> i32 { // comment\n";
    let tokens = tokenize_line(line, SyntaxLang::Rust);

    assert_eq!(
        tokens[0],
        SyntaxToken {
            text: "pub".to_string(),
            token_type: TokenType::Keyword
        }
    );
    assert_eq!(
        tokens[2],
        SyntaxToken {
            text: "fn".to_string(),
            token_type: TokenType::Keyword
        }
    );
    assert_eq!(
        tokens[9],
        SyntaxToken {
            text: "i32".to_string(),
            token_type: TokenType::Type
        }
    );
    // comment token
    let last_token = tokens.last().unwrap();
    assert_eq!(last_token.token_type, TokenType::Comment);
}

#[test]
fn test_python_tokenization() {
    let line = "def calculate(val = 42): # answer";
    let tokens = tokenize_line(line, SyntaxLang::Python);

    assert_eq!(tokens[0].token_type, TokenType::Keyword); // def
    assert_eq!(tokens[2].token_type, TokenType::Text); // calculate
    assert_eq!(tokens[8].token_type, TokenType::Number); // 42
    assert_eq!(tokens.last().unwrap().token_type, TokenType::Comment);
}

#[test]
fn test_sql_and_plsql_tokenization() {
    let sql = "SELECT id, name FROM users WHERE id = 1; -- find";
    let tokens: Vec<_> = tokenize_line(sql, SyntaxLang::Sql)
        .into_iter()
        .filter(|t| !t.text.trim().is_empty())
        .collect();
    assert_eq!(tokens[0].text, "SELECT");
    assert_eq!(tokens[0].token_type, TokenType::Keyword);
    assert_eq!(tokens[4].text, "FROM");
    assert_eq!(tokens[4].token_type, TokenType::Keyword);
    assert_eq!(tokens[6].text, "WHERE");
    assert_eq!(tokens[6].token_type, TokenType::Keyword);
    assert_eq!(tokens.last().unwrap().token_type, TokenType::Comment);

    let plsql = "PROCEDURE run(val IN NUMBER) IS BEGIN NULL; END;";
    let pl_tokens: Vec<_> = tokenize_line(plsql, SyntaxLang::PlSql)
        .into_iter()
        .filter(|t| !t.text.trim().is_empty())
        .collect();
    assert_eq!(pl_tokens[0].text, "PROCEDURE");
    assert_eq!(pl_tokens[0].token_type, TokenType::Keyword);
    assert_eq!(pl_tokens[5].text, "NUMBER");
    assert_eq!(pl_tokens[5].token_type, TokenType::Type);
    assert_eq!(pl_tokens[8].text, "BEGIN");
    assert_eq!(pl_tokens[8].token_type, TokenType::Keyword);
}

#[test]
fn test_java_tokenization() {
    let java = "public class Main { String msg = \"hi\"; } // test";
    let tokens: Vec<_> = tokenize_line(java, SyntaxLang::Java)
        .into_iter()
        .filter(|t| !t.text.trim().is_empty())
        .collect();
    assert_eq!(tokens[0].text, "public");
    assert_eq!(tokens[0].token_type, TokenType::Keyword);
    assert_eq!(tokens[1].text, "class");
    assert_eq!(tokens[1].token_type, TokenType::Keyword);
    assert_eq!(tokens[4].text, "String");
    assert_eq!(tokens[4].token_type, TokenType::Type);
    assert_eq!(tokens[7].text, "\"hi\"");
    assert_eq!(tokens[7].token_type, TokenType::StringLiteral);
    assert_eq!(tokens.last().unwrap().token_type, TokenType::Comment);
}

#[test]
fn test_html_and_bash_tokenization() {
    let html = "<div class=\"box\">hello</div> <!-- note -->";
    let tokens = tokenize_line(html, SyntaxLang::Html);
    assert_eq!(tokens[1].token_type, TokenType::Keyword); // div
    assert_eq!(tokens[3].token_type, TokenType::Type); // class
    assert_eq!(tokens[5].token_type, TokenType::StringLiteral); // "box"
    assert_eq!(tokens.last().unwrap().token_type, TokenType::Comment);

    let bash = "if [ $val -eq 0 ]; then echo \"yes\"; fi # check";
    let b_tokens = tokenize_line(bash, SyntaxLang::Bash);
    assert_eq!(b_tokens[0].token_type, TokenType::Keyword); // if
    assert_eq!(b_tokens.last().unwrap().token_type, TokenType::Comment);
}

#[test]
fn test_from_path_extensions() {
    assert_eq!(SyntaxLang::from_path("query.sql"), SyntaxLang::Sql);
    assert_eq!(SyntaxLang::from_path("pkg.pks"), SyntaxLang::PlSql);
    assert_eq!(SyntaxLang::from_path("pkg.pkb"), SyntaxLang::PlSql);
    assert_eq!(SyntaxLang::from_path("Main.java"), SyntaxLang::Java);
    assert_eq!(SyntaxLang::from_path("index.html"), SyntaxLang::Html);
    assert_eq!(SyntaxLang::from_path("index.htm"), SyntaxLang::Html);
    assert_eq!(SyntaxLang::from_path("script.sh"), SyntaxLang::Bash);
    assert_eq!(SyntaxLang::from_path("deploy.bash"), SyntaxLang::Bash);
    assert_eq!(SyntaxLang::from_path(".zshrc"), SyntaxLang::PlainText);
    assert_eq!(SyntaxLang::from_path("setup.zsh"), SyntaxLang::Bash);
    assert_eq!(SyntaxLang::from_path("main.go"), SyntaxLang::Go);
    assert_eq!(SyntaxLang::from_path("script.rb"), SyntaxLang::Ruby);
    assert_eq!(SyntaxLang::from_path("config.toml"), SyntaxLang::Toml);
    assert_eq!(SyntaxLang::from_path("deploy.ps1"), SyntaxLang::PowerShell);
    assert_eq!(SyntaxLang::from_path("data.yaml"), SyntaxLang::Yaml);
    assert_eq!(SyntaxLang::from_path("app.xml"), SyntaxLang::Xml);
    assert_eq!(SyntaxLang::from_path("run.bat"), SyntaxLang::Batch);
    assert_eq!(SyntaxLang::from_path("run.cmd"), SyntaxLang::Batch);
}

#[test]
fn test_new_languages_tokenization() {
    // Go
    let go_line = "func main() { var msg string = `hello`; } // go comment";
    let go_tokens = tokenize_line(go_line, SyntaxLang::Go);
    assert!(
        go_tokens
            .iter()
            .any(|t| t.text == "func" && t.token_type == TokenType::Keyword)
    );
    assert!(
        go_tokens
            .iter()
            .any(|t| t.text == "string" && t.token_type == TokenType::Type)
    );
    assert!(
        go_tokens
            .iter()
            .any(|t| t.token_type == TokenType::StringLiteral)
    );
    assert_eq!(go_tokens.last().unwrap().token_type, TokenType::Comment);

    // Ruby
    let rb_line = "def hello; return \"world\"; end # ruby comment";
    let rb_tokens = tokenize_line(rb_line, SyntaxLang::Ruby);
    assert!(
        rb_tokens
            .iter()
            .any(|t| t.text == "def" && t.token_type == TokenType::Keyword)
    );
    assert!(
        rb_tokens
            .iter()
            .any(|t| t.token_type == TokenType::StringLiteral)
    );
    assert_eq!(rb_tokens.last().unwrap().token_type, TokenType::Comment);

    // TOML
    let toml_line = "enabled = true # config";
    let toml_tokens = tokenize_line(toml_line, SyntaxLang::Toml);
    assert!(
        toml_tokens
            .iter()
            .any(|t| t.text == "true" && t.token_type == TokenType::Keyword)
    );
    assert_eq!(toml_tokens.last().unwrap().token_type, TokenType::Comment);

    // PowerShell
    let ps_line = "function Deploy { param([string]$name) return $true } # ps comment";
    let ps_tokens = tokenize_line(ps_line, SyntaxLang::PowerShell);
    assert!(
        ps_tokens
            .iter()
            .any(|t| t.text.eq_ignore_ascii_case("function") && t.token_type == TokenType::Keyword)
    );
    assert!(
        ps_tokens
            .iter()
            .any(|t| t.text.eq_ignore_ascii_case("string") && t.token_type == TokenType::Type)
    );
    assert_eq!(ps_tokens.last().unwrap().token_type, TokenType::Comment);

    // Batch
    let bat_line = "echo Hello World & set var=1 :: batch comment";
    let bat_tokens = tokenize_line(bat_line, SyntaxLang::Batch);
    assert!(
        bat_tokens
            .iter()
            .any(|t| t.text.eq_ignore_ascii_case("echo") && t.token_type == TokenType::Keyword)
    );
    assert!(
        bat_tokens
            .iter()
            .any(|t| t.text.eq_ignore_ascii_case("set") && t.token_type == TokenType::Keyword)
    );
    assert_eq!(bat_tokens.last().unwrap().token_type, TokenType::Comment);
}
