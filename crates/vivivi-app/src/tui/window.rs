//! エディタウィンドウおよび画面分割管理モジュール。

use vivivi_core::TextCursor;

/// 分割軸（水平分割 / 垂直分割）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    /// 上下分割 (:split)
    Horizontal,
    /// 左右分割 (:vsplit)
    Vertical,
}

/// 個別のエディタウィンドウ（カーソル位置とスクロール位置）
#[derive(Debug, Clone)]
pub struct EditorWindow {
    pub cursor: TextCursor,
    pub scroll: (usize, usize),
}

impl EditorWindow {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            cursor: TextCursor::new(line, col),
            scroll: (0, 0),
        }
    }
}
