//! アウトライン解析モジュール（Markdown見出し、関数、構造体、クラス等）。
//! 文書やソースコードの階層構造を解析し、ジャンプ可能なツリー項目を提供します。

use crate::syntax::SyntaxLang;

/// アウトライン項目の種別。
///
/// 見出しレベルやシンボル（関数、構造体、クラス等）を分類します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::outline::OutlineKind;
///
/// let heading = OutlineKind::Heading(1);
/// assert_eq!(heading, OutlineKind::Heading(1));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineKind {
    /// Markdown 見出し（レベル 1..=6）。
    Heading(usize),
    /// 関数・メソッド。
    Function,
    /// 構造体定義。
    Struct,
    /// 列挙体（Enum）定義。
    Enum,
    /// クラス定義。
    Class,
    /// 実装ブロック（例: `impl Foo`）。
    Impl,
    /// トレイト・インターフェース。
    Trait,
    /// モジュール宣言。
    Module,
    /// 汎用セクションまたはコメント見出し。
    Section,
}

/// 解析されたアウトライン項目（シンボルまたは見出し）。
///
/// ソースコードまたはドキュメント内における行番号、階層インデントレベル、
/// タイトル文字列、シンボル種別を保持します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::outline::{OutlineItem, OutlineKind};
///
/// let item = OutlineItem {
///     line_idx: 0,
///     level: 0,
///     title: "Introduction".to_string(),
///     kind: OutlineKind::Heading(1),
/// };
/// assert_eq!(item.title, "Introduction");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineItem {
    /// バッファ内の0始まりの行番号。
    pub line_idx: usize,
    /// ツリーのインデント・ネスト階層（0 = ルート）。
    pub level: usize,
    /// 表示タイトル／シグネチャ。
    pub title: String,
    /// シンボル種別。
    pub kind: OutlineKind,
}

/// ドキュメントの各行を指定された構文言語に従ってアウトライン階層へ解析します。
///
/// Markdownの見出し構造（`#` 〜 `######`）、Rustの `fn`, `struct`, `impl`、
/// Pythonの `def`, `class` など、言語仕様に応じたシンボルツリーを生成します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::outline::{parse_outline, OutlineKind};
/// use vivivi_modes::syntax::SyntaxLang;
///
/// let doc = vec![
///     "# Main Section".to_string(),
///     "Some body text".to_string(),
///     "## Subsection".to_string(),
/// ];
/// let items = parse_outline(&doc, SyntaxLang::Markdown);
/// assert_eq!(items.len(), 2);
/// assert_eq!(items[0].title, "Main Section");
/// assert_eq!(items[0].kind, OutlineKind::Heading(1));
/// assert_eq!(items[1].title, "Subsection");
/// assert_eq!(items[1].kind, OutlineKind::Heading(2));
/// ```
/// ドキュメントの各行を指定された構文言語に従ってアウトライン階層へ解析します。
///
/// Markdownの見出し構造（`#` 〜 `######`）、Rustの `fn`, `struct`, `impl`、
/// Pythonの `def`, `class` など、言語仕様に応じたシンボルツリーを生成します。
/// 言語が `PlainText` の場合は、バッファ内容から言語を自動推測して解析を試みます。
///
/// # Examples
///
/// ```
/// use vivivi_modes::outline::{parse_outline, OutlineKind};
/// use vivivi_modes::syntax::SyntaxLang;
///
/// let doc = vec![
///     "# Main Section".to_string(),
///     "Some body text".to_string(),
///     "## Subsection".to_string(),
/// ];
/// let items = parse_outline(&doc, SyntaxLang::Markdown);
/// assert_eq!(items.len(), 2);
/// assert_eq!(items[0].title, "Main Section");
/// assert_eq!(items[0].kind, OutlineKind::Heading(1));
/// assert_eq!(items[1].title, "Subsection");
/// assert_eq!(items[1].kind, OutlineKind::Heading(2));
/// ```
pub fn parse_outline(lines: &[String], lang: SyntaxLang) -> Vec<OutlineItem> {
    match lang {
        SyntaxLang::Markdown => parse_markdown_outline(lines),
        SyntaxLang::Rust => parse_rust_outline(lines),
        SyntaxLang::Python => parse_python_outline(lines),
        SyntaxLang::Cpp | SyntaxLang::JavaScript | SyntaxLang::Java => {
            parse_c_family_outline(lines)
        }
        SyntaxLang::PlainText => {
            // 無名バッファ等の場合、内容から言語を自動推測
            if let Some(detected) = detect_language_from_content(lines) {
                match detected {
                    SyntaxLang::Markdown => parse_markdown_outline(lines),
                    SyntaxLang::Rust => parse_rust_outline(lines),
                    SyntaxLang::Python => parse_python_outline(lines),
                    SyntaxLang::Cpp | SyntaxLang::JavaScript | SyntaxLang::Java => {
                        parse_c_family_outline(lines)
                    }
                    _ => parse_generic_outline(lines),
                }
            } else {
                parse_generic_outline(lines)
            }
        }
        SyntaxLang::Sql
        | SyntaxLang::PlSql
        | SyntaxLang::Bash
        | SyntaxLang::Html
        | SyntaxLang::Json => parse_generic_outline(lines),
    }
}

