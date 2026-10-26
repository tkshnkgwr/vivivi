//! Vi キー入力パーサー状態機械モジュール。
//!
//! プレフィックスカウント（`3`）、レジスタ指定（`"a`）、オペレータ（`d`, `y`, `c`）、
//! 2ストロークキー（`gg`）、およびコマンドライン入力（`:`, `/`, `?`）を順次解釈します。

mod idle;

use crate::command::{
    Motion, Operator, TextObject, TextObjectKind, TextObjectModifier, ViCommand, WordSearchKind,
};
use crate::mode::ViMode;

/// パーサーの内部状態を表す列挙体。
///
/// レジスタ指定、オペレータ待ち、多打鍵プレフィックス、コマンドライン入力中などの各フェーズを表現します。
///
/// # Examples
///
/// ```
/// use vivivi_vi::{CommandParser, ParserState};
///
/// let parser = CommandParser::new();
/// assert_eq!(parser.state(), &ParserState::Idle);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    /// 通常のキー待ち受け
    Idle,
    /// レジスタ名待ち（`"` を入力した直後）
    WaitingForRegister,
    /// オペレータの対象モーション待ち（`d`, `c`, `y` を入力した直後）
    WaitingForMotion {
        op: Operator,
        count: usize,
        register: Option<char>,
    },
    /// 文字検索の文字入力待ち (`f`, `F`, `t`, `T`)
    WaitingForFindChar {
        op: Option<Operator>,
        is_forward: bool,
        is_till: bool,
        count: usize,
        register: Option<char>,
    },
    /// テキストオブジェクトの対象文字待ち (`i` または `a` の直後)
    WaitingForTextObject {
        op: Option<Operator>,
        modifier: TextObjectModifier,
        count: usize,
        register: Option<char>,
    },
    /// `g` の2打鍵目待ち (`gg`, `g*`, `g#`, `g<`, `g=`)
    WaitingForG { count: usize },
    /// `#` の2打鍵目待ち (`#+`, `##`, `#-`, `#!`, または単語後方検索 `#`)
    WaitingForHash { count: usize },
    /// 直前行文字複製待ち (`=<char>`)
    WaitingForEqual,
    /// マーク設定のマーク名待ち (`m` の後)
    WaitingForMarkSet,
    /// マークジャンプのマーク名待ち (`'` または `` ` `` の後)
    WaitingForMarkJump { line_only: bool },
    /// マクロ記録対象レジスタ待ち (`q` の後)
    WaitingForMacroRecord,
    /// マクロ再生対象レジスタ待ち (`@` の後)
    WaitingForMacroExecute,
    /// コード折り畳み待ち (`z` の後)
    WaitingForZ,
    /// 単一文字置換の置換文字待ち (`r` の後)
    WaitingForReplaceChar { count: usize },
    /// ビジュアルモードでの一括文字置換待ち (`r` の後)
    WaitingForVisualReplaceChar,
    /// コマンドラインまたは検索文字列入力中 (`:`, `/`, `?`)
    CommandLine { prefix: char, buffer: String },
}

/// 多段階の Vi キー入力を解析して [`ViCommand`] を生成する状態機械パーサー。
///
/// # Examples
///
/// ```
/// use vivivi_vi::{CommandParser, Motion, ViCommand, ViMode};
///
/// let mut parser = CommandParser::new();
/// let cmd = parser.feed_char('j', ViMode::Normal);
/// assert_eq!(cmd, Some(ViCommand::Move(Motion::Down(1))));
/// ```
#[derive(Debug, Clone)]
pub struct CommandParser {
    pub(crate) state: ParserState,
    /// 蓄積中のカウント数値 (例: 12j の 12)
    pub(crate) count_buffer: Option<usize>,
    /// 選択中のレジスタ名
    pub(crate) selected_register: Option<char>,
    /// 現在キーボードマクロを記録中かどうか
    pub is_recording: bool,
}

impl Default for CommandParser {
    fn default() -> Self {
        Self {
            state: ParserState::Idle,
            count_buffer: None,
            selected_register: None,
            is_recording: false,
        }
    }
}

impl CommandParser {
    /// 初期状態（[`ParserState::Idle`]）で新しい [`CommandParser`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::CommandParser;
    ///
    /// let parser = CommandParser::new();
    /// assert!(!parser.is_in_command_line());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// パーサーの状態、カウントバッファ、レジスタ選択をすべてリセットします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{CommandParser, ParserState, ViMode};
    ///
    /// let mut parser = CommandParser::new();
    /// parser.feed_char('d', ViMode::Normal);
    /// assert!(matches!(parser.state(), ParserState::WaitingForMotion { .. }));
    ///
    /// parser.reset();
    /// assert_eq!(parser.state(), &ParserState::Idle);
    /// ```
    pub fn reset(&mut self) {
        self.state = ParserState::Idle;
        self.count_buffer = None;
        self.selected_register = None;
    }

