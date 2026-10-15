//! トランザクション対応および連続入力マージを備えた Undo/Redo マネージャ。
//!
//! OpenViVi の `undoMgr.h` および `undoMgr.cpp` の設計思想を忠実にモデル化し、
//! 複数打鍵のタイピング統合、Vi複合コマンドのアトミックUndoブロック、
//! 保存点（Save Point）と Modified フラグの整合性追跡を提供します。

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
    undo_stack: Vec<UndoBlock>,
    redo_stack: Vec<UndoBlock>,
    current_block: Option<UndoBlock>,
    block_nest_level: usize,
    /// ドキュメント保存時の `undo_stack` の長さ（保存点）
    save_point: Option<usize>,
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

    /// Undo/Redo スタックを永続化文字列（undofile 形式）にシリアライズします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "hello");
    /// let text = mgr.serialize();
    /// assert!(text.starts_with("VIVIVI_UNDO_V1"));
    /// ```
    pub fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str("VIVIVI_UNDO_V1\n");
        match self.save_point {
            Some(pt) => out.push_str(&format!("save_point: {}\n", pt)),
            None => out.push_str("save_point: none\n"),
        }
        out.push_str(&format!("UNDO_STACK: {}\n", self.undo_stack.len()));
        for block in &self.undo_stack {
            Self::serialize_block(&mut out, block);
        }
        out.push_str(&format!("REDO_STACK: {}\n", self.redo_stack.len()));
        for block in &self.redo_stack {
            Self::serialize_block(&mut out, block);
        }
        out
    }

    fn serialize_block(out: &mut String, block: &UndoBlock) {
        out.push_str("BLOCK_START\n");
        for action in &block.actions {
            match action {
                UndoAction::Insert {
                    pos,
                    text,
                    prohibit_merge,
                } => {
                    out.push_str(&format!(
                        "INSERT {} {} \"{}\"\n",
                        pos,
                        if *prohibit_merge { 1 } else { 0 },
                        escape_str(text)
                    ));
                }
                UndoAction::Delete {
                    pos,
                    text,
                    is_backspace,
                } => {
                    out.push_str(&format!(
                        "DELETE {} {} \"{}\"\n",
                        pos,
                        if *is_backspace { 1 } else { 0 },
                        escape_str(text)
                    ));
                }
                UndoAction::Replace {
                    pos,
                    old_text,
                    new_text,
                } => {
                    out.push_str(&format!(
                        "REPLACE {} \"{}\" \"{}\"\n",
                        pos,
                        escape_str(old_text),
                        escape_str(new_text)
                    ));
                }
            }
        }
        out.push_str("BLOCK_END\n");
    }

    /// 永続化文字列から [`UndoMgr`] を復元します。
    ///
    /// # Errors
    ///
    /// ヘッダーが `VIVIVI_UNDO_V1` でない場合や、ブロック構文の解析に失敗した場合にエラー文字列を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut orig = UndoMgr::new();
    /// orig.push_insert(0, "data");
    /// let serialized = orig.serialize();
    ///
    /// let restored = UndoMgr::deserialize(&serialized).unwrap();
    /// assert!(restored.can_undo());
    /// ```
    pub fn deserialize(data: &str) -> Result<Self, String> {
        let lines: Vec<&str> = data.lines().collect();
        if lines.is_empty() || lines[0].trim() != "VIVIVI_UNDO_V1" {
            return Err("不正な undofile ヘッダーです".into());
        }

        let mut mgr = Self::new();
        let mut idx = 1;

        if idx < lines.len() && lines[idx].starts_with("save_point:") {
            let val = lines[idx]["save_point:".len()..].trim();
            if val != "none" {
                mgr.save_point = Some(val.parse::<usize>().map_err(|e| e.to_string())?);
            } else {
                mgr.save_point = None;
            }
            idx += 1;
        }

        let mut reading_target = 0; // 0: undo, 1: redo
        while idx < lines.len() {
            let line = lines[idx].trim();
            idx += 1;
            if line.is_empty() {
                continue;
            }
            if line.starts_with("UNDO_STACK:") {
                reading_target = 0;
            } else if line.starts_with("REDO_STACK:") {
                reading_target = 1;
            } else if line == "BLOCK_START" {
                let mut block = UndoBlock::default();
                while idx < lines.len() {
                    let bline = lines[idx].trim();
                    idx += 1;
                    if bline == "BLOCK_END" {
                        break;
                    }
                    if bline.is_empty() {
                        continue;
                    }
                    let action = parse_action_line(bline)?;
                    block.actions.push(action);
                }
                if reading_target == 0 {
                    mgr.undo_stack.push(block);
                } else {
                    mgr.redo_stack.push(block);
                }
            }
        }
        Ok(mgr)
    }

    /// undofile を指定パスに保存します。
    ///
    /// 保存先ディレクトリが存在しない場合は自動作成されます。
    ///
    /// # Errors
    ///
    /// ファイルの作成や履歴データの書き込みに失敗した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_core::UndoMgr;
    ///
    /// let mgr = UndoMgr::new();
    /// mgr.save_to_file(".test.un~").unwrap();
    /// ```
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let serialized = self.serialize();
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serialized)
    }

    /// undofile を指定パスから読み込み復元します。
    ///
    /// # Errors
    ///
    /// ファイルが存在しない場合、読み込み権限がない場合、またはデータのパースに失敗した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_core::UndoMgr;
    ///
    /// let mgr = UndoMgr::load_from_file(".test.un~").unwrap();
    /// ```
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::deserialize(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape_str(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => return Err("末尾の不正なエスケープシーケンスです".into()),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// クォートで囲まれた文字列を抽出します。
fn extract_quotes(line: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_quote = false;
    let mut escape = false;
    let mut current = String::new();

    for c in line.chars() {
        if escape {
            current.push('\\');
            current.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            if in_quote {
                in_quote = false;
                results.push(current.clone());
                current.clear();
            } else {
                in_quote = true;
                current.clear();
            }
            continue;
        }
        if in_quote {
            current.push(c);
        }
    }
    results
}

fn parse_action_line(line: &str) -> Result<UndoAction, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("空の行です".into());
    }

    let quotes = extract_quotes(line);

    match parts[0] {
        "INSERT" => {
            if parts.len() < 3 || quotes.is_empty() {
                return Err(format!("不正な INSERT 行: {}", line));
            }
            let pos = parts[1].parse::<usize>().map_err(|e| e.to_string())?;
            let prohibit_merge = parts[2] == "1";
            let text = unescape_str(&quotes[0])?;
            Ok(UndoAction::Insert {
                pos,
                text,
                prohibit_merge,
            })
        }
        "DELETE" => {
            if parts.len() < 3 || quotes.is_empty() {
                return Err(format!("不正な DELETE 行: {}", line));
            }
            let pos = parts[1].parse::<usize>().map_err(|e| e.to_string())?;
            let is_backspace = parts[2] == "1";
            let text = unescape_str(&quotes[0])?;
            Ok(UndoAction::Delete {
                pos,
                text,
                is_backspace,
            })
        }
        "REPLACE" => {
            if quotes.len() < 2 {
                return Err(format!("不正な REPLACE 行: {}", line));
            }
            let pos = parts[1].parse::<usize>().map_err(|e| e.to_string())?;
            let old_text = unescape_str(&quotes[0])?;
            let new_text = unescape_str(&quotes[1])?;
            Ok(UndoAction::Replace {
                pos,
                old_text,
                new_text,
            })
        }
        other => Err(format!("未知の Undo アクション: {}", other)),
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

    #[test]
    fn test_undo_serialize_deserialize() {
        let mut undo_mgr = UndoMgr::new();
        undo_mgr.open_block();
        undo_mgr.push_insert(0, "line 1\nline 2");
        undo_mgr.push_delete(5, "\"quoted text\\escaped\"", true);
        undo_mgr.push_replace(10, "old\nfoo", "new\nbar");
        undo_mgr.close_block();
        undo_mgr.on_saved();

        let serialized = undo_mgr.serialize();
        let restored = UndoMgr::deserialize(&serialized).expect("deserialize failed");

        assert_eq!(undo_mgr.undo_stack.len(), restored.undo_stack.len());
        assert_eq!(undo_mgr.save_point, restored.save_point);
        assert_eq!(undo_mgr.undo_stack[0], restored.undo_stack[0]);
    }
}
