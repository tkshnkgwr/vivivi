//! バイナリモード (Hex Editor) コアエンジン
//!
//! 16進ダンプ＋ASCII並行表示によるバイト単位の直接閲覧・編集機能を提供します。
//! ViVi 3 / ViVi 4 のバイナリエディタ仕様に基づき、アドレスオフセット、16進ダンプ、
//! 印字可能ASCII文字列の3列同期管理、上位・下位ニブル編集、Undo/Redo、
//! バイト列検索をサポートします。

use std::path::{Path, PathBuf};

/// 編集中のペイン（16進ダンプ列 または ASCII列）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexPane {
    /// 16進ダンプ列（`48 65 6c ...`）での編集
    Hex,
    /// ASCII文字列プレビュー列での編集
    Ascii,
}

/// 16進入力時のニブル（4ビット単位）位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nibble {
    /// 上位4ビット（左側文字）
    High,
    /// 下位4ビット（右側文字）
    Low,
}

/// バイナリエディタのカーソル位置
///
/// # Examples
/// ```
/// use vivivi_modes::hex::{HexCursor, HexPane, Nibble};
///
/// let mut cur = HexCursor::new();
/// assert_eq!(cur.pane, HexPane::Hex);
/// cur.toggle_pane();
/// assert_eq!(cur.pane, HexPane::Ascii);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexCursor {
    /// 全体データにおけるバイト単位のオフセット
    pub offset: usize,
    /// 現在フォーカスされているペイン
    pub pane: HexPane,
    /// 16進ペイン時のニブル位置
    pub nibble: Nibble,
}

impl HexCursor {
    /// 新しいカーソルを先頭（オフセット0、Hexペイン、上位ニブル）で作成します。
    pub fn new() -> Self {
        Self {
            offset: 0,
            pane: HexPane::Hex,
            nibble: Nibble::High,
        }
    }

    /// ペイン（Hex ⇄ Ascii）を切り替えます。
    pub fn toggle_pane(&mut self) {
        self.pane = match self.pane {
            HexPane::Hex => HexPane::Ascii,
            HexPane::Ascii => HexPane::Hex,
        };
        self.nibble = Nibble::High;
    }
}

impl Default for HexCursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Undo/Redo用の変更アクション
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexAction {
    /// バイト置換（オフセット、旧バイト列、新バイト列）
    Replace {
        offset: usize,
        old_bytes: Vec<u8>,
        new_bytes: Vec<u8>,
    },
    /// バイト挿入（オフセット、挿入バイト列）
    Insert { offset: usize, bytes: Vec<u8> },
    /// バイト削除（オフセット、削除されたバイト列）
    Delete {
        offset: usize,
        deleted_bytes: Vec<u8>,
    },
}

/// バイナリドキュメント（生バイト列とUndo履歴の管理）
///
/// # Examples
/// ```
/// use vivivi_modes::hex::{HexDocument, Nibble};
///
/// let mut doc = HexDocument::from_bytes(b"Hello".to_vec());
/// assert_eq!(doc.len(), 5);
/// doc.set_nibble(0, Nibble::High, 0x4);
/// doc.set_nibble(0, Nibble::Low, 0x1);
/// assert_eq!(doc.get_byte(0), Some(b'A'));
/// assert!(doc.undo());
/// assert_eq!(doc.get_byte(0), Some(b'H'));
/// ```
#[derive(Debug, Clone)]
pub struct HexDocument {
    /// 生バイナリデータ
    data: Vec<u8>,
    /// 変更フラグ
    dirty: bool,
    /// ファイルパス
    file_path: Option<PathBuf>,
    /// Undoスタック
    undo_stack: Vec<HexAction>,
    /// Redoスタック
    redo_stack: Vec<HexAction>,
}

impl HexDocument {
    /// 空のバイナリドキュメントを作成します。
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            dirty: false,
            file_path: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// バイト列からバイナリドキュメントを作成します。
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            data: bytes,
            dirty: false,
            file_path: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 文字列（UTF-8）からバイナリドキュメントを作成します。
    pub fn from_text(text: &str) -> Self {
        Self::from_bytes(text.as_bytes().to_vec())
    }

