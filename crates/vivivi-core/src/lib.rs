//! vivivi-core: Text editing core engine, buffer management, undo/redo, encoding, search.

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
pub use paren::{find_matching_paren, get_cursor_matching_paren, paren_pair};
pub use undo::{UndoAction, UndoBlock, UndoMgr};
