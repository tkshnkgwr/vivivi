//! ガター変更マーカー（Change Markers / Line Change Tracking）モジュール。
//!
//! バッファ読み込み時および保存時の状態を保持し、
//! 各行の未保存変更（黄）および保存済み変更（緑）を追跡・判定します。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 行の変更状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineChangeState {
    /// 変更なし
    Unchanged,
    /// 新規追加された行（未保存）
    Added,
    /// 変更された行（未保存）
    Modified,
    /// 今回のセッションで変更され、保存済みの行
    SavedModified,
}

impl LineChangeState {
    /// ガターに表示するシンボル文字
    pub fn symbol(self) -> &'static str {
        match self {
            LineChangeState::Unchanged => " ",
            LineChangeState::Added => "▌",
            LineChangeState::Modified => "▌",
            LineChangeState::SavedModified => "▌",
        }
    }
}

/// 変更行追跡マネージャー
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
    pub fn new(initial_lines: &[String]) -> Self {
        let hashes: Vec<u64> = initial_lines.iter().map(|l| hash_line(l)).collect();
        Self {
            original_hashes: hashes.clone(),
            saved_hashes: hashes,
            enabled: true,
        }
    }

    /// 空のバッファから初期化します。
    pub fn empty() -> Self {
        Self {
            original_hashes: Vec::new(),
            saved_hashes: Vec::new(),
            enabled: true,
        }
    }

    /// ファイル保存時に呼び出し、保存済み状態を更新します。
    pub fn on_save(&mut self, current_lines: &[String]) {
        self.saved_hashes = current_lines.iter().map(|l| hash_line(l)).collect();
    }

    /// 各行の変更状態リストを算出します。
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
