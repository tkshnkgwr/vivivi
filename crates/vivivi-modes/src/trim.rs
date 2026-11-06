//! 行末空白一括削除・トリミングモジュール (`:trim`, ViVi 4 準拠)。
//!
//! 各行の末尾に存在する不要な空白文字（半角スペース、タブ、全角スペース）を
//! 検出・一括削除する機能を提供します。

/// トリム対象の空白文字種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrimKind {
    /// 半角スペース (`' '`)、タブ (`'\t'`)、全角スペース (`'　'`, `\u{3000}`) のすべてを削除
    #[default]
    All,
    /// 半角スペースとタブ記号のみを削除（全角スペースは保持）
    Half,
    /// 全角スペース (`\u{3000}`) のみを削除
    Zenkaku,
}

impl TrimKind {
    /// 指定された文字がこのトリム対象であるかを判定します。
    #[inline]
    pub fn is_target_char(&self, ch: char) -> bool {
        match self {
            Self::All => ch == ' ' || ch == '\t' || ch == '\u{3000}',
            Self::Half => ch == ' ' || ch == '\t',
            Self::Zenkaku => ch == '\u{3000}',
        }
    }
}

/// 単一行の末尾から、指定されたトリム種別の空白文字を取り除いたスライスを返します。
///
/// ゼロコピーでスライスを返却するため高速です。
///
/// # Examples
///
/// ```
/// use vivivi_modes::trim::{trim_trailing_whitespace_line, TrimKind};
///
/// assert_eq!(trim_trailing_whitespace_line("hello   \t", TrimKind::All), "hello");
/// assert_eq!(trim_trailing_whitespace_line("hello　", TrimKind::All), "hello");
/// assert_eq!(trim_trailing_whitespace_line("hello　", TrimKind::Half), "hello　");
/// assert_eq!(trim_trailing_whitespace_line("hello 　", TrimKind::Zenkaku), "hello ");
/// ```
pub fn trim_trailing_whitespace_line(line: &str, kind: TrimKind) -> &str {
    line.trim_end_matches(|c| kind.is_target_char(c))
}

/// 複数行のテキスト配列に対して末尾空白トリミングを適用し、
/// 変更された行の（0始まり行インデックス, 変更後文字列）のリストを返します。
pub fn trim_lines(
    lines: &[String],
    start_line: usize,
    end_line: usize,
    kind: TrimKind,
) -> Vec<(usize, String)> {
    let mut modified = Vec::new();
    let end_limit = end_line.min(lines.len().saturating_sub(1));

    for (idx, line) in lines
        .iter()
        .enumerate()
        .take(end_limit + 1)
        .skip(start_line)
    {
        let trimmed = trim_trailing_whitespace_line(line, kind);
        if trimmed.len() != line.len() {
            modified.push((idx, trimmed.to_string()));
        }
    }

    modified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_trailing_whitespace_line_all() {
        assert_eq!(trim_trailing_whitespace_line("abc  ", TrimKind::All), "abc");
        assert_eq!(
            trim_trailing_whitespace_line("abc\t\t", TrimKind::All),
            "abc"
        );
        assert_eq!(trim_trailing_whitespace_line("abc　", TrimKind::All), "abc");
        assert_eq!(
            trim_trailing_whitespace_line("abc \t　  ", TrimKind::All),
            "abc"
        );
        assert_eq!(
            trim_trailing_whitespace_line("  abc", TrimKind::All),
            "  abc"
        );
        assert_eq!(trim_trailing_whitespace_line("   ", TrimKind::All), "");
        assert_eq!(trim_trailing_whitespace_line("", TrimKind::All), "");
    }

    #[test]
    fn test_trim_trailing_whitespace_line_half() {
        assert_eq!(
            trim_trailing_whitespace_line("abc  \t", TrimKind::Half),
            "abc"
        );
        assert_eq!(
            trim_trailing_whitespace_line("abc　", TrimKind::Half),
            "abc　"
        );
        assert_eq!(
            trim_trailing_whitespace_line("abc　 ", TrimKind::Half),
            "abc　"
        );
    }

    #[test]
    fn test_trim_trailing_whitespace_line_zenkaku() {
        assert_eq!(
            trim_trailing_whitespace_line("abc　　", TrimKind::Zenkaku),
            "abc"
        );
        assert_eq!(
            trim_trailing_whitespace_line("abc  ", TrimKind::Zenkaku),
            "abc  "
        );
        assert_eq!(
            trim_trailing_whitespace_line("abc 　", TrimKind::Zenkaku),
            "abc "
        );
    }

    #[test]
    fn test_trim_lines_range() {
        let lines = vec![
            "line 0  ".to_string(),
            "line 1".to_string(),
            "line 2　".to_string(),
            "line 3 \t".to_string(),
        ];

        let modified = trim_lines(&lines, 1, 2, TrimKind::All);
        assert_eq!(modified.len(), 1);
        assert_eq!(modified[0], (2, "line 2".to_string()));

        let modified_all = trim_lines(&lines, 0, 3, TrimKind::All);
        assert_eq!(modified_all.len(), 3);
        assert_eq!(modified_all[0], (0, "line 0".to_string()));
        assert_eq!(modified_all[1], (2, "line 2".to_string()));
        assert_eq!(modified_all[2], (3, "line 3".to_string()));
    }
}
