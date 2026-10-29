//! 2ペイン ディレクトリビュー（ファイラー機能）モジュール。
//!
//! ViVi 3 / ViVi 4 の伝統的な2画面ファイラーをモデル化し、
//! 左右画面の独立したディレクトリ閲覧、Tabキーによるフォーカス切り替え、
//! 1キーでのファイル選択・コピー・移動・削除操作を提供します。

use std::fs;
use std::path::{Path, PathBuf};

/// ファイラー一覧内の単一エントリ（ファイルまたはディレクトリ）。
///
/// ファイル名、絶対・相対パス、種別、サイズ、更新日時、および複数選択状態を保持します。
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// use vivivi_modes::filer::FileEntry;
///
/// let entry = FileEntry {
///     name: "sample.txt".to_string(),
///     path: PathBuf::from("sample.txt"),
///     is_dir: false,
///     size: 1024,
///     modified: None,
///     is_selected: false,
/// };
/// assert_eq!(entry.name, "sample.txt");
/// assert!(!entry.is_dir);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// ファイルまたはディレクトリの名前
    pub name: String,
    /// ファイルシステムのパス
    pub path: PathBuf,
    /// ディレクトリであるかどうかのフラグ
    pub is_dir: bool,
    /// ファイルサイズ（バイト単位。ディレクトリの場合は0）
    pub size: u64,
    /// 最終更新日時（フォーマット済み文字列）
    pub modified: Option<String>,
    /// スペースキー等でマーク・選択されているか
    pub is_selected: bool,
}

/// ファイル一覧の並び替え（ソート）順序。
///
/// # Examples
/// ```
/// use vivivi_modes::filer::FileSortOrder;
///
/// let order = FileSortOrder::default();
/// assert_eq!(order, FileSortOrder::NameAsc);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileSortOrder {
    /// ファイル名 昇順（デフォルト）
    #[default]
    NameAsc,
    /// ファイル名 降順
    NameDesc,
    /// ファイルサイズ 昇順
    SizeAsc,
    /// ファイルサイズ 降順
    SizeDesc,
    /// 更新日時 昇順
    ModifiedAsc,
    /// 更新日時 降順
    ModifiedDesc,
}

/// 単一ペインのファイラー状態（1画面分）。
///
/// 現在表示中のディレクトリパス、取得されたファイル一覧、
/// 現在のカーソル位置、ソート順を管理します。
///
/// # Examples
/// ```
/// use vivivi_modes::filer::FilerPane;
///
/// let pane = FilerPane::new(".");
/// assert_eq!(pane.cursor, 0);
/// ```
#[derive(Debug, Clone)]
pub struct FilerPane {
    /// 現在表示しているディレクトリ
    pub current_dir: PathBuf,
    /// ディレクトリ内のファイル・子ディレクトリ一覧
    pub entries: Vec<FileEntry>,
    /// 一覧内で現在フォーカスされているカーソル行（0-based）
    pub cursor: usize,
    /// 現在適用されているソート順
    pub sort_order: FileSortOrder,
}

impl FilerPane {
    /// 指定されたパスをカレントディレクトリとしてファイラーペインを初期化し、一覧を読み込みます。
    ///
    /// # 引数
    /// - `dir`: 表示対象のディレクトリパス
    ///
    /// # Panics
    /// パニックは発生しません。ディレクトリが存在しない・読み込めない場合は空の一覧となります。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let pane = FilerPane::new(".");
    /// assert_eq!(pane.cursor, 0);
    /// ```
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

    /// 現在のディレクトリを再走査し、エントリ一覧とソート状態を再構築します。
    ///
    /// 親ディレクトリが存在する場合は先頭に `..` を配置し、それ以外はディレクトリ優先でソートされます。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let mut pane = FilerPane::new(".");
    /// pane.refresh();
    /// ```
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

