//! Undo/Redo manager supporting transactional undo blocks.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditAction {
    Insert { line: usize, col: usize, text: String },
    Delete { line: usize, col: usize, text: String },
}

#[derive(Debug, Clone, Default)]
pub struct UndoBlock {
    pub actions: Vec<EditAction>,
}

#[derive(Debug, Clone, Default)]
pub struct UndoMgr {
    undo_stack: Vec<UndoBlock>,
    redo_stack: Vec<UndoBlock>,
    current_block: Option<UndoBlock>,
    block_nest_level: usize,
}

impl UndoMgr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_block(&mut self) {
        if self.block_nest_level == 0 {
            self.current_block = Some(UndoBlock::default());
        }
        self.block_nest_level += 1;
    }

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

    pub fn push_action(&mut self, action: EditAction) {
        if let Some(ref mut block) = self.current_block {
            block.actions.push(action);
        } else {
            self.undo_stack.push(UndoBlock {
                actions: vec![action],
            });
            self.redo_stack.clear();
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
