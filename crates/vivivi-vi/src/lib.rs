//! vivivi-vi: OpenViVi アーキテクチャに準拠した Vi コマンドエミュレーションエンジン。
//!
//! Vi モード遷移（[`ViMode`]）、キー入力パーサー（[`CommandParser`]）、
//! モーション・オペレータコマンドモデル（[`command`]）、レジスタ管理（[`RegisterMgr`]）、
//! および統合 Vi エンジン（[`ViEngine`]）を提供します。
//!
//! # Examples
//!
//! ```
//! use vivivi_core::TextBuffer;
//! use vivivi_vi::{ViEngine, ViMode};
//!
//! let mut buf = TextBuffer::from_text("hello");
//! let mut engine = ViEngine::new();
//! assert_eq!(engine.mode(), ViMode::Normal);
//! ```

pub mod command;
pub mod engine;
pub mod mode;
pub mod parser;
pub mod register;

pub use command::{Motion, Operator, ViCommand, WordSearchKind};
pub use engine::ViEngine;
pub use mode::ViMode;
pub use parser::{CommandParser, ParserState};
pub use register::{
    ClipboardHistory, ClipboardItem, ClipboardOption, ClipboardProvider, RegisterContent,
    RegisterMgr,
};
