//! 最近使ったファイル (MRU: Most Recently Used) およびお気に入りファイル管理モジュール。
//!
//! 開いたファイルの履歴とピン留めお気に入りファイルを永続化し、
//! クイックアクセスを提供します。

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// 最近使ったファイルとお気に入りの管理マネージャー
#[derive(Debug, Clone, Default)]
pub struct RecentManager {
    /// 最近使ったファイル一覧（先頭が最新）
    pub recent_files: Vec<PathBuf>,
    /// お気に入りファイル一覧
    pub favorite_files: Vec<PathBuf>,
    /// 履歴の最大保持件数
    pub max_recent: usize,
}

impl RecentManager {
    /// デフォルト設定で新規生成します（最大30件）。
    pub fn new() -> Self {
        Self {
            recent_files: Vec::new(),
            favorite_files: Vec::new(),
            max_recent: 30,
        }
    }

    /// 最近使ったファイルにパスを追加します（先頭に追加、重複は移動、上限で切り詰め）。
    pub fn add_recent(&mut self, path: PathBuf) {
        // 既存の同一パスを削除
        self.recent_files.retain(|p| p != &path);
        // 先頭に挿入
        self.recent_files.insert(0, path);
        // 上限超えを切り詰め
        if self.recent_files.len() > self.max_recent {
            self.recent_files.truncate(self.max_recent);
        }
    }

    /// 最近使ったファイルからパスを削除します。
    pub fn remove_recent(&mut self, path: &Path) {
        self.recent_files.retain(|p| p != path);
    }

    /// お気に入りファイルの登録・解除をトグルします。
    pub fn toggle_favorite(&mut self, path: PathBuf) -> bool {
        if let Some(idx) = self.favorite_files.iter().position(|p| p == &path) {
            self.favorite_files.remove(idx);
            false
        } else {
            self.favorite_files.push(path);
            true
        }
    }

    /// お気に入りに登録されているか判定します。
    pub fn is_favorite(&self, path: &Path) -> bool {
        self.favorite_files.iter().any(|p| p == path)
    }

    /// 設定ファイルから履歴・お気に入りを読み込みます。
    pub fn load_from_file(&mut self, path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut current_section = "";
        self.recent_files.clear();
        self.favorite_files.clear();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed == "[favorites]" {
                current_section = "favorites";
                continue;
            } else if trimmed == "[recent]" {
                current_section = "recent";
                continue;
            }

            let path_buf = PathBuf::from(trimmed);
            match current_section {
                "favorites" if !self.favorite_files.contains(&path_buf) => {
                    self.favorite_files.push(path_buf);
                }
                "recent"
                    if !self.recent_files.contains(&path_buf)
                        && self.recent_files.len() < self.max_recent =>
                {
                    self.recent_files.push(path_buf);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 設定ファイルへ履歴・お気に入りを書き出します。
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(path)?;
        writeln!(file, "# vivivi recent & favorite files")?;
        writeln!(file, "[favorites]")?;
        for p in &self.favorite_files {
            writeln!(file, "{}", p.display())?;
        }
        writeln!(file)?;
        writeln!(file, "[recent]")?;
        for p in &self.recent_files {
            writeln!(file, "{}", p.display())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_manager_add_and_truncate() {
        let mut mgr = RecentManager::new();
        mgr.max_recent = 3;

        mgr.add_recent(PathBuf::from("a.txt"));
        mgr.add_recent(PathBuf::from("b.txt"));
        mgr.add_recent(PathBuf::from("c.txt"));
        mgr.add_recent(PathBuf::from("d.txt"));

        assert_eq!(mgr.recent_files.len(), 3);
        assert_eq!(mgr.recent_files[0], PathBuf::from("d.txt"));
        assert_eq!(mgr.recent_files[1], PathBuf::from("c.txt"));
        assert_eq!(mgr.recent_files[2], PathBuf::from("b.txt"));

        // 再度 b.txt を開いたら先頭に移動
        mgr.add_recent(PathBuf::from("b.txt"));
        assert_eq!(mgr.recent_files.len(), 3);
        assert_eq!(mgr.recent_files[0], PathBuf::from("b.txt"));
    }

    #[test]
    fn test_recent_manager_toggle_favorite() {
        let mut mgr = RecentManager::new();
        let path = PathBuf::from("fav.txt");

        assert!(!mgr.is_favorite(&path));
        let added = mgr.toggle_favorite(path.clone());
        assert!(added);
        assert!(mgr.is_favorite(&path));

        let removed = mgr.toggle_favorite(path.clone());
        assert!(!removed);
        assert!(!mgr.is_favorite(&path));
    }

    #[test]
    fn test_recent_manager_save_and_load() {
        let dir = std::env::temp_dir().join("vivivi_test_recent");
        let _ = fs::remove_dir_all(&dir);
        let file_path = dir.join("recent.txt");

        let mut mgr = RecentManager::new();
        mgr.add_recent(PathBuf::from("/path/to/one.rs"));
        mgr.add_recent(PathBuf::from("/path/to/two.rs"));
        mgr.toggle_favorite(PathBuf::from("/path/to/fav.rs"));

        mgr.save_to_file(&file_path).unwrap();

        let mut loaded = RecentManager::new();
        loaded.load_from_file(&file_path).unwrap();

        assert_eq!(
            loaded.favorite_files,
            vec![PathBuf::from("/path/to/fav.rs")]
        );
        assert_eq!(
            loaded.recent_files,
            vec![
                PathBuf::from("/path/to/two.rs"),
                PathBuf::from("/path/to/one.rs")
            ]
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
