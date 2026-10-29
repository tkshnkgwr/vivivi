//! テキストオブジェクト（単語、引用符、括弧等）の範囲計算モジュール。

use super::ViEngine;
use crate::command::{TextObject, TextObjectKind, TextObjectModifier};
use vivivi_core::{TextBuffer, TextCursor};

impl ViEngine {
    /// テキストオブジェクトの適用範囲 `(from, to)` （絶対文字インデックス）を計算します。
    pub(crate) fn find_text_object_range(
        &self,
        buffer: &TextBuffer,
        cursor: &TextCursor,
        obj: &TextObject,
    ) -> Option<(usize, usize)> {
        match obj.kind {
            TextObjectKind::Word => {
                self.find_word_object_range(buffer, cursor, obj.modifier, false)
            }
            TextObjectKind::BigWord => {
                self.find_word_object_range(buffer, cursor, obj.modifier, true)
            }
            TextObjectKind::DoubleQuote => {
                self.find_quote_object_range(buffer, cursor, '"', obj.modifier)
            }
            TextObjectKind::SingleQuote => {
                self.find_quote_object_range(buffer, cursor, '\'', obj.modifier)
            }
            TextObjectKind::BackQuote => {
                self.find_quote_object_range(buffer, cursor, '`', obj.modifier)
            }
            TextObjectKind::Parentheses => {
                self.find_bracket_object_range(buffer, cursor, '(', ')', obj.modifier)
            }
            TextObjectKind::Braces => {
                self.find_bracket_object_range(buffer, cursor, '{', '}', obj.modifier)
            }
            TextObjectKind::Bracket => {
                self.find_bracket_object_range(buffer, cursor, '[', ']', obj.modifier)
            }
            TextObjectKind::AngleBracket => {
                self.find_bracket_object_range(buffer, cursor, '<', '>', obj.modifier)
            }
        }
    }

    /// 単語オブジェクト (`iw`, `aw`, `iW`, `aW`) の範囲を計算します。
    fn find_word_object_range(
        &self,
        buffer: &TextBuffer,
        cursor: &TextCursor,
        modifier: TextObjectModifier,
        is_big: bool,
    ) -> Option<(usize, usize)> {
        let line_text = buffer.line_text(cursor.pos.line)?;
        let chars: Vec<char> = line_text.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let col = cursor.pos.col.min(chars.len().saturating_sub(1));
        let ch = chars[col];

        let line_start = buffer.line_start_position(cursor.pos.line);

        // カーソル文字のクラス（単語、記号、空白）
        let target_class = if is_big {
            if ch.is_whitespace() { 0 } else { 1 }
        } else if Self::is_word_char(ch) {
            1
        } else if ch.is_whitespace() {
            0
        } else {
            2
        };

        // 単語の左端を探索
        let mut start = col;
        while start > 0 {
            let prev_ch = chars[start - 1];
            let prev_class = if is_big {
                if prev_ch.is_whitespace() { 0 } else { 1 }
            } else if Self::is_word_char(prev_ch) {
                1
            } else if prev_ch.is_whitespace() {
                0
            } else {
                2
            };
            if prev_class != target_class {
                break;
            }
            start -= 1;
        }

        // 単語の右端（exclusive）を探索
        let mut end = col;
        while end < chars.len() {
            let next_ch = chars[end];
            let next_class = if is_big {
                if next_ch.is_whitespace() { 0 } else { 1 }
            } else if Self::is_word_char(next_ch) {
                1
            } else if next_ch.is_whitespace() {
                0
            } else {
                2
            };
            if next_class != target_class {
                break;
            }
            end += 1;
        }

        if modifier == TextObjectModifier::Inner {
            return Some((line_start + start, line_start + end));
        }

        // around ('aw' / 'aW'): 単語の後の空白を含める。後ろに空白が無ければ前の空白を含める。
        let mut a_end = end;
        while a_end < chars.len() && chars[a_end].is_whitespace() {
            a_end += 1;
        }

        let mut a_start = start;
        if a_end == end {
            // 後ろに空白がなかった場合、前の空白を含める
            while a_start > 0 && chars[a_start - 1].is_whitespace() {
                a_start -= 1;
            }
        }

        Some((line_start + a_start, line_start + a_end))
    }

