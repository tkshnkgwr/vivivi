//! トランザクション対応および連続入力マージを備えた Undo/Redo マネージャ。
//!
//! OpenViVi の `undoMgr.h` および `undoMgr.cpp` の設計思想を忠実にモデル化し、
//! 複数打鍵のタイピング統合、Vi複合コマンドのアトミックUndoブロック、
//! 保存点（Save Point）と Modified フラグの整合性追跡を提供します。

mod serialize;

#[cfg(test)]
mod tests;

/// 各編集操作（挿入・削除・置換）を表すアクション列挙体。
///
/// 各操作は逆操作（[`UndoAction::inverse`]）を生成可能であり、アンドゥ実行時に逆順で適用されます。
///
/// # Examples
///
/// ```
/// use vivivi_core::UndoAction;
///
/// let action = UndoAction::Insert {
///     pos: 0,
///     text: "abc".to_string(),
///     prohibit_merge: false,
/// };
/// match action.inverse() {
///     UndoAction::Delete { pos, text, .. } => {
///         assert_eq!(pos, 0);
///         assert_eq!(text, "abc");
///     }
///     _ => unreachable!(),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoAction {
    /// テキスト挿入
    Insert {
        /// 挿入位置文字オフセット
        pos: usize,
        /// 挿入された文字列
        text: String,
        /// 後続タイピングとの自動マージを禁止するフラグ
        prohibit_merge: bool,
    },
    /// テキスト削除
    Delete {
        /// 削除開始位置文字オフセット
        pos: usize,
        /// 削除された文字列
        text: String,
        /// バックスペースキーによる削除かどうか
        is_backspace: bool,
    },
    /// テキスト置換
    Replace {
        /// 置換開始位置文字オフセット
        pos: usize,
        /// 置換前の旧文字列
        old_text: String,
        /// 置換後の新文字列
        new_text: String,
    },
}

impl UndoAction {
    /// アンドゥ時に適用する逆操作アクションを生成します。
    ///
    /// - `Insert` の逆操作は `Delete`
    /// - `Delete` の逆操作は `Insert`
    /// - `Replace` の逆操作は旧文字列と新文字列を入れ替えた `Replace`
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoAction;
    ///
    /// let action = UndoAction::Delete {
    ///     pos: 5,
    ///     text: "xyz".to_string(),
    ///     is_backspace: false,
    /// };
    /// assert!(matches!(action.inverse(), UndoAction::Insert { pos: 5, .. }));
    /// ```
    pub fn inverse(&self) -> UndoAction {
        match self {
            Self::Insert { pos, text, .. } => Self::Delete {
                pos: *pos,
                text: text.clone(),
                is_backspace: false,
            },
            Self::Delete {
                pos,
                text,
                is_backspace,
            } => Self::Insert {
                pos: *pos,
                text: text.clone(),
                prohibit_merge: !*is_backspace,
            },
            Self::Replace {
                pos,
                old_text,
                new_text,
            } => Self::Replace {
                pos: *pos,
                old_text: new_text.clone(),
                new_text: old_text.clone(),
            },
        }
    }
}

/// 単一の Vi コマンドに起因する一連のアクションをアトミックにまとめるトランザクションブロック。
///
/// 複数の文字挿入や削除を1つの Undo 単位としてまとめます。
///
/// # Examples
///
/// ```
/// use vivivi_core::{UndoAction, UndoBlock};
///
/// let block = UndoBlock {
///     actions: vec![
///         UndoAction::Insert { pos: 0, text: "A".to_string(), prohibit_merge: false },
///         UndoAction::Insert { pos: 1, text: "B".to_string(), prohibit_merge: false },
///     ],
/// };
/// assert_eq!(block.actions.len(), 2);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoBlock {
    /// ブロックに含まれる一連のアクション列
    pub actions: Vec<UndoAction>,
}

/// トランザクション対応および連続入力マージを備えた Undo/Redo マネージャ本体。
///
/// Vi の複合コマンド（`dd`, `cw`, `caw` 等）のアトミックアンドゥと、
/// 保存点（Save Point）に基づく Modified 状態の追跡を提供します。
///
/// # Examples
///
/// ```
/// use vivivi_core::UndoMgr;
///
/// let mut mgr = UndoMgr::new();
/// mgr.push_insert(0, "hello");
/// assert!(mgr.can_undo());
/// assert!(mgr.is_modified());
///
/// mgr.on_saved();
/// assert!(!mgr.is_modified());
/// ```
#[derive(Debug, Clone, Default)]
pub struct UndoMgr {
    pub(super) undo_stack: Vec<UndoBlock>,
    pub(super) redo_stack: Vec<UndoBlock>,
    pub(super) current_block: Option<UndoBlock>,
    pub(super) block_nest_level: usize,
    /// ドキュメント保存時の `undo_stack` の長さ（保存点）
    pub(super) save_point: Option<usize>,
}

