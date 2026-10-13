//! Registers for copy, cut, paste.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterContent {
    CharacterWise(String),
    LineWise(String),
    BlockWise(Vec<String>),
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RegisterMgr {
    unnamed: Option<RegisterContent>,
    named: HashMap<char, RegisterContent>,
}

impl RegisterMgr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_unnamed(&mut self, content: RegisterContent) {
        self.unnamed = Some(content);
    }

    pub fn get_unnamed(&self) -> Option<&RegisterContent> {
        self.unnamed.as_ref()
    }
}
