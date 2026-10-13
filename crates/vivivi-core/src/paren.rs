//! 対応括弧（Matching Parentheses）探索モジュール。
//!
//! OpenViVi の `assocParen.cpp` に基づき、対応する括弧 `()`, `{}`, `[]`, `<>` の位置を
//! 文字列リテラルやコメントを自動スキップしながら高速に探索します。

use crate::buffer::TextBuffer;

/// 探索対象の括弧種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParenKind {
    Round,  // ( )
    Curly,  // { }
    Square, // [ ]
    Angle,  // < >
}

/// 指定文字が開き括弧か閉じ括弧かを判定し、その相方と探索方向を返します。
pub fn paren_pair(ch: char) -> Option<(char, bool)> {
    match ch {
        '(' => Some((')', true)),
        '{' => Some(('}', true)),
        '[' => Some((']', true)),
        '<' => Some(('>', true)),
        ')' => Some(('(', false)),
        '}' => Some(('{', false)),
        ']' => Some(('[', false)),
        '>' => Some(('<', false)),
        _ => None,
    }
}

/// バッファ内の指定文字オフセット `pos` にある括弧に対して、
/// 対になる対応括弧の文字オフセットを探索して返します。
pub fn find_matching_paren(buffer: &TextBuffer, pos: usize) -> Option<usize> {
    let target_ch = buffer.char_at(pos)?;
    let (match_ch, forward) = paren_pair(target_ch)?;
    let total_len = buffer.len();

    if forward {
        let mut level = 1usize;
        let mut idx = pos + 1;
        let mut in_str = None; // Some('"') or Some('\'')

        while idx < total_len {
            let ch = buffer.char_at(idx)?;

            // 文字列リテラルの追跡
            if let Some(quote) = in_str {
                if ch == '\\' {
                    idx += 2; // エスケープスキップ
                    continue;
                }
                if ch == quote {
                    in_str = None;
                }
                idx += 1;
                continue;
            } else if ch == '"' || ch == '\'' {
                in_str = Some(ch);
                idx += 1;
                continue;
            }

            // 行コメント // のスキップ
            if ch == '/' && buffer.char_at(idx + 1) == Some('/') {
                while idx < total_len && buffer.char_at(idx) != Some('\n') {
                    idx += 1;
                }
                continue;
            }

            // ブロックコメント /* ... */ のスキップ
            if ch == '/' && buffer.char_at(idx + 1) == Some('*') {
                idx += 2;
                while idx + 1 < total_len {
                    if buffer.char_at(idx) == Some('*') && buffer.char_at(idx + 1) == Some('/') {
                        idx += 2;
                        break;
                    }
                    idx += 1;
                }
                continue;
            }

            if ch == target_ch {
                level += 1;
            } else if ch == match_ch {
                level -= 1;
                if level == 0 {
                    return Some(idx);
                }
            }

            idx += 1;
        }
    } else {
        // 後方探索（閉じ括弧から対応する開き括弧を探索）
        // 先頭から pos までの文字列・コメントをスキップしながらスタックで開き括弧を追跡
        let mut stack = Vec::new();
        let mut idx = 0;
        let mut in_str = None;

        while idx <= pos {
            let ch = buffer.char_at(idx)?;

            // 文字列リテラルの追跡
            if let Some(quote) = in_str {
                if ch == '\\' {
                    idx += 2;
                    continue;
                }
                if ch == quote {
                    in_str = None;
                }
                idx += 1;
                continue;
            } else if ch == '"' || ch == '\'' {
                in_str = Some(ch);
                idx += 1;
                continue;
            }

            // 行コメント // のスキップ
            if ch == '/' && buffer.char_at(idx + 1) == Some('/') {
                while idx <= pos && buffer.char_at(idx) != Some('\n') {
                    idx += 1;
                }
                continue;
            }

            // ブロックコメント /* ... */ のスキップ
            if ch == '/' && buffer.char_at(idx + 1) == Some('*') {
                idx += 2;
                while idx < pos {
                    if buffer.char_at(idx) == Some('*') && buffer.char_at(idx + 1) == Some('/') {
                        idx += 2;
                        break;
                    }
                    idx += 1;
                }
                continue;
            }

            if ch == match_ch {
                stack.push(idx);
            } else if ch == target_ch {
                if idx == pos {
                    return stack.pop();
                } else {
                    stack.pop();
                }
            }

            idx += 1;
        }
    }

    None
}

/// カーソル位置（またはその直前）にある括弧とその対応括弧のペア `(pos, matching_pos)` を取得します。
pub fn get_cursor_matching_paren(buffer: &TextBuffer, cursor_pos: usize) -> Option<(usize, usize)> {
    let check_pos = |pos: usize| -> Option<(usize, usize)> {
        let ch = buffer.char_at(pos)?;
        let _ = paren_pair(ch)?;
        let matched = find_matching_paren(buffer, pos)?;
        Some((pos, matched))
    };

    // 1. カーソル直下の文字、または 2. カーソル直前の文字を検査
    check_pos(cursor_pos).or_else(|| {
        if cursor_pos > 0 {
            check_pos(cursor_pos - 1)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_paren_simple() {
        let buffer = TextBuffer::from_text("fn test() { let x = (1 + 2); }");

        // '(' at 7 -> ')' at 8
        assert_eq!(find_matching_paren(&buffer, 7), Some(8));
        assert_eq!(find_matching_paren(&buffer, 8), Some(7));

        // '{' at 10 -> '}' at 29
        assert_eq!(find_matching_paren(&buffer, 10), Some(29));
        assert_eq!(find_matching_paren(&buffer, 29), Some(10));

        // '(' at 20 -> ')' at 26
        assert_eq!(find_matching_paren(&buffer, 20), Some(26));
    }

    #[test]
    fn test_matching_paren_with_string_and_comments() {
        let buffer =
            TextBuffer::from_text("let a = ( \"quoted ( paren )\" + /* comment ( */ 10 );");
        // '(' at 8 -> ')' at 50
        assert_eq!(find_matching_paren(&buffer, 8), Some(50));
        assert_eq!(find_matching_paren(&buffer, 50), Some(8));
    }
}