impl UndoMgr {
    /// 新しい [`UndoMgr`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mgr = UndoMgr::new();
    /// assert!(!mgr.can_undo());
    /// assert!(!mgr.can_redo());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Undo トランザクションブロックを開始します。
    ///
    /// 多重ネスト呼び出しに対応し、最外ブロックが確定するまで内部操作を単一の Undo 単位としてまとめます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.open_block();
    /// assert!(mgr.is_block_opened());
    /// mgr.close_block();
    /// assert!(!mgr.is_block_opened());
    /// ```
    pub fn open_block(&mut self) {
        if self.block_nest_level == 0 {
            self.current_block = Some(UndoBlock::default());
        }
        self.block_nest_level += 1;
    }

    /// Undo トランザクションブロックを 1 段終了します。
    ///
    /// ネスト深度が 0 に達した時点で、蓄積されたアクション群がスタックへ確定されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.open_block();
    /// mgr.push_insert(0, "A");
    /// mgr.close_block();
    /// assert!(mgr.can_undo());
    /// ```
    pub fn close_block(&mut self) {
        if self.block_nest_level > 0 {
            self.block_nest_level -= 1;
            if self.block_nest_level == 0
                && let Some(block) = self.current_block.take()
                && !block.actions.is_empty()
            {
                self.undo_stack.push(block);
                self.redo_stack.clear();
            }
        }
    }

    /// ネストされたすべての Undo トランザクションブロックを直ちに一括確定・クローズします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.open_block();
    /// mgr.open_block();
    /// mgr.push_insert(0, "nested");
    /// mgr.close_all_blocks();
    /// assert!(!mgr.is_block_opened());
    /// assert!(mgr.can_undo());
    /// ```
    pub fn close_all_blocks(&mut self) {
        if self.block_nest_level > 0 {
            self.block_nest_level = 0;
            if let Some(block) = self.current_block.take()
                && !block.actions.is_empty()
            {
                self.undo_stack.push(block);
                self.redo_stack.clear();
            }
        }
    }

    /// 現在開いている Undo トランザクションブロックの内部にいるかどうかを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// assert!(!mgr.is_block_opened());
    /// mgr.open_block();
    /// assert!(mgr.is_block_opened());
    /// ```
    pub fn is_block_opened(&self) -> bool {
        self.block_nest_level > 0
    }

    /// 最新の文字挿入アクションをマークし、以降のタイピングとの自動マージを禁止します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "a");
    /// mgr.prohibit_merge();
    /// mgr.push_insert(1, "b");
    /// // マージが禁止されたため、2つの独立したUndoブロックになる
    /// let u1 = mgr.pop_undo();
    /// assert!(u1.is_some());
    /// assert!(mgr.can_undo());
    /// ```
    pub fn prohibit_merge(&mut self) {
        if let Some(ref mut block) = self.current_block {
            if let Some(UndoAction::Insert { prohibit_merge, .. }) = block.actions.last_mut() {
                *prohibit_merge = true;
            }
        } else if let Some(last_block) = self.undo_stack.last_mut()
            && let Some(UndoAction::Insert { prohibit_merge, .. }) = last_block.actions.last_mut()
        {
            *prohibit_merge = true;
        }
    }

    /// 文字挿入操作を記録します。
    ///
    /// 直前の操作が単一文字入力であり、改行を含まない連続タイピングの場合は自動的に直前のアクションへマージ統合されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "a");
    /// mgr.push_insert(1, "b");
    /// // 連続タイピングは1つのブロックにまとめられる
    /// assert_eq!(mgr.pop_undo().unwrap().len(), 1);
    /// ```
    pub fn push_insert(&mut self, pos: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        // 直前の挿入操作とのマージを試行
        let last_action = if let Some(ref mut block) = self.current_block {
            block.actions.last_mut()
        } else {
            self.undo_stack
                .last_mut()
                .and_then(|b| b.actions.last_mut())
        };

        if let Some(UndoAction::Insert {
            pos: last_pos,
            text: last_text,
            prohibit_merge,
        }) = last_action
        {
            let last_end = *last_pos + last_text.chars().count();
            if !*prohibit_merge
                && last_end == pos
                && !text.contains('\n')
                && !last_text.contains('\n')
            {
                last_text.push_str(text);
                self.redo_stack.clear();
                return;
            }
        }

        let action = UndoAction::Insert {
            pos,
            text: text.to_string(),
            prohibit_merge: false,
        };

        if let Some(ref mut block) = self.current_block {
            block.actions.push(action);
        } else {
            self.undo_stack.push(UndoBlock {
                actions: vec![action],
            });
            self.redo_stack.clear();
        }
    }

