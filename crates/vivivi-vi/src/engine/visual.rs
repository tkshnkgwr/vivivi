//! ビジュアルモード（文字/行/矩形選択）および矩形編集処理モジュール。

use super::ViEngine;
use crate::command::Operator;
use crate::mode::ViMode;
use crate::register::RegisterContent;
use vivivi_core::{TextBuffer, TextCursor};

/// 矩形一括挿入セッションの情報。
#[derive(Debug, Clone)]
pub struct BlockInsertSession {
    pub start_line: usize,
    pub end_line: usize,
    pub col: usize,
    pub typed_chars: Vec<char>,
}

impl ViEngine {
    /// ビジュアルモードでの操作（削除・ヤンク・変更）
    pub(crate) fn execute_visual_operate(
        &mut self,
        op: Operator,
        register: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let anchor = cursor.anchor.unwrap_or(cursor.pos);
        let pos = cursor.pos;

        let min_line = anchor.line.min(pos.line);
        let max_line = anchor.line.max(pos.line);
        let min_col = anchor.col.min(pos.col);
        let max_col = anchor.col.max(pos.col);

        if matches!(op, Operator::ShiftRight | Operator::ShiftLeft) {
            let start_line = min_line;
            let end_line = (max_line + 1).min(buffer.line_count());
            if op == Operator::ShiftRight {
                Self::indent_lines_right(buffer, start_line, end_line);
            } else {
                Self::indent_lines_left(buffer, start_line, end_line);
            }
            cursor.clear_selection();
            cursor.pos.line = min_line;
            cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            self.mode = ViMode::Normal;
            return;
        }

        if matches!(op, Operator::ToLower | Operator::ToUpper) {
            buffer.open_undo_block();
            if self.mode == ViMode::VisualBlock {
                for line in min_line..=max_line {
                    if line >= buffer.line_count() {
                        continue;
                    }
                    let line_str = buffer.line_text(line).unwrap_or_default();
                    let chars: Vec<char> = line_str.chars().collect();
                    let start = min_col.min(chars.len());
                    let end = (max_col + 1).min(chars.len());

                    if start < end {
                        let original: String = chars[start..end].iter().collect();
                        let converted = if op == Operator::ToLower {
                            original.to_lowercase()
                        } else {
                            original.to_uppercase()
                        };
                        let replace_start = buffer.line_start_position(line) + start;
                        buffer.delete_text(replace_start, end - start, false);
                        buffer.insert_text(replace_start, &converted);
                    }
                }
                cursor.pos.line = min_line;
                cursor.pos.col = min_col;
            } else if self.mode == ViMode::VisualLine {
                let start_pos = buffer.line_start_position(min_line);
                let end_pos = buffer.line_start_position(max_line + 1);
                let len = end_pos.saturating_sub(start_pos);
                if len > 0 {
                    let text = buffer.get_text(start_pos, len);
                    let converted = if op == Operator::ToLower {
                        text.to_lowercase()
                    } else {
                        text.to_uppercase()
                    };
                    buffer.delete_text(start_pos, len, false);
                    buffer.insert_text(start_pos, &converted);
                }
                cursor.pos.line = min_line.min(buffer.line_count().saturating_sub(1));
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            } else {
                let p1 = buffer.cursor_to_position(anchor);
                let p2 = buffer.cursor_to_position(pos);
                let from = p1.min(p2);
                let to = (p1.max(p2) + 1).min(buffer.len());
                let len = to.saturating_sub(from);
                if len > 0 {
                    let text = buffer.get_text(from, len);
                    let converted = if op == Operator::ToLower {
                        text.to_lowercase()
                    } else {
                        text.to_uppercase()
                    };
                    buffer.delete_text(from, len, false);
                    buffer.insert_text(from, &converted);
                }
                cursor.pos = buffer.position_to_cursor(from);
            }
            buffer.close_undo_block();
            cursor.clear_selection();
            self.mode = ViMode::Normal;
            return;
        }

        match self.mode {
            ViMode::VisualBlock => {
                buffer.open_undo_block();
                let mut block_lines = Vec::new();

                for line in min_line..=max_line {
                    if line >= buffer.line_count() {
                        continue;
                    }
                    let line_str = buffer.line_text(line).unwrap_or_default();
                    let chars: Vec<char> = line_str.chars().collect();

                    let start = min_col.min(chars.len());
                    let end = (max_col + 1).min(chars.len());

                    let slice: String = if start < end {
                        chars[start..end].iter().collect()
                    } else {
                        String::new()
                    };
                    block_lines.push(slice);

                    if (op == Operator::Delete || op == Operator::Change) && start < end {
                        let delete_start = buffer.line_start_position(line) + start;
                        buffer.delete_text(delete_start, end - start, false);
                    }
                }

                self.registers
                    .set(register, RegisterContent::BlockWise(block_lines));

                cursor.pos.line = min_line;
                cursor.pos.col = min_col;
                cursor.clear_selection();

                if op == Operator::Change {
                    self.block_insert_session = Some(BlockInsertSession {
                        start_line: min_line,
                        end_line: max_line,
                        col: min_col,
                        typed_chars: Vec::new(),
                    });
                    self.mode = ViMode::Insert;
                } else {
                    buffer.close_undo_block();
                    self.mode = ViMode::Normal;
                }
            }
            ViMode::VisualLine => {
                buffer.open_undo_block();
                let mut lines_text = String::new();
                let start_pos = buffer.line_start_position(min_line);
                let end_pos = buffer.line_start_position(max_line + 1);

                for line in min_line..=max_line {
                    if let Some(l) = buffer.line_text(line) {
                        lines_text.push_str(&l);
                        lines_text.push('\n');
                    }
                }

                self.registers
                    .set(register, RegisterContent::LineWise(lines_text));

                if op == Operator::Delete || op == Operator::Change {
                    buffer.delete_text(start_pos, end_pos.saturating_sub(start_pos), false);
                }
                buffer.close_undo_block();

                cursor.pos.line = min_line.min(buffer.line_count().saturating_sub(1));
                cursor.pos.col = 0;
                cursor.clear_selection();

                if op == Operator::Change {
                    self.mode = ViMode::Insert;
                } else {
                    self.mode = ViMode::Normal;
                }
            }
            ViMode::Visual => {
                buffer.open_undo_block();
                let p1 = buffer.cursor_to_position(anchor);
                let p2 = buffer.cursor_to_position(pos);
                let from = p1.min(p2);
                let to = (p1.max(p2) + 1).min(buffer.len());
                let len = to.saturating_sub(from);

                let text = buffer.get_text(from, len);
                self.registers
                    .set(register, RegisterContent::CharacterWise(text));

                if op == Operator::Delete || op == Operator::Change {
                    buffer.delete_text(from, len, false);
                }
                buffer.close_undo_block();

                cursor.pos = buffer.position_to_cursor(from);
                cursor.clear_selection();

                if op == Operator::Change {
                    self.mode = ViMode::Insert;
                } else {
                    self.mode = ViMode::Normal;
                }
            }
            _ => {}
        }
    }

