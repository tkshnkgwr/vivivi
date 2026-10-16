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
    /// Go
    Go,
    /// Ruby
    Ruby,
    /// TOML
    Toml,
    /// PowerShell
    PowerShell,
    /// YAML
    Yaml,
    /// XML
    Xml,
    /// Windows Batch / CMD
    Batch,
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
    /// assert_eq!(SyntaxLang::from_path("main.go"), SyntaxLang::Go);
    /// assert_eq!(SyntaxLang::from_path("app.rb"), SyntaxLang::Ruby);
    /// assert_eq!(SyntaxLang::from_path("Cargo.toml"), SyntaxLang::Toml);
    /// assert_eq!(SyntaxLang::from_path("deploy.ps1"), SyntaxLang::PowerShell);
    /// assert_eq!(SyntaxLang::from_path("config.yaml"), SyntaxLang::Yaml);
    /// assert_eq!(SyntaxLang::from_path("pom.xml"), SyntaxLang::Xml);
    /// assert_eq!(SyntaxLang::from_path("build.bat"), SyntaxLang::Batch);
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
            "go" => Self::Go,
            "rb" | "ruby" => Self::Ruby,
            "toml" => Self::Toml,
            "ps1" | "psm1" | "psd1" => Self::PowerShell,
            "yaml" | "yml" => Self::Yaml,
            "xml" | "xsd" | "xsl" | "svg" => Self::Xml,
            "bat" | "cmd" => Self::Batch,
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
        // C/Java/Rust/JS/Go スタイル行コメント: //
        if matches!(
            lang,
            SyntaxLang::Rust
                | SyntaxLang::Cpp
                | SyntaxLang::JavaScript
                | SyntaxLang::Java
                | SyntaxLang::Go
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

        // C/Java/Rust/JS/SQL/PL-SQL/Go スタイルブロックコメント: /* ... */
        if matches!(
            lang,
            SyntaxLang::Rust
                | SyntaxLang::Cpp
                | SyntaxLang::JavaScript
                | SyntaxLang::Java
                | SyntaxLang::Sql
                | SyntaxLang::PlSql
                | SyntaxLang::Go
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

        // Python / Bash / Ruby / Toml / PowerShell / Yaml 行コメント: #
        if matches!(
            lang,
            SyntaxLang::Python
                | SyntaxLang::Bash
                | SyntaxLang::Ruby
                | SyntaxLang::Toml
                | SyntaxLang::PowerShell
                | SyntaxLang::Yaml
        ) && ch == '#'
        {
            tokens.push(SyntaxToken {
                text: chars[i..].iter().collect(),
                token_type: TokenType::Comment,
            });
            break;
        }

        // PowerShell ブロックコメント: <# ... #>
        if lang == SyntaxLang::PowerShell && ch == '<' && i + 1 < len && chars[i + 1] == '#' {
            let start = i;
            i += 2;
            let mut closed = false;
            while i + 1 < len {
                if chars[i] == '#' && chars[i + 1] == '>' {
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

        // HTML / XML コメント: <!-- ... -->
        if matches!(lang, SyntaxLang::Html | SyntaxLang::Xml)
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

        // Windows Batch コメント: rem / REM / ::
        if lang == SyntaxLang::Batch {
            if ch == ':' && i + 1 < len && chars[i + 1] == ':' {
                tokens.push(SyntaxToken {
                    text: chars[i..].iter().collect(),
                    token_type: TokenType::Comment,
                });
                break;
            }
            if (ch == 'r' || ch == 'R')
                && i + 2 < len
                && (chars[i + 1] == 'e' || chars[i + 1] == 'E')
                && (chars[i + 2] == 'm' || chars[i + 2] == 'M')
                && (i + 3 == len || chars[i + 3].is_whitespace())
            {
                tokens.push(SyntaxToken {
                    text: chars[i..].iter().collect(),
                    token_type: TokenType::Comment,
                });
                break;
            }
        }

        // 3. 文字列リテラル
        // Go raw string: `...`
        if lang == SyntaxLang::Go && ch == '`' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '`' {
                i += 1;
            }
            if i < len && chars[i] == '`' {
                i += 1;
            }
            tokens.push(SyntaxToken {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::StringLiteral,
            });
            continue;
        }

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
mod keywords;
pub(crate) use keywords::{is_keyword, is_type};

#[cfg(test)]
mod tests;
