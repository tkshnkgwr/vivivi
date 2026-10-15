//! レジスタ管理モジュール（ヤンク・削除・貼り付け）。
//!
//! Viの無名レジスタ（Unnamed register `""`）、名前付きレジスタ (`"a`〜`"z`)、
//! クリップボード連携レジスタ (`"+`, `"*`)、および行指向/文字指向の属性を管理します。

use std::collections::{HashMap, VecDeque};

/// レジスタに格納されるテキストの形式・指向性。
///
/// 文字単位（インライン）、行単位（行末改行付き）、矩形ブロック選択の3形式をサポートします。
///
/// # Examples
///
/// ```
/// use vivivi_vi::RegisterContent;
///
/// let c = RegisterContent::LineWise("hello\n".to_string());
/// assert!(c.is_line_wise());
/// assert_eq!(c.as_str(), "hello\n");
/// ```
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
    /// 格納されているテキストの文字列スライスを取得します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::RegisterContent;
    ///
    /// let content = RegisterContent::CharacterWise("text".to_string());
    /// assert_eq!(content.as_str(), "text");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Self::CharacterWise(s) | Self::LineWise(s) => s.as_str(),
            Self::BlockWise(lines) => lines.first().map(|s| s.as_str()).unwrap_or(""),
        }
    }

    /// 行指向（LineWise）テキストであるかを判定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::RegisterContent;
    ///
    /// assert!(RegisterContent::LineWise("line\n".to_string()).is_line_wise());
    /// assert!(!RegisterContent::CharacterWise("word".to_string()).is_line_wise());
    /// ```
    pub fn is_line_wise(&self) -> bool {
        matches!(self, Self::LineWise(_))
    }

    /// UI表示・プレビュー用の 1 行要約文字列を生成します。
    ///
    /// 改行の手前までの文字列を抽出し、`max_len` を超える場合は末尾に `...` を付与します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::RegisterContent;
    ///
    /// let content = RegisterContent::LineWise("A very long line text for testing".to_string());
    /// assert_eq!(content.preview_text(10), "A very lon...");
    /// ```
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

    /// 格納されているテキストの総行数を取得します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::RegisterContent;
    ///
    /// let content = RegisterContent::LineWise("Line1\nLine2\n".to_string());
    /// assert_eq!(content.line_count(), 2);
    /// ```
    pub fn line_count(&self) -> usize {
        match self {
            Self::CharacterWise(s) | Self::LineWise(s) => s.lines().count().max(1),
            Self::BlockWise(lines) => lines.len().max(1),
        }
    }
}

/// クリップボード履歴の1アイテムを表す構造体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardItem {
    /// 一意な連番ID
    pub id: usize,
    /// 格納されたレジスタコンテンツ
    pub content: RegisterContent,
    /// 1行プレビュー要約文字列
    pub preview: String,
    /// 行数
    pub line_count: usize,
}

/// ヤンク・削除されたテキスト履歴を管理するクリップボード履歴マネージャ。
///
/// # Examples
///
/// ```
/// use vivivi_vi::{ClipboardHistory, RegisterContent};
///
/// let mut history = ClipboardHistory::new(10);
/// history.push(RegisterContent::CharacterWise("item 1".to_string()));
/// assert_eq!(history.len(), 1);
/// ```
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
    /// 指定最大件数で空の [`ClipboardHistory`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ClipboardHistory;
    ///
    /// let history = ClipboardHistory::new(20);
    /// assert_eq!(history.max_items, 20);
    /// assert!(history.is_empty());
    /// ```
    pub fn new(max_items: usize) -> Self {
        Self {
            max_items,
            items: VecDeque::new(),
            next_id: 1,
        }
    }

    /// 新しい履歴アイテムを追加します。
    ///
    /// 空文字列や直前と完全に同一の内容は自動的に重複排除されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{ClipboardHistory, RegisterContent};
    ///
    /// let mut history = ClipboardHistory::new(10);
    /// history.push(RegisterContent::CharacterWise("copied".to_string()));
    /// history.push(RegisterContent::CharacterWise("copied".to_string())); // 重複スキップ
    /// assert_eq!(history.len(), 1);
    /// ```
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

    /// 履歴アイテム一覧の参照を取得します（新しい順）。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ClipboardHistory;
    ///
    /// let history = ClipboardHistory::new(5);
    /// assert_eq!(history.items().len(), 0);
    /// ```
    pub fn items(&self) -> &VecDeque<ClipboardItem> {
        &self.items
    }

    /// インデックス指定で履歴アイテムを取得します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。インデックスが範囲外の場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{ClipboardHistory, RegisterContent};
    ///
    /// let mut history = ClipboardHistory::new(5);
    /// history.push(RegisterContent::CharacterWise("first".to_string()));
    /// assert!(history.get(0).is_some());
    /// assert!(history.get(10).is_none());
    /// ```
    pub fn get(&self, index: usize) -> Option<&ClipboardItem> {
        self.items.get(index)
    }

    /// 保持している履歴件数を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ClipboardHistory;
    ///
    /// let history = ClipboardHistory::new(5);
    /// assert_eq!(history.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 履歴が空かどうかを判定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ClipboardHistory;
    ///
    /// let history = ClipboardHistory::new(5);
    /// assert!(history.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 保持している履歴を全消去します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{ClipboardHistory, RegisterContent};
    ///
    /// let mut history = ClipboardHistory::new(5);
    /// history.push(RegisterContent::CharacterWise("temp".to_string()));
    /// history.clear();
    /// assert!(history.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// Vi の全レジスタ（無名、名前付き、大文字アペンド）およびクリップボード履歴を統括するマネージャ。
