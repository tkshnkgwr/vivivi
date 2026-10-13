//! シンタックスハイライト（構文強調）トークナイザーモジュール。
//!
//! OpenViVi の `tokenizer.cpp` に基づき、各種プログラミング言語の予約語、型名、
//! 文字列リテラル、数値、コメント、記号を高速にトークナイズし、
//! TUI や GUI での構文色分け描画を可能にします。

use std::path::Path;

/// シンタックスハイライト対応言語。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyntaxLang {
    #[default]
    PlainText,
    Rust,
    Cpp,
    Python,
    JavaScript,
    Json,
    Markdown,
}

impl SyntaxLang {
    /// ファイルパスの拡張子から対応言語を自動判別します。
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
            _ => Self::PlainText,
        }
    }
}

/// 構文トークン種別。
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    pub text: String,
    pub token_type: TokenType,
}

/// 行単位で構文を解析し、トークンのシーケンスを返します。
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
        // C-style line comment: //
        if (lang == SyntaxLang::Rust || lang == SyntaxLang::Cpp || lang == SyntaxLang::JavaScript)
            && ch == '/'
            && i + 1 < len
            && chars[i + 1] == '/'
        {
            tokens.push(SyntaxToken {
                text: chars[i..].iter().collect(),
                token_type: TokenType::Comment,
            });
            break;
        }

        // Python / Shell line comment: #
        if lang == SyntaxLang::Python && ch == '#' {
            tokens.push(SyntaxToken {
                text: chars[i..].iter().collect(),
                token_type: TokenType::Comment,
            });
            break;
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
}
