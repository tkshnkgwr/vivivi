//! Text cursor and selection models.

/// Represents a 0-indexed line and column position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPosition {
    pub line: usize,
    pub col: usize,
}

impl CursorPosition {
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// Text cursor with an anchor for selection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextCursor {
    pub pos: CursorPosition,
    pub anchor: Option<CursorPosition>,
}

impl TextCursor {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            pos: CursorPosition::new(line, col),
            anchor: None,
        }
    }

    pub fn has_selection(&self) -> bool {
        self.anchor.is_some() && self.anchor != Some(self.pos)
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }
}
