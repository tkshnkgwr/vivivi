use super::window::{EditorWindow, SplitAxis, WindowDirection, WindowTreeNode};
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use std::path::PathBuf;
use vivivi_core::{CharEncoding, CursorPosition, NewlineType, TextBuffer, TextCursor};
use vivivi_modes::{
    AbbrevManager, BoxAlignment, ChangeTracker, ConversionType, CsvMode, DabbrevSession,
    DynamicMacroEngine, KeisenDirection, KeisenMode, LexEditEngine, ListCharsConfig, MigemoEngine,
    OutlineItem, RecentManager, SyntaxLang, align_lines, calculate_box_selection, convert_text,
    expand_zen_coding, insert_beg_end_lines, parse_outline, renumber_lines,
};
use vivivi_vi::{RegisterContent, ViEngine, ViMode};

/// TUI アプリケーションのグローバル状態管理構造体。
///
/// テキストバッファ（[`TextBuffer`]）、カーソル、Viコマンド実行エンジン（[`ViEngine`]）、
/// 画面分割ウィンドウ一覧、各種編集拡張モード（罫線、CSV、Migemo、アウトライン、補完、マクロ等）の
/// 全状態を一元管理します。
///
/// # Examples
/// ```rust,ignore
/// use vivivi_app::tui::App;
///
/// let app = App::new("fn main() {}", None, Default::default(), Default::default());
/// assert_eq!(app.buffer.line_count(), 1);
/// ```
pub struct App {
    /// 編集中のテキストバッファ
    pub buffer: TextBuffer,
    /// アクティブウィンドウのカーソル
    pub cursor: TextCursor,
    /// Viコマンドエンジン
    pub vi: ViEngine,
    /// インラインCSV/TSVモード
    pub csv_mode: CsvMode,
    /// 罫線描画・編集モード
    pub keisen_mode: KeisenMode,
    /// CSV整列ビューが有効かどうか
    pub is_csv_active: bool,
    /// 文字エンコーディング（UTF-8, Shift_JIS, EUC-JP等）
    pub encoding: CharEncoding,
    /// 改行コード種別（LF, CRLF, CR）
    pub newline: NewlineType,
    /// シンタックスハイライト言語
    pub syntax_lang: SyntaxLang,
    /// スクロールオフセット (表示開始行, 表示開始列)
    pub scroll: (usize, usize),
    /// 開いているファイルのパス
    pub file_path: Option<PathBuf>,
    /// アプリケーション終了フラグ（true でメインループ脱出）
    pub should_quit: bool,
    /// 最下行ステータスバーに表示する一時通知メッセージ
    pub status_message: Option<String>,

    // --- 画面分割 (Split / Vsplit) ---
    /// 開かれているエディタウィンドウ一覧
    pub windows: Vec<EditorWindow>,
    /// 現在フォーカスがあるウィンドウのインデックス
    pub active_window: usize,
    /// 階層的ウィンドウレイアウトツリー（Window Layout Tree）
    pub window_tree: WindowTreeNode,
    /// 各ウィンドウの最新描画領域矩形キャッシュ [(win_idx, Rect)]
    pub window_rects: Vec<(usize, Rect)>,
    /// 直近の分割方向 (水平/垂直)
    pub split_axis: SplitAxis,
    /// Ctrl-W ウィンドウコマンド入力待ちフラグ
    pub ctrl_w_pending: bool,

    // --- アウトライン解析 ---
    /// サイドペインにアウトラインを表示するかどうか
    pub show_outline: bool,
    /// アウトラインペインにフォーカスがあるか
    pub outline_focus: bool,
    /// 解析済みのアウトライン項目一覧
    pub outline_items: Vec<OutlineItem>,
    /// 選択中のアウトライン項目のインデックス
    pub selected_outline_idx: usize,

    // --- 動的補完 (dabbrev / Ctrl-L, Ctrl-M) ---
    pub dabbrev: DabbrevSession,

    // --- 静的略語展開 (abbrev / Ctrl-;) ---
    pub abbrev: AbbrevManager,

    // --- ダイナミックマクロ (Dynamic Macro / Alt-M) ---
    pub dynamic_macro: DynamicMacroEngine,

    // --- 字句編集予測 (LexEdit / Ctrl-J) ---
    pub lex_edit: LexEditEngine,

    // --- キーボードマクロ (Ctrl-Alt-R / Ctrl-Alt-P) ---
    pub is_recording_macro: bool,
    pub recorded_macro_events: Vec<KeyEvent>,
    pub last_macro_events: Option<Vec<KeyEvent>>,

    // --- クリップボード履歴 (cbHist) ---
    pub show_clipboard_hist: bool,
    pub selected_cb_idx: usize,

    // --- ローマ字日本語検索 (Migemo / g/) ---
    pub migemo: MigemoEngine,
    pub g_pending: bool,
    pub is_migemo_search: bool,
    pub migemo_query: String,
    pub migemo_last_regex: Option<regex::Regex>,
    pub is_search_highlight_visible: bool,
    /// 直前の Esc キー押下時刻（ダブル Esc 連打による検索ハイライト消去判定用）
    pub last_esc_time: Option<std::time::Instant>,

    // --- ヘルプビューア ---
    pub show_help: bool,
    pub help_title: String,
    pub help_lines: Vec<String>,
    pub help_scroll: usize,

    // --- 印刷・整形プレビュー ---
    pub show_preview: bool,
    pub preview_lines: Vec<String>,
    pub preview_scroll: usize,

    // --- 変更行ガター追跡 (Change Markers) ---
    pub change_tracker: ChangeTracker,

    // --- 最近使ったファイル & お気に入り (Recent Files / Favorites) ---
    pub recent_manager: RecentManager,
    pub show_recent_modal: bool,
    pub selected_recent_idx: usize,

    // --- 不可視文字可視化 (:set list / :set nolist) ---
    pub list_chars_config: ListCharsConfig,

    // --- 永続設定 & 表示オプション ---
    pub config: crate::config::AppConfig,
    pub show_line_numbers: bool,
    pub relative_number: bool,
    pub tab_width: usize,

    // --- クラッシュリカバリ・スワップファイル (.swp) ---
    pub key_stroke_count: usize,
    pub has_swap_warning: bool,

    // --- マルチタブ管理 (Tab Bar UI) ---
    pub tabs: Vec<crate::tui::tab::TabPage>,
    pub active_tab: usize,
    pub next_tab_id: usize,
}

