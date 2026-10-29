use super::*;
use vivivi_core::TextBuffer;

impl KeisenMode {
    /// 罫線の描画または消去を1ステップ実行し、カーソルを移動します。
    ///
    /// 現在のカーソルセルと移動先セルの接続ビットマスクを更新し、
    /// 適切なUnicode罫線文字へ置き換えます。必要に応じて末尾空白や空行を自動拡張します。
    ///
    /// # Parameters
    /// - `buffer`: 編集対象のテキストバッファ。
    /// - `cursor`: 現在のカーソル座標（操作後に進行方向へ1ステップ更新されます）。
    /// - `dir`: 描画・消去の進行方向。
    /// - `is_erase`: `true` の場合は罫線消去、`false` の場合は描画。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::{KeisenDirection, KeisenMode};
    ///
    /// let mut buffer = TextBuffer::new();
    /// let mut cursor = CursorPosition::new(0, 0);
    /// let mode = KeisenMode::new();
    ///
    /// mode.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    /// assert_eq!(cursor.line, 1);
    /// ```
    pub fn step(
        &self,
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        dir: KeisenDirection,
        is_erase: bool,
    ) {
        buffer.open_undo_block();

        // 0. 現在位置の行・列を確実に確保
        while buffer.line_count() <= cursor.line {
            let end_pos = buffer.len();
            buffer.insert_char(end_pos, '\n');
        }
        Self::pad_line_to_col(buffer, cursor.line, cursor.col);

        // 1. 移動先の位置を確保・計算
        let next_pos = Self::ensure_target_position(buffer, *cursor, dir);
        if *cursor == next_pos {
            buffer.close_undo_block();
            return;
        }

        if is_erase {
            // 消去の場合: 現在セルから進行方向、移動先セルから逆方向の接続を除去
            Self::update_keisen_cell(buffer, *cursor, None, Some(dir), self.keisen_type);
            Self::update_keisen_cell(
                buffer,
                next_pos,
                None,
                Some(dir.opposite()),
                self.keisen_type,
            );
        } else {
            // 描画の場合: 現在セルに進行方向、移動先セルに逆方向の接続を追加
            Self::update_keisen_cell(buffer, *cursor, Some(dir), None, self.keisen_type);
            Self::update_keisen_cell(
                buffer,
                next_pos,
                Some(dir.opposite()),
                None,
                self.keisen_type,
            );
        }

        // 2. カーソルを移動先に更新
        *cursor = next_pos;

        buffer.close_undo_block();
    }

    /// セル `pos` の罫線状態を、隣接セルとの接続関係に基づいて精密に更新します。
    fn update_keisen_cell(
        buffer: &mut TextBuffer,
        pos: CursorPosition,
        add_dir: Option<KeisenDirection>,
        remove_dir: Option<KeisenDirection>,
        keisen_type: KeisenType,
    ) {
        let abs_pos = buffer.cursor_to_position(pos);
        let curr_ch = buffer.char_at(abs_pos).unwrap_or(' ');

        // 隣接セルからの本物の接続を取得
        let neighbors = Self::neighbor_connections(buffer, pos, keisen_type);

        // 新しい状態の計算
        let new_state = if let Some(dir) = add_dir {
            // 現在の隣接接続 ＋ 新たに追加する接続
            neighbors | dir.flag(keisen_type)
        } else if let Some(dir) = remove_dir {
            // 指定方向を消去
            neighbors & !dir.mask()
        } else {
            neighbors
        };

        let replace_ch = if new_state == 0 {
            if char_to_keisen_state(curr_ch).is_some() {
                ' '
            } else {
                curr_ch
            }
        } else if let Some(ch) = state_to_keisen_char(new_state) {
            ch
        } else {
            // 接続が1方向のみ（端点）の場合、直線の罫線文字を表示
            let is_horizontal = (new_state & (KT_LEFT_MASK | KT_RIGHT_MASK)) != 0;
            let is_vertical = (new_state & (KT_UP_MASK | KT_DOWN_MASK)) != 0;
            if is_horizontal && !is_vertical {
                state_to_keisen_char(KeisenDirection::Right.axis_flags(keisen_type)).unwrap_or('─')
            } else if is_vertical && !is_horizontal {
                state_to_keisen_char(KeisenDirection::Down.axis_flags(keisen_type)).unwrap_or('│')
            } else {
                curr_ch
            }
        };

        if replace_ch != curr_ch {
            buffer.delete_range(abs_pos, abs_pos + 1);
            buffer.insert_char(abs_pos, replace_ch);
        }
    }

