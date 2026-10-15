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
pub fn parse_outline(lines: &[String], lang: SyntaxLang) -> Vec<OutlineItem> {
    match lang {
        SyntaxLang::Markdown => parse_markdown_outline(lines),
        SyntaxLang::Rust => parse_rust_outline(lines),
        SyntaxLang::Python => parse_python_outline(lines),
        SyntaxLang::Cpp | SyntaxLang::JavaScript | SyntaxLang::Java => {
            parse_c_family_outline(lines)
        }
        SyntaxLang::Sql
        | SyntaxLang::PlSql
        | SyntaxLang::Bash
        | SyntaxLang::Html
        | SyntaxLang::Json
        | SyntaxLang::PlainText => parse_generic_outline(lines),
    }
}

/// Markdown の見出し (`#` 〜 `######`) を解析します。
fn parse_markdown_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if hash_count <= 6 {
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

/// Rust のシンボル (`fn`, `struct`, `enum`, `impl`, `trait`, `mod`) を解析します。
fn parse_rust_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
        let depth = (leading_spaces / 4).min(4);

        if trimmed.starts_with("pub fn ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
        {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Function,
            });
        } else if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Struct,
            });
        } else if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Enum,
            });
        } else if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Impl,
            });
        } else if trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ") {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Trait,
            });
        } else if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
            let sig = extract_signature(trimmed);
            items.push(OutlineItem {
                line_idx: idx,
                level: depth,
                title: sig,
                kind: OutlineKind::Module,
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
fn parse_generic_outline(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") || trimmed.starts_with("// ") {
            let clean = trimmed.trim_start_matches(['#', '/', ' ']);
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
}
