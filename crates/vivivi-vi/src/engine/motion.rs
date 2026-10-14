//! カーソルモーション計算および単語移動処理モジュール。

use super::ViEngine;
use crate::command::Motion;
use vivivi_core::{TextBuffer, TextCursor};

impl ViEngine {
    /// 単語構成文字（英数字またはアンダースコア）かどうかを判定します。
    pub(crate) fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    /// カーソル位置の単語を取得します。
    pub(crate) fn word_under_cursor(
        &self,
        buffer: &TextBuffer,
        cursor: &TextCursor,
    ) -> Option<String> {
        let line = buffer.line_text(cursor.pos.line)?;
        let chars: Vec<char> = line.chars().collect();
        if cursor.pos.col >= chars.len() {
            return None;
        }

        let is_target_word = Self::is_word_char(chars[cursor.pos.col]);
        let mut start = cursor.pos.col;
        while start > 0
            && Self::is_word_char(chars[start - 1]) == is_target_word
            && !chars[start - 1].is_whitespace()
        {
            start -= 1;
        }

        let mut end = cursor.pos.col;
        while end < chars.len()
            && Self::is_word_char(chars[end]) == is_target_word
            && !chars[end].is_whitespace()
        {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// モーションを計算してカーソルを移動させます。
    pub(crate) fn execute_motion(
        &mut self,
        motion: &Motion,
        buffer: &TextBuffer,
        cursor: &mut TextCursor,
    ) {
        match motion {
            Motion::Left(count) => {
                cursor.pos.col = cursor.pos.col.saturating_sub(*count);
            }
            Motion::Right(count) => {
                cursor.pos.col += *count;
                self.clamp_cursor(buffer, cursor);
            }
            Motion::Up(count) => {
                cursor.pos.line = cursor.pos.line.saturating_sub(*count);
                self.clamp_cursor(buffer, cursor);
            }
            Motion::Down(count) => {
                cursor.pos.line =
                    (cursor.pos.line + *count).min(buffer.line_count().saturating_sub(1));
                self.clamp_cursor(buffer, cursor);
            }
            Motion::StartOfLine => {
                cursor.pos.col = 0;
            }
            Motion::EndOfLine => {
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                cursor.pos.col = line_len.saturating_sub(1);
            }
            Motion::FirstNonBlank => {
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            }
            Motion::DocumentStart => {
                cursor.pos.line = 0;
                cursor.pos.col = 0;
            }
            Motion::DocumentEnd(target) => {
                if let Some(target_line) = target {
                    cursor.pos.line = (*target_line)
                        .saturating_sub(1)
                        .min(buffer.line_count().saturating_sub(1));
                } else {
                    cursor.pos.line = buffer.line_count().saturating_sub(1);
                }
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            }
            Motion::NextWord(count) => {
                for _ in 0..*count {
                    self.move_next_word(buffer, cursor, false);
                }
            }
            Motion::PrevWord(count) => {
                for _ in 0..*count {
                    self.move_prev_word(buffer, cursor, false);
                }
            }
            Motion::EndWord(count) => {
                for _ in 0..*count {
                    self.move_end_word(buffer, cursor, false);
                }
            }
            Motion::NextBigWord(count) => {
                for _ in 0..*count {
                    self.move_next_word(buffer, cursor, true);
                }
            }
            Motion::PrevBigWord(count) => {
                for _ in 0..*count {
                    self.move_prev_word(buffer, cursor, true);
                }
            }
            Motion::EndBigWord(count) => {
                for _ in 0..*count {
                    self.move_end_word(buffer, cursor, true);
                }
            }
            Motion::MatchingParen => {
                let current_pos = buffer.cursor_to_position(cursor.pos);
                if let Some((_, match_pos)) =
                    vivivi_core::get_cursor_matching_paren(buffer, current_pos)
                {
                    cursor.pos = buffer.position_to_cursor(match_pos);
                }
            }
            Motion::FindCharForward {
                target,
                till,
                count,
            } => {
                self.last_find_char = Some((*target, true, *till));
                if let Some(new_col) = buffer.line_text(cursor.pos.line).and_then(|line| {
                    vivivi_modes::find_char_forward(&line, cursor.pos.col, *target, *till, *count)
                }) {
                    cursor.pos.col = new_col;
                }
            }
            Motion::FindCharBackward {
                target,
                till,
                count,
            } => {
                self.last_find_char = Some((*target, false, *till));
                if let Some(new_col) = buffer.line_text(cursor.pos.line).and_then(|line| {
                    vivivi_modes::find_char_backward(&line, cursor.pos.col, *target, *till, *count)
                }) {
                    cursor.pos.col = new_col;
                }
            }
            Motion::RepeatFindChar { reverse, count } => {
                if let Some((target, is_forward, till)) = self.last_find_char {
                    let actual_forward = if *reverse { !is_forward } else { is_forward };
                    if let Some(line) = buffer.line_text(cursor.pos.line) {
                        let new_col = if actual_forward {
                            vivivi_modes::find_char_forward(
                                &line,
                                cursor.pos.col,
                                target,
                                till,
                                *count,
                            )
                        } else {
                            vivivi_modes::find_char_backward(
                                &line,
                                cursor.pos.col,
                                target,
                                till,
                                *count,
                            )
                        };
                        if let Some(c) = new_col {
                            cursor.pos.col = c;
                        }
                    }
                }
            }
        }
    }

    /// 次の単語先頭への移動 (`w`, `W`)
    pub(crate) fn move_next_word(
        &self,
        buffer: &TextBuffer,
        cursor: &mut TextCursor,
        is_big: bool,
    ) {
        let mut pos = buffer.cursor_to_position(cursor.pos);
        let total_len = buffer.len();
        if pos >= total_len {
            return;
        }

        let start_ch = buffer.char_at(pos).unwrap_or(' ');
        let start_type = Self::char_class(start_ch, is_big);

        // 1. 現在の単語・記号グループを抜ける
        while pos < total_len {
            if buffer
                .char_at(pos)
                .is_some_and(|ch| ch == '\n' || Self::char_class(ch, is_big) != start_type)
            {
                break;
            }
            pos += 1;
        }

        // 2. 空白文字をスキップする
        while pos < total_len {
            if buffer.char_at(pos).is_some_and(|ch| !ch.is_whitespace()) {
                break;
            }
            pos += 1;
        }

        cursor.pos = buffer.position_to_cursor(pos.min(total_len.saturating_sub(1)));
    }

    /// 前の単語先頭への移動 (`b`, `B`)
    pub(crate) fn move_prev_word(
        &self,
        buffer: &TextBuffer,
        cursor: &mut TextCursor,
        is_big: bool,
    ) {
        let mut pos = buffer.cursor_to_position(cursor.pos);
        if pos == 0 {
            return;
        }
        pos -= 1;

        // 1. 直前の空白をスキップ
        while pos > 0 {
            if buffer.char_at(pos).is_some_and(|ch| !ch.is_whitespace()) {
                break;
            }
            pos -= 1;
        }

        let target_ch = buffer.char_at(pos).unwrap_or(' ');
        let target_type = Self::char_class(target_ch, is_big);

        // 2. 同一単語クラスの先頭まで巻き戻す
        while pos > 0 {
            if buffer
                .char_at(pos - 1)
                .is_some_and(|ch| ch.is_whitespace() || Self::char_class(ch, is_big) != target_type)
            {
                break;
            }
            pos -= 1;
        }

        cursor.pos = buffer.position_to_cursor(pos);
    }

    /// 単語末尾への移動 (`e`, `E`)
    pub(crate) fn move_end_word(&self, buffer: &TextBuffer, cursor: &mut TextCursor, is_big: bool) {
        let mut pos = buffer.cursor_to_position(cursor.pos);
        let total_len = buffer.len();
        if pos + 1 >= total_len {
            return;
        }
        pos += 1;

        // 空白スキップ
        while pos < total_len {
            if buffer.char_at(pos).is_some_and(|ch| !ch.is_whitespace()) {
                break;
            }
            pos += 1;
        }

        let target_ch = buffer.char_at(pos).unwrap_or(' ');
        let target_type = Self::char_class(target_ch, is_big);

        // 同一単語クラスの末尾へ進める
        while pos + 1 < total_len {
            if buffer
                .char_at(pos + 1)
                .is_some_and(|ch| ch.is_whitespace() || Self::char_class(ch, is_big) != target_type)
            {
                break;
            }
            pos += 1;
        }

        cursor.pos = buffer.position_to_cursor(pos);
    }

    /// 文字の種類（単語、記号、空白）を分類
    pub(crate) fn char_class(ch: char, is_big: bool) -> u8 {
        if ch.is_whitespace() {
            0
        } else if is_big || Self::is_word_char(ch) {
            1
        } else {
            2
        }
    }
}
