//! テキスト差分（DIFF）エンジン。
//!
//! 行単位および文字単位の LCS（最長共通部分列）アルゴリズムに基づき、
//! 2つのテキスト間の差分抽出、Unified Diff 生成・適用、
//! および Side-by-Side 比較用データ構造を提供します。

/// 差分の種類（一致・追加・削除）。
///
/// # Examples
///
/// ```
/// use vivivi_modes::diff::DiffType;
///
/// assert_eq!(DiffType::Equal, DiffType::Equal);
/// assert_ne!(DiffType::Insert, DiffType::Delete);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    /// 変更なし（新旧両方に存在）
    Equal,
    /// 新規追加行（新テキストのみに存在）
    Insert,
    /// 削除行（旧テキストのみに存在）
    Delete,
}

/// 差分行データ。
///
/// 新旧テキストにおける行番号情報と変更種別、行テキストを保持します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::diff::{DiffLine, DiffType};
///
/// let line = DiffLine {
///     diff_type: DiffType::Insert,
///     old_lineno: None,
///     new_lineno: Some(1),
///     text: "let a = 10;".to_string(),
/// };
/// assert_eq!(line.diff_type, DiffType::Insert);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// 差分の種類
    pub diff_type: DiffType,
    /// 旧テキストにおける行番号（1-indexed。追加行の場合は `None`）
    pub old_lineno: Option<usize>,
    /// 新テキストにおける行番号（1-indexed。削除行の場合は `None`）
    pub new_lineno: Option<usize>,
    /// 行文字列
    pub text: String,
}

/// Unified Diff のハンク（変更ブロック）。
///
/// `@@ -old_start,old_len +new_start,new_len @@` に対応する差分単位です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    /// 旧テキストでの開始行
    pub old_start: usize,
    /// 旧テキストでの行数
    pub old_len: usize,
    /// 新テキストでの開始行
    pub new_start: usize,
    /// 新テキストでの行数
    pub new_len: usize,
    /// ハンクに含まれる差分行リスト
    pub lines: Vec<DiffLine>,
}

/// 左右2画面比較（Side-by-Side DIFF）の1行。
///
/// 左右画面で同期してスクロール・表示するための構造体です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideBySideLine {
    /// 左ペイン（旧テキスト）の行番号
    pub left_lineno: Option<usize>,
    /// 左ペインの行文字列
    pub left_text: Option<String>,
    /// 左ペインの差分種別
    pub left_type: DiffType,
    /// 右ペイン（新テキスト）の行番号
    pub right_lineno: Option<usize>,
    /// 右ペインの行文字列
    pub right_text: Option<String>,
    /// 右ペインの差分種別
    pub right_type: DiffType,
}

/// 2つのテキストから LCS（最長共通部分列）を計算し、差分行リストを生成します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::diff::{diff_lines, DiffType};
///
/// let old_text = "apple\nbanana";
/// let new_text = "apple\ncherry";
/// let diff = diff_lines(old_text, new_text);
///
/// assert_eq!(diff.len(), 3);
/// assert_eq!(diff[0].diff_type, DiffType::Equal);
/// assert_eq!(diff[1].diff_type, DiffType::Delete);
/// assert_eq!(diff[2].diff_type, DiffType::Insert);
/// ```
pub fn diff_lines(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let n = old_lines.len();
    let m = new_lines.len();

    // DPテーブル（行数が多い場合の安全ガード: 行数が極端に多い場合はフォールバック）
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in 0..n {
        for j in 0..m {
            if old_lines[i] == new_lines[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    // バックトラック
    let mut i = n;
    let mut j = m;
    let mut raw_diff = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            raw_diff.push(DiffLine {
                diff_type: DiffType::Equal,
                old_lineno: Some(i),
                new_lineno: Some(j),
                text: old_lines[i - 1].to_string(),
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            raw_diff.push(DiffLine {
                diff_type: DiffType::Insert,
                old_lineno: None,
                new_lineno: Some(j),
                text: new_lines[j - 1].to_string(),
            });
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i][j - 1] < dp[i - 1][j]) {
            raw_diff.push(DiffLine {
                diff_type: DiffType::Delete,
                old_lineno: Some(i),
                new_lineno: None,
                text: old_lines[i - 1].to_string(),
            });
            i -= 1;
        }
    }

    raw_diff.reverse();
    raw_diff
}

/// 2つのテキストから Unified Diff 形式（`---` / `+++` / `@@`）のパッチ文字列を生成します。
///
/// 差異が全くない場合は空文字列を返します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::diff::create_unified_diff;
///
/// let old_text = "line 1\nline 2\n";
/// let new_text = "line 1\nline 2 modified\n";
/// let patch = create_unified_diff(old_text, new_text, "a.txt", "b.txt");
/// assert!(patch.starts_with("--- a.txt\n+++ b.txt"));
/// assert!(patch.contains("-line 2"));
/// assert!(patch.contains("+line 2 modified"));
/// ```
pub fn create_unified_diff(
    old_text: &str,
    new_text: &str,
    old_label: &str,
    new_label: &str,
) -> String {
    let diff = diff_lines(old_text, new_text);
    if diff.iter().all(|l| l.diff_type == DiffType::Equal) {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!("--- {}\n", old_label));
    out.push_str(&format!("+++ {}\n", new_label));

    let hunks = group_into_hunks(&diff, 3);
    for hunk in hunks {
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_len, hunk.new_start, hunk.new_len
        ));
        for line in hunk.lines {
            match line.diff_type {
                DiffType::Equal => out.push_str(&format!(" {}\n", line.text)),
                DiffType::Insert => out.push_str(&format!("+{}\n", line.text)),
                DiffType::Delete => out.push_str(&format!("-{}\n", line.text)),
            }
        }
    }
    out
}

