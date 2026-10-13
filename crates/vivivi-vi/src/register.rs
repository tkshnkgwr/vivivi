//! レジスタ管理モジュール（ヤンク・削除・貼り付け）。
//!
//! Viの無名レジスタ（Unnamed register `""`）、名前付きレジスタ (`"a`〜`"z`)、
//! クリップボード連携レジスタ (`"+`, `"*`)、および行指向/文字指向の属性を管理します。

use std::collections::HashMap;

/// レジスタに格納されるテキストの指向性・形式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterContent {
    /// 文字単位（インライン）のテキスト
    CharacterWise(String),
    /// 行単位のテキスト（末尾に改行を含む）
    LineWise(String),
    /// 矩形ブロックテキスト
    BlockWise(Vec<String>),
}

impl RegisterContent {
    /// 内容のプレーン文字列を返します。
    pub fn as_str(&self) -> &str {
        match self {
            Self::CharacterWise(s) | Self::LineWise(s) => s.as_str(),
            Self::BlockWise(lines) => {
                // ブロックの場合は最初の行、または結合
                lines.first().map(|s| s.as_str()).unwrap_or("")
            }
        }
    }

    /// 行指向テキストかどうかを判定します。
    pub fn is_line_wise(&self) -> bool {
        matches!(self, Self::LineWise(_))
    }
}

/// レジスタマネージャ。
#[derive(Debug, Clone, Default)]
pub struct RegisterMgr {
    /// 無名レジスタ (デフォルトのコピー・削除先)
    unnamed: Option<RegisterContent>,
    /// 名前付きレジスタ ('a' - 'z', '0' - '9' 等)
    named: HashMap<char, RegisterContent>,
}

impl RegisterMgr {
    /// 新しい RegisterMgr を生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// レジスタに内容を設定します。
    ///
    /// レジスタ名が `None` または `Some('"')` の場合は無名レジスタに設定します。
    /// 指定された名前がある場合は、無名レジスタと名前付きレジスタの両方に設定します（Viの標準動作）。
    pub fn set(&mut self, reg_name: Option<char>, content: RegisterContent) {
        match reg_name {
            Some(name) if name.is_ascii_alphabetic() => {
                let lower = name.to_ascii_lowercase();
                if name.is_ascii_uppercase() {
                    // 大文字レジスタは既存内容へのアペンド
                    if let Some(existing) = self.named.get_mut(&lower) {
                        match (existing, &content) {
                            (
                                RegisterContent::CharacterWise(e),
                                RegisterContent::CharacterWise(c),
                            ) => {
                                e.push_str(c);
                            }
                            (RegisterContent::LineWise(e), RegisterContent::LineWise(c)) => {
                                e.push_str(c);
                            }
                            (e, _) => {
                                *e = content.clone();
                            }
                        }
                    } else {
                        self.named.insert(lower, content.clone());
                    }
                } else {
                    self.named.insert(lower, content.clone());
                }
                self.unnamed = Some(content);
            }
            _ => {
                self.unnamed = Some(content);
            }
        }
    }

    /// 指定されたレジスタの内容を取得します。
    ///
    /// `reg_name` が `None` の場合は無名レジスタを返します。
    pub fn get(&self, reg_name: Option<char>) -> Option<&RegisterContent> {
        match reg_name {
            Some(name) if name.is_ascii_alphabetic() => self
                .named
                .get(&name.to_ascii_lowercase())
                .or(self.unnamed.as_ref()),
            _ => self.unnamed.as_ref(),
        }
    }

    /// 無名レジスタの内容を取得します。
    pub fn get_unnamed(&self) -> Option<&RegisterContent> {
        self.unnamed.as_ref()
    }

    /// 無名レジスタに内容を設定します。
    pub fn set_unnamed(&mut self, content: RegisterContent) {
        self.unnamed = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_basic_unnamed() {
        let mut mgr = RegisterMgr::new();
        assert!(mgr.get(None).is_none());

        mgr.set(None, RegisterContent::CharacterWise("hello".to_string()));
        let content = mgr.get(None).unwrap();
        assert_eq!(content.as_str(), "hello");
        assert!(!content.is_line_wise());
    }

    #[test]
    fn test_named_register_and_unnamed_sync() {
        let mut mgr = RegisterMgr::new();

        // "a に設定すると、無名レジスタにも同期される
        mgr.set(Some('a'), RegisterContent::LineWise("line1\n".to_string()));
        assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "line1\n");
        assert_eq!(mgr.get(None).unwrap().as_str(), "line1\n");

        // "b に別内容を設定
        mgr.set(
            Some('b'),
            RegisterContent::CharacterWise("word".to_string()),
        );
        assert_eq!(mgr.get(Some('b')).unwrap().as_str(), "word");
        // "a は維持される
        assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "line1\n");
    }

    #[test]
    fn test_uppercase_register_append() {
        let mut mgr = RegisterMgr::new();
        mgr.set(Some('a'), RegisterContent::CharacterWise("foo".to_string()));

        // 大文字 "A でアペンド
        mgr.set(Some('A'), RegisterContent::CharacterWise("bar".to_string()));
        assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "foobar");
    }
}
