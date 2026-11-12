//! ガター変更マーカー（Change Markers / Line Change Tracking）モジュール。
//!
//! バッファ読み込み時および保存時の状態を保持し、
//! 各行の未保存変更（黄）および保存済み変更（緑）を追跡・判定します。

use crate::diff::{DiffType, diff_slices};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 行の変更状態（未変更・追加・未保存変更・保存済み変更）。
///
/// エディタの行番号左側（ガター部）に表示するマーカーの色分けに用いられます。
///
/// # Examples
///
/// ```
/// use vivivi_modes::gutter::LineChangeState;
///
/// let state = LineChangeState::Modified;
/// assert_eq!(state.symbol(), "▌");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineChangeState {
    /// 変更なし（ファイル読み込み時から一致）
    Unchanged,
    /// 新規追加された行（未保存）
    Added,
    /// 変更された行（未保存）
    Modified,
    /// 今回のセッションで変更され、ディスクに保存済みの行
    SavedModified,
}

impl LineChangeState {
    /// ガターに描画するシンボル文字列（`"▌"` または空白）を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::gutter::LineChangeState;
    ///
    /// assert_eq!(LineChangeState::Unchanged.symbol(), " ");
    /// assert_eq!(LineChangeState::Added.symbol(), "▌");
    /// ```
    pub fn symbol(self) -> &'static str {
        match self {
            LineChangeState::Unchanged => " ",
            LineChangeState::Added => "▌",
            LineChangeState::Modified => "▌",
            LineChangeState::SavedModified => "▌",
        }
    }
}

/// 変更行追跡マネージャー。
///
/// ファイルの初期状態（ロード時）と直近保存時の各行を保持し、
/// LCS（最長共通部分列）差分エンジンを用いて、未保存の追加/変更や保存済み変更を
/// 高速かつ正確に判定・キャッシュします。
///
/// # Examples
///
/// ```
/// use vivivi_modes::gutter::{ChangeTracker, LineChangeState};
///
/// let initial = vec!["line 1".to_string(), "line 2".to_string()];
/// let mut tracker = ChangeTracker::new(&initial);
///
/// let modified = vec!["line 1".to_string(), "line 2 modified".to_string()];
/// let markers = tracker.compute_markers(&modified);
/// assert_eq!(markers[0], LineChangeState::Unchanged);
/// assert_eq!(markers[1], LineChangeState::Modified);
/// ```
#[derive(Debug, Clone)]
pub struct ChangeTracker {
    /// 直近保存時の各行文字列
    saved_lines: Vec<String>,
    /// ファイル初回読み込み時の各行ハッシュ
    original_hashes: Vec<u64>,
    /// 直近保存時の各行ハッシュ
    saved_hashes: Vec<u64>,
    /// 最新の計算済みガターマーカーキャッシュ
    cached_markers: Vec<LineChangeState>,
    /// 前回のキャッシュ計算時の行ハッシュ
    last_current_hashes: Vec<u64>,
    /// ガターマーカー表示の有効/無効
    pub enabled: bool,
}

impl ChangeTracker {
    /// 初期行リストから追跡マネージャーを生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::gutter::ChangeTracker;
    ///
    /// let tracker = ChangeTracker::new(&["hello".to_string()]);
    /// assert!(tracker.enabled);
    /// ```
    pub fn new(initial_lines: &[String]) -> Self {
        let hashes: Vec<u64> = initial_lines.iter().map(|l| hash_line(l)).collect();
        let markers = vec![LineChangeState::Unchanged; initial_lines.len()];
        Self {
            saved_lines: initial_lines.to_vec(),
            original_hashes: hashes.clone(),
            saved_hashes: hashes.clone(),
            cached_markers: markers,
            last_current_hashes: hashes,
            enabled: true,
        }
    }

    /// 空のバッファから追跡マネージャーを生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::gutter::ChangeTracker;
    ///
    /// let tracker = ChangeTracker::empty();
    /// assert!(tracker.enabled);
    /// ```
    pub fn empty() -> Self {
        Self {
            saved_lines: Vec::new(),
            original_hashes: Vec::new(),
            saved_hashes: Vec::new(),
            cached_markers: Vec::new(),
            last_current_hashes: Vec::new(),
            enabled: true,
        }
    }

    /// ファイル保存時に呼び出し、直近保存済み状態のハッシュと行リストを更新します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::gutter::{ChangeTracker, LineChangeState};
    ///
    /// let lines = vec!["line 1".to_string()];
    /// let mut tracker = ChangeTracker::new(&lines);
    /// let new_lines = vec!["line 1 changed".to_string()];
    /// tracker.on_save(&new_lines);
    /// let markers = tracker.compute_markers(&new_lines);
    /// assert_eq!(markers[0], LineChangeState::SavedModified);
    /// ```
    pub fn on_save(&mut self, current_lines: &[String]) {
        self.saved_lines = current_lines.to_vec();
        self.saved_hashes = current_lines.iter().map(|l| hash_line(l)).collect();
        self.cached_markers = self.compute_markers(current_lines);
        self.last_current_hashes = self.saved_hashes.clone();
    }

