use super::window::{EditorWindow, SplitAxis};
use std::path::PathBuf;
use vivivi_core::{CharEncoding, CursorPosition, NewlineType, TextBuffer, TextCursor};
use vivivi_modes::{
    BoxAlignment, ConversionType, CsvMode, KeisenDirection, KeisenMode, OutlineItem, SyntaxLang,
    convert_text, parse_outline,
};
use vivivi_vi::{ViEngine, ViMode};

/// TUI アプリケーションの状態管理構造体。
pub struct App {
    /// 編集中のテキストバッファ
    pub buffer: TextBuffer,
    /// アクティブウィンドウのカーソル
    pub cursor: TextCursor,
    /// Viコマンドエンジン
    pub vi: ViEngine,
    /// インラインCSV/TSVモード
    pub csv_mode: CsvMode,
    /// 罫線モード
    pub keisen_mode: KeisenMode,
    /// CSV整列表示が有効かどうか
    pub is_csv_active: bool,
    /// 文字エンコーディング
    pub encoding: CharEncoding,
    /// 改行コード種別
    pub newline: NewlineType,
    /// シンタックスハイライト言語
    pub syntax_lang: SyntaxLang,
    /// スクロールオフセット (表示開始行, 表示開始列)
    pub scroll: (usize, usize),
    /// 開いているファイルのパス
    pub file_path: Option<PathBuf>,
    /// アプリケーション終了フラグ
    pub should_quit: bool,
    /// 最下行に表示する一時通知メッセージ
    pub status_message: Option<String>,

    // --- 画面分割 (Split / Vsplit) ---
    /// 開かれているエディタウィンドウ一覧
    pub windows: Vec<EditorWindow>,
    /// 現在フォーカスがあるウィンドウのインデックス
    pub active_window: usize,
    /// 分割方向 (水平/垂直)
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
}

impl App {
    /// 新規または指定したファイル内容から App を生成します。
    pub fn new(
        initial_text: &str,
        file_path: Option<PathBuf>,
        encoding: CharEncoding,
        newline: NewlineType,
    ) -> Self {
        let buffer = TextBuffer::from_text(initial_text);
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

        Self {
            buffer,
            cursor: initial_cursor,
            vi: ViEngine::new(),
            csv_mode,
            keisen_mode: KeisenMode::new(),
            is_csv_active: is_csv,
            encoding,
            newline,
            syntax_lang,
            scroll: (0, 0),
            file_path,
            should_quit: false,
            status_message: None,

            windows: vec![initial_window],
            active_window: 0,
            split_axis: SplitAxis::Horizontal,
            ctrl_w_pending: false,

            show_outline: false,
            outline_focus: false,
            outline_items: Vec::new(),
            selected_outline_idx: 0,
        }
    }

    /// 現在のカーソル位置（行, 列）
    pub fn cursor_pos(&self) -> CursorPosition {
        self.cursor.pos
    }

    /// 現在のViモード
    pub fn mode(&self) -> ViMode {
        self.vi.mode()
    }

    /// 単一文字をViEngineに入力し、確定したコマンドを実行します。
    pub fn feed_vi_char(&mut self, ch: char) {
        if let Some(cmd) = self.vi.feed_char(ch) {
            let res = self.vi.execute(&cmd, &mut self.buffer, &mut self.cursor);
            if let Some(msg) = res.message {
                self.status_message = Some(msg);
            }
        }
    }

