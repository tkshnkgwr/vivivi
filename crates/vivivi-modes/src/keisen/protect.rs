use super::*;
use vivivi_core::TextBuffer;

impl KeisenMode {
    /// 罫線で囲まれた枠内の文字列を指定した揃え（左・中央・右）に整列します。
    ///
    /// 行内の左右の縦罫線（または行頭・行末）に挟まれたテキスト区間を検出し、
    /// 幅に合わせて前後の空白を再配置します。
    ///
    /// # Returns
    /// 整列処理が実行された場合は `true`、縦罫線が2本未満など整列対象が存在しない場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    /// use vivivi_modes::keisen::{BoxAlignment, KeisenMode};
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ hello   │");
    /// KeisenMode::align_box_text(&mut buffer, 0, BoxAlignment::Right);
    /// assert_eq!(buffer.get_line(0).unwrap(), "│    hello│");
    /// ```
    pub fn align_box_text(
        buffer: &mut TextBuffer,
        line_idx: usize,
        alignment: BoxAlignment,
    ) -> bool {
        let line_text = match buffer.get_line(line_idx) {
            Some(t) => t,
            None => return false,
        };

        // 縦罫線の位置（charインデックス）を探索
        let vertical_indices: Vec<usize> = line_text
            .chars()
            .enumerate()
            .filter(|(_, ch)| is_vertical_keisen(*ch))
            .map(|(i, _)| i)
            .collect();

        if vertical_indices.len() < 2 {
            return false;
        }

        buffer.open_undo_block();
        let chars: Vec<char> = line_text.chars().collect();
        let mut new_chars = chars.clone();

        for window in vertical_indices.windows(2) {
            let left_v = window[0];
            let right_v = window[1];
            if right_v <= left_v + 1 {
                continue;
            }

            let segment: String = chars[left_v + 1..right_v].iter().collect();
            let trimmed = segment.trim();
            let total_width = right_v - left_v - 1;
            let text_width = UnicodeWidthStr::width(trimmed);

            if text_width > total_width {
                continue;
            }

            let space_diff = total_width.saturating_sub(text_width);
            let (left_pad, right_pad) = match alignment {
                BoxAlignment::Left => (0, space_diff),
                BoxAlignment::Right => (space_diff, 0),
                BoxAlignment::Center => (space_diff / 2, space_diff - (space_diff / 2)),
            };

            let aligned_content = format!(
                "{}{}{}",
                " ".repeat(left_pad),
                trimmed,
                " ".repeat(right_pad)
            );
            let aligned_chars: Vec<char> = aligned_content.chars().collect();

            // 長さが一致する場合に置換
            if aligned_chars.len() == total_width {
                new_chars.splice(left_v + 1..right_v, aligned_chars);
            }
        }

        let new_line_str: String = new_chars.into_iter().collect();
        if new_line_str != line_text {
            let start = buffer.line_start_position(line_idx);
            let end = start + line_text.chars().count();
            buffer.delete_range(start, end);
            buffer.insert_text(start, &new_line_str);
        }

        buffer.close_undo_block();
        true
    }

    /// カーソル位置を囲む左右の縦罫線のカラム位置（左罫線, 右罫線）を探索します。
    ///
    /// 左右に縦罫線が存在し、カーソルがその内部にある場合に `Some((左列, 右列))` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::KeisenMode;
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ test │");
    /// let cols = KeisenMode::find_enclosing_box_cols(&buffer, CursorPosition::new(0, 3));
    /// assert_eq!(cols, Some((0, 7)));
    /// ```
    pub fn find_enclosing_box_cols(
        buffer: &TextBuffer,
        pos: CursorPosition,
    ) -> Option<(usize, usize)> {
        let line_text = buffer.get_line(pos.line)?;
        let chars: Vec<char> = line_text.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let mut left_col = None;
        let mut right_col = None;

        // カーソル以前の直近の縦罫線
        for i in (0..pos.col.min(chars.len())).rev() {
            if is_vertical_keisen(chars[i]) {
                left_col = Some(i);
                break;
            }
        }

        // カーソル以降の直近の縦罫線
        for (i, &ch) in chars.iter().enumerate().skip(pos.col) {
            if is_vertical_keisen(ch) {
                right_col = Some(i);
                break;
            }
        }

        match (left_col, right_col) {
            (Some(l), Some(r)) if l < r => Some((l, r)),
            _ => None,
        }
    }

