//! ダイナミックマクロ (Dynamic Macro / Masui Repeat) モジュール。
//!
//! 増井俊之氏が提唱し、ViVi 3 / ViVi 4 に搭載された動的マクロ機能。
//! キー入力や編集アクションの履歴から直前の繰り返し規則（XX パターン）を自動推測し、
//! 明示的なマクロ記録なしに反復編集を自動実行します。

/// ダイナミックマクロエンジン。
///
/// キーストローク履歴から反復操作（増井式ダイナミックマクロ）を自動検出し、
/// 繰り返し実行を支援します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::dynamic_macro::DynamicMacroEngine;
///
/// let mut engine = DynamicMacroEngine::new();
/// for ch in "abcabc".chars() {
///     engine.record_char(ch);
/// }
/// assert_eq!(engine.detect_repeat(), Some(vec!['a', 'b', 'c']));
/// ```
#[derive(Debug, Clone, Default)]
pub struct DynamicMacroEngine {
    /// 編集アクション・キー操作の履歴ログ（最大 1000 件）
    history: Vec<char>,
    /// 直前に検出・実行した繰り返しシーケンス（再リピート用）
    last_detected: Option<Vec<char>>,
}

impl DynamicMacroEngine {
    /// 新規 `DynamicMacroEngine` を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::dynamic_macro::DynamicMacroEngine;
    ///
    /// let mut engine = DynamicMacroEngine::new();
    /// assert_eq!(engine.detect_repeat(), None);
    /// ```
    pub fn new() -> Self {
        Self {
            history: Vec::with_capacity(1024),
            last_detected: None,
        }
    }

    /// キー文字入力を履歴に記録します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::dynamic_macro::DynamicMacroEngine;
    ///
    /// let mut engine = DynamicMacroEngine::new();
    /// engine.record_char('x');
    /// ```
    pub fn record_char(&mut self, ch: char) {
        if self.history.len() >= 1000 {
            self.history.remove(0);
        }
        self.history.push(ch);
    }

    /// 直前に記録されたキーシーケンスから、反復規則（XX パターン）を検出します。
    ///
    /// 履歴の末尾から長さ L のパターンを探し、直前とその前のシーケンスが一致する最長パターンを返します。
    /// 新規パターンが見つからない場合は直前の検出パターンを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::dynamic_macro::DynamicMacroEngine;
    ///
    /// let mut engine = DynamicMacroEngine::new();
    /// for ch in "xyxy".chars() {
    ///     engine.record_char(ch);
    /// }
    /// assert_eq!(engine.detect_repeat(), Some(vec!['x', 'y']));
    /// ```
    pub fn detect_repeat(&mut self) -> Option<Vec<char>> {
        let n = self.history.len();
        if n < 2 {
            return self.last_detected.clone();
        }

        let max_l = n / 2;
        // 最長一致を優先するため、長い L から降順に探索
        for l in (1..=max_l).rev() {
            let chunk_a = &self.history[n - l..n];
            let chunk_b = &self.history[n - 2 * l..n - l];

            if chunk_a == chunk_b {
                let pattern = chunk_a.to_vec();
                self.last_detected = Some(pattern.clone());
                return Some(pattern);
            }
        }

        // 新たなパターンが見つからない場合、直前のパターンがあれば再利用
        self.last_detected.clone()
    }

    /// 履歴および直前の検出パターンをクリアします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::dynamic_macro::DynamicMacroEngine;
    ///
    /// let mut engine = DynamicMacroEngine::new();
    /// engine.record_char('a');
    /// engine.clear();
    /// assert_eq!(engine.detect_repeat(), None);
    /// ```
    pub fn clear(&mut self) {
        self.history.clear();
        self.last_detected = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_macro_simple_repeat() {
        let mut dmacro = DynamicMacroEngine::new();
        // ユーザーが "abcabc" と入力
        for ch in "abcabc".chars() {
            dmacro.record_char(ch);
        }

        // "abc" が直前の繰り返しとして検出される
        let repeat = dmacro.detect_repeat();
        assert_eq!(repeat, Some(vec!['a', 'b', 'c']));
    }

    #[test]
    fn test_dynamic_macro_longest_match() {
        let mut dmacro = DynamicMacroEngine::new();
        // "foobarfoobar"
        for ch in "foobarfoobar".chars() {
            dmacro.record_char(ch);
        }

        let repeat = dmacro.detect_repeat();
        assert_eq!(repeat, Some("foobar".chars().collect()));
    }

    #[test]
    fn test_dynamic_macro_single_char() {
        let mut dmacro = DynamicMacroEngine::new();
        dmacro.record_char('x');
        dmacro.record_char('x');

        let repeat = dmacro.detect_repeat();
        assert_eq!(repeat, Some(vec!['x']));
    }
}
