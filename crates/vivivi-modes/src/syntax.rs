//! シンタックスハイライト（構文強調）トークナイザーモジュール。
//!
//! OpenViVi の `tokenizer.cpp` に基づき、各種プログラミング言語の予約語、型名、
//! 文字列リテラル、数値、コメント、記号を高速にトークナイズし、
//! TUI や GUI での構文色分け描画を可能にします。

use std::path::Path;

/// シンタックスハイライト対応言語。
///
/// ファイル拡張子に応じた自動判別をサポートします。
///
/// # Examples
///
/// ```
/// use vivivi_modes::syntax::SyntaxLang;
///
/// let lang = SyntaxLang::from_path("src/main.rs");
/// assert_eq!(lang, SyntaxLang::Rust);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyntaxLang {
    /// プレーンテキスト（ハイライトなし）
    #[default]
    PlainText,
    /// Rust
    Rust,
    /// C / C++
    Cpp,
    /// Python
    Python,
    /// JavaScript / TypeScript
    JavaScript,
    /// JSON
    Json,
    /// Markdown
    Markdown,
    /// SQL
    Sql,
    /// PL/SQL
    PlSql,
    /// Java
    Java,
    /// HTML
    Html,
    /// Bash / Shell
    Bash,
}

impl SyntaxLang {
    /// ファイルパスの拡張子から対応言語を自動判別します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::syntax::SyntaxLang;
    ///
    /// assert_eq!(SyntaxLang::from_path("index.html"), SyntaxLang::Html);
    /// assert_eq!(SyntaxLang::from_path("script.py"), SyntaxLang::Python);
    /// assert_eq!(SyntaxLang::from_path("unknown.xyz"), SyntaxLang::PlainText);
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match ext.as_str() {
            "rs" => Self::Rust,
            "cpp" | "cxx" | "cc" | "c" | "h" | "hpp" => Self::Cpp,
            "py" | "pyw" => Self::Python,
            "js" | "ts" | "jsx" | "tsx" => Self::JavaScript,
            "json" => Self::Json,
            "md" | "markdown" => Self::Markdown,
            "sql" => Self::Sql,
            "pls" | "plb" | "pks" | "pkb" => Self::PlSql,
            "java" => Self::Java,
            "html" | "htm" => Self::Html,
            "sh" | "bash" | "zsh" => Self::Bash,
            _ => Self::PlainText,
        }
    }
}

/// 構文トークン種別。
///
/// 予約語、型名、リテラル、コメント、記号、通常テキストを分類します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::syntax::TokenType;
///
/// let kw = TokenType::Keyword;
/// assert_eq!(kw, TokenType::Keyword);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// 予約語・キーワード (`fn`, `let`, `if`, `def`, `class`, etc.)
    Keyword,
    /// 組み込み型・プリミティブ型 (`i32`, `bool`, `String`, `int`, etc.)
    Type,
    /// 文字列リテラル (`"..."`, `'...'`)
    StringLiteral,
    /// 数値リテラル (`123`, `0xFF`, `3.14`)
    Number,
    /// コメント (`//...`, `/*...*/`, `#...`)
    Comment,
    /// 演算子・区切り記号 (`(`, `)`, `{`, `}`, `::`, `->`, `;`, etc.)
    Punctuation,
    /// 通常の識別子・テキスト
    Text,
}

/// ハイライト描画用の単一トークン。
///
/// トークンの文字列スライスと構文種別を保持します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::syntax::{SyntaxToken, TokenType};
///
/// let token = SyntaxToken {
///     text: "fn".to_string(),
///     token_type: TokenType::Keyword,
/// };
/// assert_eq!(token.token_type, TokenType::Keyword);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    /// トークン文字列
    pub text: String,
    /// トークン種別
    pub token_type: TokenType,
}

