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
    use std::collections::HashMap;
    let mut counters: HashMap<(usize, u8), usize> = HashMap::new();
    let mut result = Vec::with_capacity(lines.len());

    for &line in lines {
        let (indent, delim_info, rest, pat_type) =
            if let Some(caps) = RENUM_PATTERN_DOT.captures(line) {
                let indent = caps[1].to_string();
                let delim = caps[3].to_string();
                let space = caps[4].to_string();
                let rest = line[caps[0].len()..].to_string();
                (indent, format!("{delim}{space}"), rest, 0)
            } else if let Some(caps) = RENUM_PATTERN_PAREN.captures(line) {
                let indent = caps[1].to_string();
                let space = caps[3].to_string();
                let rest = line[caps[0].len()..].to_string();
                (indent, space, rest, 1)
            } else if let Some(caps) = RENUM_PATTERN_BRACKET.captures(line) {
                let indent = caps[1].to_string();
                let space = caps[3].to_string();
                let rest = line[caps[0].len()..].to_string();
                (indent, space, rest, 2)
            } else {
                result.push(line.to_string());
                continue;
            };

        let indent_len = indent.len();

        // 自分より深い階層のカウンタはリセット（親項目が変わったため）
        counters.retain(|&(lvl, _), _| lvl <= indent_len);

        let num = counters.entry((indent_len, pat_type)).or_insert(start);
        let cur_val = *num;
        *num += step;

        match pat_type {
            0 => result.push(format!("{indent}{cur_val}{delim_info}{rest}")),
            1 => result.push(format!("{indent}({cur_val}){delim_info}{rest}")),
            2 => result.push(format!("{indent}[{cur_val}]{delim_info}{rest}")),
            _ => unreachable!(),
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

    #[test]
    fn test_renumber_nested_list() {
        let lines = vec![
            "1. 親A",
            "    1. 子1",
            "    5. 子2",
            "1. 親B",
            "    9. 子1",
            "    2. 子2",
        ];
        let renumbered = renumber_lines(&lines, 1, 1);
        assert_eq!(
            renumbered,
            vec![
                "1. 親A",
                "    1. 子1",
                "    2. 子2",
                "2. 親B",
                "    1. 子1",
                "    2. 子2",
            ]
        );
    }
}
