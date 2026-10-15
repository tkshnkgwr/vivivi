//! vivivi-core: 高速テキスト編集コアエンジン。
//!
//! バッファ管理（ギャップバッファ [`GapBuffer`]、テキストバッファ [`TextBuffer`]）、
//! アンドゥ・リドゥ管理（[`UndoMgr`]）、文字コード・改行自動判別（[`encoding`]）、
//! 対応括弧探索（[`paren`]）など、エディタのコア機能を純粋な Rust ロジック（UI非依存）で提供します。
//!
//! # Examples
//!
//! ```
//! use vivivi_core::TextBuffer;
//!
//! let mut buf = TextBuffer::from_text("Hello");
//! buf.insert_text(5, " World");
//! assert_eq!(buf.to_string(), "Hello World");
//!
//! buf.undo();
//! assert_eq!(buf.to_string(), "Hello");
//! ```

pub mod buffer;
pub mod cursor;
pub mod encoding;
pub mod gap_buffer;
pub mod line_mgr;
pub mod paren;
pub mod search;
pub mod undo;

pub use buffer::TextBuffer;
pub use cursor::{CursorPosition, TextCursor};
pub use encoding::{
    CharEncoding, NewlineType, decode_bytes, detect_encoding, detect_newline, encode_text,
};
pub use gap_buffer::GapBuffer;
pub use paren::{
    find_matching_paren, find_matching_paren_in_line, get_cursor_matching_paren, paren_pair,
};
pub use undo::{UndoAction, UndoBlock, UndoMgr};
