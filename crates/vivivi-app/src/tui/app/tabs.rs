use super::App;
use super::*;

impl App {
    /// 現在のアクティブエディタ状態を、対応するタブページ構造体に退避・同期します。
    pub fn sync_active_tab_state(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.buffer = self.buffer.clone();
            tab.cursor = self.cursor.clone();
            tab.windows = self.windows.clone();
            tab.active_window = self.active_window;
            tab.window_tree = self.window_tree.clone();
            tab.split_axis = self.split_axis;
            tab.file_path = self.file_path.clone();
            tab.encoding = self.encoding;
            tab.newline = self.newline;
            tab.syntax_lang = self.syntax_lang;
            tab.scroll = self.scroll;
            tab.change_tracker = self.change_tracker.clone();
        }
    }

    /// 指定インデックスのタブページからエディタ状態を復元・ロードします。
    pub fn load_tab_state(&mut self, index: usize) {
        if let Some(tab) = self.tabs.get(index) {
            self.buffer = tab.buffer.clone();
            self.cursor = tab.cursor.clone();
            self.windows = tab.windows.clone();
            self.active_window = tab.active_window;
            self.window_tree = tab.window_tree.clone();
            self.split_axis = tab.split_axis;
            self.file_path = tab.file_path.clone();
            self.encoding = tab.encoding;
            self.newline = tab.newline;
            self.syntax_lang = tab.syntax_lang;
            self.scroll = tab.scroll;
            self.change_tracker = tab.change_tracker.clone();
            self.active_tab = index;
            if let Some(ref path) = self.file_path {
                self.vi.current_file_path = path.to_string_lossy().to_string();
            } else {
                self.vi.current_file_path = String::new();
            }
            self.refresh_outline();
        }
    }

    /// 新規タブを開きます (`:tabnew [file]`)。
    pub fn new_tab(&mut self, file_path: Option<PathBuf>) {
        self.sync_active_tab_state();

        let (initial_text, enc, nl) = if let Some(ref path) = file_path {
            if path.exists() {
                if let Ok(bytes) = std::fs::read(path) {
                    let (content, enc, nl) = vivivi_core::decode_bytes(&bytes);
                    (content, enc, nl)
                } else {
                    (String::new(), CharEncoding::Utf8, NewlineType::default())
                }
            } else {
                (String::new(), CharEncoding::Utf8, NewlineType::default())
            }
        } else {
            (String::new(), CharEncoding::Utf8, NewlineType::default())
        };

        let tab =
            crate::tui::tab::TabPage::new(self.next_tab_id, &initial_text, file_path, enc, nl);
        self.next_tab_id += 1;
        self.tabs.push(tab);
        let new_idx = self.tabs.len() - 1;
        self.load_tab_state(new_idx);
        self.status_message = Some(format!("タブ {} を開きました", self.active_tab + 1));
    }

    /// 次のタブに切り替えます (`gt`)。
    pub fn next_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.sync_active_tab_state();
        let next_idx = (self.active_tab + 1) % self.tabs.len();
        self.load_tab_state(next_idx);
        self.status_message = Some(format!("タブ {}/{}", self.active_tab + 1, self.tabs.len()));
    }

    /// 前のタブに切り替えます (`gT`)。
    pub fn prev_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.sync_active_tab_state();
        let prev_idx = if self.active_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab - 1
        };
        self.load_tab_state(prev_idx);
        self.status_message = Some(format!("タブ {}/{}", self.active_tab + 1, self.tabs.len()));
    }

    /// 指定番号のタブに切り替えます (`[count]gt`)。1-based index。
    #[allow(dead_code)]
    pub fn switch_tab_1based(&mut self, tab_number: usize) {
        if tab_number == 0 || tab_number > self.tabs.len() {
            return;
        }
        self.sync_active_tab_state();
        self.load_tab_state(tab_number - 1);
        self.status_message = Some(format!("タブ {}/{}", self.active_tab + 1, self.tabs.len()));
    }

    /// 現在のタブを閉じます (`:tabclose`)。
    ///
    /// 未保存の変更がある場合はエラーを返し、安全のためタブを破棄しません。
    pub fn close_tab(&mut self) -> Result<(), &'static str> {
        self.close_tab_forced(false)
    }

    /// 現在のタブを閉じます（強制オプション指定可能）。
    pub fn close_tab_forced(&mut self, force: bool) -> Result<(), &'static str> {
        if self.tabs.len() <= 1 {
            return Err("最後のタブは閉じられません (:q で終了してください)");
        }
        if !force && self.buffer.is_modified() {
            return Err("E37: 前回の変更が保存されていません (! を追加して強制終了)");
        }

        self.tabs.remove(self.active_tab);
        let new_idx = if self.active_tab >= self.tabs.len() {
            self.tabs.len() - 1
        } else {
            self.active_tab
        };
        self.load_tab_state(new_idx);
        self.status_message = Some(format!(
            "タブを閉じました (現在: {}/{})",
            self.active_tab + 1,
            self.tabs.len()
        ));
        Ok(())
    }

    /// 現在のタブ以外のすべてのタブを閉じます (`:tabonly`)。
    ///
    /// 他のタブに未保存の変更がある場合は安全のため処理を中止します。
    pub fn close_other_tabs(&mut self) -> Result<(), &'static str> {
        self.close_other_tabs_forced(false)
    }

    /// 現在のタブ以外のすべてのタブを閉じます（強制オプション指定可能）。
    pub fn close_other_tabs_forced(&mut self, force: bool) -> Result<(), &'static str> {
        if self.tabs.len() <= 1 {
            return Ok(());
        }
        self.sync_active_tab_state();

        if !force {
            for (i, tab) in self.tabs.iter().enumerate() {
                if i != self.active_tab && tab.buffer.is_modified() {
                    return Err(
                        "E37: 他のタブに未保存の変更があります (! を追加して :tabonly! で強制実行)",
                    );
                }
            }
        }

        let current = self.tabs.remove(self.active_tab);
        self.tabs = vec![current];
        self.active_tab = 0;
        self.status_message = Some("他のタブをすべて閉じました".to_string());
        Ok(())
    }
}
