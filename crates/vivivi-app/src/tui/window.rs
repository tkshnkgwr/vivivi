//! エディタウィンドウおよび画面分割管理モジュール。
//!
//! 複数ウィンドウへの分割（`:split` / `:vsplit`）に対応し、
//! 各ウィンドウ独立のカーソル位置およびスクロールオフセットを管理します。

use vivivi_core::TextCursor;

/// 画面分割の分割軸（上下分割または左右分割）。
///
/// # Examples
/// ```rust,ignore
/// use vivivi_app::tui::window::SplitAxis;
///
/// let axis = SplitAxis::Horizontal;
/// assert_eq!(axis, SplitAxis::Horizontal);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    /// 上下分割（`:split` または `Ctrl-W s` 相当）
    Horizontal,
    /// 左右分割（`:vsplit` または `Ctrl-W v` 相当）
    Vertical,
}

/// 分割表示される個別のエディタウィンドウ。
///
/// 各ウィンドウは同一のテキストバッファを参照しながら、
/// 独立したカーソル座標（[`TextCursor`]）とスクロールオフセット `(line, col)` を保持します。
///
/// # Examples
/// ```rust,ignore
/// use vivivi_app::tui::window::EditorWindow;
///
/// let win = EditorWindow::new(0, 0);
/// assert_eq!(win.scroll, (0, 0));
/// ```
#[derive(Debug, Clone)]
pub struct EditorWindow {
    /// ウィンドウ内のカーソル位置
    pub cursor: TextCursor,
    /// スクロールオフセット（表示開始行, 表示開始列）
    pub scroll: (usize, usize),
}

impl EditorWindow {
    /// 指定された初期カーソル座標で新しいウィンドウを生成します。
    ///
    /// スクロールオフセットは `(0, 0)` で初期化されます。
    ///
    /// # 引数
    /// - `line`: 初期の行位置（0-based）
    /// - `col`: 初期の列位置（0-based）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::window::EditorWindow;
    ///
    /// let win = EditorWindow::new(5, 10);
    /// assert_eq!(win.cursor.pos.line, 5);
    /// assert_eq!(win.cursor.pos.col, 10);
    /// ```
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            cursor: TextCursor::new(line, col),
            scroll: (0, 0),
        }
    }
}
