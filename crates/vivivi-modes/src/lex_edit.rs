//! 字句編集予測 (LexEdit) モジュール。
//!
//! ViVi 4 の `LexEdit`（`Ctrl-j`）に準拠し、
//! 単語・字句単位の編集・置換履歴から次の編集パターンを予測し、変換候補を提案します。

use std::collections::VecDeque;

/// 単語・識別子の編集履歴レコード。
///
/// 変更前の単語文字列と変更後の単語文字列のペアを保持します。
///
/// # Examples
/// ```
/// use vivivi_modes::lex_edit::WordEditRecord;
///
/// let record = WordEditRecord {
///     before: "old_val".to_string(),
///     after: "new_val".to_string(),
/// };
/// assert_eq!(record.before, "old_val");
/// assert_eq!(record.after, "new_val");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct WordEditRecord {
    /// 変更前の単語文字列
    pub before: String,
    /// 変更後の単語文字列
    pub after: String,
}

/// 字句編集予測エンジン。
///
/// 直近の単語置換履歴（最大50件）をリングバッファとして保持し、
/// 現在カーソル下の単語に対して完全一致または部分置換（接頭辞/接尾辞等の推測）による変換候補を生成します。
///
/// # Examples
/// ```
/// use vivivi_modes::lex_edit::LexEditEngine;
///
/// let mut engine = LexEditEngine::new();
/// engine.record_edit("foo_id", "bar_id");
/// let candidates = engine.predict("foo_id");
/// assert_eq!(candidates, vec!["bar_id"]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct LexEditEngine {
    /// 編集履歴（最新50件）
    history: VecDeque<WordEditRecord>,
}

impl LexEditEngine {
    /// 空の履歴を持つ字句編集予測エンジンを生成します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::lex_edit::LexEditEngine;
    ///
    /// let engine = LexEditEngine::new();
    /// assert!(engine.predict("any_word").is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(50),
        }
    }

    /// 単語の変更・置換実績を履歴に記録します。
    ///
    /// 空文字列や変更前後の文字列が同一である場合は無視されます。
    /// また、直前の履歴と全く同一の変更である場合も重複登録はスキップされます。
    /// 履歴が50件を超えた場合は最も古い履歴から破棄されます。
    ///
    /// # 引数
    /// - `before`: 変更前の単語
    /// - `after`: 変更後の単語
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::lex_edit::LexEditEngine;
    ///
    /// let mut engine = LexEditEngine::new();
    /// engine.record_edit("apple", "orange");
    /// ```
    pub fn record_edit(&mut self, before: &str, after: &str) {
        let b = before.trim();
        let a = after.trim();
        if b.is_empty() || a.is_empty() || b == a {
            return;
        }

        // 直前の同一履歴があればスキップ
        if let Some(front) = self.history.front()
            && front.before == b
            && front.after == a
        {
            return;
        }

        if self.history.len() >= 50 {
            self.history.pop_back();
        }

        self.history.push_front(WordEditRecord {
            before: b.to_string(),
            after: a.to_string(),
        });
    }

    /// 現在の単語（カーソル下の単語）に対する編集・置換候補一覧を予測します。
    ///
    /// 過去の履歴を走査し、以下の2つのルールで候補を抽出します：
    /// 1. 完全一致: 過去に `current_word` そのものを置換した履歴がある場合
    /// 2. 部分一致: 過去の置換パターン（例: `"foo_id"` -> `"bar_id"`）に基づき、
    ///    `"foo_name"` に対する `"bar_name"` などを自動推測
    ///
    /// 重複する候補は排除されて返されます。
    ///
    /// # 引数
    /// - `current_word`: 予測対象の現在の単語文字列
    ///
    /// # 戻り値
    /// 提案される変換先候補文字列のベクタ（見つからない場合は空ベクタ）
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::lex_edit::LexEditEngine;
    ///
    /// let mut engine = LexEditEngine::new();
    /// engine.record_edit("user_id", "member_id");
    ///
    /// // 完全一致
    /// assert_eq!(engine.predict("user_id"), vec!["member_id"]);
    /// // パターン推測
    /// assert_eq!(engine.predict("user_id_count"), vec!["member_id_count"]);
    /// ```
    pub fn predict(&self, current_word: &str) -> Vec<String> {
        let word = current_word.trim();
        if word.is_empty() {
            return Vec::new();
        }

        let mut candidates = Vec::new();

        for record in &self.history {
            // 1. 完全一致: 過去に word を after に変更した実績がある
            if record.before == word {
                if !candidates.contains(&record.after) {
                    candidates.push(record.after.clone());
                }
                continue;
            }

            // 2. 部分一致・接頭辞/接尾辞の置換推測
            // 例: "foo_id" -> "bar_id" の履歴がある状態で "foo_name" が来たら "bar_name" を提案
            if word.contains(&record.before) {
                let replaced = word.replace(&record.before, &record.after);
                if !candidates.contains(&replaced) {
                    candidates.push(replaced);
                }
            }
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_prediction() {
        let mut engine = LexEditEngine::new();
        engine.record_edit("old_fn", "new_fn");

        let candidates = engine.predict("old_fn");
        assert_eq!(candidates, vec!["new_fn"]);
    }

    #[test]
    fn test_prefix_analogy_prediction() {
        let mut engine = LexEditEngine::new();
        // user_id -> member_id
        engine.record_edit("user", "member");

        let candidates = engine.predict("user_name");
        assert_eq!(candidates, vec!["member_name"]);
    }
}
