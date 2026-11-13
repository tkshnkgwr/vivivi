use super::window::{EditorWindow, SplitAxis, WindowDirection, WindowTreeNode};
use crate::types::{AppKeyEvent, LogicalRect};
use std::path::PathBuf;
use vivivi_core::{CharEncoding, CursorPosition, NewlineType, TextBuffer, TextCursor};
use vivivi_modes::{
    AbbrevManager, AstSelectionRange, BoxAlignment, ChangeTracker, ConversionType, CsvMode,
    DabbrevSession, DynamicMacroEngine, HexCursor, HexDocument, KeisenDirection, KeisenMode,
    LexEditEngine, ListCharsConfig, LspManager, MigemoEngine, OutlineItem, RecentManager,
    SyntaxLang, TreeSitterEngine, align_lines, calculate_box_selection, convert_text,
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
mod files;
mod tabs;
pub mod tools;
pub use tools::get_recent_file_path;
pub mod process;
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
                        *target_lock.lock().unwrap() =
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

    /// 更新があるかバックグラウンドで照会
    pub fn check_for_updates(&self, force: bool) {
        let mgr = std::sync::Arc::clone(&self.update_manager);
        let target_lock = std::sync::Arc::clone(&self.update_target);
        let _ = std::thread::Builder::new()
            .name("vivivi-update-check".to_string())
            .spawn(move || {
                if let Ok(moult_update::UpdateCheckResult::UpdateAvailable {
                    latest_release,
                    target_asset,
                    sha256_asset,
                    ..
                }) = mgr.check(force)
                {
                    *target_lock.lock().unwrap() =
                        Some((*latest_release, target_asset, sha256_asset));
                }
            });
    }

    /// 新バージョンのダウンロード＆展開をバックグラウンドで開始
    pub fn download_update(&self) -> bool {
        let target = self.update_target.lock().unwrap().clone();
        if let Some((release, target_asset, sha256_asset)) = target {
            let mgr = std::sync::Arc::clone(&self.update_manager);
            let staging_lock = std::sync::Arc::clone(&self.update_staging_dir);
            let _ = std::thread::Builder::new()
                .name("vivivi-update-download".to_string())
                .spawn(move || {
                    if let Ok(extracted) =
                        mgr.download(&release, &target_asset, sha256_asset.as_ref(), None)
                    {
                        *staging_lock.lock().unwrap() = Some(extracted);
                    }
                });
            true
        } else {
            false
        }
    }

    /// ダウンロード完了した新バイナリをスワップ適用して即時再起動
    pub fn apply_update_and_restart(&self) -> Result<(), String> {
        let staging = self.update_staging_dir.lock().unwrap().clone();
        if let Some(staging_dir) = staging {
            self.update_manager
                .apply_and_restart(&staging_dir)
                .map_err(|e| e.to_string())
        } else {
            Err("更新バイナリがまだ準備できていません".to_string())
        }
    }

    /// 現在のアップデートステータスを取得
    pub fn update_status(&self) -> moult_update::UpdateStatus {
        self.update_manager.status()
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
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }
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
                let tab_w = self.tab_width.max(1);
                visual_x += tab_w - (visual_x % tab_w);
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

    /// 現在のバッファ内容に合わせてガター変更マーカー（Diff キャッシュ）を最新化します。
    ///
    /// 前回の計算時から行数・各行内容に変更がない場合は Diff 計算をスキップし、
    /// 描画フレームレートへの影響をゼロにします。
    pub fn update_change_markers(&mut self) {
        if !self.change_tracker.enabled {
            return;
        }
        let lines = self.buffer.all_lines();
        self.change_tracker.update_markers(&lines);
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

    /// 2ペインファイラーを開きます (`:filer`)。
    pub fn open_filer(&mut self, path: Option<&std::path::Path>) {
        let target_dir = if let Some(p) = path {
            if p.is_dir() {
                p.to_path_buf()
            } else {
                p.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf()
            }
        } else if let Some(ref fp) = self.file_path {
            fp.parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        };

        let filer = vivivi_modes::TwoPaneFiler::new(&target_dir);
        self.filer = Some(filer);
        self.show_filer = true;
        self.status_message = Some(
            "2ペイン ファイラー起動 (Tab: 左右切替, Enter: 開く/入る, q/Esc: 閉じる)".to_string(),
        );
    }

    /// 2ペインファイラーを閉じます。
    pub fn close_filer(&mut self) {
        self.show_filer = false;
        self.filer = None;
    }

    /// 現在のバッファ内容をA4サイズのPDFファイルとして出力します。
    ///
    /// # 引数
    /// - `target_path`: 保存先ファイルパス（指定なしの場合はバッファファイル名`.pdf`、または `Untitled.pdf`）
    ///
    /// # 戻り値
    /// 成功した場合は保存先 `PathBuf`、失敗時はエラーメッセージ。
    pub fn export_pdf(&mut self, target_path: Option<&str>) -> Result<PathBuf, String> {
        let content = self.buffer.to_string();
        let doc_name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let out_path = if let Some(p) = target_path.filter(|s| !s.trim().is_empty()) {
            PathBuf::from(p.trim())
        } else if let Some(ref fp) = self.file_path {
            fp.with_extension("pdf")
        } else {
            PathBuf::from("Untitled.pdf")
        };

        let date_str = current_date_str();
        let settings = vivivi_modes::PrintSettings::default();
        let pages = vivivi_modes::paginate_text(&content, doc_name, &date_str, &settings);
        vivivi_modes::export_to_pdf(&pages, &out_path)?;

        let display_path = out_path.display().to_string();
        self.status_message = Some(format!("PDFを出力しました: {display_path}"));
        Ok(out_path)
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

    /// 指定されたクリップボードプロバイダを注入した [`App`] を初期化します。
    pub fn with_clipboard_provider(
        initial_text: &str,
        file_path: Option<PathBuf>,
        encoding: CharEncoding,
        newline: NewlineType,
        provider: std::sync::Arc<std::sync::Mutex<dyn vivivi_vi::ClipboardProvider>>,
    ) -> Self {
        let mut app = Self::new(initial_text, file_path, encoding, newline);
        app.vi.set_clipboard_provider(provider);
        app
    }

    /// テスト用のインメモリ [`crate::tui::clipboard::MockClipboard`] を注入した [`App`] を初期化します。
    pub fn with_mock_clipboard(
        initial_text: &str,
        file_path: Option<PathBuf>,
        encoding: CharEncoding,
        newline: NewlineType,
    ) -> Self {
        Self::with_clipboard_provider(
            initial_text,
            file_path,
            encoding,
            newline,
            crate::tui::clipboard::MockClipboard::new_shared(),
        )
    }

    /// TUI 設定ダイアログを開き、現在の設定をバックアップに退避します。
    pub fn open_preferences_dialog(&mut self) {
        self.sync_runtime_to_config();
        self.tui_settings_backup = Some(self.config.clone());
        self.show_preferences_dialog = true;
    }

    /// TUI 設定ダイアログをキャンセルして閉じ、開く直前の設定に安全にロールバックします。
    pub fn cancel_preferences_dialog(&mut self) {
        if let Some(backup) = self.tui_settings_backup.take() {
            self.apply_config(&backup);
        }
        self.show_preferences_dialog = false;
        self.status_message = Some("設定変更を破棄して設定画面を閉じました".to_string());
    }

    /// 外部設定構造体 [`crate::config::AppConfig`] を現在の実行時状態に適用します。
    pub fn apply_config(&mut self, cfg: &crate::config::AppConfig) {
        self.show_line_numbers = cfg.appearance.line_number;
        self.relative_number = cfg.appearance.relative_number;
        self.tab_width = cfg.editing.tab_width;
        self.wrap = cfg.appearance.wrap;
        self.change_tracker.enabled = cfg.appearance.gutter_markers;
        self.list_chars_config.enabled = cfg.invisibles.enabled;
        self.list_chars_config.zenkaku_space = cfg.invisibles.zenkaku_space.clone().into();
        self.list_chars_config.space = cfg.invisibles.space;
        self.list_chars_config.eol = cfg.invisibles.eol;
        let cb_opt = vivivi_vi::ClipboardOption::parse(&cfg.general.clipboard);
        self.vi.set_clipboard_option(cb_opt);
        self.vi.ignorecase = cfg.editing.ignorecase;
        self.vi.smartcase = cfg.editing.smartcase;
        self.config = cfg.clone();
    }

    /// 現在のエディタ実行時状態（行番号、折り返し、不可視文字など）を設定構造体に同期させます。
    pub fn sync_runtime_to_config(&mut self) {
        self.config.appearance.line_number = self.show_line_numbers;
        self.config.appearance.relative_number = self.relative_number;
        self.config.editing.tab_width = self.tab_width;
        self.config.editing.ignorecase = self.vi.ignorecase;
        self.config.editing.smartcase = self.vi.smartcase;
        self.config.appearance.wrap = self.wrap;
        self.config.appearance.gutter_markers = self.change_tracker.enabled;
        self.config.invisibles.enabled = self.list_chars_config.enabled;
        self.config.invisibles.zenkaku_space = self.list_chars_config.zenkaku_space.to_string();
        self.config.invisibles.space = self.list_chars_config.space;
        self.config.invisibles.eol = self.list_chars_config.eol;
        self.config.general.clipboard = match self.vi.clipboard_option() {
            vivivi_vi::ClipboardOption::None => "none".to_string(),
            vivivi_vi::ClipboardOption::Unnamed => "unnamed".to_string(),
            vivivi_vi::ClipboardOption::UnnamedPlus => "unnamedplus".to_string(),
        };
    }

    /// バッファの指定範囲（または全体）の末尾空白を一括削除します (`:trim`)。
    ///
    /// 単一の Undo トランザクションとして記録されるため、`u` キー1回で完全復元可能です。
    /// 戻り値として、末尾空白が削除された行数を返します。
    pub fn trim_trailing_whitespace(
        &mut self,
        range: Option<(usize, usize)>,
        kind: vivivi_modes::trim::TrimKind,
    ) -> usize {
        let line_count = self.buffer.line_count();
        if line_count == 0 {
            return 0;
        }

        let (start_line, end_line) = match range {
            Some((s, e)) => (
                s.min(line_count.saturating_sub(1)),
                e.min(line_count.saturating_sub(1)),
            ),
            None => (0, line_count.saturating_sub(1)),
        };

        let mut lines = Vec::with_capacity(line_count);
        for l in 0..line_count {
            lines.push(self.buffer.get_line(l).unwrap_or_default());
        }

        let modified_lines = vivivi_modes::trim::trim_lines(&lines, start_line, end_line, kind);
        if modified_lines.is_empty() {
            return 0;
        }

        let modified_count = modified_lines.len();

        self.buffer.open_undo_block();

        for (line_idx, new_line_str) in modified_lines.into_iter().rev() {
            let start_offset = self
                .buffer
                .cursor_to_position(CursorPosition::new(line_idx, 0));
            let old_line_len = lines[line_idx].chars().count();
            let end_offset = start_offset + old_line_len;
            self.buffer.delete_range(start_offset, end_offset);
            self.buffer.insert_text(start_offset, &new_line_str);
        }

        self.buffer.close_undo_block();
        self.clamp_cursor();

        modified_count
    }

    /// 指定行範囲（`None` の場合は末尾空行シードを除く全行）を [`vivivi_modes::SortOptions`] に従ってソートします。
    ///
    /// 処理した行数を返します。
    pub fn sort_buffer_lines(
        &mut self,
        range: Option<(usize, usize)>,
        options: vivivi_modes::SortOptions,
    ) -> usize {
        let line_count = self.buffer.line_count();
        if line_count == 0 {
            return 0;
        }

        let (start_line, end_line) = match range {
            Some((s, e)) => {
                let sc = s.min(line_count.saturating_sub(1));
                let ec = e.min(line_count.saturating_sub(1));
                (sc.min(ec), sc.max(ec))
            }
            None => {
                let mut last = line_count.saturating_sub(1);
                if last > 0 && self.buffer.get_line(last).as_deref() == Some("") {
                    last -= 1;
                }
                (0, last)
            }
        };

        if start_line > end_line {
            return 0;
        }

        let mut lines = Vec::with_capacity(end_line - start_line + 1);
        for l in start_line..=end_line {
            lines.push(self.buffer.get_line(l).unwrap_or_default());
        }
        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let sorted = vivivi_modes::sort_lines(&line_refs, options);

        let start_offset = self.buffer.line_start_position(start_line);
        let last_len = lines.last().map_or(0, |s| s.chars().count());
        let end_offset = self.buffer.line_start_position(end_line) + last_len;
        let replacement = sorted.join("\n");

        self.buffer.open_undo_block();
        self.buffer.delete_range(start_offset, end_offset);
        self.buffer.insert_text(start_offset, &replacement);
        self.buffer.close_undo_block();

        self.cursor.pos = CursorPosition::new(start_line, 0);
        self.clamp_cursor();
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }

        sorted.len()
    }

    /// 外部コマンド実行ダイアログ (cmdExecDlg) を開きます。
    pub fn open_cmd_exec_dialog(&mut self) {
        self.show_cmd_exec_dialog = true;
        self.cmd_exec_output_preview = None;
    }

    /// 外部コマンド実行ダイアログ (cmdExecDlg) を閉じます。
    pub fn close_cmd_exec_dialog(&mut self) {
        self.show_cmd_exec_dialog = false;
        self.cmd_exec_output_preview = None;
    }

    /// 外部コマンド実行ダイアログの入力コマンドと選択モードを実行します。
    pub fn execute_cmd_exec_dialog(&mut self) {
        let cmd = self.cmd_exec_input.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        match self.cmd_exec_mode {
            0 => {
                // 実行のみ (結果プレビュー)
                match self.execute_shell_command(&cmd) {
                    Ok(out) => {
                        self.cmd_exec_output_preview = Some(out);
                        self.status_message = Some(format!("!{cmd} ➔ 実行完了"));
                    }
                    Err(e) => {
                        self.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                        self.status_message = Some(format!("コマンド実行エラー: {e}"));
                    }
                }
            }
            1 => {
                // バッファ挿入 (:r !)
                match self.read_command_into_buffer(&cmd, None) {
                    Ok(count) => {
                        self.status_message = Some(format!(":r !{cmd} ➔ {count} 行を挿入しました"));
                        self.close_cmd_exec_dialog();
                    }
                    Err(e) => {
                        self.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                        self.status_message = Some(format!("コマンド読込エラー: {e}"));
                    }
                }
            }
            2 => {
                // 選択行置換 (:[range]!)
                let (start, end) = match self.cursor.anchor {
                    Some(anchor) => (
                        anchor.line.min(self.cursor.pos.line),
                        anchor.line.max(self.cursor.pos.line),
                    ),
                    None => (self.cursor.pos.line, self.cursor.pos.line),
                };
                match self.filter_range_with_command(&cmd, start, end) {
                    Ok(count) => {
                        self.status_message =
                            Some(format!(":[range]!{cmd} ➔ {count} 行に置換しました"));
                        self.close_cmd_exec_dialog();
                    }
                    Err(e) => {
                        self.cmd_exec_output_preview = Some(format!("エラー: {e}"));
                        self.status_message = Some(format!("置換エラー: {e}"));
                    }
                }
            }
            _ => {}
        }
    }

    /// バイナリモード (Hex Editor) とテキストモードを切り替えます。
    pub fn toggle_hex_mode(&mut self) {
        if self.is_hex_mode {
            // テキストモードへ復帰
            if let Some(ref hex_doc) = self.hex_doc
                && hex_doc.is_dirty()
            {
                let updated_text = hex_doc.to_utf8_lossy();
                self.buffer = TextBuffer::from_text(&updated_text);
                self.cursor = TextCursor::new(0, 0);
            }
            self.is_hex_mode = false;
            self.status_message = Some("テキストモードへ切り替えました".to_string());
        } else {
            // バイナリモードへ移行
            let data = self.buffer.to_string().into_bytes();
            let mut hex_doc = HexDocument::from_bytes(data);
            if let Some(ref path) = self.file_path {
                hex_doc.set_file_path(Some(path.clone()));
            }
            self.hex_doc = Some(hex_doc);
            self.hex_cursor = HexCursor::new();
            self.hex_scroll = 0;
            self.is_hex_mode = true;
            self.status_message = Some(
                "バイナリモード (Hex Editor) へ切り替えました (終了は :hex または :text)"
                    .to_string(),
            );
        }
    }

    /// LSP サーバーを開始します。
    pub fn start_lsp(&mut self, custom_cmd: Option<&str>) {
        let mut mgr = self.lsp_manager.take().unwrap_or_default();
        let res = if let Some(cmd) = custom_cmd {
            mgr.start_server(cmd, &[])
        } else if let Some(ref path) = self.file_path {
            mgr.start_server_for_path(path)
        } else {
            Err("ファイルパスが未設定のため、LSPサーバーを特定できません (:lsp start <cmd> で指定してください)".to_string())
        };

        match res {
            Ok(()) => {
                let name = mgr.server_name().unwrap_or("LSP").to_string();
                if let Some(ref path) = self.file_path {
                    let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
                    let text = self.buffer.to_string();
                    let lang_id = match self.syntax_lang {
                        SyntaxLang::Rust => "rust",
                        SyntaxLang::Cpp => "cpp",
                        SyntaxLang::Python => "python",
                        SyntaxLang::JavaScript => "javascript",
                        SyntaxLang::Go => "go",
                        _ => "plaintext",
                    };
                    mgr.notify_did_open(&uri, lang_id, 1, &text);
                }
                self.lsp_manager = Some(mgr);
                self.status_message = Some(format!("LSP サーバー '{name}' を起動しました"));
            }
            Err(e) => {
                self.status_message = Some(format!("LSP 起動エラー: {e}"));
            }
        }
    }

    /// LSP サーバーを停止します。
    pub fn stop_lsp(&mut self) {
        if let Some(mut mgr) = self.lsp_manager.take() {
            mgr.stop_server();
            self.status_message = Some("LSP サーバーを停止しました".to_string());
        } else {
            self.status_message = Some("LSP サーバーは起動していません".to_string());
        }
    }

    /// カーソル位置のホバー情報をリクエストします。
    pub fn request_lsp_hover(&mut self) {
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
            mgr.request_hover(
                &uri,
                self.cursor.pos.line as u32,
                self.cursor.pos.col as u32,
            );
            self.lsp_show_hover = true;
            self.status_message = Some("LSP ホバー情報を要求中...".to_string());
        } else {
            self.status_message = Some("LSP サーバーが起動していません (:lsp start)".to_string());
        }
    }

    /// カーソル位置のシンボル定義ジャンプをリクエストします。
    pub fn request_lsp_definition(&mut self) {
        if let (Some(mgr), Some(path)) = (&mut self.lsp_manager, &self.file_path) {
            let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
            mgr.request_definition(
                &uri,
                self.cursor.pos.line as u32,
                self.cursor.pos.col as u32,
            );
            self.status_message = Some("LSP 定義ジャンプを要求中...".to_string());
        } else {
            self.status_message = Some("LSP サーバーが起動していません (:lsp start)".to_string());
        }
    }

    /// LSP イベントの定期ポーリング
    pub fn poll_lsp(&mut self) {
        let mut jump_target: Option<(usize, usize)> = None;
        if let Some(ref mut mgr) = self.lsp_manager {
            mgr.poll_events();
            if let Some(hover) = mgr.take_latest_hover() {
                self.lsp_hover_text = Some(hover.contents);
            }
            if let Some(loc) = mgr.latest_definition() {
                let target_line = loc.range.start.line as usize;
                let target_col = loc.range.start.character as usize;
                jump_target = Some((target_line, target_col));
                mgr.clear_latest_definition();
            }
        }
        if let Some((target_line, target_col)) = jump_target {
            self.cursor.pos.line = target_line.min(self.buffer.line_count().saturating_sub(1));
            self.cursor.pos.col = target_col;
            self.clamp_cursor();
            self.status_message = Some(format!(
                "定義へジャンプしました: 行 {}, 列 {}",
                target_line + 1,
                target_col + 1
            ));
        }
    }

    /// Tree-sitter 構文木を現在の言語設定および最新バッファ内容で再初期化します。
    pub fn reinit_treesitter(&mut self) {
        let mut ts_engine = TreeSitterEngine::new();
        if ts_engine.set_language(self.syntax_lang) {
            let text = self.buffer.to_string();
            ts_engine.parse(&text);
            self.treesitter = Some(ts_engine);
        } else {
            self.treesitter = None;
        }
        self.treesitter_lang = self.syntax_lang;
        self.treesitter_revision = self.buffer.revision();
    }

    /// バッファ編集または言語変更があった場合のみ Tree-sitter 構文木を再同期します。
    pub fn sync_treesitter_if_dirty(&mut self) {
        if self.treesitter_lang != self.syntax_lang {
            self.reinit_treesitter();
            return;
        }
        let current_rev = self.buffer.revision();
        if self.treesitter_revision != current_rev {
            if let Some(ref mut ts) = self.treesitter {
                let text = self.buffer.to_string();
                ts.parse(&text);
            }
            self.treesitter_revision = current_rev;
        }
    }

    /// Tree-sitter 構文木を最新のバッファ内容で更新します。
    pub fn update_treesitter(&mut self) {
        if let Some(ref mut ts) = self.treesitter {
            let text = self.buffer.to_string();
            ts.parse(&text);
        }
        self.treesitter_lang = self.syntax_lang;
        self.treesitter_revision = self.buffer.revision();
    }

    /// 構文木ノードに基づくスマート選択拡大（Expand Selection）を実行します。
    pub fn expand_smart_selection(&mut self) {
        self.update_treesitter();
        if let Some(ref mut ts) = self.treesitter {
            let line = self.cursor.pos.line;
            let col = self.cursor.pos.col;
            if let Some(new_range) =
                ts.smart_expand_selection(line, col, self.current_smart_selection)
            {
                self.current_smart_selection = Some(new_range);
                self.cursor.anchor = Some(CursorPosition::new(
                    new_range.start_line,
                    new_range.start_col,
                ));
                self.cursor.pos =
                    CursorPosition::new(new_range.end_line, new_range.end_col.saturating_sub(1));
                self.vi.set_mode(ViMode::Visual);
                self.status_message = Some(format!(
                    "スマート選択 (拡大): 行 {}:{} 〜 {}:{}",
                    new_range.start_line + 1,
                    new_range.start_col + 1,
                    new_range.end_line + 1,
                    new_range.end_col
                ));
            }
        } else {
            self.status_message =
                Some("現在のファイルは Tree-sitter 構文木解析に対応していません".to_string());
        }
    }

    /// 構文木ノードに基づくスマート選択縮小（Shrink Selection）を実行します。
    pub fn shrink_smart_selection(&mut self) {
        if let Some(ref mut ts) = self.treesitter {
            if let Some(prev_range) = ts.smart_shrink_selection() {
                self.current_smart_selection = Some(prev_range);
                self.cursor.anchor = Some(CursorPosition::new(
                    prev_range.start_line,
                    prev_range.start_col,
                ));
                self.cursor.pos =
                    CursorPosition::new(prev_range.end_line, prev_range.end_col.saturating_sub(1));
                self.status_message = Some(format!(
                    "スマート選択 (縮小): 行 {}:{} 〜 {}:{}",
                    prev_range.start_line + 1,
                    prev_range.start_col + 1,
                    prev_range.end_line + 1,
                    prev_range.end_col
                ));
            } else {
                self.current_smart_selection = None;
                self.cursor.anchor = None;
                self.vi.set_mode(ViMode::Normal);
                self.status_message = Some("スマート選択を解除しました".to_string());
            }
        }
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
