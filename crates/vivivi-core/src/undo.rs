//! トランザクション対応および連続入力マージを備えた Undo/Redo マネージャ。
//!
//! OpenViVi の `undoMgr.h` および `undoMgr.cpp` の設計思想を忠実にモデル化し、
//! 複数打鍵のタイピング統合、Vi複合コマンドのアトミックUndoブロック、
//! 保存点（Save Point）と Modified フラグの整合性追跡を提供します。

/// 各編集操作（挿入・削除・置換）を表すアクション列挙体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoAction {
    Insert {
        pos: usize,
        text: String,
        prohibit_merge: bool,
    },
    Delete {
        pos: usize,
        text: String,
        is_backspace: bool,
    },
    Replace {
        pos: usize,
        old_text: String,
        new_text: String,
    },
}

impl UndoAction {
    /// 逆操作アクション（アンドゥ時に適用する操作）を生成します。
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

/// 単一の Vi コマンド（`dd` や `cw` など）に起因する一連のアクションをアトミックにまとめるトランザクションブロック。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoBlock {
    pub actions: Vec<UndoAction>,
}

/// アンドゥ・リドゥ履歴管理マネージャ本体。
#[derive(Debug, Clone, Default)]
pub struct UndoMgr {
    undo_stack: Vec<UndoBlock>,
    redo_stack: Vec<UndoBlock>,
    current_block: Option<UndoBlock>,
    block_nest_level: usize,
    /// ドキュメント保存時の `undo_stack` の長さ（保存点）
    save_point: Option<usize>,
}

impl UndoMgr {
    /// 新しい `UndoMgr` を生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// Undo トランザクションブロックを開始します。多重ネスト呼び出しに対応しています。
    pub fn open_block(&mut self) {
        if self.block_nest_level == 0 {
            self.current_block = Some(UndoBlock::default());
        }
        self.block_nest_level += 1;
    }

    /// Undo トランザクションブロックを終了します。ネスト深度が 0 に達した時点でブロックが確定します。
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
    pub fn is_block_opened(&self) -> bool {
        self.block_nest_level > 0
    }

    /// 最新の文字挿入アクションをマークし、以降のタイピングとの自動マージを禁止します。
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

    /// 文字挿入操作を記録します。連続した単一文字タイピングは自動的にマージ統合されます。
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
    pub fn pop_undo(&mut self) -> Option<Vec<UndoAction>> {
        self.close_all_blocks();
        let block = self.undo_stack.pop()?;
        let reverse_actions: Vec<UndoAction> =
            block.actions.iter().rev().map(|a| a.inverse()).collect();
        self.redo_stack.push(block);
        Some(reverse_actions)
    }

    /// Redo スタックからブロックをポップし、再適用する元の操作アクション列を返します。
    pub fn pop_redo(&mut self) -> Option<Vec<UndoAction>> {
        self.close_all_blocks();
        let block = self.redo_stack.pop()?;
        let forward_actions = block.actions.clone();
        self.undo_stack.push(block);
        Some(forward_actions)
    }

    /// Undo 可能かどうかを返します。
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Redo 可能かどうかを返します。
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 現在の状態を「保存済み（未変更）」としてマークします。
    pub fn on_saved(&mut self) {
        self.save_point = Some(self.undo_stack.len());
    }

    /// 最後に保存した時点からバッファが変更されているかどうかを返します。
    pub fn is_modified(&self) -> bool {
        match self.save_point {
            Some(pt) => pt != self.undo_stack.len(),
            None => !self.undo_stack.is_empty(),
        }
    }

    /// すべての Undo/Redo 履歴をクリアします。
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_block = None;
        self.block_nest_level = 0;
        self.save_point = Some(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuous_insert_merging() {
        let mut undo_mgr = UndoMgr::new();
        undo_mgr.push_insert(0, "h");
        undo_mgr.push_insert(1, "e");
        undo_mgr.push_insert(2, "l");
        undo_mgr.push_insert(3, "l");
        undo_mgr.push_insert(4, "o");

        assert_eq!(undo_mgr.undo_stack.len(), 1);
        if let UndoAction::Insert { text, .. } = &undo_mgr.undo_stack[0].actions[0] {
            assert_eq!(text, "hello");
        } else {
            panic!("Expected Insert action");
        }

        let undo_actions = undo_mgr.pop_undo().unwrap();
        assert_eq!(undo_actions.len(), 1);
        assert_eq!(
            undo_actions[0],
            UndoAction::Delete {
                pos: 0,
                text: "hello".to_string(),
                is_backspace: false,
            }
        );
    }

    #[test]
    fn test_undo_block_transaction() {
        let mut undo_mgr = UndoMgr::new();
        undo_mgr.open_block();
        undo_mgr.push_delete(0, "old", false);
        undo_mgr.push_insert(0, "new");
        undo_mgr.close_block();

        assert_eq!(undo_mgr.undo_stack.len(), 1);
        assert_eq!(undo_mgr.undo_stack[0].actions.len(), 2);

        let reverse = undo_mgr.pop_undo().unwrap();
        assert_eq!(reverse.len(), 2);
        // 逆順に取り消し: "new" を削除、次に "old" を挿入
        assert_eq!(
            reverse[0],
            UndoAction::Delete {
                pos: 0,
                text: "new".to_string(),
                is_backspace: false,
            }
        );
        assert_eq!(
            reverse[1],
            UndoAction::Insert {
                pos: 0,
                text: "old".to_string(),
                prohibit_merge: true,
            }
        );
    }
}