    /// バッファ変更時にマーカーキャッシュを最新化します。
    ///
    /// 前回の計算時から行数および各行ハッシュに変化がない場合は Diff 計算をスキップし、
    /// 描画フレームレートへの影響をゼロにします。
    pub fn update_markers(&mut self, current_lines: &[String]) {
        if !self.enabled {
            return;
        }

        let cur_hashes: Vec<u64> = current_lines.iter().map(|l| hash_line(l)).collect();
        if self.cached_markers.len() == current_lines.len()
            && self.last_current_hashes == cur_hashes
        {
            return;
        }

        self.cached_markers = self.compute_markers(current_lines);
        self.last_current_hashes = cur_hashes;
    }

    /// 現在の行リストに対する各行の変更状態（[`LineChangeState`]）を LCS 差分エンジンにより算出します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::gutter::{ChangeTracker, LineChangeState};
    ///
    /// let tracker = ChangeTracker::new(&["a".to_string()]);
    /// let markers = tracker.compute_markers(&["a".to_string(), "b".to_string()]);
    /// assert_eq!(markers[0], LineChangeState::Unchanged);
    /// assert_eq!(markers[1], LineChangeState::Added);
    /// ```
    pub fn compute_markers(&self, current_lines: &[String]) -> Vec<LineChangeState> {
        if !self.enabled || current_lines.is_empty() {
            return vec![LineChangeState::Unchanged; current_lines.len()];
        }

        // 1. 直近保存時（saved_lines）と現在（current_lines）の差分を LCS で算出
        let diff = diff_slices(&self.saved_lines, current_lines);

        let cur_len = current_lines.len();
        let cur_hashes: Vec<u64> = current_lines.iter().map(|l| hash_line(l)).collect();
        let mut markers = vec![LineChangeState::Unchanged; cur_len];

        let is_orig_identical_to_saved = self.original_hashes == self.saved_hashes;

        let mut idx = 0;
        while idx < diff.len() {
            match diff[idx].diff_type {
                DiffType::Equal => {
                    if let Some(new_line_1based) = diff[idx].new_lineno {
                        let cur_idx = new_line_1based - 1;
                        if cur_idx < cur_len {
                            // 保存時と一致している行。初回読み込み時と比較して SavedModified か判定
                            let is_original = if is_orig_identical_to_saved {
                                true
                            } else if let Some(old_line_1based) = diff[idx].old_lineno {
                                let saved_idx = old_line_1based - 1;
                                saved_idx < self.original_hashes.len()
                                    && cur_idx < cur_hashes.len()
                                    && self.original_hashes[saved_idx] == cur_hashes[cur_idx]
                            } else {
                                false
                            };

                            markers[cur_idx] = if is_original {
                                LineChangeState::Unchanged
                            } else {
                                LineChangeState::SavedModified
                            };
                        }
                    }
                    idx += 1;
                }
                DiffType::Delete | DiffType::Insert => {
                    // 連続する Delete と Insert のチャンクを収集
                    let mut del_count = 0;
                    let mut inserts = Vec::new();
                    while idx < diff.len() && diff[idx].diff_type != DiffType::Equal {
                        if diff[idx].diff_type == DiffType::Delete {
                            del_count += 1;
                        } else if diff[idx].diff_type == DiffType::Insert {
                            inserts.push(diff[idx].new_lineno);
                        }
                        idx += 1;
                    }

                    // 削除行数までの挿入は「変更（Modified）」、それを超える挿入は「追加（Added）」
                    for (ins_i, &new_line_opt) in inserts.iter().enumerate() {
                        if let Some(new_line_1based) = new_line_opt {
                            let cur_idx = new_line_1based - 1;
                            if cur_idx < cur_len {
                                if ins_i < del_count {
                                    markers[cur_idx] = LineChangeState::Modified;
                                } else {
                                    markers[cur_idx] = LineChangeState::Added;
                                }
                            }
                        }
                    }
                }
            }
        }

        markers
    }

