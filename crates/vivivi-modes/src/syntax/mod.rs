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
    /// hosts ファイル (`/etc/hosts`, `hosts`)
    Hosts,
    /// Cron / crontab ファイル (`crontab`, `*.cron`)
    Cron,
}

impl SyntaxLang {
    /// ファイルパスのファイル名または拡張子から対応言語を自動判別します。
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
    /// assert_eq!(SyntaxLang::from_path("/etc/hosts"), SyntaxLang::Hosts);
    /// assert_eq!(SyntaxLang::from_path("/etc/crontab"), SyntaxLang::Cron);
    /// assert_eq!(SyntaxLang::from_path("unknown.xyz"), SyntaxLang::PlainText);
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path_ref = path.as_ref();

        // 拡張子がない特殊なシステム設定ファイルをファイル名・親ディレクトリで判別
        if let Some(file_name) = path_ref
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_ascii_lowercase())
        {
            if file_name == "hosts" {
                return Self::Hosts;
            }
            if file_name == "crontab" || file_name.starts_with("crontab.") {
                return Self::Cron;
            }
        }

        if let Some(parent_name) = path_ref
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_ascii_lowercase())
            && parent_name == "cron.d"
        {
            return Self::Cron;
        }

        let ext = path_ref
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
            "hosts" => Self::Hosts,
            "cron" | "crontab" => Self::Cron,
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

    if lang == SyntaxLang::Hosts {
        return tokenize_hosts_line(line);
    }

    if lang == SyntaxLang::Cron {
        return tokenize_cron_line(line);
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
            let mut has_dot = false;
            while i < len {
                let c = chars[i];
                if c == '.' {
                    // 範囲演算子 (..) または次が数字でない場合は小数点とみなさない
                    if has_dot || i + 1 >= len || !chars[i + 1].is_ascii_digit() {
                        break;
                    }
                    has_dot = true;
                    i += 1;
                } else if c.is_ascii_alphanumeric() || c == '_' {
                    i += 1;
                } else {
                    break;
                }
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

/// hosts ファイル行専用トークナイザー。
///
/// IPv4/IPv6 アドレス（先頭フィールド）を [`TokenType::Number`]、
/// 正規ホスト名（2 番目フィールド）を [`TokenType::Keyword`]、
/// エイリアス（3 番目以降）を [`TokenType::Type`]、`#` 以降を [`TokenType::Comment`] に分類します。
fn tokenize_hosts_line(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut field_index = 0usize;

    while i < len {
        if chars[i].is_whitespace() {
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

        if chars[i] == '#' {
            tokens.push(SyntaxToken {
                text: chars[i..].iter().collect(),
                token_type: TokenType::Comment,
            });
            break;
        }

        let start = i;
        while i < len && !chars[i].is_whitespace() && chars[i] != '#' {
            i += 1;
        }
        let text: String = chars[start..i].iter().collect();
        let token_type = match field_index {
            0 => TokenType::Number,
            1 => TokenType::Keyword,
            _ => TokenType::Type,
        };
        tokens.push(SyntaxToken { text, token_type });
        field_index += 1;
    }

    tokens
}

/// Cron / crontab ファイル行専用トークナイザー。
///
/// コメント（`#`）、環境変数代入（`KEY=VALUE`）、特殊スケジュール指定（`@daily` 等）、
/// 5 フィールドの時刻式（分・時・日・月・曜日）、および実行コマンド部を色分けします。
fn tokenize_cron_line(line: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let len = line.len();
    let mut idx = 0;

    // 先頭の空白を保持
    while idx < len && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    if idx > 0 {
        tokens.push(SyntaxToken {
            text: line[..idx].to_string(),
            token_type: TokenType::Text,
        });
    }
    if idx >= len {
        return tokens;
    }

    // コメント行
    if bytes[idx] == b'#' {
        tokens.push(SyntaxToken {
            text: line[idx..].to_string(),
            token_type: TokenType::Comment,
        });
        return tokens;
    }

    let rest = &line[idx..];

    // 環境変数代入行 (例: MAILTO="admin@example.com", SHELL=/bin/bash)
    if let Some(eq_pos) = rest.find('=') {
        let key_part = &rest[..eq_pos];
        if !key_part.is_empty()
            && key_part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            tokens.push(SyntaxToken {
                text: key_part.to_string(),
                token_type: TokenType::Type,
            });
            tokens.push(SyntaxToken {
                text: "=".to_string(),
                token_type: TokenType::Punctuation,
            });
            let val_part = &rest[eq_pos + 1..];
            if !val_part.is_empty() {
                tokens.push(SyntaxToken {
                    text: val_part.to_string(),
                    token_type: TokenType::StringLiteral,
                });
            }
            return tokens;
        }
    }

    // 特殊スケジュール指定 (@reboot, @daily, @hourly 等)
    if rest.starts_with('@') {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        tokens.push(SyntaxToken {
            text: rest[..end].to_string(),
            token_type: TokenType::Keyword,
        });
        if end < rest.len() {
            tokens.extend(tokenize_line(&rest[end..], SyntaxLang::Bash));
        }
        return tokens;
    }

    // 標準 5 フィールドの時刻式 (分 時 日 月 曜日) + 実行コマンド部
    let mut pos = idx;
    let mut fields_parsed = 0usize;
    while pos < len && fields_parsed < 5 {
        let field_start = pos;
        while pos < len && !bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        let field_text = &line[field_start..pos];
        tokenize_cron_schedule_field(field_text, &mut tokens);
        fields_parsed += 1;

        let ws_start = pos;
        while pos < len && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos > ws_start {
            tokens.push(SyntaxToken {
                text: line[ws_start..pos].to_string(),
                token_type: TokenType::Text,
            });
        }
    }

    // 残りのコマンド部分は Bash トークナイザーで色分け
    if pos < len {
        tokens.extend(tokenize_line(&line[pos..], SyntaxLang::Bash));
    }

    tokens
}

/// Cron の時刻フィールド（例: `*/5`, `0-23`, `MON-FRI`）を数値/シンボルと区切り記号へ分割します。
fn tokenize_cron_schedule_field(field: &str, tokens: &mut Vec<SyntaxToken>) {
    let chars: Vec<char> = field.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        if ch.is_ascii_alphanumeric() {
            let start = i;
            while i < len && chars[i].is_ascii_alphanumeric() {
                i += 1;
            }
            tokens.push(SyntaxToken {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Number,
            });
        } else {
            tokens.push(SyntaxToken {
                text: ch.to_string(),
                token_type: TokenType::Punctuation,
            });
            i += 1;
        }
    }
}

/// 予約語判定
mod keywords;
pub(crate) use keywords::{is_keyword, is_type};

#[cfg(test)]
mod tests;
