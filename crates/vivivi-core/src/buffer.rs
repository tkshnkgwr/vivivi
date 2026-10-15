//! GapBuffer と UndoMgr を統合したテキストバッファモジュール。
//!
//! OpenViVi の `Buffer.cpp` に対応し、高水準の行・列座標変換（CursorPosition）、
//! トランザクション対応編集、文字コード処理、アンドゥ・リドゥ機能を提供します。

use crate::cursor::CursorPosition;
use crate::gap_buffer::GapBuffer;
use crate::undo::{UndoAction, UndoMgr};

/// 高速な編集処理とトランザクション Undo/Redo を提供するテキストバッファ。
///
/// 内部に [`GapBuffer<char>`] と [`UndoMgr`] を保持し、Vi コマンドの複合操作や
/// 保存状態（Modified フラグ）を整合的に管理します。
///
/// # Examples
///
/// ```
/// use vivivi_core::TextBuffer;
///
/// let mut buf = TextBuffer::from_text("Hello");
/// buf.insert_text(5, ", World");
/// assert_eq!(buf.to_string(), "Hello, World");
///
/// buf.undo();
/// assert_eq!(buf.to_string(), "Hello");
/// ```
#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    data: GapBuffer<char>,
    undo_mgr: UndoMgr,
}

impl TextBuffer {
    /// 空の新しい [`TextBuffer`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::new();
    /// assert!(buf.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            data: GapBuffer::new(),
            undo_mgr: UndoMgr::new(),
        }
    }

    /// 初期テキストを指定して新しい [`TextBuffer`] を生成します。
    ///
    /// 生成直後のバッファ状態は「保存済み（未変更）」として初期化されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("初期テキスト");
    /// assert_eq!(buf.len(), 6);
    /// assert!(!buf.is_modified());
    /// ```
    pub fn from_text(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut data = GapBuffer::with_capacity(chars.len() + 64);
        data.insert_slice(0, &chars);

        let mut undo_mgr = UndoMgr::new();
        undo_mgr.on_saved();

        Self { data, undo_mgr }
    }

    /// バッファ内に格納されている文字数（Unicode スカラー値数）を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("Rust🦀");
    /// assert_eq!(buf.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// バッファが空かどうかを判定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::new();
    /// assert!(buf.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 最後の保存時点からバッファが変更されたかどうかを返します。
    ///
    /// アンドゥ操作によって保存時点の状態に戻った場合、自動的に `false` と判定されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("test");
    /// assert!(!buf.is_modified());
    ///
    /// buf.insert_char(4, '!');
    /// assert!(buf.is_modified());
    ///
    /// buf.undo();
    /// assert!(!buf.is_modified());
    /// ```
    pub fn is_modified(&self) -> bool {
        self.undo_mgr.is_modified()
    }

    /// 現在のバッファ状態を「保存済み（未変更）」として記録します。
    ///
    /// 以降の [`TextBuffer::is_modified`] の判定基準となる保存点（Save Point）が更新されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::new();
    /// buf.insert_text(0, "edited");
    /// assert!(buf.is_modified());
    ///
    /// buf.on_saved();
    /// assert!(!buf.is_modified());
    /// ```
    pub fn on_saved(&mut self) {
        self.undo_mgr.on_saved();
    }

    /// 内部の [`UndoMgr`] への不変参照を返します。
    pub fn undo_mgr(&self) -> &UndoMgr {
        &self.undo_mgr
    }

    /// 内部の [`UndoMgr`] への可変参照を返します。
    pub fn undo_mgr_mut(&mut self) -> &mut UndoMgr {
        &mut self.undo_mgr
    }

    /// 指定されたファイルパスに対する Undo 永続化ファイル（undofile）のパスを生成します。
    ///
    /// Vim 互換の隠しファイル名形式（例: `.filename.un~`）を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use vivivi_core::TextBuffer;
    ///
    /// let p = TextBuffer::undofile_path(Path::new("src/main.rs"));
    /// assert_eq!(p, Path::new("src/.main.rs.un~"));
    /// ```
    pub fn undofile_path<P: AsRef<std::path::Path>>(file_path: P) -> std::path::PathBuf {
        let p = file_path.as_ref();
        let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
        let undofile_name = format!(".{}.un~", file_name);
        match p.parent() {
            Some(parent) => parent.join(undofile_name),
            None => std::path::PathBuf::from(undofile_name),
        }
    }

    /// undofile に Undo/Redo 履歴を保存します。
    ///
    /// # Errors
    ///
    /// ファイルの作成や履歴データの書き込みに失敗した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("sample");
    /// buf.save_undofile("sample.txt").unwrap();
    /// ```
    pub fn save_undofile<P: AsRef<std::path::Path>>(&self, file_path: P) -> std::io::Result<()> {
        let undopath = Self::undofile_path(file_path);
        self.undo_mgr.save_to_file(undopath)
    }

    /// undofile から Undo/Redo 履歴を読み込み復元します。
    ///
    /// 対象の undofile が存在しない場合は何もしません。
    ///
    /// # Errors
    ///
    /// undofile が存在し、その読み込みやデシリアライズに失敗した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::new();
    /// buf.load_undofile("sample.txt").unwrap();
    /// ```
    pub fn load_undofile<P: AsRef<std::path::Path>>(
        &mut self,
        file_path: P,
    ) -> std::io::Result<()> {
        let undopath = Self::undofile_path(file_path);
        if undopath.exists() {
            let loaded = UndoMgr::load_from_file(undopath)?;
            self.undo_mgr = loaded;
        }
        Ok(())
    }

    /// 文字オフセット `pos` にある Unicode スカラー文字を取得します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。`pos` が範囲外の場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("Rust");
    /// assert_eq!(buf.char_at(0), Some('R'));
    /// assert_eq!(buf.char_at(10), None);
    /// ```
    pub fn char_at(&self, pos: usize) -> Option<char> {
        self.data.get(pos).copied()
    }

    /// 文字オフセット `pos` から長さ `len` の部分文字列を取得します。
    ///
    /// バッファ終端を超える範囲は自動的に切り詰められます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("Hello World");
    /// assert_eq!(buf.get_text(0, 5), "Hello");
    /// assert_eq!(buf.get_text(6, 100), "World");
    /// ```
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

    /// Undo 履歴を記録せずに直接テキストを挿入します。
    ///
    /// 主に Undo/Redo 適用処理や一括ロードなどの内部処理で使用します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("ac");
    /// buf.basic_insert_text(1, "b");
    /// assert_eq!(buf.to_string(), "abc");
    /// ```
    pub fn basic_insert_text(&mut self, pos: usize, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        self.data.insert_slice(pos, &chars);
    }

    /// Undo 履歴を記録せずに直接テキストを削除し、削除された文字列を返します。
    ///
    /// 主に Undo/Redo 適用処理で使用します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("abc");
    /// let deleted = buf.basic_delete_text(1, 1);
    /// assert_eq!(deleted, "b");
    /// assert_eq!(buf.to_string(), "ac");
    /// ```
    pub fn basic_delete_text(&mut self, pos: usize, len: usize) -> String {
        let deleted = self.get_text(pos, len);
        self.data.remove_range(pos, pos + len);
        deleted
    }

    /// 文字オフセット `pos` にテキストを挿入し、Undo マネージャに記録します。
    ///
    /// `text` が空の場合は何も行いません。直前が文字入力であればタイピングマージが試みられます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::new();
    /// buf.insert_text(0, "ViVi");
    /// assert_eq!(buf.to_string(), "ViVi");
    /// ```
    pub fn insert_text(&mut self, pos: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        self.basic_insert_text(pos, text);
        self.undo_mgr.push_insert(pos, text);
    }

    /// 文字オフセット `pos` に単一文字を挿入し、Undo マネージャに記録します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("Hell");
    /// buf.insert_char(4, 'o');
    /// assert_eq!(buf.to_string(), "Hello");
    /// ```
    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let mut s = String::with_capacity(4);
        s.push(ch);
        self.insert_text(pos, &s);
    }

    /// 文字オフセット `pos` から `len` 文字を削除し、Undo マネージャに記録します。
    ///
    /// `is_backspace` が `true` の場合、直前のバックスペース削除アクションとのマージが試行されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("Hello!");
    /// let deleted = buf.delete_text(5, 1, false);
    /// assert_eq!(deleted, "!");
    /// assert_eq!(buf.to_string(), "Hello");
    /// ```
    pub fn delete_text(&mut self, pos: usize, len: usize, is_backspace: bool) -> String {
        if len == 0 || pos >= self.data.len() {
            return String::new();
        }
        let deleted = self.basic_delete_text(pos, len);
        self.undo_mgr.push_delete(pos, &deleted, is_backspace);
        deleted
    }

    /// 範囲 `[start, end)` の文字を削除し、Undo マネージャに記録します。
    ///
    /// `end <= start` の場合は空文字列を返して何も行いません。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("Hello World");
    /// buf.delete_range(5, 11);
    /// assert_eq!(buf.to_string(), "Hello");
    /// ```
    pub fn delete_range(&mut self, start: usize, end: usize) -> String {
        if end <= start {
            return String::new();
        }
        self.delete_text(start, end - start, false)
    }

    /// 複合 Vi コマンド用のアトミックな Undo トランザクションブロックを開始します。
    ///
    /// ネスト呼び出しに対応しており、最外ブロックが終了するまで変更が1つのブロックにまとめられます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("123");
    /// buf.open_undo_block();
    /// buf.insert_text(0, "A");
    /// buf.insert_text(4, "B");
    /// buf.close_undo_block();
    ///
    /// buf.undo();
    /// assert_eq!(buf.to_string(), "123");
    /// ```
    pub fn open_undo_block(&mut self) {
        self.undo_mgr.open_block();
    }

    /// Undo トランザクションブロックを終了します。
    ///
    /// ネスト深度が 0 に達した時点で、蓄積されたアクション群が単一の Undo ステップとしてスタックへプッシュされます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::new();
    /// buf.open_undo_block();
    /// buf.insert_text(0, "test");
    /// buf.close_undo_block();
    /// assert_eq!(buf.to_string(), "test");
    /// ```
    pub fn close_undo_block(&mut self) {
        self.undo_mgr.close_block();
    }

    /// アクティブなすべての Undo トランザクションブロックを一括終了・確定します。
    pub fn close_all_undo_blocks(&mut self) {
        self.undo_mgr.close_all_blocks();
    }

    /// 直前の文字挿入アクションと以降のキータイピングのマージを明示的に禁止します。
    ///
    /// カーソル移動やモード切り替え時に呼び出すことで、別個の Undo 単位として区切ります。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::new();
    /// buf.insert_text(0, "a");
    /// buf.prohibit_merge_undo();
    /// buf.insert_text(1, "b");
    ///
    /// buf.undo();
    /// assert_eq!(buf.to_string(), "a");
    /// ```
    pub fn prohibit_merge_undo(&mut self) {
        self.undo_mgr.prohibit_merge();
    }

    /// 直前の編集操作（または Undo ブロック）を 1 ステップアンドゥします。
    ///
    /// 実行された場合は `true`、取り消す履歴がない場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("Initial");
    /// buf.insert_text(7, " Edit");
    /// assert!(buf.undo());
    /// assert_eq!(buf.to_string(), "Initial");
    /// assert!(!buf.undo()); // これ以上アンドゥできない
    /// ```
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

    /// アンドゥされた操作を 1 ステップやり直します（リドゥ）。
    ///
    /// 実行された場合は `true`、やり直す履歴がない場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let mut buf = TextBuffer::from_text("Initial");
    /// buf.insert_text(7, " Edit");
    /// buf.undo();
    /// assert!(buf.redo());
    /// assert_eq!(buf.to_string(), "Initial Edit");
    /// assert!(!buf.redo()); // これ以上リドゥできない
    /// ```
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

    /// ドキュメント内の総行数を返します（空バッファでも最低 1 行）。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("Line 1\nLine 2\nLine 3");
    /// assert_eq!(buf.line_count(), 3);
    /// ```
    pub fn line_count(&self) -> usize {
        let mut count = 1;
        for i in 0..self.data.len() {
            if let Some(&'\n') = self.data.get(i) {
                count += 1;
            }
        }
        count
    }

    /// 0始まりの行番号に対応する先頭文字オフセット位置を算出します。
    ///
    /// `target_line` が総行数を超える場合はバッファ終端位置を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("abc\ndef");
    /// assert_eq!(buf.line_start_position(0), 0);
    /// assert_eq!(buf.line_start_position(1), 4);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("abc\ndef");
    /// assert_eq!(buf.position_to_line(0), 0);
    /// assert_eq!(buf.position_to_line(4), 1);
    /// ```
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

    /// 文字オフセット位置を行・列座標（[`CursorPosition`]）へ変換します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    ///
    /// let buf = TextBuffer::from_text("abc\ndef");
    /// assert_eq!(buf.position_to_cursor(5), CursorPosition::new(1, 1));
    /// ```
    pub fn position_to_cursor(&self, pos: usize) -> CursorPosition {
        let line = self.position_to_line(pos);
        let line_start = self.line_start_position(line);
        let col = pos.saturating_sub(line_start);
        CursorPosition::new(line, col)
    }

    /// 行・列座標（[`CursorPosition`]）を文字オフセット位置へ変換します。
    ///
    /// 列番号（`col`）が行の実際の長さを超える場合は行末にクランプされます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    ///
    /// let buf = TextBuffer::from_text("abc\ndef");
    /// assert_eq!(buf.cursor_to_position(CursorPosition::new(1, 1)), 5);
    /// ```
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

    /// 指定行の文字列（末尾の改行コードを除く）を取得します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。行番号が範囲外の場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    ///
    /// let buf = TextBuffer::from_text("Hello\nWorld");
    /// assert_eq!(buf.line_text(0), Some("Hello".to_string()));
    /// assert_eq!(buf.line_text(1), Some("World".to_string()));
    /// assert_eq!(buf.line_text(2), None);
    /// ```
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

    /// [`TextBuffer::line_text`] の別名エイリアスです。
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
    fn test_internal_buffer_data_and_undo_sync() {
        let mut buf = TextBuffer::new();
        // 内部フィールド data (GapBuffer) と undo_mgr の連動を直接検証
        buf.basic_insert_text(0, "abc");
        assert_eq!(buf.data.len(), 3);
        // basic_insert_text は undo_mgr に記録されない内部低レベル操作
        assert!(!buf.undo_mgr.can_undo());
    }
}
