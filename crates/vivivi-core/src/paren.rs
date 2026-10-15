//! 対応括弧（Matching Parentheses）探索モジュール。
//!
//! OpenViVi の `assocParen.cpp` に基づき、対応する括弧 `()`, `{}`, `[]`, `<>` の位置を
//! 文字列リテラルやコメントを自動スキップしながら高速に探索します。

use crate::buffer::TextBuffer;

/// 探索対象の括弧種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParenKind {
    /// 丸括弧: `( )`
    Round,
    /// 波括弧: `{ }`
    Curly,
    /// 角括弧: `[ ]`
    Square,
    /// 山括弧: `< >`
    Angle,
}

/// 指定文字が括弧であるかを判定し、対となる相方文字と探索方向を返します。
///
/// 前方探索（開き括弧）の場合は `(相方, true)`、後方探索（閉じ括弧）の場合は `(相方, false)` を返します。
/// 半角括弧だけでなく、日本語全角括弧（`「」`, `『』`, `【】` 等）にも対応しています。
///
/// # Examples
///
/// ```
/// use vivivi_core::paren_pair;
///
/// assert_eq!(paren_pair('('), Some((')', true)));
/// assert_eq!(paren_pair('}'), Some(('{', false)));
/// assert_eq!(paren_pair('「'), Some(('」', true)));
/// assert_eq!(paren_pair('a'), None);
/// ```
pub fn paren_pair(ch: char) -> Option<(char, bool)> {
    match ch {
        // 半角括弧
        '(' => Some((')', true)),
        '{' => Some(('}', true)),
        '[' => Some((']', true)),
        '<' => Some(('>', true)),
        ')' => Some(('(', false)),
        '}' => Some(('{', false)),
        ']' => Some(('[', false)),
        '>' => Some(('<', false)),

        // 全角括弧（OpenViVi / ViVi準拠の日本語編集支援）
        '（' => Some(('）', true)),
        '｛' => Some(('｝', true)),
        '［' => Some(('］', true)),
        '＜' => Some(('＞', true)),
        '「' => Some(('」', true)),
        '『' => Some(('』', true)),
        '【' => Some(('】', true)),
        '〈' => Some(('〉', true)),
        '《' => Some(('》', true)),
        '〔' => Some(('〕', true)),
        '）' => Some(('（', false)),
        '｝' => Some(('｛', false)),
        '］' => Some(('［', false)),
        '＞' => Some(('＜', false)),
        '」' => Some(('「', false)),
        '』' => Some(('『', false)),
        '】' => Some(('【', false)),
        '〉' => Some(('〈', false)),
        '》' => Some(('《', false)),
        '〕' => Some(('〔', false)),

        _ => None,
    }
}

/// バッファ内の指定文字オフセット `pos` にある括弧に対して、
/// 対になる対応括弧の文字オフセットを探索して返します。
///
/// 文字列リテラル（`"..."`, `'...'`）やコメント（`//`, `/* ... */`）内の括弧をスキップし、
/// 正しいネスト深度の相方のみを検出します。
///
/// # Panics
///
/// パニックは発生しません。対応括弧が存在しない場合は `None` を返します。
///
/// # Examples
///
/// ```
/// use vivivi_core::{TextBuffer, find_matching_paren};
///
/// let buf = TextBuffer::from_text("fn test() { let x = 1; }");
/// assert_eq!(find_matching_paren(&buf, 10), Some(23)); // '{' -> '}'
/// assert_eq!(find_matching_paren(&buf, 7), Some(8));   // '(' -> ')'
/// ```
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
                    let _ = stack.pop();
                }
            }

            idx += 1;
        }
    }

    None
}

/// カーソル位置（またはその直前）にある括弧とその対応括弧のペア `(pos, matching_pos)` を取得します。
///
/// エディタのリアルタイム括弧強調表示（カーソル周辺の括弧ペアのハイライト）に使用されます。
///
/// # Examples
///
/// ```
/// use vivivi_core::{TextBuffer, get_cursor_matching_paren};
///
/// let buf = TextBuffer::from_text("(abc)");
/// assert_eq!(get_cursor_matching_paren(&buf, 0), Some((0, 4)));
/// ```
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

/// カレント行のカーソル位置から行末に向かって最初の括弧を探し、
/// その対応括弧の文字オフセットを返します。
///
/// Vim の `%` コマンド標準挙動（カーソル直下に括弧がない場合、行内の後続括弧へ自動ジャンプして探索）をサポートします。
///
/// # Examples
///
/// ```
/// use vivivi_core::{CursorPosition, TextBuffer, find_matching_paren_in_line};
///
/// let buf = TextBuffer::from_text("println!(\"hello\");");
/// let cursor = CursorPosition::new(0, 0); // 行頭から探索
/// assert_eq!(find_matching_paren_in_line(&buf, cursor), Some(16)); // '(' の相方 ')'
/// ```
pub fn find_matching_paren_in_line(
    buffer: &TextBuffer,
    cursor: crate::cursor::CursorPosition,
) -> Option<usize> {
    let line_text = buffer.line_text(cursor.line)?;
    let chars: Vec<char> = line_text.chars().collect();
    let col = cursor.col.min(chars.len());

    // cursor.col 以降の文字から最初に見つかる括弧を探索
    for (offset, &ch) in chars[col..].iter().enumerate() {
        if paren_pair(ch).is_some() {
            let target_col = col + offset;
            let target_pos = buffer
                .cursor_to_position(crate::cursor::CursorPosition::new(cursor.line, target_col));
            return find_matching_paren(buffer, target_pos);
        }
    }

    None
}
