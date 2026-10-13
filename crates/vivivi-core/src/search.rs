//! Text search utilities.

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub ignore_case: bool,
    pub use_regex: bool,
    pub whole_word: bool,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line: usize,
    pub col: usize,
    pub length: usize,
}