    /// ビジュアル選択範囲の大文字・小文字を反転 (`~`)
    pub(crate) fn execute_visual_swap_case(
        &mut self,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let anchor = cursor.anchor.unwrap_or(cursor.pos);
        let pos = cursor.pos;

        buffer.open_undo_block();
        if self.mode == ViMode::VisualBlock {
            let min_line = anchor.line.min(pos.line);
            let max_line = anchor.line.max(pos.line);
            let min_col = anchor.col.min(pos.col);
            let max_col = anchor.col.max(pos.col);

            for line in min_line..=max_line {
                if line >= buffer.line_count() {
                    continue;
                }
                let line_str = buffer.line_text(line).unwrap_or_default();
                let chars: Vec<char> = line_str.chars().collect();
                let start = min_col.min(chars.len());
                let end = (max_col + 1).min(chars.len());

                if start < end {
                    let mut swapped = String::new();
                    for &c in &chars[start..end] {
                        if c.is_lowercase() {
                            swapped.extend(c.to_uppercase());
                        } else if c.is_uppercase() {
                            swapped.extend(c.to_lowercase());
                        } else {
                            swapped.push(c);
                        }
                    }
                    let replace_start = buffer.line_start_position(line) + start;
                    buffer.delete_text(replace_start, end - start, false);
                    buffer.insert_text(replace_start, &swapped);
                }
            }
        } else {
            let p1 = buffer.cursor_to_position(anchor);
            let p2 = buffer.cursor_to_position(pos);
            let from = p1.min(p2);
            let to = (p1.max(p2) + 1).min(buffer.len());
            let len = to.saturating_sub(from);

            let text = buffer.get_text(from, len);
            let mut swapped = String::new();
            for c in text.chars() {
                if c.is_lowercase() {
                    swapped.extend(c.to_uppercase());
                } else if c.is_uppercase() {
                    swapped.extend(c.to_lowercase());
                } else {
                    swapped.push(c);
                }
            }
            buffer.delete_text(from, len, false);
            buffer.insert_text(from, &swapped);
        }
        buffer.close_undo_block();

        cursor.clear_selection();
        self.mode = ViMode::Normal;
    }

