//! Vi modes enum and state transitions.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViMode {
    #[default]
    Normal,
    Insert,
    Replace,
    Visual,
    VisualLine,
    VisualBlock,
    CommandLine,
}

impl ViMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Replace => "REPLACE",
            Self::Visual => "VISUAL",
            Self::VisualLine => "V-LINE",
            Self::VisualBlock => "V-BLOCK",
            Self::CommandLine => "COMMAND",
        }
    }
}