    /// 指定された単一行インデックスとその内容に対する変更状態（[`LineChangeState`]）を返します。
    ///
    /// キャッシュが存在する場合は $O(1)$ かつメモリ確保ゼロで即座に返します。
    ///
    /// # 引数
    /// - `line_idx`: 判定対象の行番号（0-based）
    /// - `line`: 判定対象の行文字列スライス
    ///
    /// # 戻り値
    /// 直近保存時および初回読み込み時からの差分に基づく変更状態
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::gutter::{ChangeTracker, LineChangeState};
    ///
    /// let tracker = ChangeTracker::new(&["initial".to_string()]);
    /// assert_eq!(tracker.marker_at(0, "initial"), LineChangeState::Unchanged);
    /// assert_eq!(tracker.marker_at(0, "modified"), LineChangeState::Modified);
    /// assert_eq!(tracker.marker_at(1, "new line"), LineChangeState::Added);
    /// ```
    pub fn marker_at(&self, line_idx: usize, line: &str) -> LineChangeState {
        if !self.enabled {
            return LineChangeState::Unchanged;
        }

        let cur_hash = hash_line(line);

        // キャッシュが存在し、かつ該当行の内容がキャッシュ計算時と一致している場合は高速リターン
        if let Some(&marker) = self.cached_markers.get(line_idx)
            && self.last_current_hashes.get(line_idx) == Some(&cur_hash)
        {
            return marker;
        }

        // キャッシュ未更新時または行内容変更時のフォールバック判定
        let matches_saved =
            line_idx < self.saved_hashes.len() && self.saved_hashes[line_idx] == cur_hash;
        let matches_orig =
            line_idx < self.original_hashes.len() && self.original_hashes[line_idx] == cur_hash;

        if matches_saved {
            if matches_orig {
                LineChangeState::Unchanged
            } else {
                LineChangeState::SavedModified
            }
        } else if line_idx >= self.saved_hashes.len() {
            LineChangeState::Added
        } else {
            LineChangeState::Modified
        }
    }
}

/// 1行の文字列からハッシュ値を計算
fn hash_line(line: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_tracker_marker_at() {
        let lines = vec!["line 1".to_string(), "line 2".to_string()];
        let tracker = ChangeTracker::new(&lines);
        assert_eq!(tracker.marker_at(0, "line 1"), LineChangeState::Unchanged);
        assert_eq!(tracker.marker_at(1, "line 2"), LineChangeState::Unchanged);
        assert_eq!(
            tracker.marker_at(0, "line 1 mod"),
            LineChangeState::Modified
        );
        assert_eq!(tracker.marker_at(2, "line 3 added"), LineChangeState::Added);
    }

    #[test]
    fn test_change_tracker_unchanged() {
        let lines = vec!["line 1".to_string(), "line 2".to_string()];
        let tracker = ChangeTracker::new(&lines);
        let markers = tracker.compute_markers(&lines);
        assert_eq!(
            markers,
            vec![LineChangeState::Unchanged, LineChangeState::Unchanged]
        );
    }

    #[test]
    fn test_change_tracker_modified_and_added() {
        let lines = vec!["line 1".to_string(), "line 2".to_string()];
        let tracker = ChangeTracker::new(&lines);

        let modified_lines = vec![
            "line 1".to_string(),
            "line 2 modified".to_string(),
            "line 3 added".to_string(),
        ];
        let markers = tracker.compute_markers(&modified_lines);
        assert_eq!(
            markers,
            vec![
                LineChangeState::Unchanged,
                LineChangeState::Modified,
                LineChangeState::Added
            ]
        );
    }

    #[test]
    fn test_change_tracker_saved_modified() {
        let lines = vec!["line 1".to_string(), "line 2".to_string()];
        let mut tracker = ChangeTracker::new(&lines);

        let current = vec!["line 1 changed".to_string(), "line 2".to_string()];
        // 保存実行
        tracker.on_save(&current);

        let markers = tracker.compute_markers(&current);
        assert_eq!(
            markers,
            vec![LineChangeState::SavedModified, LineChangeState::Unchanged]
        );
    }

    #[test]
    fn test_change_tracker_insert_in_middle() {
        let lines = vec![
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        let mut tracker = ChangeTracker::new(&lines);

        // 2行目の直後（インデックス 2）に新行 "inserted" を追加
        let current = vec![
            "line 1".to_string(),
            "line 2".to_string(),
            "inserted".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "line 5".to_string(),
        ];
        tracker.update_markers(&current);

        // 追加行のみが Added、それ以外の行（元の3〜5行目を含む）はすべて Unchanged であること！
        assert_eq!(tracker.marker_at(0, "line 1"), LineChangeState::Unchanged);
        assert_eq!(tracker.marker_at(1, "line 2"), LineChangeState::Unchanged);
        assert_eq!(tracker.marker_at(2, "inserted"), LineChangeState::Added);
        assert_eq!(tracker.marker_at(3, "line 3"), LineChangeState::Unchanged);
        assert_eq!(tracker.marker_at(4, "line 4"), LineChangeState::Unchanged);
        assert_eq!(tracker.marker_at(5, "line 5"), LineChangeState::Unchanged);
    }
}