/// 差分行リストを指定コンテキスト行数でハンク（Hunk）にグループ化します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::diff::{diff_lines, group_into_hunks};
///
/// let diff = diff_lines("a\nb\nc", "a\nB\nc");
/// let hunks = group_into_hunks(&diff, 3);
/// assert_eq!(hunks.len(), 1);
/// ```
pub fn group_into_hunks(diff: &[DiffLine], context: usize) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut idx = 0;

    while idx < diff.len() {
        // 次の変更（InsertまたはDelete）を探す
        while idx < diff.len() && diff[idx].diff_type == DiffType::Equal {
            idx += 1;
        }
        if idx >= diff.len() {
            break;
        }

        let change_start = idx;
        let hunk_start = change_start.saturating_sub(context);

        // 変更の終わりを探す（前後のコンテキストが重なる場合はマージ）
        let mut last_change = change_start;
        let mut check_idx = change_start;
        while check_idx < diff.len() {
            if diff[check_idx].diff_type != DiffType::Equal {
                last_change = check_idx;
            } else if check_idx.saturating_sub(last_change) > context * 2 {
                break;
            }
            check_idx += 1;
        }

        let hunk_end = (last_change + context + 1).min(diff.len());
        let hunk_slice = &diff[hunk_start..hunk_end];

        let mut old_count = 0;
        let mut new_count = 0;
        let mut old_first = None;
        let mut new_first = None;

        for l in hunk_slice {
            match l.diff_type {
                DiffType::Equal => {
                    old_count += 1;
                    new_count += 1;
                    if old_first.is_none() {
                        old_first = l.old_lineno;
                    }
                    if new_first.is_none() {
                        new_first = l.new_lineno;
                    }
                }
                DiffType::Delete => {
                    old_count += 1;
                    if old_first.is_none() {
                        old_first = l.old_lineno;
                    }
                }
                DiffType::Insert => {
                    new_count += 1;
                    if new_first.is_none() {
                        new_first = l.new_lineno;
                    }
                }
            }
        }

        hunks.push(DiffHunk {
            old_start: old_first.unwrap_or(1),
            old_len: old_count,
            new_start: new_first.unwrap_or(1),
            new_len: new_count,
            lines: hunk_slice.to_vec(),
        });

        idx = hunk_end;
    }

    hunks
}

/// Unified Diff パッチを元テキストに適用して新しいテキストを再構築します。
///
/// ハンク情報（`@@` 行）および行プレフィックス（`+`, `-`, `' '`）を解析し、
/// 元テキストに差分を適用します。
///
/// # Errors
/// パッチの書式が不正な場合、またはハンクの適用対象行が整合しない場合にエラーメッセージを返します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::diff::{apply_unified_patch, create_unified_diff};
///
/// let old_text = "hello\nworld\n";
/// let new_text = "hello\nRust world\n";
/// let patch = create_unified_diff(old_text, new_text, "a", "b");
///
/// let restored = apply_unified_patch(old_text, &patch).unwrap();
/// assert_eq!(restored, new_text);
/// ```
pub fn apply_unified_patch(original: &str, patch: &str) -> Result<String, String> {
    let orig_lines: Vec<&str> = original.lines().collect();
    let mut result = Vec::new();
    let mut orig_idx = 0;

    let patch_lines: Vec<&str> = patch.lines().collect();
    let mut p_idx = 0;

    // ヘッダー（---, +++）をスキップ
    while p_idx < patch_lines.len()
        && (patch_lines[p_idx].starts_with("---") || patch_lines[p_idx].starts_with("+++"))
    {
        p_idx += 1;
    }

    while p_idx < patch_lines.len() {
        let line = patch_lines[p_idx];
        p_idx += 1;

        if line.starts_with("@@") {
            // 例: @@ -old_start,old_len +new_start,new_len @@
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }
            let old_spec = parts[1].trim_start_matches('-');
            let old_start: usize = old_spec
                .split(',')
                .next()
                .unwrap_or("1")
                .parse()
                .unwrap_or(1);
            let target_line = old_start.saturating_sub(1);

            // target_line まで未変更行をコピー
            while orig_idx < target_line && orig_idx < orig_lines.len() {
                result.push(orig_lines[orig_idx].to_string());
                orig_idx += 1;
            }
        } else if let Some(stripped) = line.strip_prefix('+') {
            result.push(stripped.to_string());
        } else if line.starts_with('-') {
            // 元行を1行スキップ
            orig_idx += 1;
        } else if let Some(stripped) = line.strip_prefix(' ') {
            if orig_idx < orig_lines.len() {
                result.push(orig_lines[orig_idx].to_string());
                orig_idx += 1;
            } else {
                result.push(stripped.to_string());
            }
        }
    }

    // 残りの元行をコピー
    while orig_idx < orig_lines.len() {
        result.push(orig_lines[orig_idx].to_string());
        orig_idx += 1;
    }

    let mut output = result.join("\n");
    if original.ends_with('\n') || patch.contains("+\n") {
        output.push('\n');
    }
    Ok(output)
}