    /// パーサーの現在の内部状態への参照を取得します。
    pub fn state(&self) -> &ParserState {
        &self.state
    }

    /// マクロ記録用のレジスタ入力待ち (`q` 押下直後) かどうかを判定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{CommandParser, ViMode};
    ///
    /// let mut parser = CommandParser::new();
    /// parser.feed_char('q', ViMode::Normal);
    /// assert!(parser.is_waiting_for_macro_record());
    /// ```
    pub fn is_waiting_for_macro_record(&self) -> bool {
        matches!(self.state, ParserState::WaitingForMacroRecord)
    }

    /// 現在コマンドラインまたは検索入力中（`:`, `/`, `?`）かどうかを判定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{CommandParser, ViMode};
    ///
    /// let mut parser = CommandParser::new();
    /// parser.feed_char(':', ViMode::Normal);
    /// assert!(parser.is_in_command_line());
    /// ```
    pub fn is_in_command_line(&self) -> bool {
        matches!(self.state, ParserState::CommandLine { .. })
    }

    /// 現在のコマンドライン入力プレフィックス（`:`, `/`, `?`）と入力中文字列を取得します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{CommandParser, ViMode};
    ///
    /// let mut parser = CommandParser::new();
    /// parser.feed_char(':', ViMode::Normal);
    /// parser.feed_char('w', ViMode::CommandLine);
    /// assert_eq!(parser.command_line_buffer(), Some((&':', "w")));
    /// ```
    pub fn command_line_buffer(&self) -> Option<(&char, &str)> {
        match &self.state {
            ParserState::CommandLine { prefix, buffer } => Some((prefix, buffer.as_str())),
            _ => None,
        }
    }