impl App {
    /// 新規テキストまたは指定ファイルの内容から [`App`] インスタンスを初期化・生成します。
    ///
    /// 新規作成ファイルの場合は拡張子に応じたテンプレートが自動探索・展開され、
    /// 既存ファイルの場合は永続化アンドゥログ（`.undodir`）の復元と最近使ったファイル履歴の追加が行われます。
    ///
    /// # 引数
    /// - `initial_text`: 起動時の初期テキストバッファ内容
    /// - `file_path`: 開くファイルのパス（新規・無名ファイルの場合は `None`）
    /// - `encoding`: 文字エンコーディング
    /// - `newline`: 改行コード種別
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let app = App::new("test content", None, Default::default(), Default::default());
    /// assert_eq!(app.buffer.line_count(), 1);
    /// ```
    pub fn new(
        initial_text: &str,
        file_path: Option<PathBuf>,
        encoding: CharEncoding,
        newline: NewlineType,
    ) -> Self {
        let mut buffer = TextBuffer::from_text(initial_text);
        if let Some(ref path) = file_path {
            if !path.exists() && initial_text.is_empty() {
                let tm = vivivi_modes::TemplateManager::new();
                if let Some(res) = tm.load_and_expand(path, "2026-10-14", "12:00:00") {
                    buffer = TextBuffer::from_text(&res.text);
                }
            } else {
                let _ = buffer.load_undofile(path);
            }
        }
        let mut csv_mode = CsvMode::default();
        let is_csv = file_path.as_ref().is_some_and(|p| {
            p.extension()
                .is_some_and(|ext| ext == "csv" || ext == "tsv")
        });
        if is_csv {
            let lines: Vec<&str> = initial_text.lines().collect();
            csv_mode.calculate_widths(lines);
        }

        let syntax_lang = file_path
            .as_ref()
            .map(SyntaxLang::from_path)
            .unwrap_or(SyntaxLang::PlainText);

        let initial_cursor = TextCursor::new(0, 0);
        let initial_window = EditorWindow::new(0, 0);

        let config = crate::config::AppConfig::load_or_default(None);
        let show_line_numbers = config.editor.line_number;
        let relative_number = config.editor.relative_number;
        let tab_width = config.editor.tab_width;

        let mut vi = ViEngine::new();
        vi.set_clipboard_provider(crate::tui::clipboard::SystemClipboard::new_shared());
        let cb_opt = vivivi_vi::ClipboardOption::parse(&config.editor.clipboard);
        vi.set_clipboard_option(cb_opt);
        if let Some(ref path) = file_path {
            vi.current_file_path = path.to_string_lossy().to_string();
        }

        // 変更行ガター追跡の初期化
        let lines: Vec<String> = (0..buffer.line_count())
            .filter_map(|i| buffer.get_line(i))
            .collect();
        let mut change_tracker = ChangeTracker::new(&lines);
        change_tracker.enabled = config.gutter.enabled;

        // 最近使ったファイルとお気に入りの読み込み
        let mut recent_manager = RecentManager::new();
        let recent_file_path = get_recent_file_path();
        let _ = recent_manager.load_from_file(&recent_file_path);
        if let Some(ref path) = file_path {
            recent_manager.add_recent(path.clone());
            let _ = recent_manager.save_to_file(&recent_file_path);
        }

        let list_chars_config = ListCharsConfig {
            enabled: config.invisible.enabled,
            zenkaku_space: Box::leak(config.invisible.zenkaku_space.clone().into_boxed_str()),
            space: config.invisible.space,
            eol: config.invisible.eol,
            ..Default::default()
        };

        let mut has_swap_warning = false;
        let mut initial_status_message = None;
        if let Some(ref path) = file_path
            && vivivi_core::SwapFile::exists_for(path)
        {
            has_swap_warning = true;
            initial_status_message = Some(
                "⚠️ 未保存スワップファイルが存在します。復元するには ':recover' を実行してください"
                    .to_string(),
            );
        }

        let initial_tab = crate::tui::tab::TabPage {
            id: 1,
            buffer: buffer.clone(),
            cursor: initial_cursor.clone(),
            windows: vec![initial_window.clone()],
            active_window: 0,
            window_tree: WindowTreeNode::new(0),
            split_axis: SplitAxis::Horizontal,
            file_path: file_path.clone(),
            encoding,
            newline,
            syntax_lang,
            scroll: (0, 0),
            change_tracker: change_tracker.clone(),
        };

        Self {
            buffer,
            cursor: initial_cursor,
            vi,
            csv_mode,
            keisen_mode: KeisenMode::new(),
            is_csv_active: is_csv,
            encoding,
            newline,
            syntax_lang,
            scroll: (0, 0),
            file_path,
            should_quit: false,
            status_message: initial_status_message,

            windows: vec![initial_window],
            active_window: 0,
            window_tree: WindowTreeNode::new(0),
            window_rects: Vec::new(),
            split_axis: SplitAxis::Horizontal,
            ctrl_w_pending: false,

            show_outline: false,
            outline_focus: false,
            outline_items: Vec::new(),
            selected_outline_idx: 0,

            dabbrev: DabbrevSession::new(),
            abbrev: AbbrevManager::new(),
            dynamic_macro: DynamicMacroEngine::new(),
            lex_edit: LexEditEngine::new(),
            is_recording_macro: false,
            recorded_macro_events: Vec::new(),
            last_macro_events: None,
            show_clipboard_hist: false,
            selected_cb_idx: 0,
            migemo: MigemoEngine::new(),
            g_pending: false,
            is_migemo_search: false,
            migemo_query: String::new(),
            migemo_last_regex: None,
            is_search_highlight_visible: true,
            last_esc_time: None,

            show_help: false,
            help_title: String::new(),
            help_lines: Vec::new(),
            help_scroll: 0,

            show_preview: false,
            preview_lines: Vec::new(),
            preview_scroll: 0,

            change_tracker,
            recent_manager,
            show_recent_modal: false,
            selected_recent_idx: 0,
            list_chars_config,

            config,
            show_line_numbers,
            relative_number,
            tab_width,
            key_stroke_count: 0,
            has_swap_warning,

            tabs: vec![initial_tab],
            active_tab: 0,
            next_tab_id: 2,
        }
    }

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
    pub fn close_tab(&mut self) -> Result<(), &'static str> {
        if self.tabs.len() <= 1 {
            return Err("最後のタブは閉じられません (:q で終了してください)");
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
    pub fn close_other_tabs(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.sync_active_tab_state();
        let current = self.tabs.remove(self.active_tab);
        self.tabs = vec![current];
        self.active_tab = 0;
        self.status_message = Some("他のタブをすべて閉じました".to_string());
    }

    /// 編集中バッファをスワップファイル（`.<filename>.swp`）に保存します。
    pub fn save_swap_file(&self) {
        if let Some(ref path) = self.file_path
            && self.buffer.is_modified()
        {
            let swp_path = vivivi_core::SwapFile::swap_path_for(path);
            let _ = vivivi_core::SwapFile::save(
                &swp_path,
                Some(path),
                &self.buffer.to_string(),
                self.encoding,
                self.newline,
            );
        }
    }

    /// スワップファイルを安全に削除します。
    pub fn remove_swap_file(&self) {
        if let Some(ref path) = self.file_path {
            let swp_path = vivivi_core::SwapFile::swap_path_for(path);
            let _ = vivivi_core::SwapFile::remove(&swp_path);
        }
    }

    /// スワップファイルからバッファ内容を復元します。
    pub fn recover_from_swap(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.file_path {
            let swp_path = vivivi_core::SwapFile::swap_path_for(path);
            if swp_path.exists() {
                let rec = vivivi_core::SwapFile::load(&swp_path)?;
                self.buffer = vivivi_core::TextBuffer::from_text(&rec.content);
                self.encoding = rec.encoding;
                self.newline = rec.newline;
                self.has_swap_warning = false;
                self.status_message = Some(format!(
                    "スワップファイルから復元しました ({})",
                    swp_path.display()
                ));
                return Ok(());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "復元対象のスワップファイルが見つかりません",
        ))
    }

    /// アクティブウィンドウにおける現在のカーソル座標を取得します。
    ///
    /// # 戻り値
    /// 行・列（0-based）を表す [`CursorPosition`]
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let app = App::new("test", None, Default::default(), Default::default());
    /// assert_eq!(app.cursor_pos().line, 0);
    /// ```
    pub fn cursor_pos(&self) -> CursorPosition {
        self.cursor.pos
    }

    /// 現在の Vi 操作モード（ノーマル、挿入、ビジュアル等）を取得します。
    ///
    /// # 戻り値
    /// 現在有効な [`ViMode`]
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    /// use vivivi_vi::ViMode;
    ///
    /// let app = App::new("test", None, Default::default(), Default::default());
    /// assert_eq!(app.mode(), ViMode::Normal);
    /// ```
    pub fn mode(&self) -> ViMode {
        self.vi.mode()
    }

    /// 単一文字を Vi コマンドエンジンに投入し、確定したコマンドを実行します。
    ///
    /// 検索コマンドの場合はハイライト表示を更新し、印刷プレビュー等が要求された場合は
    /// プレビュー画面を自動展開します。
    ///
    /// # 引数
    /// - `ch`: 投入する入力文字
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("hello", None, Default::default(), Default::default());
    /// app.feed_vi_char('x'); // 1文字削除
    /// ```
    pub fn feed_vi_char(&mut self, ch: char) {
        if let Some(cmd) = self.vi.feed_char(ch) {
            match &cmd {
                vivivi_vi::ViCommand::SearchForward(_)
                | vivivi_vi::ViCommand::SearchBackward(_)
                | vivivi_vi::ViCommand::SearchWordUnderCursor { .. } => {
                    self.migemo_last_regex = None;
                    self.is_search_highlight_visible = true;
                }
                vivivi_vi::ViCommand::SearchNext | vivivi_vi::ViCommand::SearchPrev => {
                    self.is_search_highlight_visible = true;
                }
                _ => {}
            }

            // 罫線保護モード有効時の 'x' / 'X' による文字削除対応
            if self.keisen_mode.is_active {
                match cmd {
                    vivivi_vi::ViCommand::DeleteChar(count) => {
                        for _ in 0..count {
                            vivivi_modes::KeisenMode::delete_char_protected(
                                &mut self.buffer,
                                &mut self.cursor.pos,
                                false,
                            );
                        }
                        if let Some(win) = self.windows.get_mut(self.active_window) {
                            win.cursor = self.cursor.clone();
                            win.scroll = self.scroll;
                        }
                        return;
                    }
                    vivivi_vi::ViCommand::DeletePrevChar(count) => {
                        for _ in 0..count {
                            vivivi_modes::KeisenMode::delete_char_protected(
                                &mut self.buffer,
                                &mut self.cursor.pos,
                                true,
                            );
                        }
                        if let Some(win) = self.windows.get_mut(self.active_window) {
                            win.cursor = self.cursor.clone();
                            win.scroll = self.scroll;
                        }
                        return;
                    }
                    _ => {}
                }
            }

            let res = self.vi.execute(&cmd, &mut self.buffer, &mut self.cursor);
            if let Some(win) = self.windows.get_mut(self.active_window) {
                win.cursor = self.cursor.clone();
                win.scroll = self.scroll;
            }
            if let Some(msg) = res.message {
                self.status_message = Some(msg);
            }
            if let Some(preview_str) = res.preview_requested {
                self.preview_lines = preview_str.lines().map(|s| s.to_string()).collect();
                self.show_preview = true;
                self.preview_scroll = 0;
            }
        }
    }

    /// 現在のバッファ内容を設定されたエンコーディングおよび改行コードでファイルに保存します (`:w` 相当)。
    ///
    /// 既存ファイルの上書き時は世代差分バックアップ（[`vivivi_modes::DiffLog`]）および
    /// 永続化アンドゥログ（`.undodir`）を自動更新し、変更行ガター追跡のベースラインを再設定します。
    ///
    /// # Errors
    /// ファイルの書き込み権限不足やディスク容量不足など、I/O エラーが発生した場合に [`std::io::Error`] を返します。
    /// ファイルパスが設定されていない無名バッファの場合は何もしません。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// let _ = app.save_file();
    /// ```
    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = self.file_path.clone() {
            let content = self.buffer.to_string();

            // 既存ファイルとの差分を DiffLog に記録
            if path.exists()
                && let Ok(old_content) = std::fs::read_to_string(&path)
            {
                let _ = vivivi_modes::DiffLog::record_generation(
                    &path,
                    &old_content,
                    &content,
                    "2026-10-14 14:00:00",
                    "auto-save",
                );
            }

            let bytes = vivivi_core::encode_text(&content, self.encoding, self.newline);
            std::fs::write(&path, bytes)?;
            self.buffer.on_saved();

            // 変更行ガター追跡の保存状態を更新
            let current_lines: Vec<String> = (0..self.buffer.line_count())
                .filter_map(|i| self.buffer.get_line(i))
                .collect();
            self.change_tracker.on_save(&current_lines);

            // 最近使ったファイルリストに追加・永続化
            self.recent_manager.add_recent(path.clone());
            let recent_file_path = get_recent_file_path();
            let _ = self.recent_manager.save_to_file(&recent_file_path);

            // 永続Undoを保存
            let _ = self.buffer.save_undofile(&path);

            // 保存完了に伴いスワップファイルをクリーンアップ
            self.remove_swap_file();

            self.syntax_lang = SyntaxLang::from_path(&path);
            self.refresh_outline();

            self.status_message = Some(format!(
                "\"{path:?}\" 書込完了 [{}/{}] (DiffLog & Undo永続化)",
                self.encoding.name(),
                self.newline.name()
            ));
            Ok(())
        } else {
            self.status_message =
                Some("ファイル名が指定されていません (:w <ファイル名>)".to_string());
            Ok(())
        }
    }

    /// 指定されたパスのファイルを読み込み、現在のエディタバッファを切り替えます。
    ///
    /// 文字コードおよび改行コードを自動判別し、拡張子に応じたシンタックスハイライト、
    /// CSV整列ビュー、永続Undoログ（`.undodir`）の復元、最近使ったファイル履歴の追加を行います。
    ///
    /// # 引数
    /// - `path`: 開くファイルのパス
    ///
    /// # Errors
    /// ファイルの読み込み権限不足など、I/O エラーが発生した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use std::path::PathBuf;
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// let _ = app.open_file(PathBuf::from("src/main.rs"));
    /// ```
    pub fn open_file(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        self.open_file_with_encoding(path, None)
    }