/// 行単位で構文を解析し、トークンのシーケンスを返します。
///
/// 言語規則に基づき、キーワードや文字列、コメント等を順次切り出して [`SyntaxToken`] の列を生成します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::syntax::{tokenize_line, SyntaxLang, TokenType};
///
/// let tokens = tokenize_line("let x = 42;", SyntaxLang::Rust);
/// assert!(tokens.iter().any(|t| t.text == "let" && t.token_type == TokenType::Keyword));
/// assert!(tokens.iter().any(|t| t.text == "42" && t.token_type == TokenType::Number));
/// ```
pub fn tokenize_line(line: &str, lang: SyntaxLang) -> Vec<SyntaxToken> {
    if lang == SyntaxLang::PlainText || line.is_empty() {
        return vec![SyntaxToken {
            text: line.to_string(),
            token_type: TokenType::Text,
        }];
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // 1. 空白スキップ
        if ch.is_whitespace() {
            let start = i;
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            tokens.push(SyntaxToken {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Text,
            });
            continue;
        }

        // 2. コメント判定
        // C/Java/Rust/JSスタイル行コメント: //
        if matches!(
            lang,
            SyntaxLang::Rust | SyntaxLang::Cpp | SyntaxLang::JavaScript | SyntaxLang::Java
        ) && ch == '/'
            && i + 1 < len
            && chars[i + 1] == '/'
        {
            tokens.push(SyntaxToken {
                text: chars[i..].iter().collect(),
                token_type: TokenType::Comment,
            });
            break;
        }

        // C/Java/Rust/JS/SQL/PL-SQL スタイルブロックコメント: /* ... */
        if matches!(
            lang,
            SyntaxLang::Rust
                | SyntaxLang::Cpp
                | SyntaxLang::JavaScript
                | SyntaxLang::Java
                | SyntaxLang::Sql
                | SyntaxLang::PlSql
        ) && ch == '/'
            && i + 1 < len
            && chars[i + 1] == '*'
        {
            let start = i;
            i += 2;
            let mut closed = false;
            while i + 1 < len {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                i = len;
            }
            tokens.push(SyntaxToken {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Comment,
            });
            continue;
        }

        // Python / Bash 行コメント: #
        if matches!(lang, SyntaxLang::Python | SyntaxLang::Bash) && ch == '#' {
            tokens.push(SyntaxToken {
                text: chars[i..].iter().collect(),
                token_type: TokenType::Comment,
            });
            break;
        }

        // SQL / PL/SQL 行コメント: --
        if matches!(lang, SyntaxLang::Sql | SyntaxLang::PlSql)
            && ch == '-'
            && i + 1 < len
            && chars[i + 1] == '-'
        {
            tokens.push(SyntaxToken {
                text: chars[i..].iter().collect(),
                token_type: TokenType::Comment,
            });
            break;
        }

        // HTML コメント: <!-- ... -->
        if lang == SyntaxLang::Html
            && ch == '<'
            && i + 3 < len
            && chars[i + 1] == '!'
            && chars[i + 2] == '-'
            && chars[i + 3] == '-'
        {
            let start = i;
            i += 4;
            let mut closed = false;
            while i + 2 < len {
                if chars[i] == '-' && chars[i + 1] == '-' && chars[i + 2] == '>' {
                    i += 3;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                i = len;
            }
            tokens.push(SyntaxToken {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Comment,
            });
            continue;
        }

        // 3. 文字列リテラル
        if ch == '"' || (ch == '\'' && lang != SyntaxLang::Rust) {
            let quote = ch;
            let start = i;
            i += 1;
            while i < len {
                if chars[i] == '\\' && i + 1 < len {
                    i += 2;
                    continue;
                }
                if chars[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            tokens.push(SyntaxToken {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::StringLiteral,
            });
            continue;
        }

        // 4. 数値リテラル
        if ch.is_ascii_digit() {
            let start = i;
            while i < len
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
            {
                i += 1;
            }
            tokens.push(SyntaxToken {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Number,
            });
            continue;
        }

        // 5. 識別子（キーワード、型名、変数名）
        if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let token_type = if is_keyword(&word, lang) {
                TokenType::Keyword
            } else if is_type(&word, lang) {
                TokenType::Type
            } else {
                TokenType::Text
            };

            tokens.push(SyntaxToken {
                text: word,
                token_type,
            });
            continue;
        }

        // 6. 記号・演算子
        let start = i;
        i += 1;
        tokens.push(SyntaxToken {
            text: chars[start..i].iter().collect(),
            token_type: TokenType::Punctuation,
        });
    }

    tokens
}

/// 予約語判定
fn is_keyword(word: &str, lang: SyntaxLang) -> bool {
    match lang {
        SyntaxLang::Rust => matches!(
            word,
            "as" | "async"
                | "await"
                | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
        ),
        SyntaxLang::Cpp => matches!(
            word,
            "auto"
                | "break"
                | "case"
                | "class"
                | "const"
                | "continue"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "enum"
                | "explicit"
                | "export"
                | "extern"
                | "false"
                | "for"
                | "friend"
                | "goto"
                | "if"
                | "inline"
                | "mutable"
                | "namespace"
                | "new"
                | "operator"
                | "private"
                | "protected"
                | "public"
                | "return"
                | "sizeof"
                | "static"
                | "struct"
                | "switch"
                | "template"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typedef"
                | "typeid"
                | "typename"
                | "using"
                | "virtual"
                | "while"
        ),
        SyntaxLang::Python => matches!(
            word,
            "and"
                | "as"
                | "assert"
                | "async"
                | "await"
                | "break"
                | "class"
                | "continue"
                | "def"
                | "del"
                | "elif"
                | "else"
                | "except"
                | "False"
                | "finally"
                | "for"
                | "from"
                | "global"
                | "if"
                | "import"
                | "in"
                | "is"
                | "lambda"
                | "None"
                | "nonlocal"
                | "not"
                | "or"
                | "pass"
                | "raise"
                | "return"
                | "True"
                | "try"
                | "while"
                | "with"
                | "yield"
        ),
        SyntaxLang::JavaScript => matches!(
            word,
            "async"
                | "await"
                | "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "let"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
                | "yield"
        ),
        SyntaxLang::Json => matches!(word, "true" | "false" | "null"),
        SyntaxLang::Sql => {
            let lower = word.to_ascii_lowercase();
            matches!(
                lower.as_str(),
                "select"
                    | "from"
                    | "where"
                    | "join"
                    | "inner"
                    | "left"
                    | "right"
                    | "outer"
                    | "cross"
                    | "full"
                    | "on"
                    | "group"
                    | "by"
                    | "having"
                    | "order"
                    | "asc"
                    | "desc"
                    | "insert"
                    | "into"
                    | "values"
                    | "update"
                    | "set"
                    | "delete"
                    | "create"
                    | "drop"
                    | "alter"
                    | "table"
                    | "view"
                    | "index"
                    | "union"
                    | "all"
                    | "distinct"
                    | "as"
                    | "and"
                    | "or"
                    | "not"
                    | "in"
                    | "is"
                    | "null"
                    | "like"
                    | "between"
                    | "exists"
                    | "case"
                    | "when"
                    | "then"
                    | "else"
                    | "end"
                    | "limit"
                    | "offset"
                    | "primary"
                    | "key"
                    | "foreign"
                    | "references"
                    | "check"
                    | "default"
                    | "unique"
                    | "constraint"
                    | "grant"
                    | "revoke"
                    | "commit"
                    | "rollback"
                    | "savepoint"
                    | "truncate"
                    | "with"
                    | "true"
                    | "false"
            )
        }
        SyntaxLang::PlSql => {
            let lower = word.to_ascii_lowercase();
            matches!(
                lower.as_str(),
                "select"
                    | "from"
                    | "where"
                    | "join"
                    | "inner"
                    | "left"
                    | "right"
                    | "outer"
                    | "cross"
                    | "full"
                    | "on"
                    | "group"
                    | "by"
                    | "having"
                    | "order"
                    | "asc"
                    | "desc"
                    | "insert"
                    | "into"
                    | "values"
                    | "update"
                    | "set"
                    | "delete"
                    | "create"
                    | "drop"
                    | "alter"
                    | "table"
                    | "view"
                    | "index"
                    | "union"
                    | "all"
                    | "distinct"
                    | "as"
                    | "and"
                    | "or"
                    | "not"
                    | "in"
                    | "is"
                    | "null"
                    | "like"
                    | "between"
                    | "exists"
                    | "case"
                    | "when"
                    | "then"
                    | "else"
                    | "end"
                    | "limit"
                    | "offset"
                    | "primary"
                    | "key"
                    | "foreign"
                    | "references"
                    | "check"
                    | "default"
                    | "unique"
                    | "constraint"
                    | "grant"
                    | "revoke"
                    | "commit"
                    | "rollback"
                    | "savepoint"
                    | "truncate"
                    | "with"
                    | "true"
                    | "false"
                    | "declare"
                    | "begin"
                    | "exception"
                    | "procedure"
                    | "function"
                    | "package"
                    | "body"
                    | "cursor"
                    | "loop"
                    | "while"
                    | "for"
                    | "exit"
                    | "elsif"
                    | "return"
                    | "pragma"
                    | "autonomous_transaction"
                    | "bulk"
                    | "collect"
                    | "forall"
                    | "execute"
                    | "immediate"
                    | "open"
                    | "fetch"
                    | "close"
                    | "raise"
                    | "record"
                    | "rowtype"
                    | "ref"
                    | "out"
                    | "nocopy"
                    | "constant"
                    | "authid"
                    | "replace"
                    | "pipe"
                    | "row"
                    | "others"
            )
        }
        SyntaxLang::Java => matches!(
            word,
            "abstract"
                | "assert"
                | "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "default"
                | "do"
                | "else"
                | "enum"
                | "extends"
                | "final"
                | "finally"
                | "for"
                | "goto"
                | "if"
                | "implements"
                | "import"
                | "instanceof"
                | "interface"
                | "native"
                | "new"
                | "package"
                | "private"
                | "protected"
                | "public"
                | "return"
                | "static"
                | "strictfp"
                | "super"
                | "switch"
                | "synchronized"
                | "this"
                | "throw"
                | "throws"
                | "transient"
                | "try"
                | "volatile"
                | "while"
                | "record"
                | "sealed"
                | "permits"
                | "var"
                | "yield"
                | "true"
                | "false"
                | "null"
        ),
        SyntaxLang::Html => {
            let lower = word.to_ascii_lowercase();
            matches!(
                lower.as_str(),
                "html"
                    | "head"
                    | "body"
                    | "title"
                    | "meta"
                    | "link"
                    | "style"
                    | "script"
                    | "div"
                    | "span"
                    | "p"
                    | "a"
                    | "img"
                    | "ul"
                    | "ol"
                    | "li"
                    | "table"
                    | "tr"
                    | "td"
                    | "th"
                    | "thead"
                    | "tbody"
                    | "tfoot"
                    | "form"
                    | "input"
                    | "button"
                    | "textarea"
                    | "select"
                    | "option"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "header"
                    | "footer"
                    | "nav"
                    | "section"
                    | "article"
                    | "main"
                    | "aside"
                    | "canvas"
                    | "svg"
                    | "video"
                    | "audio"
                    | "iframe"
                    | "label"
                    | "hr"
                    | "br"
                    | "doctype"
            )
        }
        SyntaxLang::Bash => matches!(
            word,
            "if" | "then"
                | "else"
                | "elif"
                | "fi"
                | "case"
                | "esac"
                | "for"
                | "select"
                | "while"
                | "until"
                | "do"
                | "done"
                | "in"
                | "function"
                | "time"
                | "return"
                | "exit"
                | "echo"
                | "read"
                | "export"
                | "set"
                | "unset"
                | "local"
                | "readonly"
                | "source"
                | "alias"
                | "unalias"
                | "shift"
                | "trap"
                | "eval"
                | "exec"
                | "test"
                | "true"
                | "false"
        ),
        _ => false,
    }
}

/// プリミティブ型判定
fn is_type(word: &str, lang: SyntaxLang) -> bool {
    match lang {
        SyntaxLang::Rust => matches!(
            word,
            "bool"
                | "char"
                | "str"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
                | "String"
                | "Option"
                | "Result"
                | "Vec"
                | "Box"
        ),
        SyntaxLang::Cpp => matches!(
            word,
            "int"
                | "char"
                | "bool"
                | "float"
                | "double"
                | "void"
                | "long"
                | "short"
                | "unsigned"
                | "signed"
                | "size_t"
                | "uint8_t"
                | "uint16_t"
                | "uint32_t"
                | "uint64_t"
                | "int8_t"
                | "int16_t"
                | "int32_t"
                | "int64_t"
        ),
        SyntaxLang::Sql => {
            let lower = word.to_ascii_lowercase();
            matches!(
                lower.as_str(),
                "int"
                    | "integer"
                    | "smallint"
                    | "bigint"
                    | "varchar"
                    | "char"
                    | "text"
                    | "boolean"
                    | "bool"
                    | "date"
                    | "time"
                    | "timestamp"
                    | "decimal"
                    | "numeric"
                    | "float"
                    | "double"
                    | "real"
                    | "blob"
                    | "clob"
                    | "serial"
            )
        }
        SyntaxLang::PlSql => {
            let lower = word.to_ascii_lowercase();
            matches!(
                lower.as_str(),
                "int"
                    | "integer"
                    | "smallint"
                    | "bigint"
                    | "varchar"
                    | "char"
                    | "text"
                    | "boolean"
                    | "bool"
                    | "date"
                    | "time"
                    | "timestamp"
                    | "decimal"
                    | "numeric"
                    | "float"
                    | "double"
                    | "real"
                    | "blob"
                    | "clob"
                    | "serial"
                    | "varchar2"
                    | "nvarchar2"
                    | "number"
                    | "pls_integer"
                    | "binary_integer"
                    | "sys_refcursor"
                    | "rowid"
                    | "long"
                    | "raw"
            )
        }
        SyntaxLang::Java => matches!(
            word,
            "boolean"
                | "byte"
                | "char"
                | "short"
                | "int"
                | "long"
                | "float"
                | "double"
                | "void"
                | "String"
                | "Integer"
                | "Long"
                | "Boolean"
                | "Double"
                | "Float"
                | "Byte"
                | "Short"
                | "Character"
                | "Object"
                | "Class"
                | "List"
                | "Map"
                | "Set"
                | "Optional"
                | "ArrayList"
                | "HashMap"
                | "HashSet"
                | "StringBuilder"
        ),
        SyntaxLang::Html => {
            let lower = word.to_ascii_lowercase();
            matches!(
                lower.as_str(),
                "class"
                    | "id"
                    | "href"
                    | "src"
                    | "style"
                    | "rel"
                    | "type"
                    | "name"
                    | "value"
                    | "placeholder"
                    | "action"
                    | "method"
                    | "target"
                    | "width"
                    | "height"
                    | "alt"
                    | "title"
                    | "charset"
                    | "content"
                    | "disabled"
                    | "readonly"
                    | "required"
                    | "checked"
                    | "selected"
            )
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
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
    }
}
