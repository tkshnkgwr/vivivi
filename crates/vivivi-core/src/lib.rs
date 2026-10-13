//! vivivi-core: Text editing core engine, buffer management, undo/redo, encoding, search.

pub mod buffer;
pub mod cursor;
pub mod encoding;
pub mod gap_buffer;
pub mod line_mgr;
pub mod search;
pub mod undo;

pub use buffer::TextBuffer;
pub use cursor::{CursorPosition, TextCursor};
pub use gap_buffer::GapBuffer;
pub use undo::{UndoAction, UndoBlock, UndoMgr};