    /// 生バイト列への参照を取得します。
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 全体のデータサイズ（バイト数）を取得します。
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// データが空かどうかを判定します。
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 変更されているかどうかを取得します。
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// 変更フラグを設定します。
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// ファイルパスを取得します。
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// ファイルパスを設定します。
    pub fn set_file_path(&mut self, path: Option<PathBuf>) {
        self.file_path = path;
    }

    /// 指定オフセットのバイトを取得します。
    pub fn get_byte(&self, offset: usize) -> Option<u8> {
        self.data.get(offset).copied()
    }

    /// 指定オフセットの1バイトを置換します。
    pub fn set_byte(&mut self, offset: usize, new_byte: u8) -> bool {
        if offset >= self.data.len() {
            return false;
        }
        let old_byte = self.data[offset];
        if old_byte == new_byte {
            return true;
        }

        self.data[offset] = new_byte;
        self.undo_stack.push(HexAction::Replace {
            offset,
            old_bytes: vec![old_byte],
            new_bytes: vec![new_byte],
        });
        self.redo_stack.clear();
        self.dirty = true;
        true
    }

    /// 16進ニブル（4bit）を更新します。
    ///
    /// # 引数
    /// * `offset` - バイトオフセット
    /// * `nibble` - 上位または下位ニブル
    /// * `hex_val` - 0〜15 (0x0〜0xF) の値
    pub fn set_nibble(&mut self, offset: usize, nibble: Nibble, hex_val: u8) -> bool {
        if offset >= self.data.len() || hex_val > 0x0F {
            return false;
        }
        let current = self.data[offset];
        let new_byte = match nibble {
            Nibble::High => (hex_val << 4) | (current & 0x0F),
            Nibble::Low => (current & 0xF0) | (hex_val & 0x0F),
        };
        self.set_byte(offset, new_byte)
    }

    /// 指定オフセットにバイト列を挿入します。
    pub fn insert_bytes(&mut self, offset: usize, bytes: &[u8]) {
        let insert_pos = offset.min(self.data.len());
        self.data
            .splice(insert_pos..insert_pos, bytes.iter().copied());
        self.undo_stack.push(HexAction::Insert {
            offset: insert_pos,
            bytes: bytes.to_vec(),
        });
        self.redo_stack.clear();
        self.dirty = true;
    }

    /// 指定オフセットから指定バイト数を削除します。
    pub fn delete_bytes(&mut self, offset: usize, count: usize) -> Vec<u8> {
        if offset >= self.data.len() || count == 0 {
            return Vec::new();
        }
        let end = (offset + count).min(self.data.len());
        let deleted: Vec<u8> = self.data.drain(offset..end).collect();
        if !deleted.is_empty() {
            self.undo_stack.push(HexAction::Delete {
                offset,
                deleted_bytes: deleted.clone(),
            });
            self.redo_stack.clear();
            self.dirty = true;
        }
        deleted
    }

    /// Undo操作を実行します。
    pub fn undo(&mut self) -> bool {
        let action = match self.undo_stack.pop() {
            Some(a) => a,
            None => return false,
        };

        match &action {
            HexAction::Replace {
                offset, old_bytes, ..
            } => {
                let len = old_bytes.len();
                if *offset + len <= self.data.len() {
                    self.data[*offset..*offset + len].copy_from_slice(old_bytes);
                }
            }
            HexAction::Insert { offset, bytes } => {
                let len = bytes.len();
                if *offset + len <= self.data.len() {
                    self.data.drain(*offset..*offset + len);
                }
            }
            HexAction::Delete {
                offset,
                deleted_bytes,
            } => {
                let insert_pos = (*offset).min(self.data.len());
                self.data
                    .splice(insert_pos..insert_pos, deleted_bytes.iter().copied());
            }
        }

        self.redo_stack.push(action);
        self.dirty = true;
        true
    }

