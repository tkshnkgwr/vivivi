use std::path::PathBuf;
use vivivi_core::{CharEncoding, CursorPosition, NewlineType, TextBuffer, TextCursor};
use vivivi_modes::{BoxAlignment, CsvMode, KeisenDirection, KeisenMode, SyntaxLang};
use vivivi_vi::{ViEngine, ViMode};

/// TUI アプリケーションの状態管理構造体。
pub struct App {
    /// 編集中のテキストバッファ
    pub buffer: TextBuffer,
    /// バッファ上のカーソル（行・列および選択範囲）
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

        Self {
            buffer,
            cursor: TextCursor::new(0, 0),
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
    }
}