    /// 1文字のキー入力を処理し、コマンドが確定した場合は `Some(ViCommand)` を返します。
    ///
    /// カウント数値の入力や多打鍵コマンド（例: `gg`, `dw`）の入力途中では内部状態を更新して `None` を返します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{CommandParser, Motion, ViCommand, ViMode};
    ///
    /// let mut parser = CommandParser::new();
    /// assert_eq!(parser.feed_char('l', ViMode::Normal), Some(ViCommand::Move(Motion::Right(1))));
    /// ```
    pub fn feed_char(&mut self, ch: char, mode: ViMode) -> Option<ViCommand> {
        // コマンドラインモードまたは検索入力中の処理
        if let ParserState::CommandLine {
            prefix,
            ref mut buffer,
        } = self.state
        {
            match ch {
                '\n' | '\r' => {
                    let cmd_str = std::mem::take(buffer);
                    let pref = prefix;
                    self.reset();
                    return match pref {
                        ':' => Some(ViCommand::ExCommand(cmd_str)),
                        '/' => Some(ViCommand::SearchForward(cmd_str)),
                        '?' => Some(ViCommand::SearchBackward(cmd_str)),
                        _ => None,
                    };
                }
                '\x1b' => {
                    // Escape でキャンセル
                    self.reset();
                    return Some(ViCommand::EnterMode(ViMode::Normal));
                }
                '\x08' | '\x7f' => {
                    // BackSpace で1文字削除
                    if buffer.is_empty() {
                        self.reset();
                        return Some(ViCommand::EnterMode(ViMode::Normal));
                    } else {
                        buffer.pop();
                        return None;
                    }
                }
                _ => {
                    buffer.push(ch);
                    return None;
                }
            }
        }

        // Insertモード中の特別処理
        if mode == ViMode::Insert {
            if ch == '\x1b' {
                self.reset();
                return Some(ViCommand::EnterMode(ViMode::Normal));
            }
            return None;
        }

        // Escapeキーはいつでもノーマル復帰・パーサーリセット
        if ch == '\x1b' {
            self.reset();
            return Some(ViCommand::EnterMode(ViMode::Normal));
        }

        // Visual モード（Visual, VisualLine, VisualBlock）中のキー処理
        if matches!(
            mode,
            ViMode::Visual | ViMode::VisualLine | ViMode::VisualBlock
        ) && let Some(res) = self.feed_visual_mode_char(ch, mode)
        {
            return res;
        }

        match &self.state {
            ParserState::WaitingForRegister => {
                self.selected_register = Some(ch);
                self.state = ParserState::Idle;
                None
            }

            ParserState::WaitingForFindChar {
                op,
                is_forward,
                is_till,
                count,
                register,
            } => {
                let op = *op;
                let is_forward = *is_forward;
                let is_till = *is_till;
                let count = *count;
                let register = *register;
                self.reset();

                let motion = if is_forward {
                    Motion::FindCharForward {
                        target: ch,
                        till: is_till,
                        count,
                    }
                } else {
                    Motion::FindCharBackward {
                        target: ch,
                        till: is_till,
                        count,
                    }
                };

                if let Some(op) = op {
                    Some(ViCommand::Operate {
                        op,
                        motion,
                        register,
                    })
                } else {
                    Some(ViCommand::Move(motion))
                }
            }

            ParserState::WaitingForG { count } => {
                let count = *count;
                match ch {
                    'g' => {
                        self.state = ParserState::Idle;
                        Some(ViCommand::Move(Motion::DocumentStart))
                    }
                    'u' => {
                        self.state = ParserState::WaitingForMotion {
                            op: Operator::ToLower,
                            count,
                            register: None,
                        };
                        None
                    }
                    'U' => {
                        self.state = ParserState::WaitingForMotion {
                            op: Operator::ToUpper,
                            count,
                            register: None,
                        };
                        None
                    }
                    '*' => {
                        self.state = ParserState::Idle;
                        Some(ViCommand::SearchWordUnderCursor {
                            forward: true,
                            kind: WordSearchKind::Partial,
                        })
                    }
                    '#' => {
                        self.state = ParserState::Idle;
                        Some(ViCommand::SearchWordUnderCursor {
                            forward: false,
                            kind: WordSearchKind::Partial,
                        })
                    }
                    '<' => {
                        self.state = ParserState::Idle;
                        Some(ViCommand::SearchWordUnderCursor {
                            forward: true,
                            kind: WordSearchKind::Prefix,
                        })
                    }
                    '=' => {
                        self.state = ParserState::Idle;
                        Some(ViCommand::SearchWordUnderCursor {
                            forward: true,
                            kind: WordSearchKind::Suffix,
                        })
                    }
                    _ => {
                        self.state = ParserState::Idle;
                        None
                    }
                }
            }

            ParserState::WaitingForHash { count } => {
                let count = *count;
                self.state = ParserState::Idle;
                match ch {
                    '+' | '#' => Some(ViCommand::Increment {
                        delta: count as i64,
                    }),
                    '-' => Some(ViCommand::Increment {
                        delta: -(count as i64),
                    }),
                    '!' => Some(ViCommand::ToggleBoolean),
                    _ => Some(ViCommand::SearchWordUnderCursor {
                        forward: false,
                        kind: WordSearchKind::ExactWord,
                    }),
                }
            }

            ParserState::WaitingForEqual => {
                self.state = ParserState::Idle;
                match ch {
                    '\n' | '\r' => Some(ViCommand::CopyFromAboveLine { target: None }),
                    _ => Some(ViCommand::CopyFromAboveLine { target: Some(ch) }),
                }
            }

            ParserState::WaitingForMarkSet => {
                self.state = ParserState::Idle;
                if ch.is_ascii_alphabetic() {
                    Some(ViCommand::SetMark(ch))
                } else {
                    None
                }
            }

            ParserState::WaitingForMarkJump { line_only } => {
                let line_only = *line_only;
                self.state = ParserState::Idle;
                if ch == '@' || ch == '.' {
                    Some(ViCommand::JumpToLastEdit { line_only })
                } else if ch.is_ascii_alphabetic() {
                    Some(ViCommand::JumpMark {
                        mark: ch,
                        line_only,
                    })
                } else {
                    None
                }
            }

            ParserState::WaitingForMacroRecord => {
                self.state = ParserState::Idle;
                if ch.is_ascii_alphanumeric() {
                    self.is_recording = true;
                    Some(ViCommand::RecordMacro(ch))
                } else {
                    None
                }
            }

            ParserState::WaitingForMacroExecute => {
                self.state = ParserState::Idle;
                if ch.is_ascii_alphanumeric() || ch == '@' {
                    Some(ViCommand::ExecuteMacro(ch))
                } else {
                    None
                }
            }

            ParserState::WaitingForZ => {
                self.state = ParserState::Idle;
                match ch {
                    'z' => Some(ViCommand::ScrollAdjust(
                        crate::command::ScrollTarget::Center,
                    )),
                    't' => Some(ViCommand::ScrollAdjust(crate::command::ScrollTarget::Top)),
                    'b' => Some(ViCommand::ScrollAdjust(
                        crate::command::ScrollTarget::Bottom,
                    )),
                    'c' => Some(ViCommand::Fold(crate::command::FoldAction::Close)),
                    'o' => Some(ViCommand::Fold(crate::command::FoldAction::Open)),
                    'a' => Some(ViCommand::Fold(crate::command::FoldAction::Toggle)),
                    'M' => Some(ViCommand::Fold(crate::command::FoldAction::CloseAll)),
                    'R' => Some(ViCommand::Fold(crate::command::FoldAction::OpenAll)),
                    d if d.is_ascii_digit() => {
                        let lvl = d.to_digit(10).unwrap() as usize;
                        Some(ViCommand::Fold(crate::command::FoldAction::CloseLevel(lvl)))
                    }
                    _ => None,
                }
            }

            ParserState::WaitingForReplaceChar { count } => {
                let count = *count;
                self.reset();
                if ch == '\x1b' {
                    None
                } else {
                    Some(ViCommand::ReplaceChar { ch, count })
                }
            }

            ParserState::WaitingForVisualReplaceChar => {
                self.reset();
                if ch == '\x1b' {
                    None
                } else {
                    Some(ViCommand::VisualReplaceChar(ch))
                }
            }

            ParserState::WaitingForMotion {
                op,
                count,
                register,
            } => {
                let op = *op;
                let count = *count;
                let register = *register;

                // オペレータ連打（例: dd, yy, cc, >>, <<, guu, gUU）は行操作
                let is_double_op = matches!(
                    (op, ch),
                    (Operator::Delete, 'd')
                        | (Operator::Yank, 'y')
                        | (Operator::Change, 'c')
                        | (Operator::ShiftRight, '>')
                        | (Operator::ShiftLeft, '<')
                        | (Operator::ToLower, 'u')
                        | (Operator::ToLower, 'g')
                        | (Operator::ToUpper, 'U')
                        | (Operator::ToUpper, 'g')
                );

                if is_double_op {
                    self.reset();
                    return Some(ViCommand::OperateLine {
                        op,
                        count,
                        register,
                    });
                }

                // テキストオブジェクト（i または a）待ちへ遷移
                if ch == 'i' || ch == 'a' {
                    let modifier = if ch == 'i' {
                        TextObjectModifier::Inner
                    } else {
                        TextObjectModifier::Around
                    };
                    self.state = ParserState::WaitingForTextObject {
                        op: Some(op),
                        modifier,
                        count,
                        register,
                    };
                    return None;
                }

                // f, F, t, T は文字入力待ちへ遷移
                if matches!(ch, 'f' | 'F' | 't' | 'T') {
                    self.state = ParserState::WaitingForFindChar {
                        op: Some(op),
                        is_forward: matches!(ch, 'f' | 't'),
                        is_till: matches!(ch, 't' | 'T'),
                        count,
                        register,
                    };
                    return None;
                }

                // モーションのパース
                if let Some(motion) = self.parse_motion(ch, count) {
                    self.reset();
                    Some(ViCommand::Operate {
                        op,
                        motion,
                        register,
                    })
                } else {
                    self.reset();
                    None
                }
            }

            ParserState::WaitingForTextObject {
                op,
                modifier,
                count,
                register,
            } => {
                let op = *op;
                let modifier = *modifier;
                let count = *count;
                let register = *register;
                self.reset();

                let kind = match ch {
                    'w' => Some(TextObjectKind::Word),
                    'W' => Some(TextObjectKind::BigWord),
                    '"' => Some(TextObjectKind::DoubleQuote),
                    '\'' => Some(TextObjectKind::SingleQuote),
                    '`' => Some(TextObjectKind::BackQuote),
                    '(' | ')' | 'b' => Some(TextObjectKind::Parentheses),
                    '{' | '}' | 'B' => Some(TextObjectKind::Braces),
                    '[' | ']' => Some(TextObjectKind::Bracket),
                    '<' | '>' => Some(TextObjectKind::AngleBracket),
                    _ => None,
                };

                if let Some(kind) = kind {
                    let motion = Motion::TextObject(TextObject {
                        modifier,
                        kind,
                        count,
                    });
                    if let Some(op) = op {
                        Some(ViCommand::Operate {
                            op,
                            motion,
                            register,
                        })
                    } else {
                        Some(ViCommand::Move(motion))
                    }
                } else {
                    None
                }
            }

            ParserState::Idle => self.feed_idle_char(ch),

            ParserState::CommandLine { .. } => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_parser_state_transitions() {
        let mut parser = CommandParser::new();
        assert_eq!(parser.state, ParserState::Idle);

        // '2' を入力 -> カウントバッファに蓄積
        parser.feed_char('2', ViMode::Normal);
        assert_eq!(parser.count_buffer, Some(2));

        // '"' を入力 -> レジスタプレフィックス待機状態へ遷移
        parser.feed_char('"', ViMode::Normal);
        assert_eq!(parser.state, ParserState::WaitingForRegister);

        // 'a' を入力 -> selected_register に設定され Idle 状態へ
        parser.feed_char('a', ViMode::Normal);
        assert_eq!(parser.selected_register, Some('a'));
        assert_eq!(parser.state, ParserState::Idle);
    }
}