/// バッファの内容（先頭数十行）からプログラミング言語または文書形式をヒューリスティックに推測します。
pub fn detect_language_from_content(lines: &[String]) -> Option<SyntaxLang> {
    let sample = lines.iter().take(50);
    let mut has_markdown_heading = false;
    let mut has_rust_kw = false;
    let mut has_python_kw = false;

    for line in sample {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let count = trimmed.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&count) && trimmed[count..].starts_with(' ') {
                has_markdown_heading = true;
            }
        }
        if trimmed.starts_with("use ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub(crate) fn ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("impl<")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("let mut ")
        {
            has_rust_kw = true;
        }
        if trimmed.starts_with("def ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
        {
            has_python_kw = true;
        }
    }

    if has_rust_kw {
        Some(SyntaxLang::Rust)
    } else if has_markdown_heading {
        Some(SyntaxLang::Markdown)
    } else if has_python_kw {
        Some(SyntaxLang::Python)
    } else {
        None
    }
}

/// Markdown の見出し (`#` 〜 `######`) を解析します。
fn parse_markdown_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&hash_count) {
                let rest = trimmed[hash_count..].trim();
                if !rest.is_empty() {
                    items.push(OutlineItem {
                        line_idx: idx,
                        level: hash_count.saturating_sub(1),
                        title: rest.to_string(),
                        kind: OutlineKind::Heading(hash_count),
                    });
                }
            }
        }
    }
    items
}

/// Rust のシンボル (`fn`, `struct`, `enum`, `impl`, `trait`, `mod`, `type`) を解析します。
///
/// `pub`, `pub(crate)`, `pub(super)`, `async`, `const`, `unsafe`, `extern "C"` などの
/// 各種可視性修飾子や実行修飾子を透過的に解析してシンボルを網羅抽出します。
fn parse_rust_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
        let depth = (leading_spaces / 4).min(4);

        // 修飾子（pub, pub(...), async, const, unsafe, extern "C" 等）を除去したコア宣言を抽出
        let mut rest = trimmed;

        // 可視性修飾子の除去
        if rest.starts_with("pub ") {
            rest = rest[4..].trim_start();
        } else if rest.starts_with("pub(")
            && let Some(close_paren) = rest.find(')')
        {
            rest = rest[close_paren + 1..].trim_start();
        }

        // 実行・性質修飾子の除去 (async, const, unsafe, extern 等)
        loop {
            if rest.starts_with("async ") || rest.starts_with("const ") {
                rest = rest[6..].trim_start();
            } else if rest.starts_with("unsafe ") || rest.starts_with("extern ") {
                rest = rest[7..].trim_start();
            } else if rest.starts_with("extern \"C\" ") {
                rest = rest[11..].trim_start();
            } else {
                break;
            }
        }

        if rest.starts_with("fn ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Function,
            });
        } else if rest.starts_with("struct ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Struct,
            });
        } else if rest.starts_with("enum ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Enum,
            });
        } else if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Impl,
            });
        } else if rest.starts_with("trait ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Trait,
            });
        } else if rest.starts_with("mod ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Module,
            });
        } else if rest.starts_with("type ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Struct,
            });
        } else if trimmed.starts_with("macro_rules! ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Function,
            });
        }
    }
    items
}

/// Python のシンボル (`def`, `class`) を解析します。
fn parse_python_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
        let depth = leading_spaces / 4;

        if trimmed.starts_with("def ") {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Function,
            });
        } else if trimmed.starts_with("class ") {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Class,
            });
        }
    }
    items
}

/// C/C++ および JavaScript スタイルの関数・クラスを解析します。
fn parse_c_family_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
        let depth = (leading_spaces / 4).min(4);

        if trimmed.starts_with("class ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Class,
            });
        } else if trimmed.starts_with("struct ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Struct,
            });
        } else if trimmed.starts_with("function ") || trimmed.starts_with("export function ") {
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: extract_signature(trimmed),
                kind: OutlineKind::Function,
            });
        }
    }
    items
}

