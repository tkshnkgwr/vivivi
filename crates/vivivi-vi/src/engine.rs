//! ViEngine state machine and dispatch.

use crate::command::ViCommand;
use crate::mode::ViMode;
use crate::register::RegisterMgr;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ViEngine {
    mode: ViMode,
    registers: RegisterMgr,
    count_buffer: Option<usize>,
    pending_operator: Option<char>,
}

impl ViEngine {
    pub fn new() -> Self {
        Self {
            mode: ViMode::Normal,
            registers: RegisterMgr::new(),
            count_buffer: None,
            pending_operator: None,
        }
    }

    pub fn mode(&self) -> ViMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ViMode) {
        self.mode = mode;
    }

    pub fn registers(&self) -> &RegisterMgr {
        &self.registers
    }

    pub fn registers_mut(&mut self) -> &mut RegisterMgr {
        &mut self.registers
    }

    /// Feeds a single character into the Vi state machine.
    pub fn feed_char(&mut self, ch: char) -> Option<ViCommand> {
        match self.mode {
            ViMode::Normal => self.handle_normal(ch),
            ViMode::Insert => {
                if ch == '\x1b' {
                    self.mode = ViMode::Normal;
                    Some(ViCommand::EnterMode(ViMode::Normal))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn handle_normal(&mut self, ch: char) -> Option<ViCommand> {
        match ch {
            'i' => {
                self.mode = ViMode::Insert;
                Some(ViCommand::InsertBefore)
            }
            'a' => {
                self.mode = ViMode::Insert;
                Some(ViCommand::InsertAfter)
            }
            'I' => {
                self.mode = ViMode::Insert;
                Some(ViCommand::InsertLineStart)
            }
            'A' => {
                self.mode = ViMode::Insert;
                Some(ViCommand::InsertLineEnd)
            }
            'o' => {
                self.mode = ViMode::Insert;
                Some(ViCommand::OpenLineBelow)
            }
            'O' => {
                self.mode = ViMode::Insert;
                Some(ViCommand::OpenLineAbove)
            }
            'u' => Some(ViCommand::Undo),
            'v' => {
                self.mode = ViMode::Visual;
                Some(ViCommand::EnterMode(ViMode::Visual))
            }
            'V' => {
                self.mode = ViMode::VisualLine;
                Some(ViCommand::EnterMode(ViMode::VisualLine))
            }
            _ => None,
        }
    }
}
