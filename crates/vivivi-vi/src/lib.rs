//! vivivi-vi: Vi engine emulation based on OpenViVi architecture.

pub mod command;
pub mod engine;
pub mod mode;
pub mod register;

pub use command::ViCommand;
pub use engine::ViEngine;
pub use mode::ViMode;