    /// 指定位置の隣接4セルから、こちら側に向かって接続している方向ビットマスクを取得します。
    ///
    /// 周囲のセル（上・下・左・右）が現在位置に対して接続を持つ罫線文字であるかを検査し、
    /// 現在セルが接続すべき方向のビットフラグを合成して返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::{KeisenMode, KeisenType};
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│\n \n");
    /// // 2行目の位置 (1, 0) から見て上の行 (0, 0) に垂直罫線がある
    /// let mask = KeisenMode::neighbor_connections(&buffer, CursorPosition::new(1, 0), KeisenType::Thin);
    /// assert_ne!(mask, 0);
    /// ```
    pub fn neighbor_connections(
        buffer: &TextBuffer,
        pos: CursorPosition,
        keisen_type: KeisenType,
    ) -> u8 {
        let mut mask = 0u8;

        // 上のセルが下への接続を持っているか
        if pos.line > 0 {
            let up_state = Self::get_cell_keisen_state(buffer, pos.line - 1, pos.col);
            if (up_state & (KT_DOWN_THIN | KT_DOWN_THICK)) != 0 {
                mask |= KeisenDirection::Up.flag(keisen_type);
            }
        }

        // 下のセルが上への接続を持っているか
        let down_state = Self::get_cell_keisen_state(buffer, pos.line + 1, pos.col);
        if (down_state & (KT_UP_THIN | KT_UP_THICK)) != 0 {
            mask |= KeisenDirection::Down.flag(keisen_type);
        }

        // 左のセルが右への接続を持っているか
        if pos.col > 0 {
            let left_state = Self::get_cell_keisen_state(buffer, pos.line, pos.col - 1);
            if (left_state & (KT_RIGHT_THIN | KT_RIGHT_THICK)) != 0 {
                mask |= KeisenDirection::Left.flag(keisen_type);
            }
        }

        // 右のセルが左への接続を持っているか
        let right_state = Self::get_cell_keisen_state(buffer, pos.line, pos.col + 1);
        if (right_state & (KT_LEFT_THIN | KT_LEFT_THICK)) != 0 {
            mask |= KeisenDirection::Right.flag(keisen_type);
        }

        mask
    }

    /// 指定セルの罫線状態ビットマスクを取得します（範囲外や空白なら 0）。
    ///
    /// 行番号や列番号が存在しない範囲を指している場合、またはセルが罫線文字ではない場合は `0` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    /// use vivivi_modes::keisen::{KeisenMode, KT_LEFT_THIN, KT_RIGHT_THIN};
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "─");
    /// let state = KeisenMode::get_cell_keisen_state(&buffer, 0, 0);
    /// assert_eq!(state, KT_LEFT_THIN | KT_RIGHT_THIN);
    /// ```
    pub fn get_cell_keisen_state(buffer: &TextBuffer, line: usize, col: usize) -> u8 {
        if line >= buffer.line_count() {
            return 0;
        }
        let line_text = match buffer.get_line(line) {
            Some(t) => t,
            None => return 0,
        };
        let chars: Vec<char> = line_text.chars().collect();
        if col >= chars.len() {
            return 0;
        }
        char_to_keisen_state(chars[col]).unwrap_or(0)
    }

    /// 移動先の座標を確保します（行末外や存在しない行への移動時に空白・改行を補完）。
    fn ensure_target_position(
        buffer: &mut TextBuffer,
        pos: CursorPosition,
        dir: KeisenDirection,
    ) -> CursorPosition {
        match dir {
            KeisenDirection::Right => {
                let target = CursorPosition::new(pos.line, pos.col + 1);
                Self::pad_line_to_col(buffer, target.line, target.col);
                target
            }
            KeisenDirection::Left => CursorPosition::new(pos.line, pos.col.saturating_sub(1)),
            KeisenDirection::Down => {
                let next_line = pos.line + 1;
                while buffer.line_count() <= next_line {
                    let end_pos = buffer.len();
                    buffer.insert_char(end_pos, '\n');
                }
                Self::pad_line_to_col(buffer, next_line, pos.col);
                CursorPosition::new(next_line, pos.col)
            }
            KeisenDirection::Up => {
                let prev_line = pos.line.saturating_sub(1);
                Self::pad_line_to_col(buffer, prev_line, pos.col);
                CursorPosition::new(prev_line, pos.col)
            }
        }
    }

    /// 指定行の文字数が `target_col` に満たない場合、スペースでパディングします。
    fn pad_line_to_col(buffer: &mut TextBuffer, line: usize, target_col: usize) {
        if let Some(line_text) = buffer.get_line(line) {
            let char_count = line_text.chars().count();
            if char_count < target_col + 1 {
                let end_of_line_pos = buffer.line_start_position(line) + char_count;
                let spaces_needed = target_col + 1 - char_count;
                let padding = " ".repeat(spaces_needed);
                buffer.insert_text(end_of_line_pos, &padding);
            }
        }
    }
}
