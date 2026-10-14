//! 2ペイン ディレクトリビュー（ファイラー機能）モジュール。
//!
//! ViVi 3 / ViVi 4 の伝統的な2ペインファイルマネージャをモデル化し、
//! 左右画面の独立したディレクトリ閲覧、Tabキー切り替え、
//! 1キーでのファイル選択・コピー・移動・削除操作を提供します。

use std::fs;
use std::path::{Path, PathBuf};

/// ファイル一覧エントリ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub is_selected: bool,
}

/// ソート基準。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileSortOrder {
    #[default]
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
    ModifiedAsc,
    ModifiedDesc,
}

/// 単一ファイラーペイン。
#[derive(Debug, Clone)]
pub struct FilerPane {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub sort_order: FileSortOrder,
}

impl FilerPane {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        let current_dir = dir.as_ref().to_path_buf();
        let mut pane = Self {
            current_dir,
            entries: Vec::new(),
            cursor: 0,
            sort_order: FileSortOrder::NameAsc,
        };
        pane.refresh();
        pane
    }

    /// 現在のディレクトリを再読み込みし、エントリ一覧を更新します。
    pub fn refresh(&mut self) {
        self.entries.clear();

        // 親ディレクトリ ".." を先頭に追加（ルートでなければ）
        if self.current_dir.parent().is_some() {
            self.entries.push(FileEntry {
                name: "..".to_string(),
                path: self.current_dir.parent().unwrap().to_path_buf(),
                is_dir: true,
                size: 0,
                modified: None,
                is_selected: false,
            });
        }

        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            let mut items = Vec::new();
            for entry in read_dir.flatten() {
                let path = entry.path();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();

                let metadata = entry.metadata().ok();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                items.push(FileEntry {
                    name: file_name,
                    path,
                    is_dir,
                    size,
                    modified: None,
                    is_selected: false,
                });
            }

            // ディレクトリ優先ソート
            items.sort_by(|a, b| {
                if a.is_dir != b.is_dir {
                    b.is_dir.cmp(&a.is_dir)
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            });

            self.entries.extend(items);
        }

        if self.cursor >= self.entries.len() {
            self.cursor = self.entries.len().saturating_sub(1);
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        if !self.entries.is_empty() && self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
        }
    }

    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    /// 現在行のエントリを開く／移動します（ディレクトリなら移動し true、ファイルなら false）。
    pub fn enter(&mut self) -> Result<Option<PathBuf>, String> {
        let entry = match self.current_entry() {
            Some(e) => e.clone(),
            None => return Ok(None),
        };

        if entry.is_dir {
            self.current_dir = entry.path;
            self.cursor = 0;
            self.refresh();
            Ok(None)
        } else {
            Ok(Some(entry.path))
        }
    }

    /// 親ディレクトリに移動します。
    pub fn up_dir(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.cursor = 0;
            self.refresh();
        }
    }

    /// 現在行の選択状態をトグルします（Spaceキー）。
    pub fn toggle_select(&mut self) {
        if let Some(entry) = self.entries.get_mut(self.cursor)
            && entry.name != ".."
        {
            entry.is_selected = !entry.is_selected;
        }
        self.cursor_down();
    }

    /// 選択されているエントリ一覧（無選択の場合は現在のカーソル位置のエントリ）を取得します。
    pub fn selected_entries(&self) -> Vec<FileEntry> {
        let selected: Vec<FileEntry> = self
            .entries
            .iter()
            .filter(|e| e.is_selected && e.name != "..")
            .cloned()
            .collect();

        if selected.is_empty()
            && let Some(entry) = self.current_entry()
            && entry.name != ".."
        {
            return vec![entry.clone()];
        }
        selected
    }
}

/// 左右どちらのアクティブペインか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveFilerPane {
    Left,
    Right,
}

/// 2ペインファイラーマネージャ。
#[derive(Debug, Clone)]
pub struct TwoPaneFiler {
    pub left: FilerPane,
    pub right: FilerPane,
    pub active: ActiveFilerPane,
}

impl TwoPaneFiler {
    pub fn new<P: AsRef<Path>>(initial_dir: P) -> Self {
        let left = FilerPane::new(initial_dir.as_ref());
        let right = FilerPane::new(initial_dir.as_ref());
        Self {
            left,
            right,
            active: ActiveFilerPane::Left,
        }
    }

    pub fn active_pane(&self) -> &FilerPane {
        match self.active {
            ActiveFilerPane::Left => &self.left,
            ActiveFilerPane::Right => &self.right,
        }
    }

    pub fn active_pane_mut(&mut self) -> &mut FilerPane {
        match self.active {
            ActiveFilerPane::Left => &mut self.left,
            ActiveFilerPane::Right => &mut self.right,
        }
    }

    pub fn other_pane(&self) -> &FilerPane {
        match self.active {
            ActiveFilerPane::Left => &self.right,
            ActiveFilerPane::Right => &self.left,
        }
    }

    pub fn other_pane_mut(&mut self) -> &mut FilerPane {
        match self.active {
            ActiveFilerPane::Left => &mut self.right,
            ActiveFilerPane::Right => &mut self.left,
        }
    }

    /// 左右ペインのアクティブをトグルします（Tabキー）。
    pub fn switch_pane(&mut self) {
        self.active = match self.active {
            ActiveFilerPane::Left => ActiveFilerPane::Right,
            ActiveFilerPane::Right => ActiveFilerPane::Left,
        };
    }

    /// 選択されたファイルをもう一方のペインのディレクトリへコピーします (`c` キー)。
    pub fn copy_to_other(&mut self) -> Result<usize, String> {
        let targets = self.active_pane().selected_entries();
        let dest_dir = self.other_pane().current_dir.clone();
        let mut count = 0;

        for item in targets {
            let dest_file = dest_dir.join(&item.name);
            if item.is_dir {
                // ディレクトリの再帰コピー
                copy_dir_recursive(&item.path, &dest_file).map_err(|e| e.to_string())?;
            } else {
                fs::copy(&item.path, &dest_file).map_err(|e| e.to_string())?;
            }
            count += 1;
        }

        self.left.refresh();
        self.right.refresh();
        Ok(count)
    }

    /// 選択されたファイルをもう一方のペインのディレクトリへ移動します (`m` キー)。
    pub fn move_to_other(&mut self) -> Result<usize, String> {
        let targets = self.active_pane().selected_entries();
        let dest_dir = self.other_pane().current_dir.clone();
        let mut count = 0;

        for item in targets {
            let dest_file = dest_dir.join(&item.name);
            fs::rename(&item.path, &dest_file).map_err(|e| e.to_string())?;
            count += 1;
        }

        self.left.refresh();
        self.right.refresh();
        Ok(count)
    }

    /// 選択されたファイルを削除します (`d` キー)。
    pub fn delete_selected(&mut self) -> Result<usize, String> {
        let targets = self.active_pane().selected_entries();
        let mut count = 0;

        for item in targets {
            if item.is_dir {
                fs::remove_dir_all(&item.path).map_err(|e| e.to_string())?;
            } else {
                fs::remove_file(&item.path).map_err(|e| e.to_string())?;
            }
            count += 1;
        }

        self.left.refresh();
        self.right.refresh();
        Ok(count)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filer_switch_and_select() {
        let mut filer = TwoPaneFiler::new(".");
        assert_eq!(filer.active, ActiveFilerPane::Left);
        filer.switch_pane();
        assert_eq!(filer.active, ActiveFilerPane::Right);
        filer.switch_pane();
        assert_eq!(filer.active, ActiveFilerPane::Left);
    }
}
