//! テキストカーソルおよび選択範囲モデル。

/// 0始まりの行番号と列番号を表すカーソル座標構造体。
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

/// 選択アンカー（開始点）を持つテキストカーソル構造体。
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

    /// 選択範囲が存在するかどうかを返します。
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some() && self.anchor != Some(self.pos)
    }

    /// 選択範囲を解除します。
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }
}
