use super::window::{EditorWindow, SplitAxis, WindowDirection, WindowTreeNode};
use crate::types::{AppKeyEvent, LogicalRect};
use std::path::PathBuf;
use vivivi_core::{CharEncoding, CursorPosition, NewlineType, TextBuffer, TextCursor};
use vivivi_modes::{
    AbbrevManager, AstSelectionRange, ChangeTracker, ConversionType, CsvMode, DabbrevSession,
    DynamicMacroEngine, HexCursor, HexDocument, KeisenMode, LexEditEngine, ListCharsConfig,
    LspManager, MigemoEngine, OutlineItem, RecentManager, SyntaxLang, TreeSitterEngine,
    align_lines, calculate_box_selection, convert_text, expand_zen_coding, insert_beg_end_lines,
    parse_outline, renumber_lines,
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
mod dialogs_and_config;
mod editing_ops;
mod files;
mod lsp_treesitter;
pub mod process;
mod tabs;
pub mod tools;
pub use tools::get_recent_file_path;
mod update;
mod windows;

#[cfg(test)]
mod tests;

/// ダウンロード対象のリリース情報・アセット情報のタプル
pub type UpdateTargetInfo = (
    moult_update::ReleaseInfo,
    moult_update::AssetInfo,
    Option<moult_update::AssetInfo>,
);

/// アップデート対象情報のスレッド間共有ハンドル型
pub type SharedUpdateTarget = std::sync::Arc<std::sync::Mutex<Option<UpdateTargetInfo>>>;

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
    /// 各ウィンドウの最新描画領域矩形キャッシュ [(win_idx, LogicalRect)]
    pub window_rects: Vec<(usize, LogicalRect)>,
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
    /// アウトライン解析済みのバッファリビジョン
    pub outline_revision: Option<u64>,
    /// アウトライン解析時の言語種別
    pub outline_lang: SyntaxLang,

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
    pub recorded_macro_events: Vec<AppKeyEvent>,
    pub last_macro_events: Option<Vec<AppKeyEvent>>,

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

    // --- 2ペイン ファイラー (:filer) ---
    pub show_filer: bool,
    pub filer: Option<vivivi_modes::TwoPaneFiler>,

    /// サイドバー・ファイルツリーの表示トグル要求フラグ (:tree, :explore)
    pub toggle_tree_request: bool,
    /// サイドバー・アウトラインタブの表示トグル要求フラグ (:outline, F8)
    pub toggle_outline_request: bool,
    /// プロジェクト可視化グラフ (Visualize / ViVi 4) の表示フラグ (:visualize, :graph)
    pub show_visualize_graph: bool,

    // --- 変更行ガター追跡 (Change Markers) ---
    pub change_tracker: ChangeTracker,

    // --- 最近使ったファイル & お気に入り (Recent Files / Favorites) ---
    pub recent_manager: RecentManager,
    pub show_recent_modal: bool,
    pub selected_recent_idx: usize,
    /// 設定ダイアログ（GUI / TUI）の表示フラグ
    pub show_preferences_dialog: bool,
    /// TUI 設定ダイアログの選択中カテゴリインデックス (0..6)
    pub tui_settings_category: usize,
    /// TUI 設定ダイアログの選択中項目インデックス
    pub tui_settings_item: usize,
    /// TUI 設定ダイアログで項目ペインにフォーカスがあるか
    pub tui_settings_in_items: bool,
    /// TUI 設定ダイアログを開いた時点の設定バックアップ（キャンセル・ロールバック用）
    pub tui_settings_backup: Option<crate::config::AppConfig>,

    /// 外部コマンド実行ダイアログ (cmdExecDlg) の表示フラグ
    pub show_cmd_exec_dialog: bool,
    /// 外部コマンド実行ダイアログの入力文字列
    pub cmd_exec_input: String,
    /// 外部コマンド実行ダイアログの実行モード (0: 実行のみ, 1: バッファ挿入 `:r!`, 2: 選択行置換 `:[range]!`)
    pub cmd_exec_mode: usize,
    /// 外部コマンド実行結果のプレビュー
    pub cmd_exec_output_preview: Option<String>,

    // --- 不可視文字可視化 (:set list / :set nolist) ---
    pub list_chars_config: ListCharsConfig,

    // --- 永続設定 & 表示オプション ---
    pub config: crate::config::AppConfig,
    /// 現在適用中のカラーテーマ（TUI / GUI 共通）
    pub active_theme: crate::theme::Theme,
    pub show_line_numbers: bool,
    pub relative_number: bool,
    pub tab_width: usize,
    pub wrap: bool,

    // --- クラッシュリカバリ・スワップファイル (.swp) ---
    pub key_stroke_count: usize,
    pub has_swap_warning: bool,

    // --- バイナリモード (Hex Editor) ---
    pub is_hex_mode: bool,
    pub hex_doc: Option<HexDocument>,
    pub hex_cursor: HexCursor,
    pub hex_scroll: usize,

    // --- LSP (Language Server Protocol) ---
    pub lsp_manager: Option<LspManager>,
    pub lsp_hover_text: Option<String>,
    pub lsp_show_hover: bool,
    pub lsp_show_diagnostics_panel: bool,

    // --- Tree-sitter 構文木解析 & スマート選択 ---
    pub treesitter: Option<TreeSitterEngine>,
    pub treesitter_revision: u64,
    pub treesitter_lang: SyntaxLang,
    pub current_smart_selection: Option<AstSelectionRange>,

    // --- マルチタブ管理 (Tab Bar UI) ---
    pub tabs: Vec<crate::tui::tab::TabPage>,
    pub active_tab: usize,
    pub next_tab_id: usize,

    // --- 自動アップデート (moult-update) ---
    /// アップデート管理エンジン
    pub update_manager: std::sync::Arc<moult_update::UpdateManager>,
    /// ダウンロード対象のリリース情報キャッシュ
    pub update_target: SharedUpdateTarget,
    /// ダウンロード・解凍済みステージングディレクトリ（適用準備完了時）
    pub update_staging_dir: std::sync::Arc<std::sync::Mutex<Option<PathBuf>>>,
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
                let date_str = current_date_str();
                let dt_str = current_datetime_str();
                let time_str = dt_str.split_whitespace().nth(1).unwrap_or("00:00:00");
                if let Some(res) = tm.load_and_expand(path, &date_str, time_str) {
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

        #[cfg(not(test))]
        let config = crate::config::AppConfig::load_or_default(None);
        #[cfg(test)]
        let config = crate::config::AppConfig::default();

        let active_theme = crate::theme::Theme::load_by_name(&config.appearance.theme);
        let show_line_numbers = config.appearance.line_number;
        let relative_number = config.appearance.relative_number;
        let tab_width = config.editing.tab_width;
        let wrap = config.appearance.wrap;

        let mut vi = ViEngine::new();
        vi.ignorecase = config.editing.ignorecase;
        vi.smartcase = config.editing.smartcase;
        #[cfg(not(test))]
        let clipboard_provider = crate::tui::clipboard::SystemClipboard::new_shared();
        #[cfg(test)]
        let clipboard_provider = crate::tui::clipboard::MockClipboard::new_shared();
        vi.set_clipboard_provider(clipboard_provider);

        let cb_opt = vivivi_vi::ClipboardOption::parse(&config.general.clipboard);
        vi.set_clipboard_option(cb_opt);
        if let Some(ref path) = file_path {
            vi.current_file_path = path.to_string_lossy().to_string();
        }

        // 変更行ガター追跡の初期化
        let lines: Vec<String> = buffer.all_lines();
        let mut change_tracker = ChangeTracker::new(&lines);
        change_tracker.enabled = config.appearance.gutter_markers;

        // 最近使ったファイルとお気に入りの読み込み
        let mut recent_manager = RecentManager::new();
        #[cfg(not(test))]
        {
            let recent_file_path = get_recent_file_path();
            let _ = recent_manager.load_from_file(&recent_file_path);
            if let Some(ref path) = file_path {
                recent_manager.add_recent(path.clone());
                let _ = recent_manager.save_to_file(&recent_file_path);
            }
        }
        #[cfg(test)]
        {
            if let Some(ref path) = file_path {
                recent_manager.add_recent(path.clone());
            }
        }

        let list_chars_config = ListCharsConfig {
            enabled: config.invisibles.enabled,
            zenkaku_space: config.invisibles.zenkaku_space.clone().into(),
            space: config.invisibles.space,
            eol: config.invisibles.eol,
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

        // --- 自動アップデート (moult-update) マネージャー初期化 ---
        let current_exe_name = std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "vivivi.exe".to_string());

        let (target_bin, pair_bin) = if current_exe_name.to_lowercase().contains("gui") {
            ("vivivi-gui.exe".to_string(), "vivivi.exe".to_string())
        } else {
            ("vivivi.exe".to_string(), "vivivi-gui.exe".to_string())
        };

        let update_config = moult_update::UpdateConfig::new(
            "tkshnkgwr",
            "vivivi",
            env!("CARGO_PKG_VERSION"),
            target_bin,
        )
        .with_pair_bin(pair_bin)
        .with_asset_filter("windows-x86_64");

        let update_manager = std::sync::Arc::new(moult_update::UpdateManager::new(update_config));
        let update_target = std::sync::Arc::new(std::sync::Mutex::new(None));
        let update_staging_dir = std::sync::Arc::new(std::sync::Mutex::new(None));

        #[cfg(not(test))]
        if config.general.check_update {
            let mgr = std::sync::Arc::clone(&update_manager);
            let target_lock = std::sync::Arc::clone(&update_target);
            let _ = std::thread::Builder::new()
                .name("vivivi-update-check".to_string())
                .spawn(move || {
                    if let Ok(moult_update::UpdateCheckResult::UpdateAvailable {
                        latest_release,
                        target_asset,
                        sha256_asset,
                        ..
                    }) = mgr.check(false)
                    {
                        *target_lock.lock().unwrap_or_else(|e| e.into_inner()) =
                            Some((*latest_release, target_asset, sha256_asset));
                    }
                });
        }

        let mut ts_engine = TreeSitterEngine::new();
        let treesitter = if ts_engine.set_language(syntax_lang) {
            ts_engine.parse(initial_text);
            Some(ts_engine)
        } else {
            None
        };

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

        let treesitter_revision = buffer.revision();

        let mut app = Self {
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
            outline_revision: None,
            outline_lang: syntax_lang,

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

            show_filer: false,
            filer: None,
            toggle_tree_request: false,
            toggle_outline_request: false,
            show_visualize_graph: false,

            change_tracker,
            recent_manager,
            show_recent_modal: false,
            selected_recent_idx: 0,
            show_preferences_dialog: false,
            tui_settings_category: 0,
            tui_settings_item: 0,
            tui_settings_in_items: false,
            tui_settings_backup: None,
            show_cmd_exec_dialog: false,
            cmd_exec_input: String::new(),
            cmd_exec_mode: 0,
            cmd_exec_output_preview: None,
            list_chars_config,

            active_theme,
            config,
            show_line_numbers,
            relative_number,
            tab_width,
            wrap,
            key_stroke_count: 0,
            has_swap_warning,

            is_hex_mode: false,
            hex_doc: None,
            hex_cursor: HexCursor::new(),
            hex_scroll: 0,

            lsp_manager: None,
            lsp_hover_text: None,
            lsp_show_hover: false,
            lsp_show_diagnostics_panel: false,

            treesitter,
            treesitter_revision,
            treesitter_lang: syntax_lang,
            current_smart_selection: None,

            tabs: vec![initial_tab],
            active_tab: 0,
            next_tab_id: 2,

            update_manager,
            update_target,
            update_staging_dir,
        };
        app.refresh_outline();
        app
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
            if let Some(adjust) = res.scroll_adjust {
                match adjust {
                    vivivi_vi::ScrollTarget::Top => {
                        self.scroll.0 = self.cursor.pos.line;
                    }
                    vivivi_vi::ScrollTarget::Center => {
                        let view_height = 24usize;
                        let half = view_height / 2;
                        self.scroll.0 = self.cursor.pos.line.saturating_sub(half);
                    }
                    vivivi_vi::ScrollTarget::Bottom => {
                        let view_height = 24usize;
                        self.scroll.0 = (self.cursor.pos.line + 1).saturating_sub(view_height);
                    }
                }
            }
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

        // 水平スクロール（nowrap時のみ適用、wrap時は常に0）
        if self.wrap {
            self.scroll.1 = 0;
        } else {
            let cur_visual_col = self.visual_col_at(cur.line, cur.col);
            if cur_visual_col < self.scroll.1 {
                self.scroll.1 = cur_visual_col;
            } else if cur_visual_col >= self.scroll.1 + view_width {
                self.scroll.1 = cur_visual_col - view_width + 1;
            }
        }

        // アクティブウィンドウの状態と同期
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }
    }

    /// フロントエンド非依存のキーイベントを処理します（GUI / TUI 共通）。
    pub fn handle_key(&mut self, event: impl Into<AppKeyEvent>) {
        let app_ev: AppKeyEvent = event.into();
        crate::tui::event::handle_key_event(self, app_ev.into());
    }

    /// Exコマンド文字列を実行します（`:w`, `:q`, `:set fenc=...` 等、GUI / TUI 共通）。
    pub fn execute_ex(&mut self, cmd: &str) {
        crate::tui::event::execute_ex_command(self, cmd);
    }
}

/// 現在日時（UTC）を YYYY-MM-DD HH:MM:SS 形式の文字列で取得します。
pub fn current_datetime_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    crate::panic_log::format_utc_datetime_from_secs(secs)
}

/// 現在日付（UTC）を YYYY-MM-DD 形式の文字列で取得します。
pub fn current_date_str() -> String {
    current_datetime_str()[..10].to_string()
}