    /// Redo操作を実行します。
    pub fn redo(&mut self) -> bool {
        let action = match self.redo_stack.pop() {
            Some(a) => a,
            None => return false,
        };

        match &action {
            HexAction::Replace {
                offset, new_bytes, ..
            } => {
                let len = new_bytes.len();
                if *offset + len <= self.data.len() {
                    self.data[*offset..*offset + len].copy_from_slice(new_bytes);
                }
            }
            HexAction::Insert { offset, bytes } => {
                let insert_pos = (*offset).min(self.data.len());
                self.data
                    .splice(insert_pos..insert_pos, bytes.iter().copied());
            }
            HexAction::Delete {
                offset,
                deleted_bytes,
            } => {
                let len = deleted_bytes.len();
                if *offset + len <= self.data.len() {
                    self.data.drain(*offset..*offset + len);
                }
            }
        }

        self.undo_stack.push(action);
        self.dirty = true;
        true
    }

    /// バイト列パターンを検索し、一致した先頭オフセット一覧を返します。
    pub fn search_bytes(&self, pattern: &[u8]) -> Vec<usize> {
        if pattern.is_empty() || pattern.len() > self.data.len() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        for i in 0..=(self.data.len() - pattern.len()) {
            if &self.data[i..i + pattern.len()] == pattern {
                matches.push(i);
            }
        }
        matches
    }

    /// データをUTF-8文字列として損失許容で変換します。
    pub fn to_utf8_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}

impl Default for HexDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// 1行（16バイト）のフォーマット済みHex表示行
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexRow {
    /// 行の先頭アドレスオフセット
    pub offset: usize,
    /// 16進ダンプ文字列（例: `48 65 6c 6c 6f 20 57 6f  72 6c 64 21 00 00 00 00`）
    pub hex_dump: String,
    /// ASCIIプレビュー文字列（例: `Hello World!....`）
    pub ascii_dump: String,
    /// 該当行に含まれるバイト数（通常16、末尾行は1〜16）
    pub byte_count: usize,
}

/// 1行あたりの標準バイト数（16バイト）
pub const BYTES_PER_ROW: usize = 16;

/// 指定オフセットから1行分のHexRowを生成します。
pub fn format_hex_row(data: &[u8], row_index: usize) -> Option<HexRow> {
    let offset = row_index * BYTES_PER_ROW;
    if offset >= data.len() && !data.is_empty() {
        return None;
    }

    if data.is_empty() {
        if row_index == 0 {
            return Some(HexRow {
                offset: 0,
                hex_dump: String::new(),
                ascii_dump: String::new(),
                byte_count: 0,
            });
        }
        return None;
    }

    let end = (offset + BYTES_PER_ROW).min(data.len());
    let slice = &data[offset..end];
    let byte_count = slice.len();

    // 16進文字列の生成 (8バイトごとに余分なスペース1文字)
    let mut hex_dump = String::with_capacity(50);
    #[allow(clippy::needless_range_loop)]
    for i in 0..BYTES_PER_ROW {
        if i == 8 {
            hex_dump.push(' ');
        }
        if i < byte_count {
            if i > 0 && i != 8 {
                hex_dump.push(' ');
            }
            hex_dump.push_str(&format!("{:02x}", slice[i]));
        } else {
            // パディング
            if i > 0 && i != 8 {
                hex_dump.push(' ');
            }
            hex_dump.push_str("  ");
        }
    }

    // ASCII文字列の生成
    let mut ascii_dump = String::with_capacity(BYTES_PER_ROW);
    for &b in slice {
        // 印字可能ASCII文字 (0x20..=0x7E) 以外は '.'
        if (0x20..=0x7E).contains(&b) {
            ascii_dump.push(b as char);
        } else {
            ascii_dump.push('.');
        }
    }

    Some(HexRow {
        offset,
        hex_dump,
        ascii_dump,
        byte_count,
    })
}

/// 全データの総行数（16バイト単位）を計算します。
///
/// # Examples
/// ```
/// use vivivi_modes::hex::calculate_row_count;
///
/// assert_eq!(calculate_row_count(0), 1);
/// assert_eq!(calculate_row_count(16), 1);
/// assert_eq!(calculate_row_count(17), 2);
/// ```
pub fn calculate_row_count(len: usize) -> usize {
    if len == 0 {
        1
    } else {
        len.div_ceil(BYTES_PER_ROW)
    }
}