///
/// # Examples
///
/// ```
/// use vivivi_vi::{RegisterContent, RegisterMgr};
///
/// let mut mgr = RegisterMgr::new();
/// mgr.set(Some('a'), RegisterContent::CharacterWise("yanked".to_string()));
/// assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "yanked");
/// assert_eq!(mgr.get_unnamed().unwrap().as_str(), "yanked");
/// ```
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
    /// 新しい空の [`RegisterMgr`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::RegisterMgr;
    ///
    /// let mgr = RegisterMgr::new();
    /// assert!(mgr.get_unnamed().is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 指定されたレジスタにテキスト内容を格納します。
    ///
    /// - レジスタ名が `None` の場合は無名レジスタに設定します。
    /// - 小文字レジスタ（`'a'`〜`'z'`）は上書き保存されます。
    /// - 大文字レジスタ（`'A'`〜`'Z'`）は対応する小文字レジスタの既存内容へ末尾追記（アペンド）されます。
    /// - 格納された内容は、無名レジスタおよびクリップボード履歴にも自動同期されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{RegisterContent, RegisterMgr};
    ///
    /// let mut mgr = RegisterMgr::new();
    /// mgr.set(Some('a'), RegisterContent::CharacterWise("Hello".to_string()));
    /// mgr.set(Some('A'), RegisterContent::CharacterWise(" World".to_string()));
    /// assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "Hello World");
    /// ```
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

    /// クリップボード履歴の不変参照を取得します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{RegisterContent, RegisterMgr};
    ///
    /// let mut mgr = RegisterMgr::new();
    /// mgr.set(None, RegisterContent::CharacterWise("copy".to_string()));
    /// assert_eq!(mgr.history().len(), 1);
    /// ```
    pub fn history(&self) -> &ClipboardHistory {
        &self.history
    }

    /// クリップボード履歴の可変参照を取得します。
    pub fn history_mut(&mut self) -> &mut ClipboardHistory {
        &mut self.history
    }

    /// 指定されたレジスタの内容を取得します。
    ///
    /// `reg_name` が `None` の場合は無名レジスタの内容を返します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。未登録または存在しない場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{RegisterContent, RegisterMgr};
    ///
    /// let mut mgr = RegisterMgr::new();
    /// mgr.set(Some('b'), RegisterContent::CharacterWise("data".to_string()));
    /// assert_eq!(mgr.get(Some('b')).unwrap().as_str(), "data");
    /// assert!(mgr.get(Some('z')).is_some()); // 未登録時は無名レジスタにフォールバック
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{RegisterContent, RegisterMgr};
    ///
    /// let mut mgr = RegisterMgr::new();
    /// mgr.set_unnamed(RegisterContent::CharacterWise("unnamed".to_string()));
    /// assert_eq!(mgr.get_unnamed().unwrap().as_str(), "unnamed");
    /// ```
    pub fn get_unnamed(&self) -> Option<&RegisterContent> {
        self.unnamed.as_ref()
    }

    /// 無名レジスタに内容を設定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{RegisterContent, RegisterMgr};
    ///
    /// let mut mgr = RegisterMgr::new();
    /// mgr.set_unnamed(RegisterContent::CharacterWise("test".to_string()));
    /// assert!(mgr.get_unnamed().is_some());
    /// ```
    pub fn set_unnamed(&mut self, content: RegisterContent) {
        self.unnamed = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_register_storage_and_limit() {
        let mut mgr = RegisterMgr::new();
        // 60 回異なる内容をセットして内部履歴上限 (50) を直接検証
        for i in 0..60 {
            mgr.set(None, RegisterContent::CharacterWise(format!("item_{i}")));
        }
        assert_eq!(mgr.history.items.len(), 50);
        assert_eq!(mgr.history.items[0].content.as_str(), "item_59");
    }
}
