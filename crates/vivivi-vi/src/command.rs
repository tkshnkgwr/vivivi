//! Vi commands parsed from user keystrokes.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
    StartOfLine,
    EndOfLine,
    FirstNonBlank,
    NextWord(usize),
    PrevWord(usize),
    EndWord(usize),
    GotoLine(Option<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViCommand {
    Move(Motion),
    Operate { op: Operator, motion: Motion },
    InsertBefore,
    InsertAfter,
    InsertLineStart,
    InsertLineEnd,
    OpenLineBelow,
    OpenLineAbove,
    Undo,
    Redo,
    PasteAfter,
    PasteBefore,
    EnterMode(crate::mode::ViMode),
    ExCommand(String),
}
