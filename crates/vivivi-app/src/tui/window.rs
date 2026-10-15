//! エディタウィンドウおよび画面分割管理モジュール。
//!
//! 複数ウィンドウへの分割（`:split` / `:vsplit`）に対応し、
//! 各ウィンドウ独立のカーソル位置およびスクロールオフセット、
//! ならびに Vim ライクな階層的ウィンドウ分割ツリー（Window Layout Tree）を管理します。

use ratatui::layout::{Constraint, Layout, Rect};
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

/// ウィンドウ間移動の方向（`Ctrl-W h/j/k/l`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowDirection {
    /// 左のウィンドウ（`Ctrl-W h` / `Left`）
    Left,
    /// 下のウィンドウ（`Ctrl-W j` / `Down`）
    Down,
    /// 上のウィンドウ（`Ctrl-W k` / `Up`）
    Up,
    /// 右のウィンドウ（`Ctrl-W l` / `Right`）
    Right,
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

/// ウィンドウレイアウトツリーの階層ノード。
///
/// Vim のウィンドウ管理と同様に、画面の分割状態を木構造で表現します。
/// 単一ウィンドウは [`WindowTreeNode::Leaf`]、分割された領域は [`WindowTreeNode::Split`] で表され、
/// 任意の深さで水平（上下）・垂直（左右）の分割をネストできます。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowTreeNode {
    /// 個別のエディタウィンドウ（インデックス）
    Leaf(usize),
    /// 子ノードを分割軸に沿って等分割配置するコンテナ
    Split {
        /// 分割軸（水平または垂直）
        axis: SplitAxis,
        /// 分割された子ノード一覧
        children: Vec<WindowTreeNode>,
    },
}

impl WindowTreeNode {
    /// 初期ウィンドウインデックスを持つ単一ノードを生成します。
    pub fn new(initial_win: usize) -> Self {
        Self::Leaf(initial_win)
    }

    /// 指定された領域 `area` 内で各ウィンドウの矩形領域を再帰的に計算し、`(win_idx, Rect)` のリストを返します。
    pub fn compute_rects(&self, area: Rect) -> Vec<(usize, Rect)> {
        let mut out = Vec::new();
        self.collect_rects(area, &mut out);
        out
    }

    fn collect_rects(&self, area: Rect, out: &mut Vec<(usize, Rect)>) {
        match self {
            WindowTreeNode::Leaf(idx) => {
                out.push((*idx, area));
            }
            WindowTreeNode::Split { axis, children } => {
                if children.is_empty() {
                    return;
                }
                let count = children.len();
                let constraints = vec![Constraint::Ratio(1, count as u32); count];
                let chunks = match axis {
                    SplitAxis::Horizontal => Layout::vertical(constraints).split(area),
                    SplitAxis::Vertical => Layout::horizontal(constraints).split(area),
                };
                for (child, &chunk) in children.iter().zip(chunks.iter()) {
                    child.collect_rects(chunk, out);
                }
            }
        }
    }

