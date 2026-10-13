//! GapBuffer と UndoMgr を統合したテキストバッファモジュール。
//!
//! OpenViVi の `Buffer.cpp` に対応し、高水準の行・列座標変換（CursorPosition）、
//! トランザクション対応編集、文字コード処理、アンドゥ・リドゥ機能を提供します。

use crate::cursor::CursorPosition;
use crate::gap_buffer::GapBuffer;
use crate::undo::{UndoAction, UndoMgr};

/// テキストバッファ本体。
///
/// 内部に `GapBuffer<char>` と `UndoMgr` をカプセル化し、
/// 安全かつ高速なテキスト編集と履歴管理を行います。
#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    data: GapBuffer<char>,
    undo_mgr: UndoMgr,
}

impl TextBuffer {
    /// Creates a new empty `TextBuffer`.
    pub fn new() -> Self {
        Self {
            data: GapBuffer::new(),
            undo_mgr: UndoMgr::new(),
        }
    }

    /// Creates a `TextBuffer` from initial text string.
    pub fn from_text(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut data = GapBuffer::with_capacity(chars.len() + 64);
        data.insert_slice(0, &chars);

        let mut undo_mgr = UndoMgr::new();
        undo_mgr.on_saved();

        Self { data, undo_mgr }
    }

    /// Number of Unicode scalar characters stored in the buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the buffer empty?
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Has the buffer been modified since last save?
    pub fn is_modified(&self) -> bool {
        self.undo_mgr.is_modified()
    }

    /// Marks the buffer state as saved.
    pub fn on_saved(&mut self) {
        self.undo_mgr.on_saved();
    }

    /// Returns the character at character offset `pos`.
    pub fn char_at(&self, pos: usize) -> Option<char> {
        self.data.get(pos).copied()
    }

    /// Returns a substring from character offset `pos` of length `len`.
    pub fn get_text(&self, pos: usize, len: usize) -> String {
        let end = (pos + len).min(self.data.len());
        let mut s = String::with_capacity(end.saturating_sub(pos));
        for i in pos..end {
            if let Some(&ch) = self.data.get(i) {
                s.push(ch);
            }
        }
        s
    }

    /// Basic insert without recording undo actions (used by undo/redo application).
    pub fn basic_insert_text(&mut self, pos: usize, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        self.data.insert_slice(pos, &chars);
    }

    /// Basic delete without recording undo actions (used by undo/redo application).
    pub fn basic_delete_text(&mut self, pos: usize, len: usize) -> String {
        let deleted = self.get_text(pos, len);
        self.data.remove_range(pos, pos + len);
        deleted
    }

    /// Inserts text at character offset `pos`, recording to the Undo manager.
    pub fn insert_text(&mut self, pos: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        self.basic_insert_text(pos, text);
        self.undo_mgr.push_insert(pos, text);
    }