    /// カーソルを1行上へ移動します。先頭行にいる場合は変化しません。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let mut pane = FilerPane::new(".");
    /// pane.cursor = 1;
    /// pane.cursor_up();
    /// assert_eq!(pane.cursor, 0);
    /// ```
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// カーソルを1行下へ移動します。末尾行にいる場合は変化しません。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let mut pane = FilerPane::new(".");
    /// pane.cursor = 0;
    /// pane.cursor_down();
    /// ```
    pub fn cursor_down(&mut self) {
        if !self.entries.is_empty() && self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
        }
    }

    /// 現在カーソルが位置しているエントリを取得します。
    ///
    /// エントリが1件も存在しない場合は `None` を返します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let pane = FilerPane::new(".");
    /// let _entry = pane.current_entry();
    /// ```
    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    /// 現在カーソル行のエントリを開きます。
    ///
    /// 対象がディレクトリの場合はそのディレクトリへ階層を移動し、エントリ一覧を更新して `Ok(None)` を返します。
    /// 対象が通常ファイル等の場合は移動せず、そのファイルのパス `Ok(Some(PathBuf))` を返します。
    ///
    /// # Errors
    /// ファイルシステムや権限エラーが発生した場合にエラー文字列を返します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let mut pane = FilerPane::new(".");
    /// let _result = pane.enter();
    /// ```
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

    /// 親ディレクトリに移動し、エントリ一覧を再読み込みします。
    ///
    /// ルートディレクトリ等、親ディレクトリが存在しない場合は何も行いません。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let mut pane = FilerPane::new(".");
    /// pane.up_dir();
    /// ```
    pub fn up_dir(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.cursor = 0;
            self.refresh();
        }
    }

    /// 現在カーソル行の選択状態（チェックマーク）をトグルし、カーソルを1行下へ進めます。
    ///
    /// 親ディレクトリ `..` は選択対象外です。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let mut pane = FilerPane::new(".");
    /// pane.toggle_select();
    /// ```
    pub fn toggle_select(&mut self) {
        if let Some(entry) = self.entries.get_mut(self.cursor)
            && entry.name != ".."
        {
            entry.is_selected = !entry.is_selected;
        }
        self.cursor_down();
    }

    /// 選択マークが付いているエントリ一覧を取得します。
    ///
    /// 1件も明示的に選択されていない場合、現在カーソル位置のエントリ（`..` を除く）が単一要素として返されます。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::FilerPane;
    ///
    /// let pane = FilerPane::new(".");
    /// let targets = pane.selected_entries();
    /// ```
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

/// 左右どちらのファイラー画面が現在アクティブ（操作対象）であるかを示します。
///
/// # Examples
/// ```
/// use vivivi_modes::filer::ActiveFilerPane;
///
/// let active = ActiveFilerPane::Left;
/// assert_eq!(active, ActiveFilerPane::Left);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveFilerPane {
    /// 左画面
    Left,
    /// 右画面
    Right,
}

/// 2画面（左右ペイン）ファイラーマネージャ。
///
/// 左右それぞれの画面で独立したディレクトリ階層を閲覧でき、
/// Tabキーで画面を行き来しながらファイルの一括コピー・移動・削除を行えます。
///
/// # Examples
/// ```
/// use vivivi_modes::filer::{TwoPaneFiler, ActiveFilerPane};
///
/// let filer = TwoPaneFiler::new(".");
/// assert_eq!(filer.active, ActiveFilerPane::Left);
/// ```
#[derive(Debug, Clone)]
pub struct TwoPaneFiler {
    /// 左ペイン
    pub left: FilerPane,
    /// 右ペイン
    pub right: FilerPane,
    /// 現在フォーカスされているアクティブペイン
    pub active: ActiveFilerPane,
}

impl TwoPaneFiler {
    /// 左右ペインを指定されたディレクトリで初期化し、左画面をアクティブとして生成します。
    ///
    /// # 引数
    /// - `initial_dir`: 起動時の初期ディレクトリパス
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::{TwoPaneFiler, ActiveFilerPane};
    ///
    /// let filer = TwoPaneFiler::new(".");
    /// assert_eq!(filer.active, ActiveFilerPane::Left);
    /// ```
    pub fn new<P: AsRef<Path>>(initial_dir: P) -> Self {
        let left = FilerPane::new(initial_dir.as_ref());
        let right = FilerPane::new(initial_dir.as_ref());
        Self {
            left,
            right,
            active: ActiveFilerPane::Left,
        }
    }

    /// 現在アクティブなペインへの不変参照を取得します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::TwoPaneFiler;
    ///
    /// let filer = TwoPaneFiler::new(".");
    /// let active = filer.active_pane();
    /// assert_eq!(active.cursor, 0);
    /// ```
    pub fn active_pane(&self) -> &FilerPane {
        match self.active {
            ActiveFilerPane::Left => &self.left,
            ActiveFilerPane::Right => &self.right,
        }
    }

    /// 現在アクティブなペインへの可変参照を取得します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::TwoPaneFiler;
    ///
    /// let mut filer = TwoPaneFiler::new(".");
    /// filer.active_pane_mut().cursor_down();
    /// ```
    pub fn active_pane_mut(&mut self) -> &mut FilerPane {
        match self.active {
            ActiveFilerPane::Left => &mut self.left,
            ActiveFilerPane::Right => &mut self.right,
        }
    }

