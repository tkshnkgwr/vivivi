//! Vi キー入力パーサー状態機械モジュール。
//!
//! プレフィックスカウント（`3`）、レジスタ指定（`"a`）、オペレータ（`d`, `y`, `c`）、
//! 2ストロークキー（`gg`）、およびコマンドライン入力（`:`, `/`, `?`）を順次解釈します。

use crate::command::{Motion, Operator, ViCommand};
use crate::mode::ViMode;

/// パーサーの内部状態。
#[derive(Debug, Clone, PartialEq, Eq)]
enum ParserState {
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
    /// `g` の2打鍵目待ち (`gg`)
    WaitingForSecondG { count: usize },
    /// コマンドラインまたは検索文字列入力中 (`:`, `/`, `?`)
    CommandLine { prefix: char, buffer: String },
}

/// Vi キー入力パーサー。
#[derive(Debug, Clone)]
pub struct CommandParser {
    state: ParserState,
    /// 蓄積中のカウント数値 (例: 12j の 12)
    count_buffer: Option<usize>,
    /// 選択中のレジスタ名
    selected_register: Option<char>,
}

impl Default for CommandParser {
    fn default() -> Self {
        Self {
            state: ParserState::Idle,
            count_buffer: None,
            selected_register: None,
        }
    }
}