    /// Inserts a single character at character offset `pos`, recording to the Undo manager.
    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let mut s = String::with_capacity(4);
        s.push(ch);
        self.insert_text(pos, &s);
    }

    /// Deletes `len` characters starting at offset `pos`, recording to the Undo manager.
    pub fn delete_text(&mut self, pos: usize, len: usize, is_backspace: bool) -> String {
        if len == 0 || pos >= self.data.len() {
            return String::new();
        }
        let deleted = self.basic_delete_text(pos, len);
        self.undo_mgr.push_delete(pos, &deleted, is_backspace);
        deleted
    }

    /// Deletes characters in range `[start, end)`, recording to the Undo manager.
    pub fn delete_range(&mut self, start: usize, end: usize) -> String {
        if end <= start {
            return String::new();
        }
        self.delete_text(start, end - start, false)
    }

    /// Opens an undo transaction block.
    pub fn open_undo_block(&mut self) {
        self.undo_mgr.open_block();
    }

    /// Closes an undo transaction block.
    pub fn close_undo_block(&mut self) {
        self.undo_mgr.close_block();
    }

    /// Closes all active undo transaction blocks.
    pub fn close_all_undo_blocks(&mut self) {
        self.undo_mgr.close_all_blocks();
    }

    /// Prohibits merging the last insert action with subsequent typings.
    pub fn prohibit_merge_undo(&mut self) {
        self.undo_mgr.prohibit_merge();
    }

    /// Executes an Undo step. Returns true if undo was performed.
    pub fn undo(&mut self) -> bool {
        if let Some(actions) = self.undo_mgr.pop_undo() {
            for action in actions {
                match action {
                    UndoAction::Insert { pos, text, .. } => {
                        self.basic_insert_text(pos, &text);
                    }
                    UndoAction::Delete { pos, text, .. } => {
                        self.basic_delete_text(pos, text.chars().count());
                    }
                    UndoAction::Replace {
                        pos,
                        old_text,
                        new_text,
                    } => {
                        self.basic_delete_text(pos, old_text.chars().count());
                        self.basic_insert_text(pos, &new_text);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Executes a Redo step. Returns true if redo was performed.
    pub fn redo(&mut self) -> bool {
        if let Some(actions) = self.undo_mgr.pop_redo() {
            for action in actions {
                match action {
                    UndoAction::Insert { pos, text, .. } => {
                        self.basic_insert_text(pos, &text);
                    }
                    UndoAction::Delete { pos, text, .. } => {
                        self.basic_delete_text(pos, text.chars().count());
                    }
                    UndoAction::Replace {
                        pos,
                        old_text,
                        new_text,
                    } => {
                        self.basic_delete_text(pos, old_text.chars().count());
                        self.basic_insert_text(pos, &new_text);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Total number of lines in the document (1-indexed minimum of 1).
    pub fn line_count(&self) -> usize {
        let mut count = 1;
        for i in 0..self.data.len() {
            if let Some(&'\n') = self.data.get(i) {
                count += 1;
            }
        }
        count
    }

    /// Converts a 0-indexed line number into character offset where the line starts.
    pub fn line_start_position(&self, target_line: usize) -> usize {
        if target_line == 0 {
            return 0;
        }
        let mut current_line = 0;
        for i in 0..self.data.len() {
            if let Some(&'\n') = self.data.get(i) {
                current_line += 1;
                if current_line == target_line {
                    return i + 1;
                }
            }
        }
        self.data.len()
    }

    /// Converts a character offset into a 0-indexed line number.
    pub fn position_to_line(&self, pos: usize) -> usize {
        let limit = pos.min(self.data.len());
        let mut line = 0;
        for i in 0..limit {
            if let Some(&'\n') = self.data.get(i) {
                line += 1;
            }
        }
        line
    }

    /// Converts a character offset into a `CursorPosition` (line and column).
    pub fn position_to_cursor(&self, pos: usize) -> CursorPosition {
        let line = self.position_to_line(pos);
        let line_start = self.line_start_position(line);
        let col = pos.saturating_sub(line_start);
        CursorPosition::new(line, col)
    }

    /// Converts a `CursorPosition` (line and column) into a character offset.
    pub fn cursor_to_position(&self, cursor: CursorPosition) -> usize {
        let line_start = self.line_start_position(cursor.line);
        let next_line_start = self.line_start_position(cursor.line + 1);
        let line_len = if cursor.line + 1 < self.line_count() {
            next_line_start.saturating_sub(line_start + 1) // exclude newline
        } else {
            self.data.len().saturating_sub(line_start)
        };
        line_start + cursor.col.min(line_len)
    }

    /// Returns the content of specified line without the trailing newline.
    pub fn line_text(&self, line: usize) -> Option<String> {
        if line >= self.line_count() {
            return None;
        }
        let start = self.line_start_position(line);
        let mut text = String::new();
        let mut idx = start;
        while idx < self.data.len() {
            match self.data.get(idx) {
                Some(&'\n') => break,
                Some(&ch) => text.push(ch),
                None => break,
            }
            idx += 1;
        }
        Some(text)
    }

    /// Alias for `line_text(line)`.
    pub fn get_line(&self, line: usize) -> Option<String> {
        self.line_text(line)
    }
}

impl std::fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.get_text(0, self.data.len());
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_insert_delete_undo() {
        let mut buf = TextBuffer::from_text("Hello");
        assert_eq!(buf.to_string(), "Hello");
        assert!(!buf.is_modified());

        buf.insert_text(5, " World");
        assert_eq!(buf.to_string(), "Hello World");
        assert!(buf.is_modified());

        assert!(buf.undo());
        assert_eq!(buf.to_string(), "Hello");
        assert!(!buf.is_modified());

        assert!(buf.redo());
        assert_eq!(buf.to_string(), "Hello World");
        assert!(buf.is_modified());
    }

    #[test]
    fn test_undo_block() {
        let mut buf = TextBuffer::from_text("12345");
        buf.open_undo_block();
        buf.delete_text(0, 2, false); // deletes "12" -> "345"
        buf.insert_text(0, "ABC"); // inserts "ABC" -> "ABC345"
        buf.close_undo_block();

        assert_eq!(buf.to_string(), "ABC345");

        // One undo reverts both operations in the block!
        assert!(buf.undo());
        assert_eq!(buf.to_string(), "12345");

        assert!(buf.redo());
        assert_eq!(buf.to_string(), "ABC345");
    }

    #[test]
    fn test_lines_and_positions() {
        let buf = TextBuffer::from_text("alpha\nbeta\ngamma");
        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.line_text(0), Some("alpha".to_string()));
        assert_eq!(buf.line_text(1), Some("beta".to_string()));
        assert_eq!(buf.line_text(2), Some("gamma".to_string()));

        assert_eq!(buf.line_start_position(0), 0);
        assert_eq!(buf.line_start_position(1), 6); // "alpha\n" is 6 chars
        assert_eq!(buf.line_start_position(2), 11); // "alpha\nbeta\n" is 11 chars

        assert_eq!(buf.position_to_line(0), 0);
        assert_eq!(buf.position_to_line(5), 0);
        assert_eq!(buf.position_to_line(6), 1);
        assert_eq!(buf.position_to_line(12), 2);
    }

    #[test]
    fn test_prohibit_merge_and_newlines() {
        let mut buf = TextBuffer::new();
        // Type 'a', 'b', 'c'
        buf.insert_text(0, "a");
        buf.insert_text(1, "b");
        buf.insert_text(2, "c");

        // Cursor moved or mode changed: prohibit merge
        buf.prohibit_merge_undo();

        // Type 'd', 'e'
        buf.insert_text(3, "d");
        buf.insert_text(4, "e");

        assert_eq!(buf.to_string(), "abcde");

        // First undo should only revert "de"
        assert!(buf.undo());
        assert_eq!(buf.to_string(), "abc");

        // Second undo reverts "abc"
        assert!(buf.undo());
        assert_eq!(buf.to_string(), "");

        // Redo brings back "abc", then "abcde"
        assert!(buf.redo());
        assert_eq!(buf.to_string(), "abc");
        assert!(buf.redo());
        assert_eq!(buf.to_string(), "abcde");
    }

    #[test]
    fn test_cursor_and_position_conversion() {
        let buf = TextBuffer::from_text("hello\nworld");
        let cur = buf.position_to_cursor(7); // 'o' in "world" (line 1, col 1)
        assert_eq!(cur.line, 1);
        assert_eq!(cur.col, 1);

        let pos = buf.cursor_to_position(CursorPosition::new(1, 3));
        assert_eq!(pos, 9); // 'l' in "world"
        assert_eq!(buf.char_at(pos), Some('l'));
    }
}