    /// 引用符オブジェクト (`i"`, `a"`, `i'`, `a'`, `i```, `a```) の範囲を計算します。
    fn find_quote_object_range(
        &self,
        buffer: &TextBuffer,
        cursor: &TextCursor,
        quote: char,
        modifier: TextObjectModifier,
    ) -> Option<(usize, usize)> {
        let line_text = buffer.line_text(cursor.pos.line)?;
        let chars: Vec<char> = line_text.chars().collect();
        let col = cursor.pos.col;

        // 行内の全 quote 位置を収集（エスケープされた quote は除外）
        let mut quote_indices = Vec::new();
        let mut escaped = false;
        for (i, &c) in chars.iter().enumerate() {
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }
            if c == quote {
                quote_indices.push(i);
            }
        }

        // 偶数個のペア (q0, q1), (q2, q3)... のみ有効な引用符ペアとして抽出
        let mut valid_pairs = Vec::new();
        for chunk in quote_indices.chunks_exact(2) {
            valid_pairs.push((chunk[0], chunk[1]));
        }

        if valid_pairs.is_empty() {
            return None;
        }

        // 1. カーソル位置を内包するペアを探す
        let mut target_pair = None;
        for &(q1, q2) in &valid_pairs {
            if col >= q1 && col <= q2 {
                target_pair = Some((q1, q2));
                break;
            }
        }

        // 2. カーソルより右にある最初のペアを探す
        if target_pair.is_none() {
            for &(q1, q2) in &valid_pairs {
                if q1 > col {
                    target_pair = Some((q1, q2));
                    break;
                }
            }
        }

        let (q_start, q_end) = target_pair?;
        let line_start = buffer.line_start_position(cursor.pos.line);

        if modifier == TextObjectModifier::Inner {
            if q_start < q_end {
                Some((line_start + q_start + 1, line_start + q_end))
            } else {
                Some((line_start + q_start + 1, line_start + q_start + 1))
            }
        } else {
            // around: 引用符記号を含める
            let mut a_end = q_end + 1;
            // クォート後ろの空白も含める（Vim仕様）
            if a_end < chars.len() && chars[a_end].is_whitespace() {
                a_end += 1;
            }
            Some((line_start + q_start, line_start + a_end))
        }
    }

    /// 括弧オブジェクト (`i(`, `a(`, `i{`, `a{`, `i[`, `a[`, `i<`, `a<`) の範囲を計算します。
    fn find_bracket_object_range(
        &self,
        buffer: &TextBuffer,
        cursor: &TextCursor,
        open_ch: char,
        close_ch: char,
        modifier: TextObjectModifier,
    ) -> Option<(usize, usize)> {
        let cur_pos = buffer.cursor_to_position(cursor.pos);
        let total_len = buffer.len();

        // 1. カーソル位置から前方（先頭方向）に向かって対応する開き括弧を探す
        let mut depth = 0isize;
        let mut open_idx = None;

        let mut p = cur_pos;
        loop {
            if let Some(ch) = buffer.char_at(p) {
                if ch == close_ch && p != cur_pos {
                    depth += 1;
                } else if ch == open_ch {
                    if depth == 0 {
                        open_idx = Some(p);
                        break;
                    } else {
                        depth -= 1;
                    }
                }
            }
            if p == 0 {
                break;
            }
            p -= 1;
        }

        // カーソルより右にある開き括弧も探す（カーソルが括弧の手前にある場合）
        if open_idx.is_none() {
            let mut p = cur_pos;
            while p < total_len {
                if buffer.char_at(p) == Some(open_ch) {
                    open_idx = Some(p);
                    break;
                }
                p += 1;
            }
        }

        let open_pos = open_idx?;

        // 2. 開き括弧から後方に向かって対応する閉じ括弧を探す
        let mut depth = 0isize;
        let mut close_idx = None;
        let mut p = open_pos + 1;
        while p < total_len {
            if let Some(ch) = buffer.char_at(p) {
                if ch == open_ch {
                    depth += 1;
                } else if ch == close_ch {
                    if depth == 0 {
                        close_idx = Some(p);
                        break;
                    } else {
                        depth -= 1;
                    }
                }
            }
            p += 1;
        }

        let close_pos = close_idx?;

        if modifier == TextObjectModifier::Inner {
            Some((open_pos + 1, close_pos))
        } else {
            Some((open_pos, close_pos + 1))
        }
    }
}