impl CommandParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// パーサーの状態をリセットします。
    pub fn reset(&mut self) {
        self.state = ParserState::Idle;
        self.count_buffer = None;
        self.selected_register = None;
    }

    /// 現在コマンドライン入力中かどうかを取得します。
    pub fn is_in_command_line(&self) -> bool {
        matches!(self.state, ParserState::CommandLine { .. })
    }

    /// 現在のコマンドライン入力バッファを取得します。
    pub fn command_line_buffer(&self) -> Option<(&char, &str)> {
        match &self.state {
            ParserState::CommandLine { prefix, buffer } => Some((prefix, buffer.as_str())),
            _ => None,
        }
    }

    /// 1文字のキー入力を処理し、コマンドが確定した場合は `Some(ViCommand)` を返します。
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

        match &self.state {
            ParserState::WaitingForRegister => {
                self.selected_register = Some(ch);
                self.state = ParserState::Idle;
                None
            }

            ParserState::WaitingForSecondG { count: _ } => {
                self.state = ParserState::Idle;
                if ch == 'g' {
                    Some(ViCommand::Move(Motion::DocumentStart))
                } else {
                    None
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

                // オペレータ連打（例: dd, yy, cc）は行操作
                let is_double_op = matches!(
                    (op, ch),
                    (Operator::Delete, 'd') | (Operator::Yank, 'y') | (Operator::Change, 'c')
                );

                if is_double_op {
                    self.reset();
                    return Some(ViCommand::OperateLine {
                        op,
                        count,
                        register,
                    });
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

            ParserState::Idle => {
                // レジスタ指定開始 (`"`)
                if ch == '"' {
                    self.state = ParserState::WaitingForRegister;
                    return None;
                }

                // 数値カウントの蓄積 (1〜9 で開始、以降 0〜9)
                if ch.is_ascii_digit() && (ch != '0' || self.count_buffer.is_some()) {
                    let digit = ch.to_digit(10).unwrap() as usize;
                    self.count_buffer = Some(self.count_buffer.unwrap_or(0) * 10 + digit);
                    return None;
                }

                let count = self.count_buffer.take().unwrap_or(1);
                let register = self.selected_register.take();

                // コマンドライン・検索開始
                if ch == ':' || ch == '/' || ch == '?' {
                    self.state = ParserState::CommandLine {
                        prefix: ch,
                        buffer: String::new(),
                    };
                    return Some(ViCommand::EnterMode(ViMode::CommandLine));
                }

                // オペレータ (`d`, `c`, `y`)
                if let Some(op) = match ch {
                    'd' => Some(Operator::Delete),
                    'c' => Some(Operator::Change),
                    'y' => Some(Operator::Yank),
                    _ => None,
                } {
                    self.state = ParserState::WaitingForMotion {
                        op,
                        count,
                        register,
                    };
                    return None;
                }

                // `g` プレフィックス (`gg`)
                if ch == 'g' {
                    self.state = ParserState::WaitingForSecondG { count };
                    return None;
                }

                // 行末までの操作 (`D`, `C`, `Y`)
                match ch {
                    'D' => return Some(ViCommand::DeleteToEol(register)),
                    'C' => return Some(ViCommand::ChangeToEol(register)),
                    'Y' => {
                        return Some(ViCommand::OperateLine {
                            op: Operator::Yank,
                            count,
                            register,
                        });
                    }
                    'x' => return Some(ViCommand::DeleteChar(count)),
                    'X' => return Some(ViCommand::DeletePrevChar(count)),
                    'p' => return Some(ViCommand::PasteAfter { count, register }),
                    'P' => return Some(ViCommand::PasteBefore { count, register }),
                    'u' => return Some(ViCommand::Undo),
                    '\x12' => return Some(ViCommand::Redo), // Ctrl-R
                    'n' => return Some(ViCommand::SearchNext),
                    'N' => return Some(ViCommand::SearchPrev),
                    '*' => return Some(ViCommand::SearchWordUnderCursor { forward: true }),
                    '#' => return Some(ViCommand::SearchWordUnderCursor { forward: false }),
                    _ => {}
                }

                // 挿入系コマンド
                match ch {
                    'i' => return Some(ViCommand::InsertBefore),
                    'a' => return Some(ViCommand::InsertAfter),
                    'I' => return Some(ViCommand::InsertLineStart),
                    'A' => return Some(ViCommand::InsertLineEnd),
                    'o' => return Some(ViCommand::OpenLineBelow),
                    'O' => return Some(ViCommand::OpenLineAbove),
                    'v' => return Some(ViCommand::EnterMode(ViMode::Visual)),
                    'V' => return Some(ViCommand::EnterMode(ViMode::VisualLine)),
                    '\x16' => return Some(ViCommand::EnterMode(ViMode::VisualBlock)), // Ctrl-V
                    _ => {}
                }

                // 単独モーション
                if let Some(motion) = self.parse_motion(ch, count) {
                    return Some(ViCommand::Move(motion));
                }

                None
            }

            ParserState::CommandLine { .. } => unreachable!(),
        }
    }

    /// 単一文字キーからモーションをパースします。
    fn parse_motion(&self, ch: char, count: usize) -> Option<Motion> {
        match ch {
            'h' | '\x08' => Some(Motion::Left(count)),
            'l' | ' ' => Some(Motion::Right(count)),
            'j' => Some(Motion::Down(count)),
            'k' => Some(Motion::Up(count)),
            '0' => Some(Motion::StartOfLine),
            '$' => Some(Motion::EndOfLine),
            '^' => Some(Motion::FirstNonBlank),
            'w' => Some(Motion::NextWord(count)),
            'b' => Some(Motion::PrevWord(count)),
            'e' => Some(Motion::EndWord(count)),
            'W' => Some(Motion::NextBigWord(count)),
            'B' => Some(Motion::PrevBigWord(count)),
            'E' => Some(Motion::EndBigWord(count)),
            'G' => {
                let target_line = self
                    .count_buffer
                    .or(if count > 1 { Some(count) } else { None });
                Some(Motion::DocumentEnd(target_line))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_and_motion() {
        let mut parser = CommandParser::new();

        // 3j -> Move(Down(3))
        assert_eq!(parser.feed_char('3', ViMode::Normal), None);
        assert_eq!(
            parser.feed_char('j', ViMode::Normal),
            Some(ViCommand::Move(Motion::Down(3)))
        );

        // 12w -> Move(NextWord(12))
        assert_eq!(parser.feed_char('1', ViMode::Normal), None);
        assert_eq!(parser.feed_char('2', ViMode::Normal), None);
        assert_eq!(
            parser.feed_char('w', ViMode::Normal),
            Some(ViCommand::Move(Motion::NextWord(12)))
        );
    }

    #[test]
    fn test_operator_and_motion() {
        let mut parser = CommandParser::new();

        // dw -> Operate(Delete, NextWord(1))
        assert_eq!(parser.feed_char('d', ViMode::Normal), None);
        assert_eq!(
            parser.feed_char('w', ViMode::Normal),
            Some(ViCommand::Operate {
                op: Operator::Delete,
                motion: Motion::NextWord(1),
                register: None,
            })
        );

        // dd -> OperateLine(Delete, 1)
        assert_eq!(parser.feed_char('d', ViMode::Normal), None);
        assert_eq!(
            parser.feed_char('d', ViMode::Normal),
            Some(ViCommand::OperateLine {
                op: Operator::Delete,
                count: 1,
                register: None,
            })
        );
    }

    #[test]
    fn test_register_and_command() {
        let mut parser = CommandParser::new();

        // "a yy -> OperateLine(Yank, 1, Some('a'))
        assert_eq!(parser.feed_char('"', ViMode::Normal), None);
        assert_eq!(parser.feed_char('a', ViMode::Normal), None);
        assert_eq!(parser.feed_char('y', ViMode::Normal), None);
        assert_eq!(
            parser.feed_char('y', ViMode::Normal),
            Some(ViCommand::OperateLine {
                op: Operator::Yank,
                count: 1,
                register: Some('a'),
            })
        );
    }

    #[test]
    fn test_command_line_parsing() {
        let mut parser = CommandParser::new();

        // :w<Enter> -> ExCommand("w")
        assert_eq!(
            parser.feed_char(':', ViMode::Normal),
            Some(ViCommand::EnterMode(ViMode::CommandLine))
        );
        assert_eq!(parser.feed_char('w', ViMode::CommandLine), None);
        assert_eq!(
            parser.feed_char('\n', ViMode::CommandLine),
            Some(ViCommand::ExCommand("w".to_string()))
        );

        // /hello<Enter> -> SearchForward("hello")
        assert_eq!(
            parser.feed_char('/', ViMode::Normal),
            Some(ViCommand::EnterMode(ViMode::CommandLine))
        );
        assert_eq!(parser.feed_char('h', ViMode::CommandLine), None);
        assert_eq!(parser.feed_char('i', ViMode::CommandLine), None);
        assert_eq!(
            parser.feed_char('\n', ViMode::CommandLine),
            Some(ViCommand::SearchForward("hi".to_string()))
        );
    }
}