    /// ビジュアルブロックでの一括挿入 (`I` または `A`)
    pub(crate) fn execute_visual_block_insert(
        &mut self,
        at_end: bool,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let anchor = cursor.anchor.unwrap_or(cursor.pos);
        let pos = cursor.pos;

        let min_line = anchor.line.min(pos.line);
        let max_line = anchor.line.max(pos.line);
        let min_col = anchor.col.min(pos.col);
        let max_col = anchor.col.max(pos.col);

        let insert_col = if at_end { max_col + 1 } else { min_col };

        cursor.pos.line = min_line;
        cursor.pos.col = insert_col;
        cursor.clear_selection();

        buffer.open_undo_block();
        self.block_insert_session = Some(BlockInsertSession {
            start_line: min_line,
            end_line: max_line,
            col: insert_col,
            typed_chars: Vec::new(),
        });
        self.mode = ViMode::Insert;
    }

    /// 矩形一括挿入セッションが完了したときの他行への文字列反映処理
    pub(crate) fn finish_block_insert(&mut self, buffer: &mut TextBuffer) {
        if let Some(session) = self.block_insert_session.take() {
            let typed_str: String = session.typed_chars.into_iter().collect();
            if !typed_str.is_empty() {
                for line in (session.start_line + 1)..=session.end_line {
                    if line < buffer.line_count() {
                        let line_len = buffer
                            .line_text(line)
                            .map(|s| s.chars().count())
                            .unwrap_or(0);
                        if line_len < session.col {
                            let pad = " ".repeat(session.col - line_len);
                            let insert_pos = buffer.line_start_position(line) + line_len;
                            buffer.insert_text(insert_pos, &pad);
                        }
                        let insert_pos = buffer.line_start_position(line) + session.col;
                        buffer.insert_text(insert_pos, &typed_str);
                    }
                }
            }
            buffer.close_undo_block();
        }
    }

    /// ビジュアルモードでの選択領域一括文字置換 (`r<char>`)
    pub(crate) fn execute_visual_replace_char(
        &mut self,
        replacement: char,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let anchor = cursor.anchor.unwrap_or(cursor.pos);
        let pos = cursor.pos;

        let min_line = anchor.line.min(pos.line);
        let max_line = anchor.line.max(pos.line);
        let min_col = anchor.col.min(pos.col);
        let max_col = anchor.col.max(pos.col);

        buffer.open_undo_block();
        match self.mode {
            ViMode::VisualBlock => {
                for line in min_line..=max_line {
                    if line >= buffer.line_count() {
                        continue;
                    }
                    let line_str = buffer.line_text(line).unwrap_or_default();
                    let chars: Vec<char> = line_str.chars().collect();
                    let start = min_col.min(chars.len());
                    let end = (max_col + 1).min(chars.len());

                    if start < end {
                        let count = end - start;
                        let replaced: String = std::iter::repeat_n(replacement, count).collect();
                        let replace_start = buffer.line_start_position(line) + start;
                        buffer.delete_text(replace_start, count, false);
                        buffer.insert_text(replace_start, &replaced);
                    }
                }
            }
            ViMode::VisualLine => {
                for line in min_line..=max_line {
                    if line >= buffer.line_count() {
                        continue;
                    }
                    let line_str = buffer.line_text(line).unwrap_or_default();
                    let count = line_str.chars().count();
                    if count > 0 {
                        let replaced: String = std::iter::repeat_n(replacement, count).collect();
                        let start_pos = buffer.line_start_position(line);
                        buffer.delete_text(start_pos, count, false);
                        buffer.insert_text(start_pos, &replaced);
                    }
                }
            }
            ViMode::Visual => {
                let p1 = buffer.cursor_to_position(anchor);
                let p2 = buffer.cursor_to_position(pos);
                let from = p1.min(p2);
                let to = (p1.max(p2) + 1).min(buffer.len());
                let count = to.saturating_sub(from);
                if count > 0 {
                    let replaced: String = std::iter::repeat_n(replacement, count).collect();
                    buffer.delete_text(from, count, false);
                    buffer.insert_text(from, &replaced);
                }
            }
            _ => {}
        }
        buffer.close_undo_block();

        cursor.pos.line = min_line;
        cursor.pos.col = min_col;
        cursor.clear_selection();
        self.mode = ViMode::Normal;
    }
}
