//! レジスタ管理モジュール（ヤンク・削除・貼り付け）。
//!
//! Viの無名レジスタ（Unnamed register `""`）、名前付きレジスタ (`"a`〜`"z`)、
//! クリップボード連携レジスタ (`"+`, `"*`)、および行指向/文字指向の属性を管理します。

use std::collections::{HashMap, VecDeque};

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
            Self::BlockWise(lines) => lines.first().map(|s| s.as_str()).unwrap_or(""),
        }
    }

    /// 行指向テキストかどうかを判定します。
    pub fn is_line_wise(&self) -> bool {
        matches!(self, Self::LineWise(_))
    }

    /// プレビュー用の一行サマリー文字列を返します。
    pub fn preview_text(&self, max_len: usize) -> String {
        let raw = match self {
            Self::CharacterWise(s) | Self::LineWise(s) => s.lines().next().unwrap_or(s),
            Self::BlockWise(lines) => lines.first().map(|s| s.as_str()).unwrap_or(""),
        };
        let trimmed = raw.trim();
        if trimmed.chars().count() > max_len {
            let truncated: String = trimmed.chars().take(max_len).collect();
            format!("{}...", truncated)
        } else {
            trimmed.to_string()
        }
    }

    /// 行数を返します。
    pub fn line_count(&self) -> usize {
        match self {
            Self::CharacterWise(s) | Self::LineWise(s) => s.lines().count().max(1),
            Self::BlockWise(lines) => lines.len().max(1),
        }
    }
}

/// クリップボード履歴の1アイテム。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardItem {
    pub id: usize,
    pub content: RegisterContent,
    pub preview: String,
    pub line_count: usize,
}

/// クリップボード履歴マネージャ。
#[derive(Debug, Clone)]
pub struct ClipboardHistory {
    pub max_items: usize,
    items: VecDeque<ClipboardItem>,
    next_id: usize,
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self {
            max_items: 50,
            items: VecDeque::new(),
            next_id: 1,
        }
    }
}

impl ClipboardHistory {
    /// 指定最大件数でクリップボード履歴を生成します。
    pub fn new(max_items: usize) -> Self {
        Self {
            max_items,
            items: VecDeque::new(),
            next_id: 1,
        }
    }

    /// 新しい履歴アイテムを追加します。空文字列や直前と完全に同一の内容は重複排除します。
    pub fn push(&mut self, content: RegisterContent) {
        let text = content.as_str().trim();
        if text.is_empty() {
            return;
        }

        // 直前のアイテムと同一内容ならスキップ
        if let Some(front) = self.items.front()
            && front.content == content
        {
            return;
        }

        let item = ClipboardItem {
            id: self.next_id,
            preview: content.preview_text(40),
            line_count: content.line_count(),
            content,
        };
        self.next_id += 1;

        self.items.push_front(item);
        while self.items.len() > self.max_items {
            self.items.pop_back();
        }
    }

    /// 履歴アイテム一覧のスライスを返します（新しい順）。
    pub fn items(&self) -> &VecDeque<ClipboardItem> {
        &self.items
    }

    /// インデックス指定でアイテムを取得します。
    pub fn get(&self, index: usize) -> Option<&ClipboardItem> {
        self.items.get(index)
    }

    /// 履歴件数を返します。
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 履歴が空かどうかを返します。
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 履歴を全消去します。
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// レジスタマネージャ。
#[derive(Debug, Clone, Default)]
pub struct RegisterMgr {
    /// 無名レジスタ (デフォルトのコピー・削除先)
    unnamed: Option<RegisterContent>,
    /// 名前付きレジスタ ('a' - 'z', '0' - '9' 等)
    named: HashMap<char, RegisterContent>,
    /// クリップボード履歴
    history: ClipboardHistory,
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
        // 履歴に自動蓄積
        self.history.push(content.clone());

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

    /// クリップボード履歴の参照を取得します。
    pub fn history(&self) -> &ClipboardHistory {
        &self.history
    }

    /// クリップボード履歴の可変参照を取得します。
    pub fn history_mut(&mut self) -> &mut ClipboardHistory {
        &mut self.history
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

    #[test]
    fn test_clipboard_history() {
        let mut mgr = RegisterMgr::new();
        mgr.set(None, RegisterContent::CharacterWise("first".to_string()));
        mgr.set(
            Some('a'),
            RegisterContent::CharacterWise("second".to_string()),
        );
        // 同一内容の連続は重複排除される
        mgr.set(None, RegisterContent::CharacterWise("second".to_string()));
        mgr.set(None, RegisterContent::CharacterWise("third".to_string()));

        let history = mgr.history();
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().content.as_str(), "third");
        assert_eq!(history.get(1).unwrap().content.as_str(), "second");
        assert_eq!(history.get(2).unwrap().content.as_str(), "first");
    }
}