    /// 現在非アクティブ（反対側）のペインへの不変参照を取得します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::TwoPaneFiler;
    ///
    /// let filer = TwoPaneFiler::new(".");
    /// let other = filer.other_pane();
    /// assert_eq!(other.cursor, 0);
    /// ```
    pub fn other_pane(&self) -> &FilerPane {
        match self.active {
            ActiveFilerPane::Left => &self.right,
            ActiveFilerPane::Right => &self.left,
        }
    }

    /// 現在非アクティブ（反対側）のペインへの可変参照を取得します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::TwoPaneFiler;
    ///
    /// let mut filer = TwoPaneFiler::new(".");
    /// filer.other_pane_mut().cursor_down();
    /// ```
    pub fn other_pane_mut(&mut self) -> &mut FilerPane {
        match self.active {
            ActiveFilerPane::Left => &mut self.right,
            ActiveFilerPane::Right => &mut self.left,
        }
    }

    /// 左右ペインのアクティブ状態をトグル（切り替え）します（Tabキー相当）。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::filer::{TwoPaneFiler, ActiveFilerPane};
    ///
    /// let mut filer = TwoPaneFiler::new(".");
    /// assert_eq!(filer.active, ActiveFilerPane::Left);
    /// filer.switch_pane();
    /// assert_eq!(filer.active, ActiveFilerPane::Right);
    /// ```
    pub fn switch_pane(&mut self) {
        self.active = match self.active {
            ActiveFilerPane::Left => ActiveFilerPane::Right,
            ActiveFilerPane::Right => ActiveFilerPane::Left,
        };
    }

    /// アクティブペインで選択されたファイルを反対側ペインのディレクトリへコピーします (`c` キー相当)。
    ///
    /// コピー完了後、左右両ペインの一覧を自動的にリフレッシュします。
    ///
    /// # Errors
    /// ファイルの読み書き権限不足やディスク容量不足など、ファイルシステムI/Oに失敗した場合にエラー文字列を返します。
    ///
    /// # Examples
    /// ```rust,no_run
    /// use vivivi_modes::filer::TwoPaneFiler;
    ///
    /// let mut filer = TwoPaneFiler::new(".");
    /// let _count = filer.copy_to_other().unwrap();
    /// ```
    pub fn copy_to_other(&mut self) -> Result<usize, String> {
        let targets = self.active_pane().selected_entries();
        let dest_dir = self.other_pane().current_dir.clone();
        let mut count = 0;

        for item in targets {
            let dest_file = dest_dir.join(&item.name);
            if item.path == dest_file {
                return Err("コピー元とコピー先のパスが同一です".to_string());
            }
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

    /// アクティブペインで選択されたファイルを反対側ペインのディレクトリへ移動します (`m` キー相当)。
    ///
    /// 移動完了後、左右両ペインの一覧を自動的にリフレッシュします。
    ///
    /// # Errors
    /// ファイルの移動や権限エラーなど、ファイルシステムI/Oに失敗した場合にエラー文字列を返します。
    ///
    /// # Examples
    /// ```rust,no_run
    /// use vivivi_modes::filer::TwoPaneFiler;
    ///
    /// let mut filer = TwoPaneFiler::new(".");
    /// let _count = filer.move_to_other().unwrap();
    /// ```
    pub fn move_to_other(&mut self) -> Result<usize, String> {
        let targets = self.active_pane().selected_entries();
        let dest_dir = self.other_pane().current_dir.clone();
        let mut count = 0;

        for item in targets {
            let dest_file = dest_dir.join(&item.name);
            if item.path == dest_file {
                continue;
            }
            if fs::rename(&item.path, &dest_file).is_err() {
                // 異なるデバイス/ドライブ間の移動に対するフォールバック（コピー後に元を削除）
                if item.is_dir {
                    copy_dir_recursive(&item.path, &dest_file).map_err(|e| e.to_string())?;
                    fs::remove_dir_all(&item.path).map_err(|e| e.to_string())?;
                } else {
                    fs::copy(&item.path, &dest_file).map_err(|e| e.to_string())?;
                    fs::remove_file(&item.path).map_err(|e| e.to_string())?;
                }
            }
            count += 1;
        }

        self.left.refresh();
        self.right.refresh();
        Ok(count)
    }

    /// アクティブペインで選択されたファイルをファイルシステムから削除します (`d` キー相当)。
    ///
    /// 削除完了後、左右両ペインの一覧を自動的にリフレッシュします。
    ///
    /// # Errors
    /// 削除権限不足など、ファイルシステムI/Oに失敗した場合にエラー文字列を返します。
    ///
    /// # Examples
    /// ```rust,no_run
    /// use vivivi_modes::filer::TwoPaneFiler;
    ///
    /// let mut filer = TwoPaneFiler::new(".");
    /// let _count = filer.delete_selected().unwrap();
    /// ```
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
