//! 通常モードのオペレータ実行（削除・ヤンク・変更）および貼り付け処理モジュール。

use super::ViEngine;
use crate::command::{Motion, Operator};
use crate::mode::ViMode;
use crate::register::RegisterContent;
use vivivi_core::{TextBuffer, TextCursor};

impl ViEngine {
    /// オペレータ＋モーションの実行 (`dw`, `cw`, `y$`)
    pub(crate) fn execute_operate(
        &mut self,
        op: Operator,
        motion: &Motion,
        reg: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let start_pos = buffer.cursor_to_position(cursor.pos);
        let mut target_cursor = cursor.clone();
        self.execute_motion(motion, buffer, &mut target_cursor);
        let end_pos = buffer.cursor_to_position(target_cursor.pos);

        let is_inclusive = matches!(
            motion,
            Motion::FindCharForward { till: false, .. }
                | Motion::EndOfLine
                | Motion::EndWord(_)
                | Motion::EndBigWord(_)
                | Motion::MatchingParen
        );

        let (from, mut to) = if start_pos <= end_pos {
            (start_pos, end_pos)
        } else {
            (end_pos, start_pos)
        };

        if is_inclusive && to < buffer.len() {
            to += 1;
        }

        let len = to - from;
        if len == 0 {
            return;
        }

        let text = buffer.get_text(from, len);
        self.registers
            .set(reg, RegisterContent::CharacterWise(text));

        match op {
            Operator::Yank => {
                // ヤンクはバッファを変更しない
            }
            Operator::Delete => {
                buffer.open_undo_block();
                buffer.delete_text(from, len, false);
                buffer.close_undo_block();
                cursor.pos = buffer.position_to_cursor(from);
                self.clamp_cursor(buffer, cursor);
            }
            Operator::Change => {
                buffer.open_undo_block();
                buffer.delete_text(from, len, false);
                buffer.close_undo_block();
                cursor.pos = buffer.position_to_cursor(from);
                self.mode = ViMode::Insert;
            }
            Operator::ShiftRight | Operator::ShiftLeft => {
                let start_line = cursor.pos.line.min(target_cursor.pos.line);
                let end_line =
                    (cursor.pos.line.max(target_cursor.pos.line) + 1).min(buffer.line_count());
                if op == Operator::ShiftRight {
                    Self::indent_lines_right(buffer, start_line, end_line);
                } else {
                    Self::indent_lines_left(buffer, start_line, end_line);
                }
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            }
        }
    }

    /// 指定行範囲を右にインデント（4スペース追加）します。
    pub(crate) fn indent_lines_right(buffer: &mut TextBuffer, start_line: usize, end_line: usize) {
        buffer.open_undo_block();
        for l in start_line..end_line {
            let line_text = buffer.line_text(l).unwrap_or_default();
            if !line_text.is_empty() {
                let pos = buffer.line_start_position(l);
                buffer.insert_text(pos, "    ");
            }
        }
        buffer.close_undo_block();
    }

    /// 指定行範囲を左にアンインデント（最大4スペースまたはタブ1つ削除）します。
    pub(crate) fn indent_lines_left(buffer: &mut TextBuffer, start_line: usize, end_line: usize) {
        buffer.open_undo_block();
        for l in start_line..end_line {
            let line_text = buffer.line_text(l).unwrap_or_default();
            if line_text.is_empty() {
                continue;
            }
            let pos = buffer.line_start_position(l);
            let mut to_remove = 0;
            let mut spaces = 0;
            for ch in line_text.chars() {
                if ch == '\t' {
                    to_remove += 1;
                    break;
                } else if ch == ' ' {
                    to_remove += 1;
                    spaces += 1;
                    if spaces == 4 {
                        break;
                    }
                } else {
                    break;
                }
            }
            if to_remove > 0 {
                buffer.delete_text(pos, to_remove, false);
            }
        }
        buffer.close_undo_block();
    }

    /// 行単位オペレータの実行 (`dd`, `yy`, `cc`, `>>`, `<<`)
    pub(crate) fn execute_operate_line(
        &mut self,
        op: Operator,
        count: usize,
        reg: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let start_line = cursor.pos.line;
        let end_line = (start_line + count).min(buffer.line_count());
        let start_pos = buffer.line_start_position(start_line);
        let end_pos = buffer.line_start_position(end_line);

        let len = end_pos.saturating_sub(start_pos);
        if len == 0 {
            return;
        }

        if matches!(op, Operator::Yank | Operator::Delete | Operator::Change) {
            let text = buffer.get_text(start_pos, len);
            self.registers.set(reg, RegisterContent::LineWise(text));
        }

        match op {
            Operator::Yank => {
                // ヤンクは変更なし
            }
            Operator::Delete => {
                buffer.open_undo_block();
                buffer.delete_text(start_pos, len, false);
                buffer.close_undo_block();
                self.clamp_cursor(buffer, cursor);
            }
            Operator::Change => {
                buffer.open_undo_block();
                buffer.delete_text(start_pos, len, false);
                // 新しい行を1行挿入してインデント位置へ
                buffer.insert_text(start_pos, "\n");
                buffer.close_undo_block();
                cursor.pos.col = 0;
                self.mode = ViMode::Insert;
            }
            Operator::ShiftRight => {
                Self::indent_lines_right(buffer, start_line, end_line);
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            }
            Operator::ShiftLeft => {
                Self::indent_lines_left(buffer, start_line, end_line);
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            }
        }
    }

    /// 貼り付けの実行 (`p`, `P`)
    pub(crate) fn execute_paste(
        &mut self,
        after: bool,
        count: usize,
        reg: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let content = match self.registers.fetch(reg) {
            Some(c) => c,
            None => return,
        };

        buffer.open_undo_block();
        for _ in 0..count {
            match &content {
                RegisterContent::LineWise(text) => {
                    if after {
                        let insert_pos = buffer.line_start_position(cursor.pos.line + 1);
                        buffer.insert_text(insert_pos, text);
                        cursor.pos.line += 1;
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    } else {
                        let insert_pos = buffer.line_start_position(cursor.pos.line);
                        buffer.insert_text(insert_pos, text);
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    }
                }
                RegisterContent::CharacterWise(text) => {
                    let cur_pos = buffer.cursor_to_position(cursor.pos);
                    let insert_pos = if after {
                        (cur_pos + 1).min(buffer.len())
                    } else {
                        cur_pos
                    };
                    buffer.insert_text(insert_pos, text);
                    let new_pos = insert_pos + text.chars().count().saturating_sub(1);
                    cursor.pos = buffer.position_to_cursor(new_pos);
                }
                RegisterContent::BlockWise(lines) => {
                    let start_line = cursor.pos.line;
                    let paste_col = if after {
                        cursor.pos.col + 1
                    } else {
                        cursor.pos.col
                    };

                    for (i, block_line) in lines.iter().enumerate() {
                        let cur_line = start_line + i;
                        while cur_line >= buffer.line_count() {
                            buffer.insert_char(buffer.len(), '\n');
                        }

                        let line_len = buffer
                            .line_text(cur_line)
                            .map(|s| s.chars().count())
                            .unwrap_or(0);

                        if line_len < paste_col {
                            let padding = " ".repeat(paste_col - line_len);
                            let insert_pos = buffer.line_start_position(cur_line) + line_len;
                            buffer.insert_text(insert_pos, &padding);
                        }

                        let insert_pos = buffer.line_start_position(cur_line) + paste_col;
                        buffer.insert_text(insert_pos, block_line);
                    }
                }
            }
        }
        buffer.close_undo_block();
    }
}