    /// ファイル保存処理 (:w 等から呼ばれる)
    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.file_path {
            let content = self.buffer.to_string();
            let bytes = vivivi_core::encode_text(&content, self.encoding, self.newline);
            std::fs::write(path, bytes)?;
            self.buffer.on_saved();
            self.status_message = Some(format!(
                "\"{path:?}\" 書込完了 [{}/{}]",
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

    /// 罫線枠内テキスト整列の実行
    pub fn align_keisen_box(&mut self, alignment: BoxAlignment) {
        let cur_line = self.cursor_pos().line;
        if KeisenMode::align_box_text(&mut self.buffer, cur_line, alignment) {
            self.status_message = Some(format!("行 {cur_line} の罫線枠内テキストを整列しました"));
        } else {
            self.status_message = Some("罫線枠が検出されませんでした".to_string());
        }
    }

    /// 罫線描画・消去ステップの実行
    pub fn step_keisen(&mut self, dir: KeisenDirection, is_erase: bool) {
        self.keisen_mode
            .step(&mut self.buffer, &mut self.cursor.pos, dir, is_erase);
    }

    /// 画面サイズに合わせてスクロール位置を調整します。
    pub fn adjust_scroll(&mut self, view_height: usize, view_width: usize) {
        let cur = self.cursor_pos();

        // 垂直スクロール
        if cur.line < self.scroll.0 {
            self.scroll.0 = cur.line;
        } else if cur.line >= self.scroll.0 + view_height {
            self.scroll.0 = cur.line - view_height + 1;
        }

        // 水平スクロール
        if cur.col < self.scroll.1 {
            self.scroll.1 = cur.col;
        } else if cur.col >= self.scroll.1 + view_width {
            self.scroll.1 = cur.col - view_width + 1;
        }

        // アクティブウィンドウの状態と同期
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }
    }

    // --- 画面分割 (Split / Vsplit) 操作 ---

    /// 画面を水平分割 (:split)
    pub fn split_horizontal(&mut self) {
        let cur = self.cursor.clone();
        let scroll = self.scroll;
        self.split_axis = SplitAxis::Horizontal;
        self.windows.push(EditorWindow {
            cursor: cur,
            scroll,
        });
        self.active_window = self.windows.len() - 1;
        self.status_message = Some(format!(
            "水平分割: ウィンドウ {}/{}",
            self.active_window + 1,
            self.windows.len()
        ));
    }

    /// 画面を垂直分割 (:vsplit)
    pub fn split_vertical(&mut self) {
        let cur = self.cursor.clone();
        let scroll = self.scroll;
        self.split_axis = SplitAxis::Vertical;
        self.windows.push(EditorWindow {
            cursor: cur,
            scroll,
        });
        self.active_window = self.windows.len() - 1;
        self.status_message = Some(format!(
            "垂直分割: ウィンドウ {}/{}",
            self.active_window + 1,
            self.windows.len()
        ));
    }

    /// 次のウィンドウへフォーカス切り替え (Ctrl-w w)
    pub fn cycle_window(&mut self) {
        if self.windows.len() > 1 {
            // 現在のウィンドウ状態を退避
            self.windows[self.active_window].cursor = self.cursor.clone();
            self.windows[self.active_window].scroll = self.scroll;

            self.active_window = (self.active_window + 1) % self.windows.len();
            self.cursor = self.windows[self.active_window].cursor.clone();
            self.scroll = self.windows[self.active_window].scroll;
            self.status_message = Some(format!(
                "ウィンドウ: {}/{}",
                self.active_window + 1,
                self.windows.len()
            ));
        }
    }

    /// 現在のウィンドウを閉じる (:close, Ctrl-w c)
    pub fn close_window(&mut self) {
        if self.windows.len() > 1 {
            self.windows.remove(self.active_window);
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

    /// 現在のウィンドウのみを残す (:only)
    pub fn only_window(&mut self) {
        let cur = self.cursor.clone();
        let scroll = self.scroll;
        self.windows = vec![EditorWindow {
            cursor: cur,
            scroll,
        }];
        self.active_window = 0;
        self.status_message = Some("単一ウィンドウに統合しました".to_string());
    }

    // --- アウトライン解析操作 ---

    /// アウトラインペインの表示トグル (:outline, F8)
    pub fn toggle_outline(&mut self) {
        self.show_outline = !self.show_outline;
        if self.show_outline {
            self.refresh_outline();
            self.status_message = Some("アウトライン表示: ON (F8 で切替)".to_string());
        } else {
            self.outline_focus = false;
            self.status_message = Some("アウトライン表示: OFF".to_string());
        }
    }

    /// アウトライン情報の再解析
    pub fn refresh_outline(&mut self) {
        let lines: Vec<String> = (0..self.buffer.line_count())
            .filter_map(|i| self.buffer.line_text(i))
            .collect();
        self.outline_items = parse_outline(&lines, self.syntax_lang);
        if self.selected_outline_idx >= self.outline_items.len() {
            self.selected_outline_idx = 0;
        }
    }

    /// 選択中のアウトライン項目へジャンプ
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

    /// バッファ全文に対する文字変換 (:zenkaku, :hankaku, etc.)
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

        // 単一ウィンドウ統合
        app.only_window();
        assert_eq!(app.windows.len(), 1);
        assert_eq!(app.active_window, 0);
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
}
