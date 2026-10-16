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
mod files;
mod tabs;
pub mod tools;
pub use tools::get_recent_file_path;
mod windows;

#[cfg(test)]
mod tests;

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
}
