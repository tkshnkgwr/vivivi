//! 罫線操作・スクロール・テキスト一括整形（トリム/ソート）・Hexモード切替モジュール。

use super::App;
use vivivi_core::{CursorPosition, TextBuffer, TextCursor};
use vivivi_modes::{BoxAlignment, HexCursor, HexDocument, KeisenDirection, KeisenMode};
use vivivi_vi::ViMode;

impl App {
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
}