/// 汎用テキストやコメントによる見出しセクションを解析します。
///
/// `#`, `##`, `###` などのMarkdown見出しや、`// ` によるコメント区切りを
/// 見出しレベルを保持したまま汎用的に抽出します。
fn parse_generic_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&hash_count) {
                let rest = trimmed[hash_count..].trim();
                if !rest.is_empty() {
                    items.push(OutlineItem {
                        line_idx: idx,
                        level: hash_count.saturating_sub(1),
                        title: rest.to_string(),
                        kind: OutlineKind::Heading(hash_count),
                    });
                }
            }
        } else if trimmed.starts_with("// ") {
            let clean = trimmed.trim_start_matches(['/', ' ']);
            if !clean.is_empty() {
                items.push(OutlineItem {
                    line_idx: idx,
                    level: 0,
                    title: clean.to_string(),
                    kind: OutlineKind::Section,
                });
            }
        }
    }
    items
}

/// 行末の波括弧、セミコロン、コロンを除去してシグネチャを整形します。
fn extract_signature(line: &str) -> String {
    let s = line
        .trim_end_matches('{')
        .trim_end_matches(':')
        .trim_end_matches(';')
        .trim();
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_outline() {
        let lines = vec![
            "# Title".to_string(),
            "Some text here".to_string(),
            "## Section 1".to_string(),
            "### Subsection 1.1".to_string(),
            "## Section 2".to_string(),
        ];
        let outline = parse_outline(&lines, SyntaxLang::Markdown);
        assert_eq!(outline.len(), 4);
        assert_eq!(outline[0].title, "Title");
        assert_eq!(outline[0].level, 0);
        assert_eq!(outline[1].title, "Section 1");
        assert_eq!(outline[1].level, 1);
        assert_eq!(outline[2].title, "Subsection 1.1");
        assert_eq!(outline[2].level, 2);
    }

    #[test]
    fn test_rust_outline() {
        let lines = vec![
            "pub struct MyEditor {".to_string(),
            "    name: String,".to_string(),
            "}".to_string(),
            "impl MyEditor {".to_string(),
            "    pub fn new() -> Self {".to_string(),
            "        Self { name: String::new() }".to_string(),
            "    }".to_string(),
            "}".to_string(),
        ];
        let outline = parse_outline(&lines, SyntaxLang::Rust);
        assert_eq!(outline.len(), 3);
        assert_eq!(outline[0].kind, OutlineKind::Struct);
        assert_eq!(outline[1].kind, OutlineKind::Impl);
        assert_eq!(outline[2].kind, OutlineKind::Function);
    }

    #[test]
    fn test_plaintext_markdown_auto_detect() {
        let lines = vec![
            "# Heading 1".to_string(),
            "Normal text".to_string(),
            "## Heading 2".to_string(),
            "### Heading 3".to_string(),
        ];
        // PlainText（無名バッファ等）であっても、内容推測または汎用フォールバックでH1〜H3が全階層抽出される
        let outline = parse_outline(&lines, SyntaxLang::PlainText);
        assert_eq!(outline.len(), 3);
        assert_eq!(outline[0].title, "Heading 1");
        assert_eq!(outline[0].level, 0);
        assert_eq!(outline[1].title, "Heading 2");
        assert_eq!(outline[1].level, 1);
        assert_eq!(outline[2].title, "Heading 3");
        assert_eq!(outline[2].level, 2);
    }

    #[test]
    fn test_rust_advanced_modifiers() {
        let lines = vec![
            "pub(crate) fn helper() {}".to_string(),
            "pub const fn max_val() -> usize { 10 }".to_string(),
            "unsafe fn raw_ptr() {}".to_string(),
            "pub type Result<T> = std::result::Result<T, MyError>;".to_string(),
            "macro_rules! my_macro { () => {}; }".to_string(),
        ];
        let outline = parse_outline(&lines, SyntaxLang::Rust);
        assert_eq!(outline.len(), 5);
        assert_eq!(outline[0].kind, OutlineKind::Function);
        assert_eq!(outline[1].kind, OutlineKind::Function);
        assert_eq!(outline[2].kind, OutlineKind::Function);
        assert_eq!(outline[3].kind, OutlineKind::Struct); // Type alias
        assert_eq!(outline[4].kind, OutlineKind::Function); // Macro
    }
}