    /// 文字削除操作を記録します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_delete(0, "xyz", false);
    /// assert!(mgr.can_undo());
    /// ```
    pub fn push_delete(&mut self, pos: usize, text: &str, is_backspace: bool) {
        if text.is_empty() {
            return;
        }
        let action = UndoAction::Delete {
            pos,
            text: text.to_string(),
            is_backspace,
        };

        if let Some(ref mut block) = self.current_block {
            block.actions.push(action);
        } else {
            self.undo_stack.push(UndoBlock {
                actions: vec![action],
            });
            self.redo_stack.clear();
        }
    }

    /// 文字置換操作を記録します。
    ///
    /// 旧文字列と新文字列が等しい場合は何も記録しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_replace(0, "old", "new");
    /// assert!(mgr.can_undo());
    /// ```
    pub fn push_replace(&mut self, pos: usize, old_text: &str, new_text: &str) {
        if old_text == new_text {
            return;
        }
        let action = UndoAction::Replace {
            pos,
            old_text: old_text.to_string(),
            new_text: new_text.to_string(),
        };

        if let Some(ref mut block) = self.current_block {
            block.actions.push(action);
        } else {
            self.undo_stack.push(UndoBlock {
                actions: vec![action],
            });
            self.redo_stack.clear();
        }
    }

    /// Undo スタックからブロックをポップし、逆順の取り消しアクション列を返します。
    ///
    /// 取り消されたブロックは同時に Redo スタックへ格納されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{UndoAction, UndoMgr};
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "A");
    /// let actions = mgr.pop_undo().unwrap();
    /// assert!(matches!(actions[0], UndoAction::Delete { pos: 0, .. }));
    /// assert!(mgr.can_redo());
    /// ```
    pub fn pop_undo(&mut self) -> Option<Vec<UndoAction>> {
        self.close_all_blocks();
        let block = self.undo_stack.pop()?;
        let reverse_actions: Vec<UndoAction> =
            block.actions.iter().rev().map(|a| a.inverse()).collect();
        self.redo_stack.push(block);
        Some(reverse_actions)
    }

    /// Redo スタックからブロックをポップし、再適用する元の操作アクション列を返します。
    ///
    /// 再適用されたブロックは同時に Undo スタックへ格納されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "A");
    /// mgr.pop_undo();
    /// let actions = mgr.pop_redo().unwrap();
    /// assert_eq!(actions.len(), 1);
    /// assert!(mgr.can_undo());
    /// ```
    pub fn pop_redo(&mut self) -> Option<Vec<UndoAction>> {
        self.close_all_blocks();
        let block = self.redo_stack.pop()?;
        let forward_actions = block.actions.clone();
        self.undo_stack.push(block);
        Some(forward_actions)
    }

    /// Undo 可能かどうかを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// assert!(!mgr.can_undo());
    /// mgr.push_insert(0, "A");
    /// assert!(mgr.can_undo());
    /// ```
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Redo 可能かどうかを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// assert!(!mgr.can_redo());
    /// mgr.push_insert(0, "A");
    /// mgr.pop_undo();
    /// assert!(mgr.can_redo());
    /// ```
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 現在の状態を「保存済み（未変更）」としてマークします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "edited");
    /// assert!(mgr.is_modified());
    /// mgr.on_saved();
    /// assert!(!mgr.is_modified());
    /// ```
    pub fn on_saved(&mut self) {
        self.save_point = Some(self.undo_stack.len());
    }

    /// 最後に保存した時点からバッファが変更されているかどうかを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// assert!(!mgr.is_modified());
    /// mgr.push_insert(0, "a");
    /// assert!(mgr.is_modified());
    /// mgr.pop_undo();
    /// assert!(!mgr.is_modified());
    /// ```
    pub fn is_modified(&self) -> bool {
        match self.save_point {
            Some(pt) => pt != self.undo_stack.len(),
            None => !self.undo_stack.is_empty(),
        }
    }

    /// すべての Undo/Redo 履歴をクリアし、初期状態にリセットします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "test");
    /// mgr.clear();
    /// assert!(!mgr.can_undo());
    /// assert!(!mgr.is_modified());
    /// ```
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_block = None;
        self.block_nest_level = 0;
        self.save_point = Some(0);
    }
}
