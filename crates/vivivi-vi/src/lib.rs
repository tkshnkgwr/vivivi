//! vivivi-vi: Vi engine emulation based on OpenViVi architecture.

pub mod command;
pub mod engine;
pub mod mode;
pub mod parser;
pub mod register;

pub use command::{Motion, Operator, ViCommand};
pub use engine::ViEngine;
pub use mode::ViMode;
pub use parser::CommandParser;
pub use register::{RegisterContent, RegisterMgr};
