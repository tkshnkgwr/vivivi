//! テキストカーソルおよび選択範囲モデル。
//!
//! 0始まりの行・列座標（[`CursorPosition`]）と、選択範囲アンカーを持つ
//! カーソル状態（[`TextCursor`]）を定義します。

/// 0始まりの行番号と列番号を表すカーソル座標構造体。
///
/// テキストバッファ上の論理的な文字位置を表し、画面上の描画セル位置とは区別されます。
///
/// # Examples
///
/// ```
/// use vivivi_core::CursorPosition;
///
/// let pos = CursorPosition::new(0, 5);
/// assert_eq!(pos.line, 0);
/// assert_eq!(pos.col, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPosition {
    /// 0始まりの論理行番号
    pub line: usize,
    /// 0始まりの論理文字（Unicodeスカラー値）列番号
    pub col: usize,
}

impl CursorPosition {
    /// 指定された行番号と列番号で新しい [`CursorPosition`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::CursorPosition;
    ///
    /// let pos = CursorPosition::new(10, 2);
    /// assert_eq!(pos.line, 10);
    /// assert_eq!(pos.col, 2);
    /// ```
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// 選択アンカー（開始点）を持つテキストカーソル構造体。
///
/// 通常のカーソル位置 `pos` に加え、ビジュアル選択中の基点となる `anchor` を保持します。
///
/// # Examples
///
/// ```
/// use vivivi_core::{CursorPosition, TextCursor};
///
/// let mut cursor = TextCursor::new(0, 0);
/// assert!(!cursor.has_selection());
///
/// cursor.anchor = Some(CursorPosition::new(0, 0));
/// cursor.pos = CursorPosition::new(0, 5);
/// assert!(cursor.has_selection());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextCursor {
    /// 現在のカーソルヘッド位置
    pub pos: CursorPosition,
    /// 選択範囲の開始基点（選択中のみ `Some`）
    pub anchor: Option<CursorPosition>,
}

impl TextCursor {
    /// 指定された初期座標で選択範囲を持たない新しい [`TextCursor`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextCursor;
    ///
    /// let cursor = TextCursor::new(1, 0);
    /// assert_eq!(cursor.pos.line, 1);
    /// assert!(cursor.anchor.is_none());
    /// ```
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            pos: CursorPosition::new(line, col),
            anchor: None,
        }
    }

    /// 有効なテキスト選択範囲が存在するかどうかを判定します。
    ///
    /// アンカーが設定されており、かつ現在のカーソル位置と異なる場合に `true` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextCursor};
    ///
    /// let mut cursor = TextCursor::new(0, 0);
    /// cursor.anchor = Some(CursorPosition::new(0, 0));
    /// assert!(!cursor.has_selection()); // 開始点と同一位置は選択なし扱い
    ///
    /// cursor.pos.col = 3;
    /// assert!(cursor.has_selection());
    /// ```
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some() && self.anchor != Some(self.pos)
    }

    /// 現在の選択範囲を解除し、通常カーソル状態に戻します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextCursor};
    ///
    /// let mut cursor = TextCursor::new(0, 5);
    /// cursor.anchor = Some(CursorPosition::new(0, 0));
    /// assert!(cursor.has_selection());
    ///
    /// cursor.clear_selection();
    /// assert!(!cursor.has_selection());
    /// assert!(cursor.anchor.is_none());
    /// ```
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }
}
