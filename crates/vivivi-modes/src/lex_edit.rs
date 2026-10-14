//! 字句編集予測 (LexEdit) モジュール。
//!
//! ViVi 4 の `LexEdit`（`Ctrl-j`）に準拠し、
//! 単語・字句単位の編集・置換履歴から次の編集パターンを予測し、変換候補を提案します。

use std::collections::VecDeque;

/// 単語編集の履歴レコード
#[derive(Debug, Clone, PartialEq)]
pub struct WordEditRecord {
    pub before: String,
    pub after: String,
}

/// 字句編集予測エンジン
#[derive(Debug, Clone, Default)]
pub struct LexEditEngine {
    /// 編集履歴（最新50件）
    history: VecDeque<WordEditRecord>,
}

impl LexEditEngine {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(50),
        }
    }

    /// 単語の変更・置換実績を記録します。
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

    /// 現在の単語（カーソル下の単語）に対する編集・置換候補を予測します。
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