    /// 指定された文字エンコーディング（省略時は自動判別）でファイルを開きます。
    pub fn open_file_with_encoding(
        &mut self,
        path: PathBuf,
        forced_encoding: Option<CharEncoding>,
    ) -> Result<(), std::io::Error> {
        let (content, enc, nl) = if path.exists() {
            let bytes = std::fs::read(&path)?;
            if let Some(forced) = forced_encoding {
                vivivi_core::decode_bytes_with_encoding(&bytes, forced)
            } else {
                vivivi_core::decode_bytes(&bytes)
            }
        } else {
            (
                String::new(),
                forced_encoding.unwrap_or(CharEncoding::Utf8),
                NewlineType::default(),
            )
        };

        self.buffer = TextBuffer::from_text(&content);
        let _ = self.buffer.load_undofile(&path);

        let lines: Vec<String> = (0..self.buffer.line_count())
            .filter_map(|i| self.buffer.get_line(i))
            .collect();
        self.change_tracker = ChangeTracker::new(&lines);

        let is_csv = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "csv" | "tsv"))
            .unwrap_or(false);
        self.is_csv_active = is_csv;

        self.syntax_lang = SyntaxLang::from_path(&path);
        self.vi.current_file_path = path.to_string_lossy().to_string();
        self.file_path = Some(path.clone());
        self.encoding = enc;
        self.newline = nl;
        self.cursor = TextCursor::new(0, 0);
        self.scroll = (0, 0);

        self.recent_manager.add_recent(path);
        let recent_file_path = get_recent_file_path();
        let _ = self.recent_manager.save_to_file(&recent_file_path);

        self.status_message = Some(format!(
            "ファイルを開きました [{}/{}]: {:?}",
            self.encoding.name(),
            self.newline.name(),
            self.file_path
        ));
        Ok(())
    }

    /// 現在開いているファイルを、指定された文字エンコーディングで再読み込みします。
    pub fn reload_with_encoding(&mut self, enc: CharEncoding) -> Result<(), std::io::Error> {
        if let Some(path) = self.file_path.clone() {
            self.open_file_with_encoding(path, Some(enc))
        } else {
            self.encoding = enc;
            self.status_message = Some(format!(
                "文字エンコーディングを設定しました: {}",
                enc.name()
            ));
            Ok(())
        }
    }

    /// 現在開いているファイルのお気に入り登録／解除をトグル（切り替え）します。
    ///
    /// 無名バッファの場合はエラーメッセージをステータスラインに表示します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.toggle_favorite_current_file();
    /// ```
    pub fn toggle_favorite_current_file(&mut self) {
        if let Some(ref path) = self.file_path {
            let is_fav = self.recent_manager.toggle_favorite(path.clone());
            let recent_file_path = get_recent_file_path();
            let _ = self.recent_manager.save_to_file(&recent_file_path);
            if is_fav {
                self.status_message = Some(format!("お気に入りに追加: {:?}", path));
            } else {
                self.status_message = Some(format!("お気に入りを解除: {:?}", path));
            }
        } else {
            self.status_message =
                Some("名前のないバッファはお気に入りに登録できません".to_string());
        }
    }

    /// 現在カーソル行の罫線枠内テキストを指定方向（左揃え・中央揃え・右揃え）に整列します。
    ///
    /// # 引数
    /// - `alignment`: 整列方向（[`BoxAlignment`]）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    /// use vivivi_modes::BoxAlignment;
    ///
    /// let mut app = App::new("│ text │", None, Default::default(), Default::default());
    /// app.align_keisen_box(BoxAlignment::Center);
    /// ```
    pub fn align_keisen_box(&mut self, alignment: BoxAlignment) {
        let cur_line = self.cursor_pos().line;
        if KeisenMode::align_box_text(&mut self.buffer, cur_line, alignment) {
            self.status_message = Some(format!("行 {cur_line} の罫線枠内テキストを整列しました"));
        } else {
            self.status_message = Some("罫線枠が検出されませんでした".to_string());
        }
    }

    /// 罫線モードで指定方向に1セル進み、罫線の描画または消去を実行します。
    ///
    /// # 引数
    /// - `dir`: 描画方向（[`KeisenDirection`]）
    /// - `is_erase`: `true` の場合、罫線を消去
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    /// use vivivi_modes::KeisenDirection;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.step_keisen(KeisenDirection::Right, false);
    /// ```
    pub fn step_keisen(&mut self, dir: KeisenDirection, is_erase: bool) {
        self.keisen_mode
            .step(&mut self.buffer, &mut self.cursor.pos, dir, is_erase);
    }

    /// 指定された行・列座標における画面表示カラム位置（全角幅2、半角幅1、タブ幅を考慮）を計算します。
    pub fn visual_col_at(&self, line: usize, col: usize) -> usize {
        let Some(line_str) = self.buffer.get_line(line) else {
            return col;
        };
        let mut visual_x = 0;
        for (i, ch) in line_str.chars().enumerate() {
            if i >= col {
                break;
            }
            if ch == '\t' {
                visual_x += 4 - (visual_x % 4);
            } else {
                visual_x += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            }
        }
        visual_x
    }

    /// 現在の表示領域の高さ・幅に合わせてエディタ画面のスクロールオフセットを自動調整します。
    ///
    /// カーソルが常に可視領域内に収まるよう、垂直・水平スクロール位置をクランプします。
    ///
    /// # 引数
    /// - `view_height`: 可視行数
    /// - `view_width`: 可視カラム幅
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.adjust_scroll(24, 80);
    /// ```
    pub fn adjust_scroll(&mut self, view_height: usize, view_width: usize) {
        let cur = self.cursor_pos();

        // 垂直スクロール
        if cur.line < self.scroll.0 {
            self.scroll.0 = cur.line;
        } else if cur.line >= self.scroll.0 + view_height {
            self.scroll.0 = cur.line - view_height + 1;
        }

        // 水平スクロール（全角・タブ幅を考慮した表示カラム幅基準）
        let cur_visual_col = self.visual_col_at(cur.line, cur.col);
        if cur_visual_col < self.scroll.1 {
            self.scroll.1 = cur_visual_col;
        } else if cur_visual_col >= self.scroll.1 + view_width {
            self.scroll.1 = cur_visual_col - view_width + 1;
        }

        // アクティブウィンドウの状態と同期
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }
    }

    /// 半画面分（10行）カーソルを上下にスクロール移動します (`Ctrl-D` / `Ctrl-U` 相当)。
    ///
    /// # 引数
    /// - `down`: `true` で下方向、`false` で上方向
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("line1\nline2", None, Default::default(), Default::default());
    /// app.scroll_half_page(true);
    /// ```
    pub fn scroll_half_page(&mut self, down: bool) {
        let step = 10usize;
        let total_lines = self.buffer.line_count();
        if down {
            self.cursor.pos.line = (self.cursor.pos.line + step).min(total_lines.saturating_sub(1));
        } else {
            self.cursor.pos.line = self.cursor.pos.line.saturating_sub(step);
        }
        self.clamp_cursor();
    }

    /// 1画面分（20行）カーソルを上下にスクロール移動します (`Ctrl-F` / `Ctrl-B`, `PageDown` / `PageUp` 相当)。
    ///
    /// # 引数
    /// - `down`: `true` で下方向、`false` で上方向
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("line1\nline2", None, Default::default(), Default::default());
    /// app.scroll_full_page(true);
    /// ```
    pub fn scroll_full_page(&mut self, down: bool) {
        let step = 20usize;
        let total_lines = self.buffer.line_count();
        if down {
            self.cursor.pos.line = (self.cursor.pos.line + step).min(total_lines.saturating_sub(1));
        } else {
            self.cursor.pos.line = self.cursor.pos.line.saturating_sub(step);
        }
        self.clamp_cursor();
    }

    /// カーソルの列位置を現在行の有効な文字境界内にクランプします。
    ///
    /// 挿入モード時は行末文字の直後まで、ノーマルモード時は末尾文字上に制限されます。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("abc", None, Default::default(), Default::default());
    /// app.cursor.pos.col = 100;
    /// app.clamp_cursor();
    /// assert!(app.cursor.pos.col < 100);
    /// ```
    pub fn clamp_cursor(&mut self) {
        let line_len = self
            .buffer
            .line_text(self.cursor.pos.line)
            .map(|s| s.chars().count())
            .unwrap_or(0);
        let max_col = if self.mode() == ViMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        self.cursor.pos.col = self.cursor.pos.col.min(max_col);
    }

    // --- 画面分割 (Split / Vsplit) 操作 ---

    /// 画面を上下（水平）に2分割し、新しいウィンドウを作成します (`:split` または `Ctrl-W s` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_horizontal();
    /// assert_eq!(app.windows.len(), 2);
    /// ```
    pub fn split_horizontal(&mut self) {
        // 現在のアクティブウィンドウの最新状態を退避
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }

        let cur = self.cursor.clone();
        let scroll = self.scroll;
        let new_idx = self.windows.len();
        self.split_axis = SplitAxis::Horizontal;
        self.windows.push(EditorWindow {
            cursor: cur,
            scroll,
        });
        self.window_tree
            .split_window(self.active_window, new_idx, SplitAxis::Horizontal);
        self.active_window = new_idx;
        self.status_message = Some(format!(
            "水平分割: ウィンドウ {}/{}",
            self.active_window + 1,
            self.windows.len()
        ));
    }

    /// 画面を左右（垂直）に2分割し、新しいウィンドウを作成します (`:vsplit` または `Ctrl-W v` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_vertical();
    /// assert_eq!(app.windows.len(), 2);
    /// ```
    pub fn split_vertical(&mut self) {
        // 現在のアクティブウィンドウの最新状態を退避
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }

        let cur = self.cursor.clone();
        let scroll = self.scroll;
        let new_idx = self.windows.len();
        self.split_axis = SplitAxis::Vertical;
        self.windows.push(EditorWindow {
            cursor: cur,
            scroll,
        });
        self.window_tree
            .split_window(self.active_window, new_idx, SplitAxis::Vertical);
        self.active_window = new_idx;
        self.status_message = Some(format!(
            "垂直分割: ウィンドウ {}/{}",
            self.active_window + 1,
            self.windows.len()
        ));
    }

    /// 指定されたインデックスのウィンドウへフォーカスを切り替えます。
    pub fn switch_to_window(&mut self, target_idx: usize) {
        if target_idx < self.windows.len() && target_idx != self.active_window {
            // 現在のウィンドウ状態を退避
            self.windows[self.active_window].cursor = self.cursor.clone();
            self.windows[self.active_window].scroll = self.scroll;

            self.active_window = target_idx;
            self.cursor = self.windows[self.active_window].cursor.clone();
            self.scroll = self.windows[self.active_window].scroll;
            self.status_message = Some(format!(
                "ウィンドウ: {}/{}",
                self.active_window + 1,
                self.windows.len()
            ));
        }
    }

    /// フォーカスを次のウィンドウへ巡回移動します (`Ctrl-W w` 相当)。
    pub fn cycle_window(&mut self) {
        if self.windows.len() > 1 {
            let next_idx = (self.active_window + 1) % self.windows.len();
            self.switch_to_window(next_idx);
        }
    }

    /// フォーカスを前のウィンドウへ逆巡回移動します (`Ctrl-W W` 相当)。
    pub fn cycle_window_prev(&mut self) {
        if self.windows.len() > 1 {
            let prev_idx = if self.active_window == 0 {
                self.windows.len() - 1
            } else {
                self.active_window - 1
            };
            self.switch_to_window(prev_idx);
        }
    }

    /// 幾何学的な画面位置に基づいて、指定方向（上下左右）の隣接ウィンドウへフォーカスを移動します (`Ctrl-W h/j/k/l` 相当)。
    pub fn navigate_window(&mut self, dir: WindowDirection) {
        if self.windows.len() <= 1 {
            return;
        }

        let cur_idx = self.active_window;
        let cur_rect = self
            .window_rects
            .iter()
            .find(|(idx, _)| *idx == cur_idx)
            .map(|(_, r)| *r);

        let Some(cur_rect) = cur_rect else {
            // 矩形キャッシュがない場合のフォールバック
            match dir {
                WindowDirection::Left | WindowDirection::Up => self.cycle_window_prev(),
                WindowDirection::Right | WindowDirection::Down => self.cycle_window(),
            }
            return;
        };

        let mut best_target = None;
        let mut best_score = u32::MAX;

        for &(idx, rect) in &self.window_rects {
            if idx == cur_idx {
                continue;
            }

            let is_in_direction = match dir {
                WindowDirection::Left => {
                    rect.x + rect.width <= cur_rect.x + 1 && rect.x < cur_rect.x
                }
                WindowDirection::Right => {
                    rect.x + 1 >= cur_rect.x + cur_rect.width || rect.x > cur_rect.x
                }
                WindowDirection::Up => {
                    rect.y + rect.height <= cur_rect.y + 1 && rect.y < cur_rect.y
                }
                WindowDirection::Down => {
                    rect.y + 1 >= cur_rect.y + cur_rect.height || rect.y > cur_rect.y
                }
            };

            if is_in_direction {
                let score = match dir {
                    WindowDirection::Left => {
                        let dx = (cur_rect.x as i32 - (rect.x + rect.width) as i32).unsigned_abs();
                        let cy_cur = cur_rect.y as i32 + (cur_rect.height as i32 / 2);
                        let cy_rect = rect.y as i32 + (rect.height as i32 / 2);
                        let dy = (cy_cur - cy_rect).unsigned_abs();
                        dx * 10 + dy
                    }
                    WindowDirection::Right => {
                        let dx =
                            (rect.x as i32 - (cur_rect.x + cur_rect.width) as i32).unsigned_abs();
                        let cy_cur = cur_rect.y as i32 + (cur_rect.height as i32 / 2);
                        let cy_rect = rect.y as i32 + (rect.height as i32 / 2);
                        let dy = (cy_cur - cy_rect).unsigned_abs();
                        dx * 10 + dy
                    }
                    WindowDirection::Up => {
                        let dy = (cur_rect.y as i32 - (rect.y + rect.height) as i32).unsigned_abs();
                        let cx_cur = cur_rect.x as i32 + (cur_rect.width as i32 / 2);
                        let cx_rect = rect.x as i32 + (rect.width as i32 / 2);
                        let dx = (cx_cur - cx_rect).unsigned_abs();
                        dy * 10 + dx
                    }
                    WindowDirection::Down => {
                        let dy =
                            (rect.y as i32 - (cur_rect.y + cur_rect.height) as i32).unsigned_abs();
                        let cx_cur = cur_rect.x as i32 + (cur_rect.width as i32 / 2);
                        let cx_rect = rect.x as i32 + (rect.width as i32 / 2);
                        let dx = (cx_cur - cx_rect).unsigned_abs();
                        dy * 10 + dx
                    }
                };

                if score < best_score {
                    best_score = score;
                    best_target = Some(idx);
                }
            }
        }

        if let Some(target) = best_target {
            self.switch_to_window(target);
        }
    }

    /// 現在フォーカスされているウィンドウを閉じます (`:close` または `Ctrl-W c` 相当)。
    ///
    /// 最後の1ウィンドウの場合はウィンドウを閉じず、未保存変更がなければ終了フラグを立てます。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_horizontal();
    /// app.close_window();
    /// assert_eq!(app.windows.len(), 1);
    /// ```
    pub fn close_window(&mut self) {
        if self.windows.len() > 1 {
            let target_idx = self.active_window;
            self.window_tree.remove_window(target_idx);
            self.window_tree.adjust_indices_after_remove(target_idx);
            self.windows.remove(target_idx);
            if self.active_window >= self.windows.len() {
                self.active_window = self.windows.len() - 1;
            }
            self.cursor = self.windows[self.active_window].cursor.clone();
            self.scroll = self.windows[self.active_window].scroll;
            self.status_message = Some(format!("ウィンドウ閉鎖 (残り: {})", self.windows.len()));
        } else if self.buffer.is_modified() {
            self.status_message = Some("未保存の変更があります (:q! で強制終了)".to_string());
        } else {
            self.should_quit = true;
        }
    }

    /// 現在フォーカスされているウィンドウ以外をすべて閉じ、単一ウィンドウに統合します (`:only` または `Ctrl-W o` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_horizontal();
    /// app.split_vertical();
    /// app.only_window();
    /// assert_eq!(app.windows.len(), 1);
    /// ```
    pub fn only_window(&mut self) {
        let cur = self.cursor.clone();
        let scroll = self.scroll;
        self.windows = vec![EditorWindow {
            cursor: cur,
            scroll,
        }];
        self.active_window = 0;
        self.window_tree = WindowTreeNode::new(0);
        self.status_message = Some("単一ウィンドウに統合しました".to_string());
    }

    // --- アウトライン解析操作 ---

    /// アウトラインサイドペインの表示状態をトグル（切り替え）します (`:outline` または `F8` 相当)。
    ///
    /// 表示を有効化した場合はバッファ内容を自動再解析します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("# Heading", None, Default::default(), Default::default());
    /// app.toggle_outline();
    /// assert!(app.show_outline);
    /// ```
    pub fn toggle_outline(&mut self) {
        self.show_outline = !self.show_outline;
        if self.show_outline {
            self.refresh_outline();
            self.outline_focus = true;
            self.status_message = Some(
                "アウトライン: ON (j/k:選択, Enter:ジャンプ, Tab/Esc:エディタ復帰, F2/F8:閉じる)"
                    .to_string(),
            );
        } else {
            self.outline_focus = false;
            self.status_message = Some("アウトライン: OFF".to_string());
        }
    }

    /// 現在のバッファ内容をシンタックス言語に基づいて再解析し、アウトライン項目一覧を更新します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("fn main() {}", None, Default::default(), Default::default());
    /// app.refresh_outline();
    /// ```
    pub fn refresh_outline(&mut self) {
        let lines: Vec<String> = (0..self.buffer.line_count())
            .filter_map(|i| self.buffer.line_text(i))
            .collect();
        // 無名バッファ等でPlainTextの場合、内容から言語を自動検出してsyntax_langも同期
        if self.file_path.is_none()
            && self.syntax_lang == SyntaxLang::PlainText
            && let Some(detected) = vivivi_modes::detect_language_from_content(&lines)
        {
            self.syntax_lang = detected;
        }
        self.outline_items = parse_outline(&lines, self.syntax_lang);
        if self.selected_outline_idx >= self.outline_items.len() {
            self.selected_outline_idx = 0;
        }
    }

    /// サイドペインで現在選択されているアウトライン項目の定義行へカーソルをジャンプさせます。
    ///
    /// ジャンプ完了後、アウトラインペインのフォーカスを解除してエディタバッファへ戻します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("fn main() {}", None, Default::default(), Default::default());
    /// app.refresh_outline();
    /// app.jump_to_selected_outline();
    /// ```
    pub fn jump_to_selected_outline(&mut self) {
        if let Some(item) = self.outline_items.get(self.selected_outline_idx) {
            let target_line = item
                .line_idx
                .min(self.buffer.line_count().saturating_sub(1));
            self.cursor.pos.line = target_line;
            self.cursor.pos.col = 0;
            self.outline_focus = false;
            self.status_message =
                Some(format!("ジャンプ: 行 {} - {}", target_line + 1, item.title));
        }
    }

    // --- 文字変換操作 ---

    /// バッファ全文に対して指定された文字種変換（全角/半角、ひらがな/カタカナ等）を実行します (`:zenkaku`, `:hankaku` 等)。
    ///
    /// 操作全体は単一のアンドゥブロックとして記録され、1回のアンドゥ操作で復元できます。
    ///
    /// # 引数
    /// - `conv_type`: 適用する変換ルール（[`ConversionType`]）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    /// use vivivi_modes::ConversionType;
    ///
    /// let mut app = App::new("hello", None, Default::default(), Default::default());
    /// app.convert_buffer_text(ConversionType::ToUppercase);
    /// assert_eq!(app.buffer.line_text(0).unwrap(), "HELLO");
    /// ```
    pub fn convert_buffer_text(&mut self, conv_type: ConversionType) {
        self.buffer.open_undo_block();
        let mut count = 0;
        for line in 0..self.buffer.line_count() {
            if let Some(line_str) = self.buffer.line_text(line) {
                let converted = convert_text(&line_str, conv_type);
                if converted != line_str {
                    let start_pos = self.buffer.line_start_position(line);
                    let old_len = line_str.chars().count();
                    self.buffer.delete_text(start_pos, old_len, false);
                    self.buffer.insert_text(start_pos, &converted);
                    count += 1;
                }
            }
        }
        self.buffer.close_undo_block();
        self.status_message = Some(format!("文字変換完了 ({count} 行変更)"));
    }

    // --- BOX選択数値集計 (:calc / :sum) ---

    /// 現在のBOX選択（矩形選択）範囲内の数値を抽出し、合計・平均・最大・最小・標準偏差を集計します (`:calc` / `:sum`)。
    ///
    /// 集計結果はステータスラインに表示され、合計値（Sum）は自動的に無名レジスタ（`"`）にコピーされます。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("10\n20\n30", None, Default::default(), Default::default());
    /// app.calculate_current_box_selection();
    /// ```
    pub fn calculate_current_box_selection(&mut self) {
        let anchor = self.cursor.anchor.unwrap_or(self.cursor.pos);
        let pos = self.cursor.pos;

        let min_line = anchor.line.min(pos.line);
        let max_line = anchor.line.max(pos.line);
        let min_col = anchor.col.min(pos.col);
        let max_col = anchor.col.max(pos.col);

        if let Some(result) =
            calculate_box_selection(&self.buffer, min_line, max_line, min_col, max_col)
        {
            let summary = result.formatted_summary();
            // 合計値を無名レジスタにコピー
            let sum_str = format!("{}", result.sum);
            self.vi
                .registers_mut()
                .set(None, RegisterContent::CharacterWise(sum_str));
            self.status_message = Some(format!("{summary} [合計値をレジスタにコピーしました]"));
        } else {
            self.status_message =
                Some("BOX選択範囲から有効な数値を検出できませんでした".to_string());
        }
    }

    // --- 動的補完 (dabbrev / Ctrl-L, Ctrl-M) ---

    /// 動的単語補完（dabbrev）を実行します (`Ctrl-L` で次候補、`Ctrl-M` で前候補)。
    ///
    /// 初回呼び出し時はカーソル直前の単語プレフィックスからバッファ全体を走査して候補一覧を生成し、
    /// 2回目以降は補完文字列を循環的に差し替えます。
    ///
    /// # 引数
    /// - `is_forward`: `true` で次の候補へ進む、`false` で前の候補へ戻る
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("function_apple function_ap", None, Default::default(), Default::default());
    /// app.cursor.pos.col = 26;
    /// app.trigger_dabbrev(true);
    /// ```
    pub fn trigger_dabbrev(&mut self, is_forward: bool) {
        let cur = self.cursor.pos;
        if !self.dabbrev.is_active() {
            // 初回開始 (前方に検索)
            let opt_candidate = self
                .dabbrev
                .start(&self.buffer, cur.line, cur.col)
                .map(|s| s.to_string());
            if let Some(candidate) = opt_candidate {
                // プレフィックスの後に続く残りの文字列を挿入
                let prefix_len = self.dabbrev.prefix.chars().count();
                let candidate_chars: Vec<char> = candidate.chars().collect();
                let remaining: String = candidate_chars[prefix_len..].iter().collect();

                let insert_pos = self.buffer.cursor_to_position(cur);
                self.buffer.insert_text(insert_pos, &remaining);
                self.cursor.pos.col += remaining.chars().count();

                let total = self.dabbrev.candidates.len();
                self.status_message = Some(format!("[動的補完 1/{total}] {candidate}"));
            } else {
                self.status_message = Some("動的補完の候補が見つかりません".to_string());
            }
        } else {
            // 2回目以降: 既存の補完部分を差し替え
            let old_inserted_len = self.dabbrev.last_inserted_len;
            let next_cand = if is_forward {
                self.dabbrev.next_candidate().map(|s| s.to_string())
            } else {
                self.dabbrev.prev_candidate().map(|s| s.to_string())
            };

            if let Some(candidate) = next_cand {
                let prefix_len = self.dabbrev.prefix.chars().count();
                let candidate_chars: Vec<char> = candidate.chars().collect();
                let new_remaining: String = candidate_chars[prefix_len..].iter().collect();

                // 直前に挿入した文字を削除して新しい残りを挿入
                let cur_pos = self.buffer.cursor_to_position(self.cursor.pos);
                let del_start = cur_pos.saturating_sub(old_inserted_len);
                self.buffer.delete_text(del_start, old_inserted_len, false);
                self.buffer.insert_text(del_start, &new_remaining);

                self.cursor.pos.col = self.cursor.pos.col.saturating_sub(old_inserted_len)
                    + new_remaining.chars().count();

                let curr_idx = self.dabbrev.current_index.unwrap_or(0) + 1;
                let total = self.dabbrev.candidates.len();
                self.status_message = Some(format!("[動的補完 {curr_idx}/{total}] {candidate}"));
            }
        }
    }

    // --- クリップボード履歴 (cbHist) 操作 ---

    /// クリップボード履歴ダイアログから指定インデックスのテキストをカーソル位置に貼り付けます。
    ///
    /// 文字単位、行単位、ブロック単位（矩形）のレジスタ種別に応じた適切なペースト処理を行い、
    /// モーダルダイアログを自動的に閉じます。
    ///
    /// # 引数
    /// - `index`: クリップボード履歴リスト内のインデックス（0-based）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.paste_clipboard_item(0);
    /// ```
    pub fn paste_clipboard_item(&mut self, index: usize) {
        if let Some(item) = self.vi.registers().history().get(index).cloned() {
            let pos = self.buffer.cursor_to_position(self.cursor.pos);
            match item.content {
                RegisterContent::CharacterWise(s) => {
                    let char_count = s.chars().count();
                    self.buffer.insert_text(pos, &s);
                    self.cursor.pos.col += char_count;
                }
                RegisterContent::LineWise(s) => {
                    let next_line_start = self.buffer.line_start_position(self.cursor.pos.line + 1);
                    self.buffer.insert_text(next_line_start, &s);
                    self.cursor.pos.line += 1;
                    self.cursor.pos.col = 0;
                }
                RegisterContent::BlockWise(lines) => {
                    for (i, line) in lines.iter().enumerate() {
                        let l = self.cursor.pos.line + i;
                        if l >= self.buffer.line_count() {
                            self.buffer.insert_char(self.buffer.len(), '\n');
                        }
                        let insert_pos = self.buffer.line_start_position(l) + self.cursor.pos.col;
                        self.buffer.insert_text(insert_pos, line);
                    }
                }
            }
            self.show_clipboard_hist = false;
            self.status_message = Some(format!("履歴から貼り付けました: {}", item.preview));
        }
    }

    // --- ローマ字日本語検索 (Migemo / g/) ---

    /// ローマ字クエリを日本語正規表現パターンにコンパイルし、インクリメンタル検索を実行します (`g/` 相当)。
    ///
    /// マッチ箇所が存在する場合はカーソルを移動させ、全画面検索ハイライトを自動有効化します。
    ///
    /// # 引数
    /// - `romaji`: 入力されたローマ字文字列（例: `"kensaku"`）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("日本語の検索テスト", None, Default::default(), Default::default());
    /// app.execute_migemo_search("kensaku");
    /// ```
    pub fn execute_migemo_search(&mut self, romaji: &str) {
        match self.migemo.compile_query(romaji) {
            Ok(regex) => {
                self.migemo_last_regex = Some(regex.clone());
                self.is_search_highlight_visible = true;
                let full_text = self.buffer.to_string();
                let cur_pos = self.buffer.cursor_to_position(self.cursor.pos);
                let next_byte_offset = full_text
                    .char_indices()
                    .nth(cur_pos + 1)
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                // カーソル以降を検索、なければ先頭から検索（ラップアラウンド）
                let found_match = regex
                    .find_at(&full_text, next_byte_offset)
                    .or_else(|| regex.find(&full_text));

                if let Some(m) = found_match {
                    let char_idx = full_text[..m.start()].chars().count();
                    self.cursor.pos = self.buffer.position_to_cursor(char_idx);
                    if let Some(win) = self.windows.get_mut(self.active_window) {
                        win.cursor = self.cursor.clone();
                        win.scroll = self.scroll;
                    }
                    self.status_message = Some(format!("g/{romaji} [Migemo一致: {}]", m.as_str()));
                } else {
                    self.status_message = Some(format!("g/{romaji} [Migemo: 一致なし]"));
                }
            }
            Err(e) => {
                self.status_message = Some(format!("Migemo正規表現エラー: {e}"));
            }
        }
    }

    /// Migemo 検索結果の次の一致箇所（または前の一致箇所）へカーソルをジャンプさせます (`n` / `N` 相当)。
    ///
    /// # 引数
    /// - `is_forward`: `true` で正方向（次の一致）、`false` で逆方向（前の一致）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("検索 検索", None, Default::default(), Default::default());
    /// app.execute_migemo_search("kensaku");
    /// app.jump_next_migemo(true);
    /// ```
    pub fn jump_next_migemo(&mut self, is_forward: bool) {
        let regex = match &self.migemo_last_regex {
            Some(r) => r.clone(),
            None => return,
        };

        self.is_search_highlight_visible = true;
        let full_text = self.buffer.to_string();
        let cur_pos = self.buffer.cursor_to_position(self.cursor.pos);

        if is_forward {
            let next_byte_offset = full_text
                .char_indices()
                .nth(cur_pos + 1)
                .map(|(i, _)| i)
                .unwrap_or(0);

            if let Some(m) = regex
                .find_at(&full_text, next_byte_offset)
                .or_else(|| regex.find(&full_text))
            {
                let char_idx = full_text[..m.start()].chars().count();
                self.cursor.pos = self.buffer.position_to_cursor(char_idx);
                if let Some(win) = self.windows.get_mut(self.active_window) {
                    win.cursor = self.cursor.clone();
                    win.scroll = self.scroll;
                }
                self.status_message = Some(format!("[Migemo] 一致: {}", m.as_str()));
            }
        } else {
            // 逆方向検索: cur_byte_offset より手前の最後の一致を探索
            let cur_byte_offset = full_text
                .char_indices()
                .nth(cur_pos)
                .map(|(i, _)| i)
                .unwrap_or(full_text.len());

            let matches: Vec<_> = regex.find_iter(&full_text).collect();
            if let Some(m) = matches
                .iter()
                .rev()
                .find(|m| m.start() < cur_byte_offset)
                .or_else(|| matches.last())
            {
                let char_idx = full_text[..m.start()].chars().count();
                self.cursor.pos = self.buffer.position_to_cursor(char_idx);
                if let Some(win) = self.windows.get_mut(self.active_window) {
                    win.cursor = self.cursor.clone();
                    win.scroll = self.scroll;
                }
                self.status_message = Some(format!("[Migemo] 一致: {}", m.as_str()));
            }
        }
    }

    /// 現在有効な検索正規表現オブジェクトを取得します。
    ///
    /// Migemo検索の正規表現が優先され、存在しない場合は通常のVi検索パターンが使用されます。
    /// `:nohlsearch` 等でハイライトが非表示化されている場合は `None` を返します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let app = App::new("test", None, Default::default(), Default::default());
    /// assert!(app.active_search_regex().is_none());
    /// ```
    pub fn active_search_regex(&self) -> Option<regex::Regex> {
        if !self.is_search_highlight_visible {
            return None;
        }
        if let Some(ref r) = self.migemo_last_regex {
            return Some(r.clone());
        }
        if let Some(pat) = self.vi.last_search_pattern()
            && !pat.is_empty()
        {
            return regex::Regex::new(pat)
                .ok()
                .or_else(|| regex::Regex::new(&regex::escape(pat)).ok());
        }
        None
    }

    /// 画面上の検索マッチハイライト表示を一時的に消去します (`:noh` / `:nohlsearch` 相当)。
    ///
    /// 次に検索を実行するか `n` / `N` を押すと自動的にハイライトが再開されます。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.clear_search_highlight();
    /// assert!(!app.is_search_highlight_visible);
    /// ```
    pub fn clear_search_highlight(&mut self) {
        self.is_search_highlight_visible = false;
    }

    // --- ヘルプビューア (:help / F1) ---

    /// ヘルプドキュメント画面をモーダルビューとして開きます (`:help` または `F1` 相当)。
    ///
    /// 指定されたトピック名（例: `"keisen"`, `"csv"`）に対応する Markdown ヘルプを探索・ロードします。
    /// トピックが省略された場合は総合インデックス（`index.md`）を表示します。
    ///
    /// # 引数
    /// - `topic`: 閲覧したいヘルプトピック名（オプション）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.open_help(Some("keisen"));
    /// assert!(app.show_help);
    /// ```
    pub fn open_help(&mut self, topic: Option<&str>) {
        let topic_name = topic.unwrap_or("index").trim();
        let file_name = if topic_name.ends_with(".md") {
            topic_name.to_string()
        } else {
            format!("{topic_name}.md")
        };

        // 探索候補パス（doc/help, docs/help, help）
        let candidates = [
            PathBuf::from("doc").join("help").join(&file_name),
            PathBuf::from("docs").join("help").join(&file_name),
            PathBuf::from("help").join(&file_name),
        ];

        let mut content = None;
        let mut loaded_path = None;
        for path in &candidates {
            if let Ok(text) = std::fs::read_to_string(path) {
                content = Some(text);
                loaded_path = Some(path.display().to_string());
                break;
            }
        }

        let text = content.unwrap_or_else(|| {
            // 外部ファイルが見つからない場合のフォールバック（埋め込みインデックス）
            include_str!("../../../../doc/help/index.md").to_string()
        });

        self.help_title = loaded_path.unwrap_or_else(|| format!("ヘルプ: {topic_name}"));
        self.help_lines = text.lines().map(|s| s.to_string()).collect();
        self.help_scroll = 0;
        self.show_help = true;
        self.status_message = Some(format!(
            "ヘルプを開きました (j/k:スクロール, q/Esc:閉じる): {topic_name}"
        ));
    }

    // --- 編集支援: 対象行範囲取得 ---

    /// ビジュアル選択範囲またはバッファ全体の対象行インデックス範囲 `(start_line, end_line)` を取得します。
    ///
    /// ビジュアルモード中の場合はアンカー位置と現在カーソル位置から選択範囲の開始行・終了行を算出し、
    /// それ以外の場合はバッファの先頭行（0）から末尾行までの全行範囲を返します。
    ///
    /// # 戻り値
    /// `(開始行, 終了行)` （0-based、inclusive）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let app = App::new("line 1\nline 2", None, Default::default(), Default::default());
    /// assert_eq!(app.get_target_line_range(), (0, 1));
    /// ```
    pub fn get_target_line_range(&self) -> (usize, usize) {
        if self.vi.mode().is_visual()
            && let Some(anchor) = self.cursor.anchor
        {
            let start = anchor.line.min(self.cursor.pos.line);
            let end = anchor.line.max(self.cursor.pos.line);
            return (start, end);
        }
        let total = self.buffer.line_count();
        if total == 0 { (0, 0) } else { (0, total - 1) }
    }

    // --- テキストアライン (:align <target>) ---

    /// 指定した区切り文字列の位置で対象行範囲のテキストを垂直整列します (`:align <target>` 相当)。
    ///
    /// 選択範囲内の各行で最初に出現する `target` 文字列の位置を揃えるよう空白を挿入し、
    /// 実行後はノーマルモードへ戻ります。操作全体は単一のアンドゥブロックとして記録されます。
    ///
    /// # 引数
    /// - `target`: 整列基準となる区切り文字列（例: `"="`, `":"`, `","`）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("a = 1\nbbb = 2", None, Default::default(), Default::default());
    /// app.align_selection_or_buffer("=");
    /// ```
    pub fn align_selection_or_buffer(&mut self, target: &str) {
        if target.is_empty() || self.buffer.line_count() == 0 {
            return;
        }
        let (start_line, end_line) = self.get_target_line_range();
        let mut lines = Vec::new();
        for l in start_line..=end_line {
            lines.push(self.buffer.line_text(l).unwrap_or_default());
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let aligned = align_lines(&line_refs, target);

        self.buffer.open_undo_block();
        for (i, new_line) in aligned.into_iter().enumerate() {
            let cur_line = start_line + i;
            let old_line_len = self
                .buffer
                .line_text(cur_line)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line, 0));
            let end_pos = start_pos + old_line_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &new_line);
        }
        self.buffer.close_undo_block();

        self.vi.set_mode(ViMode::Normal);
        self.cursor.anchor = None;
        self.status_message = Some(format!(
            "テキストアライン実行: '{}' ({}〜{} 行)",
            target,
            start_line + 1,
            end_line + 1
        ));
    }

    // --- リナンバ (:renumber [start] [step]) ---

    /// 対象行範囲の箇条書き・リスト連番を自動検出し、連番を再計算・更新します (`:renumber` 相当)。
    ///
    /// `1.`, `2)`, `[3]` などの連番パターンを検出し、開始番号 `start` と増分 `step` に応じて連番を振り直します。
    ///
    /// # 引数
    /// - `start`: 開始番号
    /// - `step`: 加算ステップ数
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("1. first\n1. second", None, Default::default(), Default::default());
    /// app.renumber_selection_or_buffer(1, 1);
    /// ```
    pub fn renumber_selection_or_buffer(&mut self, start: usize, step: usize) {
        if self.buffer.line_count() == 0 {
            return;
        }
        let (start_line, end_line) = self.get_target_line_range();
        let mut lines = Vec::new();
        for l in start_line..=end_line {
            lines.push(self.buffer.line_text(l).unwrap_or_default());
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let renumbered = renumber_lines(&line_refs, start, step);

        self.buffer.open_undo_block();
        for (i, new_line) in renumbered.into_iter().enumerate() {
            let cur_line = start_line + i;
            let old_line_len = self
                .buffer
                .line_text(cur_line)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line, 0));
            let end_pos = start_pos + old_line_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &new_line);
        }
        self.buffer.close_undo_block();

        self.vi.set_mode(ViMode::Normal);
        self.cursor.anchor = None;
        self.status_message = Some(format!(
            "リナンバ実行: 開始={}, 増分={} ({}〜{} 行)",
            start,
            step,
            start_line + 1,
            end_line + 1
        ));
    }

    // --- 行頭行末一括挿入 (:insbegend, :insbeg, :insend) ---

    /// 対象行範囲の行頭および行末に文字列を一括挿入します (`:insbegend` 相当)。
    ///
    /// インデント維持フラグ（`keep_indent`）や空行スキップフラグ（`skip_empty`）をサポートし、
    /// コメントアウトやマークダウン箇条書き化などを高速に行えます。
    ///
    /// # 引数
    /// - `beg`: 行頭に挿入する文字列
    /// - `end`: 行末に挿入する文字列
    /// - `keep_indent`: `true` の場合、既存インデントの直後に `beg` を挿入
    /// - `skip_empty`: `true` の場合、空行をスキップ
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("item 1\nitem 2", None, Default::default(), Default::default());
    /// app.insert_beg_end_in_range("- ", "", true, true);
    /// ```
    pub fn insert_beg_end_in_range(
        &mut self,
        beg: &str,
        end: &str,
        keep_indent: bool,
        skip_empty: bool,
    ) {
        if self.buffer.line_count() == 0 {
            return;
        }
        let (start_line, end_line) = self.get_target_line_range();
        let mut lines = Vec::new();
        for l in start_line..=end_line {
            lines.push(self.buffer.line_text(l).unwrap_or_default());
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let edited = insert_beg_end_lines(&line_refs, beg, end, keep_indent, skip_empty);

        self.buffer.open_undo_block();
        for (i, new_line) in edited.into_iter().enumerate() {
            let cur_line = start_line + i;
            let old_line_len = self
                .buffer
                .line_text(cur_line)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line, 0));
            let end_pos = start_pos + old_line_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &new_line);
        }
        self.buffer.close_undo_block();

        self.vi.set_mode(ViMode::Normal);
        self.cursor.anchor = None;
        self.status_message = Some(format!(
            "行頭行末挿入実行: beg='{}', end='{}' ({}〜{} 行)",
            beg,
            end,
            start_line + 1,
            end_line + 1
        ));
    }

    // --- 静的略語展開 (abbrev / Ctrl-;) ---

    /// カーソル直前の単語を検出し、略語辞書から定型コード・スニペットを展開します (`Ctrl-;` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("fnm", None, Default::default(), Default::default());
    /// app.cursor.pos.col = 3;
    /// app.trigger_abbrev_expansion();
    /// ```
    pub fn trigger_abbrev_expansion(&mut self) {
        let cur_line_idx = self.cursor.pos.line;
        let line_text = match self.buffer.line_text(cur_line_idx) {
            Some(t) => t,
            None => return,
        };
        let chars: Vec<char> = line_text.chars().collect();
        let col = self.cursor.pos.col.min(chars.len());

        let mut word_start = col;
        while word_start > 0 {
            let c = chars[word_start - 1];
            if c.is_alphanumeric() || c == '_' {
                word_start -= 1;
            } else {
                break;
            }
        }

        let word: String = chars[word_start..col].iter().collect();
        if word.is_empty() {
            self.status_message = Some("略語展開: カーソル直前に単語がありません".to_string());
            return;
        }

        if let Some(snippet) = self.abbrev.expand(&word) {
            let snippet = snippet.to_string();
            let word_len = word.chars().count();
            self.buffer.open_undo_block();

            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line_idx, word_start));
            let end_pos = start_pos + word_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &snippet);
            self.buffer.close_undo_block();

            let lines: Vec<&str> = snippet.lines().collect();
            if lines.len() <= 1 {
                self.cursor.pos.col = word_start + snippet.chars().count();
            } else {
                self.cursor.pos.line = cur_line_idx + lines.len() - 1;
                let last_line = lines.last().unwrap_or(&"");
                self.cursor.pos.col = last_line.chars().count();
            }

            self.status_message = Some(format!("略語展開成功: '{}'", word));
        } else {
            self.status_message = Some(format!("略語が見つかりません: '{}'", word));
        }
    }

    // --- Zen-Coding (Emmet) 展開 (Ctrl-,) ---

    /// カーソル直前の短縮記法（`ul>li*3` 等）を検出し、HTMLタグ構造へ展開します (`Ctrl-,` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("div>p", None, Default::default(), Default::default());
    /// app.cursor.pos.col = 5;
    /// app.trigger_zen_coding();
    /// ```
    pub fn trigger_zen_coding(&mut self) {
        let cur_line_idx = self.cursor.pos.line;
        let line_text = match self.buffer.line_text(cur_line_idx) {
            Some(t) => t,
            None => return,
        };
        let chars: Vec<char> = line_text.chars().collect();
        let col = self.cursor.pos.col.min(chars.len());

        let mut zen_start = col;
        while zen_start > 0 {
            let c = chars[zen_start - 1];
            if c.is_whitespace() {
                break;
            }
            zen_start -= 1;
        }

        let zen_str: String = chars[zen_start..col].iter().collect();
        if zen_str.is_empty() {
            self.status_message = Some("Zen-Coding: 展開対象の記述がありません".to_string());
            return;
        }

        if let Some(expanded) = expand_zen_coding(&zen_str) {
            let query_len = zen_str.chars().count();
            self.buffer.open_undo_block();

            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line_idx, zen_start));
            let end_pos = start_pos + query_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &expanded);
            self.buffer.close_undo_block();

            let lines: Vec<&str> = expanded.lines().collect();
            if lines.len() <= 1 {
                self.cursor.pos.col = zen_start + expanded.chars().count();
            } else {
                self.cursor.pos.line = cur_line_idx + lines.len() - 1;
                let last_line = lines.last().unwrap_or(&"");
                self.cursor.pos.col = last_line.chars().count();
            }

            self.status_message = Some(format!("Zen-Coding展開: '{}'", zen_str));
        } else {
            self.status_message = Some(format!("Zen-Coding構文エラー: '{}'", zen_str));
        }
    }

    // --- ダイナミックマクロ (Dynamic Macro / Alt-M) ---

    /// 直前の入力・編集履歴から増井式反復規則を検出し、マクロを自動補完・実行します (`Alt-M` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.trigger_dynamic_macro();
    /// ```
    pub fn trigger_dynamic_macro(&mut self) {
        if let Some(chars) = self.dynamic_macro.detect_repeat() {
            let seq_str: String = chars.iter().collect();
            self.buffer.open_undo_block();
            for &ch in &chars {
                let pos = self.buffer.cursor_to_position(self.cursor.pos);
                self.buffer.insert_char(pos, ch);
                if ch == '\n' {
                    self.cursor.pos.line += 1;
                    self.cursor.pos.col = 0;
                } else {
                    self.cursor.pos.col += 1;
                }
            }
            self.buffer.close_undo_block();
            self.status_message = Some(format!("ダイナミックマクロ実行: '{}'", seq_str));
        } else {
            self.status_message =
                Some("ダイナミックマクロ: 反復規則を検出できませんでした".to_string());
        }
    }

    // --- 字句編集予測 (LexEdit / Ctrl-J) ---

    /// カーソル位置の単語に対して過去の置換履歴から次の編集パターンを予測し適用します (`Ctrl-J` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("old_fn", None, Default::default(), Default::default());
    /// app.trigger_lex_edit();
    /// ```
    pub fn trigger_lex_edit(&mut self) {
        let cur_line_idx = self.cursor.pos.line;
        let line_text = match self.buffer.line_text(cur_line_idx) {
            Some(t) => t,
            None => return,
        };
        let chars: Vec<char> = line_text.chars().collect();
        let col = self.cursor.pos.col.min(chars.len());

        let mut word_start = col;
        while word_start > 0
            && (chars[word_start - 1].is_alphanumeric() || chars[word_start - 1] == '_')
        {
            word_start -= 1;
        }
        let mut word_end = col;
        while word_end < chars.len()
            && (chars[word_end].is_alphanumeric() || chars[word_end] == '_')
        {
            word_end += 1;
        }

        let word: String = chars[word_start..word_end].iter().collect();
        if word.is_empty() {
            self.status_message = Some("字句編集予測: 対象単語がありません".to_string());
            return;
        }

        let candidates = self.lex_edit.predict(&word);
        if let Some(target) = candidates.first() {
            let word_len = word.chars().count();
            self.buffer.open_undo_block();

            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line_idx, word_start));
            let end_pos = start_pos + word_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, target);
            self.buffer.close_undo_block();

            self.cursor.pos.col = word_start + target.chars().count();
            self.status_message = Some(format!("字句編集予測適用: '{}' -> '{}'", word, target));
        } else {
            self.status_message = Some(format!("字句編集予測候補なし: '{}'", word));
        }
    }

    // --- キーボードマクロ (Ctrl-Alt-R / Ctrl-Alt-P) ---

    /// キーボードマクロの記録開始／停止をトグル（切り替え）します (`F9` または `Ctrl+Alt+R` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.toggle_macro_recording(); // 記録開始
    /// assert!(app.is_recording_macro);
    /// app.toggle_macro_recording(); // 記録終了
    /// assert!(!app.is_recording_macro);
    /// ```
    pub fn toggle_macro_recording(&mut self) {
        if self.is_recording_macro {
            self.is_recording_macro = false;
            self.last_macro_events = Some(self.recorded_macro_events.clone());
            let count = self.recorded_macro_events.len();
            self.status_message = Some(format!("マクロ記録終了 ({count} 件のキーイベントを保存)"));
        } else {
            self.is_recording_macro = true;
            self.recorded_macro_events.clear();
            self.status_message =
                Some("● マクロ記録中... (Ctrl+Alt+R または F9 で終了)".to_string());
        }
    }
}

