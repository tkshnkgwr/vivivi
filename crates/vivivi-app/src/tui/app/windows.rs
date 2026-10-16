use super::App;
use super::*;
use crate::tui::window::{EditorWindow, SplitAxis};

impl App {
    // --- 画面分割 (Split / Vsplit) 操作 ---

    /// 画面を上下（水平）に2分割し、新しいウィンドウを作成します (`:split` または `Ctrl-W s` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_horizontal();
    /// assert_eq!(app.windows.len(), 2);
    /// ```
    pub fn split_horizontal(&mut self) {
        // 現在のアクティブウィンドウの最新状態を退避
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }

        let cur = self.cursor.clone();
        let scroll = self.scroll;
        let new_idx = self.windows.len();
        self.split_axis = SplitAxis::Horizontal;
        self.windows.push(EditorWindow {
            cursor: cur,
            scroll,
        });
        self.window_tree
            .split_window(self.active_window, new_idx, SplitAxis::Horizontal);
        self.active_window = new_idx;
        self.status_message = Some(format!(
            "水平分割: ウィンドウ {}/{}",
            self.active_window + 1,
            self.windows.len()
        ));
    }

    /// 画面を左右（垂直）に2分割し、新しいウィンドウを作成します (`:vsplit` または `Ctrl-W v` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_vertical();
    /// assert_eq!(app.windows.len(), 2);
    /// ```
    pub fn split_vertical(&mut self) {
        // 現在のアクティブウィンドウの最新状態を退避
        if let Some(win) = self.windows.get_mut(self.active_window) {
            win.cursor = self.cursor.clone();
            win.scroll = self.scroll;
        }

        let cur = self.cursor.clone();
        let scroll = self.scroll;
        let new_idx = self.windows.len();
        self.split_axis = SplitAxis::Vertical;
        self.windows.push(EditorWindow {
            cursor: cur,
            scroll,
        });
        self.window_tree
            .split_window(self.active_window, new_idx, SplitAxis::Vertical);
        self.active_window = new_idx;
        self.status_message = Some(format!(
            "垂直分割: ウィンドウ {}/{}",
            self.active_window + 1,
            self.windows.len()
        ));
    }

    /// 指定されたインデックスのウィンドウへフォーカスを切り替えます。
    pub fn switch_to_window(&mut self, target_idx: usize) {
        if target_idx < self.windows.len() && target_idx != self.active_window {
            // 現在のウィンドウ状態を退避
            self.windows[self.active_window].cursor = self.cursor.clone();
            self.windows[self.active_window].scroll = self.scroll;

            self.active_window = target_idx;
            self.cursor = self.windows[self.active_window].cursor.clone();
            self.scroll = self.windows[self.active_window].scroll;
            self.status_message = Some(format!(
                "ウィンドウ: {}/{}",
                self.active_window + 1,
                self.windows.len()
            ));
        }
    }

    /// フォーカスを次のウィンドウへ巡回移動します (`Ctrl-W w` 相当)。
    pub fn cycle_window(&mut self) {
        if self.windows.len() > 1 {
            let next_idx = (self.active_window + 1) % self.windows.len();
            self.switch_to_window(next_idx);
        }
    }

    /// フォーカスを前のウィンドウへ逆巡回移動します (`Ctrl-W W` 相当)。
    pub fn cycle_window_prev(&mut self) {
        if self.windows.len() > 1 {
            let prev_idx = if self.active_window == 0 {
                self.windows.len() - 1
            } else {
                self.active_window - 1
            };
            self.switch_to_window(prev_idx);
        }
    }

    /// 幾何学的な画面位置に基づいて、指定方向（上下左右）の隣接ウィンドウへフォーカスを移動します (`Ctrl-W h/j/k/l` 相当)。
    pub fn navigate_window(&mut self, dir: WindowDirection) {
        if self.windows.len() <= 1 {
            return;
        }

        let cur_idx = self.active_window;
        let cur_rect = self
            .window_rects
            .iter()
            .find(|(idx, _)| *idx == cur_idx)
            .map(|(_, r)| *r);

        let Some(cur_rect) = cur_rect else {
            // 矩形キャッシュがない場合のフォールバック
            match dir {
                WindowDirection::Left | WindowDirection::Up => self.cycle_window_prev(),
                WindowDirection::Right | WindowDirection::Down => self.cycle_window(),
            }
            return;
        };

        let mut best_target = None;
        let mut best_score = u32::MAX;

        for &(idx, rect) in &self.window_rects {
            if idx == cur_idx {
                continue;
            }

            let is_in_direction = match dir {
                WindowDirection::Left => {
                    rect.x + rect.width <= cur_rect.x + 1 && rect.x < cur_rect.x
                }
                WindowDirection::Right => {
                    rect.x + 1 >= cur_rect.x + cur_rect.width || rect.x > cur_rect.x
                }
                WindowDirection::Up => {
                    rect.y + rect.height <= cur_rect.y + 1 && rect.y < cur_rect.y
                }
                WindowDirection::Down => {
                    rect.y + 1 >= cur_rect.y + cur_rect.height || rect.y > cur_rect.y
                }
            };

            if is_in_direction {
                let score = match dir {
                    WindowDirection::Left => {
                        let dx = (cur_rect.x as i32 - (rect.x + rect.width) as i32).unsigned_abs();
                        let cy_cur = cur_rect.y as i32 + (cur_rect.height as i32 / 2);
                        let cy_rect = rect.y as i32 + (rect.height as i32 / 2);
                        let dy = (cy_cur - cy_rect).unsigned_abs();
                        dx * 10 + dy
                    }
                    WindowDirection::Right => {
                        let dx =
                            (rect.x as i32 - (cur_rect.x + cur_rect.width) as i32).unsigned_abs();
                        let cy_cur = cur_rect.y as i32 + (cur_rect.height as i32 / 2);
                        let cy_rect = rect.y as i32 + (rect.height as i32 / 2);
                        let dy = (cy_cur - cy_rect).unsigned_abs();
                        dx * 10 + dy
                    }
                    WindowDirection::Up => {
                        let dy = (cur_rect.y as i32 - (rect.y + rect.height) as i32).unsigned_abs();
                        let cx_cur = cur_rect.x as i32 + (cur_rect.width as i32 / 2);
                        let cx_rect = rect.x as i32 + (rect.width as i32 / 2);
                        let dx = (cx_cur - cx_rect).unsigned_abs();
                        dy * 10 + dx
                    }
                    WindowDirection::Down => {
                        let dy =
                            (rect.y as i32 - (cur_rect.y + cur_rect.height) as i32).unsigned_abs();
                        let cx_cur = cur_rect.x as i32 + (cur_rect.width as i32 / 2);
                        let cx_rect = rect.x as i32 + (rect.width as i32 / 2);
                        let dx = (cx_cur - cx_rect).unsigned_abs();
                        dy * 10 + dx
                    }
                };

                if score < best_score {
                    best_score = score;
                    best_target = Some(idx);
                }
            }
        }

        if let Some(target) = best_target {
            self.switch_to_window(target);
        }
    }

    /// 現在フォーカスされているウィンドウを閉じます (`:close` または `Ctrl-W c` 相当)。
    ///
    /// 最後の1ウィンドウの場合はウィンドウを閉じず、未保存変更がなければ終了フラグを立てます。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_horizontal();
    /// app.close_window();
    /// assert_eq!(app.windows.len(), 1);
    /// ```
    pub fn close_window(&mut self) {
        if self.windows.len() > 1 {
            let target_idx = self.active_window;
            self.window_tree.remove_window(target_idx);
            self.window_tree.adjust_indices_after_remove(target_idx);
            self.windows.remove(target_idx);
            if self.active_window >= self.windows.len() {
                self.active_window = self.windows.len() - 1;
            }
            self.cursor = self.windows[self.active_window].cursor.clone();
            self.scroll = self.windows[self.active_window].scroll;
            self.status_message = Some(format!("ウィンドウ閉鎖 (残り: {})", self.windows.len()));
        } else if self.buffer.is_modified() {
            self.status_message = Some("未保存の変更があります (:q! で強制終了)".to_string());
        } else {
            self.should_quit = true;
        }
    }

    /// 現在フォーカスされているウィンドウ以外をすべて閉じ、単一ウィンドウに統合します (`:only` または `Ctrl-W o` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.split_horizontal();
    /// app.split_vertical();
    /// app.only_window();
    /// assert_eq!(app.windows.len(), 1);
    /// ```
    pub fn only_window(&mut self) {
        let cur = self.cursor.clone();
        let scroll = self.scroll;
        self.windows = vec![EditorWindow {
            cursor: cur,
            scroll,
        }];
        self.active_window = 0;
        self.window_tree = WindowTreeNode::new(0);
        self.status_message = Some("単一ウィンドウに統合しました".to_string());
    }
}
