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
    /// 空の新しい `TextBuffer` を生成します。
    pub fn new() -> Self {
        Self {
            data: GapBuffer::new(),
            undo_mgr: UndoMgr::new(),
        }
    }

    /// 初期文字列から `TextBuffer` を生成します。
    pub fn from_text(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut data = GapBuffer::with_capacity(chars.len() + 64);
        data.insert_slice(0, &chars);

        let mut undo_mgr = UndoMgr::new();
        undo_mgr.on_saved();

        Self { data, undo_mgr }
    }

    /// バッファ内に格納されている Unicode スカラー文字の総数を返します。
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// バッファが空かどうかを判定します。
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 前回の保存以降、バッファが変更されたかどうかを返します。
    pub fn is_modified(&self) -> bool {
        self.undo_mgr.is_modified()
    }

    /// 現在のバッファ状態を「保存済み（未変更）」として記録します。
    pub fn on_saved(&mut self) {
        self.undo_mgr.on_saved();
    }

    /// 文字オフセット `pos` にある文字を取得します。
    pub fn char_at(&self, pos: usize) -> Option<char> {
        self.data.get(pos).copied()
    }

    /// 文字オフセット `pos` から長さ `len` の部分文字列を取得します。
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

    /// Undo 履歴を記録せずに直接テキストを挿入します（Undo/Redo 適用処理用）。
    pub fn basic_insert_text(&mut self, pos: usize, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        self.data.insert_slice(pos, &chars);
    }

    /// Undo 履歴を記録せずに直接テキストを削除します（Undo/Redo 適用処理用）。
    pub fn basic_delete_text(&mut self, pos: usize, len: usize) -> String {
        let deleted = self.get_text(pos, len);
        self.data.remove_range(pos, pos + len);
        deleted
    }

    /// 文字オフセット `pos` にテキストを挿入し、Undo マネージャに記録します。
    pub fn insert_text(&mut self, pos: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        self.basic_insert_text(pos, text);
        self.undo_mgr.push_insert(pos, text);
    }

    /// 文字オフセット `pos` に単一文字を挿入し、Undo マネージャに記録します。
    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let mut s = String::with_capacity(4);
        s.push(ch);
        self.insert_text(pos, &s);
    }

    /// 文字オフセット `pos` から `len` 文字を削除し、Undo マネージャに記録します。
    pub fn delete_text(&mut self, pos: usize, len: usize, is_backspace: bool) -> String {
        if len == 0 || pos >= self.data.len() {
            return String::new();
        }
        let deleted = self.basic_delete_text(pos, len);
        self.undo_mgr.push_delete(pos, &deleted, is_backspace);
        deleted
    }

    /// 範囲 `[start, end)` の文字を削除し、Undo マネージャに記録します。
    pub fn delete_range(&mut self, start: usize, end: usize) -> String {
        if end <= start {
            return String::new();
        }
        self.delete_text(start, end - start, false)
    }

    /// Undo トランザクションブロックを開始します。
    pub fn open_undo_block(&mut self) {
        self.undo_mgr.open_block();
    }

    /// Undo トランザクションブロックを終了します。
    pub fn close_undo_block(&mut self) {
        self.undo_mgr.close_block();
    }

    /// アクティブなすべての Undo トランザクションブロックを一括終了します。
    pub fn close_all_undo_blocks(&mut self) {
        self.undo_mgr.close_all_blocks();
    }

    /// 直前の文字挿入アクションと以降のキータイピングのマージを禁止します。
    pub fn prohibit_merge_undo(&mut self) {
        self.undo_mgr.prohibit_merge();
    }

    /// Undo ステップを実行します。実行された場合は true を返します。
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

    /// Redo ステップを実行します。実行された場合は true を返します。
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

    /// 文書内の総行数を返します（最低でも 1 行）。
    pub fn line_count(&self) -> usize {
        let mut count = 1;
        for i in 0..self.data.len() {
            if let Some(&'\n') = self.data.get(i) {
                count += 1;
            }
        }
        count
    }

    /// 0始まりの行番号を開始する文字オフセット位置へ変換します。
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

    /// 文字オフセット位置を 0 始まりの行番号へ変換します。
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

    /// 文字オフセット位置を行・列座標（CursorPosition）へ変換します。
    pub fn position_to_cursor(&self, pos: usize) -> CursorPosition {
        let line = self.position_to_line(pos);
        let line_start = self.line_start_position(line);
        let col = pos.saturating_sub(line_start);
        CursorPosition::new(line, col)
    }

    /// 行・列座標（CursorPosition）を文字オフセット位置へ変換します。
    pub fn cursor_to_position(&self, cursor: CursorPosition) -> usize {
        let line_start = self.line_start_position(cursor.line);
        let next_line_start = self.line_start_position(cursor.line + 1);
        let line_len = if cursor.line + 1 < self.line_count() {
            next_line_start.saturating_sub(line_start + 1) // 改行文字を除く
        } else {
            self.data.len().saturating_sub(line_start)
        };
        line_start + cursor.col.min(line_len)
    }

    /// 指定行の文字列（末尾の改行コードを除く）を返します。
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

    /// `line_text(line)` の別名エイリアスです。
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
        buf.delete_text(0, 2, false); // "12" を削除 -> "345"
        buf.insert_text(0, "ABC"); // "ABC" を挿入 -> "ABC345"
        buf.close_undo_block();

        assert_eq!(buf.to_string(), "ABC345");

        // 1回のUndoでブロック内の両操作が一括アンドゥされる
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
        assert_eq!(buf.line_start_position(1), 6); // "alpha\n" は 6 文字
        assert_eq!(buf.line_start_position(2), 11); // "alpha\nbeta\n" は 11 文字

        assert_eq!(buf.position_to_line(0), 0);
        assert_eq!(buf.position_to_line(5), 0);
        assert_eq!(buf.position_to_line(6), 1);
        assert_eq!(buf.position_to_line(12), 2);
    }

    #[test]
    fn test_prohibit_merge_and_newlines() {
        let mut buf = TextBuffer::new();
        // 'a', 'b', 'c' を入力
        buf.insert_text(0, "a");
        buf.insert_text(1, "b");
        buf.insert_text(2, "c");

        // カーソル移動やモード変更によりマージ禁止を設定
        buf.prohibit_merge_undo();

        // 'd', 'e' を入力
        buf.insert_text(3, "d");
        buf.insert_text(4, "e");

        assert_eq!(buf.to_string(), "abcde");

        // 最初のUndoは "de" のみをアンドゥ
        assert!(buf.undo());
        assert_eq!(buf.to_string(), "abc");

        // 2回目のUndoで "abc" をアンドゥ
        assert!(buf.undo());
        assert_eq!(buf.to_string(), "");

        // Redoで "abc"、続いて "abcde" が復元される
        assert!(buf.redo());
        assert_eq!(buf.to_string(), "abc");
        assert!(buf.redo());
        assert_eq!(buf.to_string(), "abcde");
    }

    #[test]
    fn test_cursor_and_position_conversion() {
        let buf = TextBuffer::from_text("hello\nworld");
        let cur = buf.position_to_cursor(7); // "world" の 'o' (行 1, 列 1)
        assert_eq!(cur.line, 1);
        assert_eq!(cur.col, 1);

        let pos = buf.cursor_to_position(CursorPosition::new(1, 3));
        assert_eq!(pos, 9); // "world" の 'l'
        assert_eq!(buf.char_at(pos), Some('l'));
    }
}