/// 最近使ったファイルおよびお気に入りの永続化保存ファイルパスを取得します。
///
/// Windows では `%USERPROFILE%/.vivivi/recent.txt`、Linux/macOS では `~/.vivivi/recent.txt` を返します。
///
/// # Examples
/// ```rust,ignore
/// use vivivi_app::tui::app::get_recent_file_path;
///
/// let path = get_recent_file_path();
/// assert!(path.ends_with("recent.txt"));
/// ```
pub fn get_recent_file_path() -> PathBuf {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile)
            .join(".vivivi")
            .join("recent.txt")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".vivivi").join("recent.txt")
    } else {
        PathBuf::from(".vivivi_recent.txt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_split_and_cycle() {
        let mut app = App::new(
            "line1\nline2\nline3",
            None,
            CharEncoding::Utf8,
            NewlineType::Lf,
        );
        assert_eq!(app.windows.len(), 1);
        assert_eq!(app.active_window, 0);

        // 水平分割
        app.split_horizontal();
        assert_eq!(app.windows.len(), 2);
        assert_eq!(app.active_window, 1);
        assert_eq!(app.split_axis, SplitAxis::Horizontal);

        // 垂直分割
        app.split_vertical();
        assert_eq!(app.windows.len(), 3);
        assert_eq!(app.active_window, 2);
        assert_eq!(app.split_axis, SplitAxis::Vertical);

        // ウィンドウ巡回
        app.cycle_window();
        assert_eq!(app.active_window, 0);

        // ウィンドウ閉鎖
        app.close_window();
        assert_eq!(app.windows.len(), 2);

        // 独立スクロール・独立カーソル検証（同期スクロール防止）
        let multi_line_text = (0..100)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut app_scroll = App::new(&multi_line_text, None, CharEncoding::Utf8, NewlineType::Lf);
        assert_eq!(app_scroll.scroll, (0, 0));
        assert_eq!(app_scroll.cursor.pos.line, 0);

        // 分割して新ウィンドウを50行目にスクロール移動
        app_scroll.split_horizontal();
        assert_eq!(app_scroll.active_window, 1);
        app_scroll.cursor.pos.line = 50;
        app_scroll.adjust_scroll(20, 80);
        assert!(
            app_scroll.scroll.0 > 0,
            "アクティブウィンドウはスクロールしている"
        );

        // ウィンドウ 0 (非アクティブ) のスクロールとカーソルは同期せず (0, 0) を維持している
        assert_eq!(app_scroll.windows[0].scroll, (0, 0));
        assert_eq!(app_scroll.windows[0].cursor.pos.line, 0);

        // ウィンドウ 0 へ切り替え -> スクロール位置 (0, 0) が復元される
        app_scroll.cycle_window();
        assert_eq!(app_scroll.active_window, 0);
        assert_eq!(app_scroll.scroll, (0, 0));
        assert_eq!(app_scroll.cursor.pos.line, 0);

        // ウィンドウ 0 で10行目に移動
        app_scroll.cursor.pos.line = 10;
        app_scroll.adjust_scroll(20, 80);

        // ウィンドウ 1 へ戻る -> ウィンドウ 1 のスクロール位置が独立して保持されている
        app_scroll.cycle_window();
        assert_eq!(app_scroll.active_window, 1);
        assert!(app_scroll.scroll.0 > 0);
        assert_eq!(app_scroll.cursor.pos.line, 50);

        // cycle_window_prev でウィンドウ 0 へ戻る -> ウィンドウ 0 のカーソル位置 (10行目) が保持されている
        app_scroll.cycle_window_prev();
        assert_eq!(app_scroll.active_window, 0);
        assert_eq!(app_scroll.cursor.pos.line, 10);

        // 単一ウィンドウ統合
        app.only_window();
        assert_eq!(app.windows.len(), 1);
        assert_eq!(app.active_window, 0);
    }

    #[test]
    fn test_nested_split_vim_behavior() {
        let mut app = App::new("test vim split", None, CharEncoding::Utf8, NewlineType::Lf);
        assert_eq!(app.windows.len(), 1);
        assert_eq!(app.active_window, 0);

        // 1. :split (水平分割) -> 上: win 0, 下: win 1
        app.split_horizontal();
        assert_eq!(app.windows.len(), 2);
        assert_eq!(app.active_window, 1); // 下の win 1 がアクティブ

        // 2. 下ウィンドウ (win 1) で :vsplit (垂直分割)
        // Vim では下ウィンドウのみが左右に2分割され、全体が左・中・右の3分割にならない！
        app.split_vertical();
        assert_eq!(app.windows.len(), 3);
        assert_eq!(app.active_window, 2); // 右下の win 2 がアクティブ

        // 画面サイズ 100 x 40 で領域計算を検証
        let rects = app.window_tree.compute_rects(Rect::new(0, 0, 100, 40));
        assert_eq!(rects.len(), 3);
        // 上半分 (win 0): y=0, height=20, width=100
        assert_eq!(rects[0], (0, Rect::new(0, 0, 100, 20)));
        // 左下 (win 1): y=20, height=20, width=50
        assert_eq!(rects[1], (1, Rect::new(0, 20, 50, 20)));
        // 右下 (win 2): y=20, height=20, width=50, x=50
        assert_eq!(rects[2], (2, Rect::new(50, 20, 50, 20)));

        // 3. 幾何学的ナビゲーション (Ctrl-W h/j/k/l) の検証
        app.window_rects = rects;
        // 現在は win 2 (右下) にフォーカス
        // Left (Ctrl-W h) -> 左下 (win 1) へ移動
        app.navigate_window(WindowDirection::Left);
        assert_eq!(app.active_window, 1);

        // Up (Ctrl-W k) -> 上半分 (win 0) へ移動
        app.navigate_window(WindowDirection::Up);
        assert_eq!(app.active_window, 0);

        // Down (Ctrl-W j) -> 下 (win 1) へ移動
        app.navigate_window(WindowDirection::Down);
        assert_eq!(app.active_window, 1);

        // Right (Ctrl-W l) -> 右下 (win 2) へ移動
        app.navigate_window(WindowDirection::Right);
        assert_eq!(app.active_window, 2);

        // 4. close_window で右下 (win 2) を閉じる -> 下半分が win 1 のみになり、上下2分割に戻る
        app.close_window();
        assert_eq!(app.windows.len(), 2);
        let rects2 = app.window_tree.compute_rects(Rect::new(0, 0, 100, 40));
        assert_eq!(rects2.len(), 2);
        assert_eq!(rects2[0], (0, Rect::new(0, 0, 100, 20)));
        assert_eq!(rects2[1], (1, Rect::new(0, 20, 100, 20)));
    }

    #[test]
    fn test_outline_toggle_and_jump() {
        let text = "# Title\nSome description\n## Section 1\nContent";
        let mut app = App::new(
            text,
            Some(PathBuf::from("test.md")),
            CharEncoding::Utf8,
            NewlineType::Lf,
        );

        // アウトライン表示ON
        app.toggle_outline();
        assert!(app.show_outline);
        assert_eq!(app.outline_items.len(), 2);
        assert_eq!(app.outline_items[0].title, "Title");
        assert_eq!(app.outline_items[1].title, "Section 1");

        // 2番目の見出しを選択してジャンプ
        app.selected_outline_idx = 1;
        app.jump_to_selected_outline();
        assert_eq!(app.cursor.pos.line, 2); // "## Section 1" は行2 (0-indexed)
    }

    #[test]
    fn test_convert_buffer_text() {
        let mut app = App::new("Hello 123", None, CharEncoding::Utf8, NewlineType::Lf);
        app.convert_buffer_text(ConversionType::ToZenkakuAlphanumeric);
        assert_eq!(app.buffer.to_string(), "Ｈｅｌｌｏ　１２３");

        app.convert_buffer_text(ConversionType::ToHankakuAlphanumeric);
        assert_eq!(app.buffer.to_string(), "Hello 123");
    }

    #[test]
    fn test_open_help() {
        let mut app = App::new("test", None, CharEncoding::Utf8, NewlineType::Lf);
        assert!(!app.show_help);

        // インデックスヘルプ
        app.open_help(None);
        assert!(app.show_help);
        assert!(!app.help_lines.is_empty());
        assert_eq!(app.help_scroll, 0);

        // トピック指定ヘルプ (keisen)
        app.open_help(Some("keisen"));
        assert!(app.show_help);
        assert!(!app.help_lines.is_empty());

        // トピック指定ヘルプ (syntax)
        app.open_help(Some("syntax"));
        assert!(app.show_help);
        assert!(!app.help_lines.is_empty());
        assert!(
            app.help_lines
                .iter()
                .any(|l| l.contains("シンタックスハイライト"))
        );

        // トピック指定ヘルプ (visual)
        app.open_help(Some("visual"));
        assert!(app.show_help);
        assert!(!app.help_lines.is_empty());
        assert!(
            app.help_lines
                .iter()
                .any(|l| l.contains("視覚・編集支援機能"))
        );

        // 存在しないトピック（フォールバック動作）
        app.open_help(Some("non_existent_topic_12345"));
        assert!(app.show_help);
        assert!(!app.help_lines.is_empty());
    }

    #[test]
    fn test_align_text() {
        let text = "let a = 1;\nlet count = 2;\nlet total_value = 300;";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        app.align_selection_or_buffer("=");
        assert_eq!(
            app.buffer.to_string(),
            "let a           = 1;\nlet count       = 2;\nlet total_value = 300;"
        );
    }

    #[test]
    fn test_renumber() {
        let text = "1. りんご\n1. みかん\n9. バナナ";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        app.renumber_selection_or_buffer(1, 1);
        assert_eq!(app.buffer.to_string(), "1. りんご\n2. みかん\n3. バナナ");

        // ステップ10
        app.renumber_selection_or_buffer(10, 10);
        assert_eq!(app.buffer.to_string(), "10. りんご\n20. みかん\n30. バナナ");
    }

    #[test]
    fn test_insert_beg_end() {
        let text = "println!(\"hello\");\nprintln!(\"world\");";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        app.insert_beg_end_in_range("// ", " // comment", true, true);
        assert_eq!(
            app.buffer.to_string(),
            "// println!(\"hello\"); // comment\n// println!(\"world\"); // comment"
        );
    }

    #[test]
    fn test_abbrev_expansion() {
        let text = "hw";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        app.abbrev.register("hw", "Hello World!");
        app.cursor.pos.col = 2; // "hw"の直後
        app.trigger_abbrev_expansion();
        assert_eq!(app.buffer.to_string(), "Hello World!");
    }

    #[test]
    fn test_zen_coding_expansion() {
        let text = "ul>li*2";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        app.cursor.pos.col = 7;
        app.trigger_zen_coding();
        assert_eq!(
            app.buffer.to_string(),
            "<ul>\n  <li></li>\n  <li></li>\n</ul>"
        );
    }

    #[test]
    fn test_dynamic_macro() {
        let text = "";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        // "abcabc" を記録
        for ch in "abcabc".chars() {
            app.dynamic_macro.record_char(ch);
        }
        app.trigger_dynamic_macro();
        assert_eq!(app.buffer.to_string(), "abc");

        // もう一度実行すると同じパターンが再実行される
        app.trigger_dynamic_macro();
        assert_eq!(app.buffer.to_string(), "abcabc");
    }

    #[test]
    fn test_lex_edit() {
        let text = "user_name = 10;";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        app.lex_edit.record_edit("user", "member");
        app.cursor.pos.col = 2; // "user_name" の途中
        app.trigger_lex_edit();
        assert_eq!(app.buffer.to_string(), "member_name = 10;");
    }

    #[test]
    fn test_macro_recording_toggle() {
        let mut app = App::new("test", None, CharEncoding::Utf8, NewlineType::Lf);
        assert!(!app.is_recording_macro);

        app.toggle_macro_recording();
        assert!(app.is_recording_macro);

        app.toggle_macro_recording();
        assert!(!app.is_recording_macro);
        assert!(app.last_macro_events.is_some());
    }

    #[test]
    fn test_expression_substitute_in_app() {
        let text = "price: 100\nprice: 250";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        let res = app.vi.execute(
            &vivivi_vi::ViCommand::ExCommand("%s/([0-9]+)/$1 * 2/ge".to_string()),
            &mut app.buffer,
            &mut app.cursor,
        );
        assert_eq!(res.message, Some("2 箇所の式置換を行いました".to_string()));
        assert_eq!(app.buffer.to_string(), "price: 200\nprice: 500");
    }

    #[test]
    fn test_phase4_4_features_in_app() {
        let text = "count = 41;\nflag = false;\nfunc();";
        let mut app = App::new(
            text,
            Some(PathBuf::from("test.rs")),
            CharEncoding::Utf8,
            NewlineType::Lf,
        );
        app.cursor.pos = CursorPosition::new(0, 8); // '4'

        // 1. #+ (41 -> 42)
        let cmd = app.vi.feed_char('#');
        assert_eq!(cmd, None);
        let cmd = app.vi.feed_char('+');
        assert!(cmd.is_some());
        app.vi
            .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
        assert!(app.buffer.to_string().contains("count = 42;"));

        // 2. #! (false -> true)
        app.cursor.pos = CursorPosition::new(1, 7); // "false"
        let cmd = app.vi.feed_char('#');
        assert_eq!(cmd, None);
        let cmd = vi_cmd_bang(&mut app.vi);
        assert!(cmd.is_some());
        app.vi
            .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
        assert!(app.buffer.to_string().contains("flag = true;"));

        // 3. マーク設定と一覧
        app.cursor.pos = CursorPosition::new(1, 0);
        let cmd = app.vi.feed_char('m');
        assert_eq!(cmd, None);
        let cmd = app.vi.feed_char('Z');
        assert!(cmd.is_some());
        app.vi
            .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
        let res = app.vi.execute(
            &vivivi_vi::ViCommand::ListMarks,
            &mut app.buffer,
            &mut app.cursor,
        );
        assert!(res.message.unwrap().contains("'Z': 2行"));
    }

    #[test]
    fn test_phase4_5_features_in_app() {
        let text =
            "fn main() {\n    // TODO: refactor logic\n    let x = 10;\n    let y = 20;\n}\n";
        let mut app = App::new(
            text,
            Some(PathBuf::from("dummy.rs")),
            CharEncoding::Utf8,
            NewlineType::Lf,
        );

        // 1. コード折り畳み (:fold, zc, zo, za)
        let res = app.vi.execute(
            &vivivi_vi::ViCommand::ExCommand("fold".to_string()),
            &mut app.buffer,
            &mut app.cursor,
        );
        assert!(res.message.unwrap().contains("コード折り畳み: 1 箇所"));
        assert_eq!(app.vi.fold_manager.folds.len(), 1);

        let cmd = app.vi.feed_char('z');
        assert_eq!(cmd, None);
        let cmd = app.vi.feed_char('c');
        assert!(cmd.is_some());
        let res = app
            .vi
            .execute(&cmd.unwrap(), &mut app.buffer, &mut app.cursor);
        assert_eq!(res.message, Some("折り畳みを閉じました (zc)".to_string()));

        // 2. コード内 ToDo 抽出 (:todo)
        let res = app.vi.execute(
            &vivivi_vi::ViCommand::ExCommand("todo".to_string()),
            &mut app.buffer,
            &mut app.cursor,
        );
        assert!(res.message.unwrap().contains("ToDo: 1 件見つかりました"));
        assert_eq!(app.cursor.pos.line, 1); // 2行目 (0-indexed 1)

        // 3. 印刷プレビュー (:preview)
        let res = app.vi.execute(
            &vivivi_vi::ViCommand::ExCommand("preview".to_string()),
            &mut app.buffer,
            &mut app.cursor,
        );
        assert!(res.preview_requested.is_some());
        assert!(
            res.message
                .unwrap()
                .contains("印刷プレビューを生成しました")
        );

        // 4. Grep検索 (:grep)
        let res = app.vi.execute(
            &vivivi_vi::ViCommand::ExCommand("grep main src".to_string()),
            &mut app.buffer,
            &mut app.cursor,
        );
        assert!(res.message.unwrap().contains("Grep完了"));
    }

    #[test]
    fn test_phase4_6_gutter_and_recent() {
        let initial_text = "line1\nline2\nline3";
        let mut app = App::new(initial_text, None, CharEncoding::Utf8, NewlineType::Lf);

        // ガターマーカー初期状態
        let lines: Vec<String> = (0..app.buffer.line_count())
            .filter_map(|i| app.buffer.get_line(i))
            .collect();
        let markers = app.change_tracker.compute_markers(&lines);
        assert_eq!(
            markers,
            vec![
                vivivi_modes::LineChangeState::Unchanged,
                vivivi_modes::LineChangeState::Unchanged,
                vivivi_modes::LineChangeState::Unchanged,
            ]
        );

        // 1行目を編集
        app.buffer.insert_char(0, 'x');
        let lines: Vec<String> = (0..app.buffer.line_count())
            .filter_map(|i| app.buffer.get_line(i))
            .collect();
        let markers = app.change_tracker.compute_markers(&lines);
        assert_eq!(markers[0], vivivi_modes::LineChangeState::Modified);

        // お気に入りの登録トグル
        app.recent_manager = RecentManager::new();
        let unique_fav = PathBuf::from("unique_test_fav_file_12345.rs");
        app.file_path = Some(unique_fav.clone());
        app.toggle_favorite_current_file();
        assert!(app.recent_manager.is_favorite(&unique_fav));

        // 不可視文字設定の初期値
        assert!(!app.list_chars_config.enabled);
        app.list_chars_config.enabled = true;
        assert!(app.list_chars_config.enabled);
    }

    #[test]
    fn test_key_release_ignored() {
        let mut app = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);

        // Releaseイベントを作成
        let release_event = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('x'),
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Release,
            state: crossterm::event::KeyEventState::empty(),
        };

        // handle_key_event に渡しても Release イベントは完全に無視されること
        crate::tui::event::handle_key_event(&mut app, release_event);
        assert_eq!(app.buffer.to_string(), "hello");
        assert_eq!(app.cursor.pos.col, 0);
    }

    #[test]
    fn test_ctrl_shortcuts_and_basic_keys() {
        let mut app = App::new(
            "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13",
            None,
            CharEncoding::Utf8,
            NewlineType::Lf,
        );

        // 1. Home / End キーテスト
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::End,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.cursor.pos.col, 5); // "line 1" is 6 chars, end is 5 in normal mode

        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Home,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.cursor.pos.col, 0);

        // 2. Ctrl-D / Ctrl-U (半画面スクロール) テスト
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('d'),
                crossterm::event::KeyModifiers::CONTROL,
            ),
        );
        assert_eq!(app.cursor.pos.line, 10); // 10行進む

        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('u'),
                crossterm::event::KeyModifiers::CONTROL,
            ),
        );
        assert_eq!(app.cursor.pos.line, 0); // 10行戻る

        // 3. Ctrl-V (VisualBlock) テスト
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('v'),
                crossterm::event::KeyModifiers::CONTROL,
            ),
        );
        assert_eq!(app.mode(), ViMode::VisualBlock);

        // もう一度 Ctrl-V で Normal モード復帰
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('v'),
                crossterm::event::KeyModifiers::CONTROL,
            ),
        );
        assert_eq!(app.mode(), ViMode::Normal);

        // 4. Ctrl-R (Redo) テスト
        // 'x' で1文字削除
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('x'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.buffer.line_text(0).unwrap(), "ine 1");

        // 'u' で Undo
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('u'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.buffer.line_text(0).unwrap(), "line 1");

        // Ctrl-R で Redo
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('r'),
                crossterm::event::KeyModifiers::CONTROL,
            ),
        );
        assert_eq!(app.buffer.line_text(0).unwrap(), "ine 1");
    }

    #[test]
    fn test_search_jump_to_head_and_highlight() {
        let text = "こんにちは世界\nHello Antigravity\nさようなら世界";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

        // 1. 通常検索 "/Antigravity"
        app.feed_vi_char('/');
        for c in "Antigravity\n".chars() {
            app.feed_vi_char(c);
        }
        // "Hello Antigravity" の行1、"Hello " の直後（col 6）に着地
        assert_eq!(app.cursor.pos.line, 1);
        assert_eq!(app.cursor.pos.col, 6);

        // active_search_regex でハイライト用Regexが取得できること
        let re = app.active_search_regex();
        assert!(re.is_some());
        let re = re.unwrap();
        assert!(re.is_match("Hello Antigravity"));

        // 2. 日本語文字列検索 "/世界"
        app.cursor.pos = vivivi_core::CursorPosition::new(0, 0);
        app.feed_vi_char('/');
        for c in "世界\n".chars() {
            app.feed_vi_char(c);
        }
        // "こんにちは世界" の先頭文字 "世" は 5文字目（col 5）
        assert_eq!(app.cursor.pos.line, 0);
        assert_eq!(app.cursor.pos.col, 5);

        // 'n' で次の "世界" へジャンプ
        app.feed_vi_char('n');
        // "さようなら世界" の先頭文字 "世" は 5文字目（col 5）
        assert_eq!(app.cursor.pos.line, 2);
        assert_eq!(app.cursor.pos.col, 5);
        assert!(app.active_search_regex().is_some());

        // 3. :noh で検索ハイライトを消去
        app.clear_search_highlight();
        assert!(app.active_search_regex().is_none());

        // 4. 'n' を押すとハイライトが再表示される
        app.feed_vi_char('n');
        assert!(app.active_search_regex().is_some());

        // 5. ノーマルモードで Esc 1回押しではハイライトは消えない（誤操作防止）
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Esc,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert!(app.active_search_regex().is_some());

        // 6. ノーマルモードで素早く Esc をもう一度押す（ダブルEsc連打）とハイライトが消去される
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Esc,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert!(app.active_search_regex().is_none());
    }

    #[test]
    fn test_keisen_key_modifier_requirement() {
        let text = "hello\nworld";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

        // 罫線モードを有効化
        app.keisen_mode.is_active = true;

        // 1. 修飾キーなしの通常の矢印キー（Right）を押す
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Right,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        // 罫線は引かれず、カーソルだけが右（col 1）に移動
        assert_eq!(app.cursor.pos.col, 1);
        assert_eq!(app.buffer.line_text(0).unwrap(), "hello");

        // 2. 修飾キーなしの通常の矢印キー（Down）を押す
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Down,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        // 罫線は引かれず、カーソルだけが下（line 1）に移動
        assert_eq!(app.cursor.pos.line, 1);
        assert_eq!(app.buffer.line_text(1).unwrap(), "world");

        // 3. Ctrl+Right を押す -> 罫線が描画される！
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Right,
                crossterm::event::KeyModifiers::CONTROL,
            ),
        );
        // line 1 の文字が罫線（─）に更新されていること
        assert_eq!(
            app.buffer.char_at(app.buffer.line_start_position(1) + 1),
            Some('─')
        );
    }

    #[test]
    fn test_replace_mode_workflow() {
        let text = "hello world";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

        // 1. 'R' を押して置換モードへ入る
        app.feed_vi_char('R');
        assert_eq!(app.mode(), ViMode::Replace);

        // 2. "HELLO" とタイプして上書き
        for c in "HELLO".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        assert_eq!(app.buffer.line_text(0).unwrap(), "HELLO world");
        assert_eq!(app.cursor.pos.col, 5);

        // 3. Esc でノーマルモードへ復帰
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Esc,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.mode(), ViMode::Normal);
        assert_eq!(app.cursor.pos.col, 4);
    }

    #[test]
    fn test_keisen_normal_x_protected() {
        // 罫線ボックス内のテキスト
        let text = "┌──────────┐\n│ hello    │\n└──────────┘";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);
        app.keisen_mode.is_active = true;

        // 1行目の 'h' (line: 1, col: 2) にカーソルを合わせる
        app.cursor.pos = CursorPosition::new(1, 2);

        // ノーマルモードで 'x' を押す
        app.feed_vi_char('x');

        // 'h' が削除され、右枠線手前に半角スペースが1つ自動補填されて全体の行幅（枠線位置）が完全維持される
        let line1 = app.buffer.line_text(1).unwrap();
        assert_eq!(line1, "│ ello     │");
        assert_eq!(line1.chars().count(), 12);
    }

    #[test]
    fn test_horizontal_scroll_over_80_columns() {
        // 120文字の長い行
        let long_line = "a".repeat(120);
        let mut app = App::new(&long_line, None, CharEncoding::Utf8, NewlineType::Lf);
        assert_eq!(app.scroll, (0, 0));

        // 行末 '$' へ移動
        app.feed_vi_char('$');
        assert_eq!(app.cursor.pos.col, 119);

        // 表示幅 80 カラムでスクロール調整
        app.adjust_scroll(24, 80);
        // 水平スクロール位置が 0 から右にスクロールしていること
        assert!(
            app.scroll.1 > 0,
            "水平スクロールが発生していること (scroll.1 = {})",
            app.scroll.1
        );
        // カーソルがスクロール後の画面可視範囲 (scroll.1..scroll.1 + 80) 内に収まっていること
        assert!(app.cursor.pos.col >= app.scroll.1);
        assert!(app.cursor.pos.col < app.scroll.1 + 80);
    }

    #[test]
    fn test_migemo_builtin_kanji_search() {
        let text = "これは日本語の検索テストです。世界中どこでも動きます。";
        let mut app = App::new(text, None, CharEncoding::Utf8, NewlineType::Lf);

        // 1. "kensaku" で "検索"（漢字）にマッチしてカーソルがジャンプすること
        app.execute_migemo_search("kensaku");
        assert_eq!(app.cursor.pos.col, 7); // "検索" の開始位置

        // 2. "sekai" で "世界" にマッチすること
        app.execute_migemo_search("sekai");
        assert_eq!(app.cursor.pos.col, 15); // "世界" の開始位置

        // 3. "kakko" で "（" や "括弧" にマッチすること
        let mut app2 = App::new(
            "テキスト（重要）および矢印→はこちら",
            None,
            CharEncoding::Utf8,
            NewlineType::Lf,
        );
        app2.execute_migemo_search("kakko");
        assert_eq!(app2.cursor.pos.col, 4); // "（" の開始位置

        // 4. "yajirushi" で "矢印" や "→" にマッチすること
        app2.execute_migemo_search("yajirushi");
        assert_eq!(app2.cursor.pos.col, 11); // "矢印" の開始位置
    }

    #[test]
    fn test_insert_key_mode_toggle() {
        let mut app = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);
        assert_eq!(app.mode(), ViMode::Normal);

        // 1. Normalモードで Insertキーを押す -> Insertモードへ突入
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Insert,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.mode(), ViMode::Insert);

        // 2. Insertモードで Insertキーを押す -> Replaceモードへ切り替え
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Insert,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.mode(), ViMode::Replace);

        // 3. Replaceモードで Insertキーを押す -> Insertモードへ復帰
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Insert,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.mode(), ViMode::Insert);

        // 4. Escキーで Normalモードへ復帰
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Esc,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.mode(), ViMode::Normal);
    }

    #[test]
    fn test_encoding_and_newline_ex_commands() {
        let mut app = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);

        // :set fenc=sjis
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "set fenc=sjis".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.encoding, CharEncoding::ShiftJis);

        // :set ff=dos
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "set ff=dos".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.newline, NewlineType::Crlf);

        // :e ++enc=euc-jp
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "e ++enc=euc-jp".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.encoding, CharEncoding::EucJp);
        assert_eq!(app.newline, NewlineType::Crlf);

        // :set clipboard=unnamed
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "set clipboard=unnamed".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(
            app.vi.clipboard_option(),
            vivivi_vi::ClipboardOption::Unnamed
        );
    }

    #[test]
    fn test_swap_file_and_recovery() {
        let temp_dir = std::env::temp_dir();
        let target_file = temp_dir.join("vivivi_test_doc.txt");
        let swp_file = vivivi_core::SwapFile::swap_path_for(&target_file);

        // 事前クリーンアップ
        let _ = vivivi_core::SwapFile::remove(&swp_file);

        // 1. スワップファイルが存在しない状態で起動
        let mut app = App::new(
            "original text",
            Some(target_file.clone()),
            CharEncoding::Utf8,
            NewlineType::Lf,
        );
        assert!(!app.has_swap_warning);

        // 2. バッファ変更後にスワップ保存
        app.buffer.insert_text(13, " modified");
        assert!(app.buffer.is_modified());
        app.save_swap_file();
        assert!(swp_file.exists());

        // 3. 次回起動時にスワップファイル検出と警告メッセージ
        let mut new_app = App::new(
            "original text",
            Some(target_file.clone()),
            CharEncoding::Utf8,
            NewlineType::Lf,
        );
        assert!(new_app.has_swap_warning);
        assert!(
            new_app
                .status_message
                .as_ref()
                .unwrap()
                .contains("スワップ")
        );

        // 4. :recover コマンドでスワップから復元
        crate::tui::event::handle_key_event(
            &mut new_app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "recover".chars() {
            crate::tui::event::handle_key_event(
                &mut new_app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut new_app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(new_app.buffer.to_string(), "original text modified");
        assert!(!new_app.has_swap_warning);

        // 5. 保存するとスワップファイルが削除される
        let _ = new_app.save_file();
        assert!(!swp_file.exists());
        let _ = std::fs::remove_file(&target_file);
    }

    #[test]
    fn test_multi_tab_workflow() {
        let mut app = App::new("tab1 content", None, CharEncoding::Utf8, NewlineType::Lf);
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab, 0);

        // 1. :tabnew で新規タブ作成
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "tabnew".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab, 1);
        assert_eq!(app.buffer.to_string(), "");

        // タブ2を編集
        app.buffer.insert_text(0, "tab2 content");
        assert_eq!(app.buffer.to_string(), "tab2 content");

        // 2. gt で次のタブ（循環してタブ1へ）
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('g'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('t'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.active_tab, 0);
        assert_eq!(app.buffer.to_string(), "tab1 content");

        // 3. gT で前のタブ（タブ2へ）
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('g'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('T'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.active_tab, 1);
        assert_eq!(app.buffer.to_string(), "tab2 content");

        // 4. :tabclose でタブ2を閉じる
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "tabclose".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab, 0);
        assert_eq!(app.buffer.to_string(), "tab1 content");

        // 5. 最後の1タブで :tabclose を実行しても閉じずにエラーメッセージ
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "tabc".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.tabs.len(), 1);
        assert!(app.status_message.as_ref().unwrap().contains("最後のタブ"));

        // 6. switch_tab_1based と :tabonly の検証
        app.new_tab(None);
        app.new_tab(None);
        assert_eq!(app.tabs.len(), 3);
        assert_eq!(app.tabs[0].id, 1);
        app.switch_tab_1based(2);
        assert_eq!(app.active_tab, 1);

        // :tabonly で他タブを一括クローズ
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(':'),
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        for c in "tabonly".chars() {
            crate::tui::event::handle_key_event(
                &mut app,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(c),
                    crossterm::event::KeyModifiers::empty(),
                ),
            );
        }
        crate::tui::event::handle_key_event(
            &mut app,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::empty(),
            ),
        );
        assert_eq!(app.tabs.len(), 1);
    }

    fn vi_cmd_bang(vi: &mut ViEngine) -> Option<vivivi_vi::ViCommand> {
        vi.feed_char('!')
    }
}
