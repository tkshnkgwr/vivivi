//! ガター変更マーカー（Change Markers / Line Change Tracking）モジュール。
//!
//! バッファ読み込み時および保存時の状態を保持し、
//! 各行の未保存変更（黄）および保存済み変更（緑）を追跡・判定します。

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
/// ファイルの初期状態（ロード時）と直近保存時の各行ハッシュを保持し、
/// バッファの各行について未保存の追加/変更か、保存済みの変更かを高速に判定します。
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
    /// ファイル初回読み込み時の各行ハッシュ
    original_hashes: Vec<u64>,
    /// 直近保存時の各行ハッシュ
    saved_hashes: Vec<u64>,
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
        Self {
            original_hashes: hashes.clone(),
            saved_hashes: hashes,
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
            original_hashes: Vec::new(),
            saved_hashes: Vec::new(),
            enabled: true,
        }
    }

    /// ファイル保存時に呼び出し、直近保存済み状態のハッシュを更新します。
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
        self.saved_hashes = current_lines.iter().map(|l| hash_line(l)).collect();
    }

    /// 現在の行リストに対する各行の変更状態（[`LineChangeState`]）を算出します。
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

        let cur_hashes: Vec<u64> = current_lines.iter().map(|l| hash_line(l)).collect();
        let mut markers = Vec::with_capacity(cur_hashes.len());

        let orig_len = self.original_hashes.len();
        let saved_len = self.saved_hashes.len();

        for (i, &cur_hash) in cur_hashes.iter().enumerate() {
            let matches_saved = i < saved_len && self.saved_hashes[i] == cur_hash;
            let matches_orig = i < orig_len && self.original_hashes[i] == cur_hash;

            if matches_saved {
                if matches_orig {
                    markers.push(LineChangeState::Unchanged);
                } else {
                    markers.push(LineChangeState::SavedModified);
                }
            } else if i >= saved_len {
                markers.push(LineChangeState::Added);
            } else {
                markers.push(LineChangeState::Modified);
            }
        }

        markers
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
}
