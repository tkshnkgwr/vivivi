//! 動的単語補完 (Dynamic Abbreviation / dabbrev) モジュール。
//!
//! ViVi 3 / ViVi 4 の「動的補完 (Ctrl-L / Ctrl-M)」に準拠し、
//! バッファ内をカーソル直前から前方／後方に高速探索して、
//! 一致する単語候補をサイクリックに提示・補完します。

use vivivi_core::TextBuffer;

/// 動的補完セッションの状態。
#[derive(Debug, Clone, Default)]
pub struct DabbrevSession {
    /// 補完開始時に入力されていたプレフィックス（基底文字列）
    pub prefix: String,
    /// 収集された補完候補リスト（カーソルに近い順）
    pub candidates: Vec<String>,
    /// 現在適用中の候補インデックス
    pub current_index: Option<usize>,
    /// 直前に補完挿入された文字列の文字数
    pub last_inserted_len: usize,
}

impl DabbrevSession {
    /// 新規セッションを生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// セッションをリセットします。
    pub fn reset(&mut self) {
        self.prefix.clear();
        self.candidates.clear();
        self.current_index = None;
        self.last_inserted_len = 0;
    }

    /// セッションが現在アクティブ（候補巡回中）かどうか。
    pub fn is_active(&self) -> bool {
        self.current_index.is_some() && !self.candidates.is_empty()
    }

    /// カーソル位置のプレフィックスから補完候補を探索し、セッションを開始します。
    pub fn start(
        &mut self,
        buffer: &TextBuffer,
        cursor_line: usize,
        cursor_col: usize,
    ) -> Option<&str> {
        self.reset();

        // 1. カーソル直前の単語プレフィックスを抽出
        let prefix = extract_word_prefix(buffer, cursor_line, cursor_col)?;
        if prefix.is_empty() {
            return None;
        }

        // 2. バッファ内から候補を探索
        let candidates = find_candidates(buffer, cursor_line, cursor_col, &prefix);
        if candidates.is_empty() {
            return None;
        }

        self.prefix = prefix;
        self.candidates = candidates;
        self.current_index = Some(0);
        let first_candidate = &self.candidates[0];
        self.last_inserted_len = first_candidate.chars().count() - self.prefix.chars().count();

        Some(first_candidate.as_str())
    }

    /// 次の候補（より離れた前方/後方）へ進めます (Ctrl-L)。
    pub fn next_candidate(&mut self) -> Option<&str> {
        if self.candidates.is_empty() {
            return None;
        }
        let next_idx = match self.current_index {
            Some(curr) => (curr + 1) % self.candidates.len(),
            None => 0,
        };
        self.current_index = Some(next_idx);
        let cand = &self.candidates[next_idx];
        self.last_inserted_len = cand.chars().count() - self.prefix.chars().count();
        Some(cand.as_str())
    }

    /// 前の候補（直前の候補）へ戻します (Ctrl-M)。
    pub fn prev_candidate(&mut self) -> Option<&str> {
        if self.candidates.is_empty() {
            return None;
        }
        let prev_idx = match self.current_index {
            Some(curr) => {
                if curr == 0 {
                    self.candidates.len() - 1
                } else {
                    curr - 1
                }
            }
            None => 0,
        };
        self.current_index = Some(prev_idx);
        let cand = &self.candidates[prev_idx];
        self.last_inserted_len = cand.chars().count() - self.prefix.chars().count();
        Some(cand.as_str())
    }
}

/// 単語を構成する文字かどうかを判定します（英数字、アンダースコア、非ASCII文字）。
pub fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric()
        || ch == '_'
        || (!ch.is_ascii() && !ch.is_whitespace() && !is_ascii_punctuation(ch))
}

fn is_ascii_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// カーソル直前の単語プレフィックスを抽出します。
pub fn extract_word_prefix(
    buffer: &TextBuffer,
    cursor_line: usize,
    cursor_col: usize,
) -> Option<String> {
    let line_text = buffer.get_line(cursor_line)?;
    let chars: Vec<char> = line_text.chars().collect();
    if cursor_col == 0 || chars.is_empty() {
        return None;
    }

    let end_idx = cursor_col.min(chars.len());
    let mut start_idx = end_idx;

    while start_idx > 0 && is_word_char(chars[start_idx - 1]) {
        start_idx -= 1;
    }

    if start_idx == end_idx {
        return None;
    }

    let prefix: String = chars[start_idx..end_idx].iter().collect();
    Some(prefix)
}