    /// 対象のウィンドウインデックス `target_idx` を指定した軸 `axis` で分割し、
    /// 新しいウィンドウインデックス `new_idx` を追加します。
    pub fn split_window(&mut self, target_idx: usize, new_idx: usize, axis: SplitAxis) -> bool {
        match self {
            WindowTreeNode::Leaf(idx) if *idx == target_idx => {
                *self = WindowTreeNode::Split {
                    axis,
                    children: vec![
                        WindowTreeNode::Leaf(target_idx),
                        WindowTreeNode::Leaf(new_idx),
                    ],
                };
                true
            }
            WindowTreeNode::Leaf(_) => false,
            WindowTreeNode::Split {
                axis: cur_axis,
                children,
            } => {
                // 親の分割軸と一致し、直下に target_idx が存在する場合は兄弟として追加
                if *cur_axis == axis {
                    let mut insert_pos = None;
                    for (i, child) in children.iter().enumerate() {
                        if matches!(child, WindowTreeNode::Leaf(idx) if *idx == target_idx) {
                            insert_pos = Some(i + 1);
                            break;
                        }
                    }
                    if let Some(pos) = insert_pos {
                        children.insert(pos, WindowTreeNode::Leaf(new_idx));
                        return true;
                    }
                }

                // 子ノードへ再帰的に委譲
                for child in children.iter_mut() {
                    if child.split_window(target_idx, new_idx, axis) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// 対象のウィンドウインデックス `target_idx` をツリーから削除し、不要になったコンテナを縮退します。
    pub fn remove_window(&mut self, target_idx: usize) -> bool {
        let removed = self.remove_recursive(target_idx);
        if removed {
            self.compact();
        }
        removed
    }

    fn remove_recursive(&mut self, target_idx: usize) -> bool {
        match self {
            WindowTreeNode::Leaf(_) => false,
            WindowTreeNode::Split { children, .. } => {
                let initial_len = children.len();
                children.retain(|c| match c {
                    WindowTreeNode::Leaf(idx) => *idx != target_idx,
                    _ => true,
                });
                if children.len() < initial_len {
                    return true;
                }
                for child in children.iter_mut() {
                    if child.remove_recursive(target_idx) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// 子が1つだけになったコンテナを、その子自身に昇格（縮退）させます。
    pub fn compact(&mut self) {
        if let WindowTreeNode::Split { children, .. } = self {
            for child in children.iter_mut() {
                child.compact();
            }
            if children.len() == 1 {
                *self = children.remove(0);
            }
        }
    }

    /// ウィンドウ削除によるインデックスの詰まりを反映し、`removed_idx` より大きいインデックスをデクリメントします。
    pub fn adjust_indices_after_remove(&mut self, removed_idx: usize) {
        match self {
            WindowTreeNode::Leaf(idx) => {
                if *idx > removed_idx {
                    *idx -= 1;
                }
            }
            WindowTreeNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    child.adjust_indices_after_remove(removed_idx);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_nested_splits() {
        let mut tree = WindowTreeNode::new(0);

        // 1. 0 を水平分割 -> [0, 1]
        assert!(tree.split_window(0, 1, SplitAxis::Horizontal));
        assert_eq!(
            tree,
            WindowTreeNode::Split {
                axis: SplitAxis::Horizontal,
                children: vec![WindowTreeNode::Leaf(0), WindowTreeNode::Leaf(1)],
            }
        );

        // 2. 下の 1 を垂直分割 -> 0 の下で [1, 2] が垂直分割
        assert!(tree.split_window(1, 2, SplitAxis::Vertical));
        assert_eq!(
            tree,
            WindowTreeNode::Split {
                axis: SplitAxis::Horizontal,
                children: vec![
                    WindowTreeNode::Leaf(0),
                    WindowTreeNode::Split {
                        axis: SplitAxis::Vertical,
                        children: vec![WindowTreeNode::Leaf(1), WindowTreeNode::Leaf(2)],
                    }
                ],
            }
        );

        // 3. 領域計算の検証 (Rect: 100 x 40)
        let rects = tree.compute_rects(Rect::new(0, 0, 100, 40));
        assert_eq!(rects.len(), 3);
        // 上半分 (0): y=0, height=20, width=100
        assert_eq!(rects[0], (0, Rect::new(0, 0, 100, 20)));
        // 左下 (1): y=20, height=20, width=50
        assert_eq!(rects[1], (1, Rect::new(0, 20, 50, 20)));
        // 右下 (2): y=20, height=20, width=50, x=50
        assert_eq!(rects[2], (2, Rect::new(50, 20, 50, 20)));
    }

    #[test]
    fn test_tree_remove_and_compact() {
        let mut tree = WindowTreeNode::new(0);
        tree.split_window(0, 1, SplitAxis::Horizontal);
        tree.split_window(1, 2, SplitAxis::Vertical);

        // 2 を削除 -> 1 だけになり縮退して [0, 1] に戻る
        assert!(tree.remove_window(2));
        tree.adjust_indices_after_remove(2);
        assert_eq!(
            tree,
            WindowTreeNode::Split {
                axis: SplitAxis::Horizontal,
                children: vec![WindowTreeNode::Leaf(0), WindowTreeNode::Leaf(1)],
            }
        );

        // 0 を削除 -> 1 だけになり縮退して Leaf(0) になる (adjust により 1 -> 0)
        assert!(tree.remove_window(0));
        tree.adjust_indices_after_remove(0);
        assert_eq!(tree, WindowTreeNode::Leaf(0));
    }
}