    /// 罫線枠保護モードでの文字挿入。
    ///
    /// 枠線内にカーソルがある場合、カーソルから右枠線までの間にある末尾の空白文字を削ることで、
    /// 外側の枠線位置を保ったまま文字を挿入します。空白の余白がない場合は枠の崩れを防ぐため挿入を拒否（`false`）します。
    /// 枠外の場合は通常の文字挿入を行います。
    ///
    /// # Returns
    /// 挿入に成功した場合は `true`、枠内に余白がなく拒否された場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::KeisenMode;
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ a   │");
    /// let mut cursor = CursorPosition::new(0, 3);
    /// assert!(KeisenMode::insert_char_protected(&mut buffer, &mut cursor, 'b'));
    /// assert_eq!(buffer.get_line(0).unwrap(), "│ ab  │");
    ///
    /// // 全角文字（幅2）の入力時は末尾の半角スペース2文字分を自動消費して枠幅を完全維持
    /// let mut cursor_zenkaku = CursorPosition::new(0, 1);
    /// assert!(KeisenMode::insert_char_protected(&mut buffer, &mut cursor_zenkaku, 'あ'));
    /// assert_eq!(buffer.get_line(0).unwrap(), "│あ ab│");
    /// ```
    pub fn insert_char_protected(
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        ch: char,
    ) -> bool {
        if let Some((left_col, right_col)) = Self::find_enclosing_box_cols(buffer, *cursor)
            && cursor.col > left_col
            && cursor.col < right_col
        {
            let line_text = match buffer.get_line(cursor.line) {
                Some(t) => t,
                None => return false,
            };
            let chars: Vec<char> = line_text.chars().collect();
            let char_w = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);

            // カーソル位置から右罫線直前までの範囲で、末尾側から必要な幅分の空白を探索
            let mut collected_spaces: Vec<(usize, usize)> = Vec::new(); // (col, width)
            let mut accumulated_width = 0;

            for i in (cursor.col..right_col).rev() {
                let c = chars[i];
                if c == ' ' {
                    collected_spaces.push((i, 1));
                    accumulated_width += 1;
                } else if c == '　' {
                    collected_spaces.push((i, 2));
                    accumulated_width += 2;
                }
                if accumulated_width >= char_w {
                    break;
                }
            }

            if accumulated_width >= char_w {
                buffer.open_undo_block();
                let line_start = buffer.line_start_position(cursor.line);

                // 1. 空白を末尾側から削除 (インデックスを保持するため降順に削除)
                for &(col, _) in &collected_spaces {
                    buffer.delete_range(line_start + col, line_start + col + 1);
                }

                // 2. 削りすぎた幅（全角スペースを半角文字で消費した場合等）があれば半角スペースを補填
                let extra_width = accumulated_width - char_w;
                let adjusted_right_col = right_col - collected_spaces.len();
                for _ in 0..extra_width {
                    buffer.insert_char(line_start + adjusted_right_col, ' ');
                }

                // 3. カーソル位置に文字を挿入
                let insert_pos = line_start + cursor.col;
                buffer.insert_char(insert_pos, ch);

                cursor.col += 1;
                buffer.close_undo_block();
                return true;
            } else {
                // 枠内に十分な余白がないため、枠線を壊さないよう挿入拒否
                return false;
            }
        }

        // 枠外の場合は通常の文字挿入
        let abs_pos = buffer.cursor_to_position(*cursor);
        buffer.insert_char(abs_pos, ch);
        cursor.col += 1;
        true
    }

    /// 罫線枠保護モードでの文字削除（BackSpaceまたはDelete）。
    ///
    /// 枠線内にカーソルがある場合、文字を削除した分（全角文字なら2文字分、半角文字なら1文字分）だけ
    /// 右枠線の直前に空白を自動補填し、外側の枠線位置を保ちます。枠線の文字自体は誤消去されないよう保護されます。
    ///
    /// # Returns
    /// 削除に成功した場合は `true`、枠線自体の削除試行など無効な操作の場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::KeisenMode;
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ a b │");
    /// let mut cursor = CursorPosition::new(0, 5); // 'b' の直後
    /// assert!(KeisenMode::delete_char_protected(&mut buffer, &mut cursor, true));
    /// assert_eq!(buffer.get_line(0).unwrap(), "│ a   │");
    /// ```
    pub fn delete_char_protected(
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        is_backspace: bool,
    ) -> bool {
        if let Some((left_col, right_col)) = Self::find_enclosing_box_cols(buffer, *cursor) {
            let line_text = match buffer.get_line(cursor.line) {
                Some(t) => t,
                None => return false,
            };
            let chars: Vec<char> = line_text.chars().collect();

            let delete_col = if is_backspace {
                if cursor.col <= left_col + 1 {
                    return false; // 左枠線は削除させない
                }
                cursor.col - 1
            } else {
                if cursor.col >= right_col {
                    return false; // 右枠線は削除させない
                }
                cursor.col
            };

            if delete_col > left_col && delete_col < right_col && delete_col < chars.len() {
                let del_ch = chars[delete_col];
                let del_width = UnicodeWidthChar::width(del_ch).unwrap_or(1).max(1);

                buffer.open_undo_block();
                let line_start = buffer.line_start_position(cursor.line);

                // 1. 指定位置の文字を削除
                buffer.delete_range(line_start + delete_col, line_start + delete_col + 1);

                // 2. 削除した文字の表示幅分（半角なら1、全角なら2）右枠線の手前に空白を補填
                let pad_pos = line_start + right_col - 1;
                for _ in 0..del_width {
                    buffer.insert_char(pad_pos, ' ');
                }

                if is_backspace {
                    cursor.col -= 1;
                }
                buffer.close_undo_block();
                return true;
            }
        }

        // 枠外の場合は通常の削除
        let abs_pos = buffer.cursor_to_position(*cursor);
        if is_backspace {
            if abs_pos > 0 {
                buffer.delete_range(abs_pos - 1, abs_pos);
                cursor.col = cursor.col.saturating_sub(1);
                return true;
            }
        } else if abs_pos < buffer.len() {
            buffer.delete_range(abs_pos, abs_pos + 1);
            return true;
        }
        false
    }
}
