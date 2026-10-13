//! Undo/Redo manager supporting transactional blocks and continuous typing merging,
//! faithfully modeled after OpenViVi's `undoMgr.h` and `undoMgr.cpp`.

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
    pub fn inverse(&self) -> UndoAction {
        match self {
            Self::Insert { pos, text, .. } => Self::Delete {
                pos: *pos,
                text: text.clone(),
                is_backspace: false,
            },
            Self::Delete { pos, text, is_backspace } => Self::Insert {
                pos: *pos,
                text: text.clone(),
                prohibit_merge: !*is_backspace,
            },
            Self::Replace { pos, old_text, new_text } => Self::Replace {
                pos: *pos,
                old_text: new_text.clone(),
                new_text: old_text.clone(),
            },
        }
    }
}

/// A transactional group of Undo actions (e.g. from a single vi command like `dd` or `cw`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoBlock {
    pub actions: Vec<UndoAction>,
}

#[derive(Debug, Clone, Default)]
pub struct UndoMgr {
    undo_stack: Vec<UndoBlock>,
    redo_stack: Vec<UndoBlock>,
    current_block: Option<UndoBlock>,
    block_nest_level: usize,
    /// The index in `undo_stack` where the document was saved.
    save_point: Option<usize>,
}

impl UndoMgr {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts an undo transaction block. Multiple nested calls are supported.
    pub fn open_block(&mut self) {
        if self.block_nest_level == 0 {
            self.current_block = Some(UndoBlock::default());
        }
        self.block_nest_level += 1;
    }

    /// Closes an undo transaction block. When nest level reaches 0, the block is finalized.
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

    /// Closes all nested undo transaction blocks immediately.
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

    /// Are we currently inside an open undo block?
    pub fn is_block_opened(&self) -> bool {
        self.block_nest_level > 0
    }

    /// Marks the latest insert action so it will not merge with subsequent typing.
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

    /// Records a text insertion, merging consecutive single-character typings if contiguous.
    pub fn push_insert(&mut self, pos: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        // Try merging with the last insert action
        let last_action = if let Some(ref mut block) = self.current_block {
            block.actions.last_mut()
        } else {
            self.undo_stack.last_mut().and_then(|b| b.actions.last_mut())
        };

        if let Some(UndoAction::Insert {
            pos: last_pos,
            text: last_text,
            prohibit_merge,
        }) = last_action {
            let last_end = *last_pos + last_text.chars().count();
            if !*prohibit_merge && last_end == pos && !text.contains('\n') && !last_text.contains('\n') {
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

    /// Records a text deletion.
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

    /// Records a text replacement.
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

    /// Pops an undo block from the undo stack, returning actions to reverse (in reverse execution order).
    pub fn pop_undo(&mut self) -> Option<Vec<UndoAction>> {
        self.close_all_blocks();
        let block = self.undo_stack.pop()?;
        let reverse_actions: Vec<UndoAction> = block.actions.iter().rev().map(|a| a.inverse()).collect();
        self.redo_stack.push(block);
        Some(reverse_actions)
    }

    /// Pops a redo block from the redo stack, returning original actions to re-apply.
    pub fn pop_redo(&mut self) -> Option<Vec<UndoAction>> {
        self.close_all_blocks();
        let block = self.redo_stack.pop()?;
        let forward_actions = block.actions.clone();
        self.undo_stack.push(block);
        Some(forward_actions)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Marks the current state as saved.
    pub fn on_saved(&mut self) {
        self.save_point = Some(self.undo_stack.len());
    }

    /// Checks whether the buffer has been modified since the last save point.
    pub fn is_modified(&self) -> bool {
        match self.save_point {
            Some(pt) => pt != self.undo_stack.len(),
            None => !self.undo_stack.is_empty(),
        }
    }

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
        // Inverse in reverse order: delete "new", then insert "old"
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