/// バッファ内をスキャンし、指定プレフィックスで始まる単語候補を探索します。
/// カーソル位置から逆方向（手前）、次に順方向（後方）の順にスキャンして優先順位をつけます。
pub fn find_candidates(
    buffer: &TextBuffer,
    cursor_line: usize,
    cursor_col: usize,
    prefix: &str,
) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 自分自身のプレフィックスは候補から除外
    seen.insert(prefix.to_string());

    let line_count = buffer.line_count();

    // 1. カーソル行のカーソルより前の部分を逆順スキャン
    if let Some(line_str) = buffer.get_line(cursor_line) {
        let words = extract_words(&line_str);
        for (start_col, word) in words.into_iter().rev() {
            let end_col = start_col + word.chars().count();
            // カーソルがこの単語の末尾または内部にある場合（現在入力中の単語自身）は除外
            if cursor_col >= start_col && cursor_col <= end_col {
                continue;
            }
            if start_col < cursor_col && word.starts_with(prefix) && seen.insert(word.clone()) {
                candidates.push(word);
            }
        }
    }

    // 2. カーソル行より上の行を逆順スキャン（カーソルに近い上方向）
    for l in (0..cursor_line).rev() {
        if let Some(line_str) = buffer.get_line(l) {
            let words = extract_words(&line_str);
            for (_, word) in words.into_iter().rev() {
                if word.starts_with(prefix) && seen.insert(word.clone()) {
                    candidates.push(word);
                }
            }
        }
    }

    // 3. カーソル行のカーソルより後の部分を順方向スキャン
    if let Some(line_str) = buffer.get_line(cursor_line) {
        let words = extract_words(&line_str);
        for (start_col, word) in words {
            if start_col >= cursor_col && word.starts_with(prefix) && seen.insert(word.clone()) {
                candidates.push(word);
            }
        }
    }

    // 4. カーソル行より下の行を順方向スキャン（下方向）
    for l in (cursor_line + 1)..line_count {
        if let Some(line_str) = buffer.get_line(l) {
            let words = extract_words(&line_str);
            for (_, word) in words {
                if word.starts_with(prefix) && seen.insert(word.clone()) {
                    candidates.push(word);
                }
            }
        }
    }

    candidates
}

/// 行文字列からすべての単語と、その開始文字インデックスを抽出します。
fn extract_words(line: &str) -> Vec<(usize, String)> {
    let mut words = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if is_word_char(chars[i]) {
            let start = i;
            while i < chars.len() && is_word_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            words.push((start, word));
        } else {
            i += 1;
        }
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word_prefix() {
        let buffer = TextBuffer::from_text("let my_variable_name = 100;\nlet my_");
        let prefix = extract_word_prefix(&buffer, 1, 7).expect("should extract prefix");
        assert_eq!(prefix, "my_");
    }

    #[test]
    fn test_find_candidates_priority() {
        let text = "alpha_one\nbeta\nalpha_two\nalpha_target\nalpha_three";
        let buffer = TextBuffer::from_text(text);
        // 行 3: "alpha_target" の直後で "alpha_" 補完
        let candidates = find_candidates(&buffer, 3, 12, "alpha_");
        // カーソル直前の行（alpha_two）が最初に来て、その上が続き、下（alpha_three）が続く
        assert_eq!(candidates, vec!["alpha_two", "alpha_one", "alpha_three"]);
    }

    #[test]
    fn test_dabbrev_session_cycling() {
        let text = "apple_pie\napple_juice\napple_sauce\napp";
        let buffer = TextBuffer::from_text(text);
        let mut session = DabbrevSession::new();

        let first = session.start(&buffer, 3, 3).expect("should start session");
        assert_eq!(first, "apple_sauce");

        let second = session.next_candidate().expect("next candidate");
        assert_eq!(second, "apple_juice");

        let third = session.next_candidate().expect("third candidate");
        assert_eq!(third, "apple_pie");

        // 一周して戻る
        let wrapped = session.next_candidate().expect("wrap to first");
        assert_eq!(wrapped, "apple_sauce");

        // 逆順に戻る
        let prev = session.prev_candidate().expect("prev candidate");
        assert_eq!(prev, "apple_pie");
    }
}