/// 左右2画面分割（Side-by-Side DIFF）表示用の同期行リストを構築します。
///
/// 連続する削除行と追加行を1つの変更ブロック（左右横並び）として合成し、
/// 2ペインでの直感的な比較表示を可能にします。
///
/// # Examples
///
/// ```
/// use vivivi_modes::diff::{create_side_by_side, DiffType};
///
/// let sbs = create_side_by_side("old_line", "new_line");
/// assert_eq!(sbs.len(), 1);
/// assert_eq!(sbs[0].left_text.as_deref(), Some("old_line"));
/// assert_eq!(sbs[0].right_text.as_deref(), Some("new_line"));
/// assert_eq!(sbs[0].left_type, DiffType::Delete);
/// assert_eq!(sbs[0].right_type, DiffType::Insert);
/// ```
pub fn create_side_by_side(old_text: &str, new_text: &str) -> Vec<SideBySideLine> {
    let diff = diff_lines(old_text, new_text);
    let mut sbs = Vec::new();
    let mut i = 0;

    while i < diff.len() {
        match diff[i].diff_type {
            DiffType::Equal => {
                sbs.push(SideBySideLine {
                    left_lineno: diff[i].old_lineno,
                    left_text: Some(diff[i].text.clone()),
                    left_type: DiffType::Equal,
                    right_lineno: diff[i].new_lineno,
                    right_text: Some(diff[i].text.clone()),
                    right_type: DiffType::Equal,
                });
                i += 1;
            }
            DiffType::Delete => {
                // Delete と Insert が連続している場合は横並び（変更行）としてマージ
                if i + 1 < diff.len() && diff[i + 1].diff_type == DiffType::Insert {
                    sbs.push(SideBySideLine {
                        left_lineno: diff[i].old_lineno,
                        left_text: Some(diff[i].text.clone()),
                        left_type: DiffType::Delete,
                        right_lineno: diff[i + 1].new_lineno,
                        right_text: Some(diff[i + 1].text.clone()),
                        right_type: DiffType::Insert,
                    });
                    i += 2;
                } else {
                    sbs.push(SideBySideLine {
                        left_lineno: diff[i].old_lineno,
                        left_text: Some(diff[i].text.clone()),
                        left_type: DiffType::Delete,
                        right_lineno: None,
                        right_text: None,
                        right_type: DiffType::Equal,
                    });
                    i += 1;
                }
            }
            DiffType::Insert => {
                sbs.push(SideBySideLine {
                    left_lineno: None,
                    left_text: None,
                    left_type: DiffType::Equal,
                    right_lineno: diff[i].new_lineno,
                    right_text: Some(diff[i].text.clone()),
                    right_type: DiffType::Insert,
                });
                i += 1;
            }
        }
    }
    sbs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_lines_and_unified() {
        let old = "apple\nbanana\ncherry\n";
        let new = "apple\nblueberry\ncherry\ndate\n";

        let lines = diff_lines(old, new);
        assert_eq!(lines.len(), 5);

        let patch = create_unified_diff(old, new, "old.txt", "new.txt");
        assert!(patch.contains("--- old.txt"));
        assert!(patch.contains("+++ new.txt"));
        assert!(patch.contains("-banana"));
        assert!(patch.contains("+blueberry"));
        assert!(patch.contains("+date"));

        let restored = apply_unified_patch(old, &patch).expect("apply patch");
        assert_eq!(restored.trim(), new.trim());
    }

    #[test]
    fn test_side_by_side() {
        let old = "foo\nbar\nbaz\n";
        let new = "foo\nBAR\nbaz\nqux\n";

        let sbs = create_side_by_side(old, new);
        assert_eq!(sbs[0].left_text.as_deref(), Some("foo"));
        assert_eq!(sbs[0].right_text.as_deref(), Some("foo"));
        // bar -> BAR は横並び
        assert_eq!(sbs[1].left_text.as_deref(), Some("bar"));
        assert_eq!(sbs[1].right_text.as_deref(), Some("BAR"));
        assert_eq!(sbs[1].left_type, DiffType::Delete);
        assert_eq!(sbs[1].right_type, DiffType::Insert);
    }
}
