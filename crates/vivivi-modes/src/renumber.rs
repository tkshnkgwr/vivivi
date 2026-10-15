//! リナンバ (Renumbering) モジュール。
//!
//! ViVi 3 / ViVi 4 に準拠し、箇条書きリストや行頭の連番（`1.`, `2.`, `(1)`, `[1]` 等）を検出し、
//! 指定した開始番号から順に自動再ナンバリングします。

use regex::Regex;
use std::sync::LazyLock;

/// 連番パターンの正規表現
/// 1: `^\s*(\d+)([.)])\s+`  例: "  1. item", "2) item"
/// 2: `^\s*\((\d+)\)\s+`   例: " (1) item"
/// 3: `^\s*\[(\d+)\]\s+`   例: " [1] item"
static RENUM_PATTERN_DOT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(\d+)([.)])(\s+)").unwrap());
static RENUM_PATTERN_PAREN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)\((\d+)\)(\s+)").unwrap());
static RENUM_PATTERN_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)\[(\d+)\](\s+)").unwrap());

/// 複数行のリストテキスト内の連番を再付番します。
///
/// 箇条書き番号（`1.` 型、`2)` 型、`(1)` 型、`[1]` 型）を自動検出し、
/// 指定した開始番号と増分ステップで連番を再計算・更新します。
/// パターンに合致しない行はそのまま維持されます。
///
/// # Parameters
/// - `lines`: 対象行のスライス。
/// - `start`: 開始番号（通常は 1）。
/// - `step`: 増分ステップ（通常は 1）。
///
/// # Examples
///
/// ```
/// use vivivi_modes::renumber::renumber_lines;
///
/// let lines = vec![
///     "1. First item",
///     "5. Second item",
///     "3. Third item",
/// ];
/// let renumbered = renumber_lines(&lines, 1, 1);
/// assert_eq!(renumbered[0], "1. First item");
/// assert_eq!(renumbered[1], "2. Second item");
/// assert_eq!(renumbered[2], "3. Third item");
/// ```
pub fn renumber_lines(lines: &[&str], start: usize, step: usize) -> Vec<String> {
    let mut current_num = start;
    let mut result = Vec::with_capacity(lines.len());

    for &line in lines {
        if let Some(caps) = RENUM_PATTERN_DOT.captures(line) {
            let indent = &caps[1];
            let delim = &caps[3];
            let space = &caps[4];
            let rest = &line[caps[0].len()..];
            result.push(format!("{indent}{current_num}{delim}{space}{rest}"));
            current_num += step;
        } else if let Some(caps) = RENUM_PATTERN_PAREN.captures(line) {
            let indent = &caps[1];
            let space = &caps[3];
            let rest = &line[caps[0].len()..];
            result.push(format!("{indent}({current_num}){space}{rest}"));
            current_num += step;
        } else if let Some(caps) = RENUM_PATTERN_BRACKET.captures(line) {
            let indent = &caps[1];
            let space = &caps[3];
            let rest = &line[caps[0].len()..];
            result.push(format!("{indent}[{current_num}]{space}{rest}"));
            current_num += step;
        } else {
            result.push(line.to_string());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renumber_dot_list() {
        let lines = vec!["1. りんご", "1. みかん", "5. バナナ", "4. ぶどう"];
        let renumbered = renumber_lines(&lines, 1, 1);
        assert_eq!(
            renumbered,
            vec!["1. りんご", "2. みかん", "3. バナナ", "4. ぶどう",]
        );
    }

    #[test]
    fn test_renumber_with_indent_and_step() {
        let lines = vec!["  (1) ステップA", "  (1) ステップB", "  (9) ステップC"];
        let renumbered = renumber_lines(&lines, 10, 10);
        assert_eq!(
            renumbered,
            vec!["  (10) ステップA", "  (20) ステップB", "  (30) ステップC",]
        );
    }

    #[test]
    fn test_renumber_bracket_list() {
        let lines = vec!["[0] 準備", "[0] 実行", "[0] 確認"];
        let renumbered = renumber_lines(&lines, 1, 1);
        assert_eq!(renumbered, vec!["[1] 準備", "[2] 実行", "[3] 確認",]);
    }
}
