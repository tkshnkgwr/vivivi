//! `CommandParser` の Idle（初期待ち受け）状態、Visual モード、および単一モーション解決の実装。

use super::{CommandParser, ParserState};
use crate::command::{Motion, Operator, TextObjectModifier, ViCommand, WordSearchKind};
use crate::mode::ViMode;

impl CommandParser {
    /// Visual モード（`Visual`, `VisualLine`, `VisualBlock`）特有のキー入力を解釈します。
    ///
    /// Visual モード固有のコマンドまたは状態遷移に該当した場合は `Some(Option<ViCommand>)` を返し、
    /// 通常のモーションやレジスタ指定へフォールスルーすべき場合は `None` を返します。
    pub(super) fn feed_visual_mode_char(
        &mut self,
        ch: char,
        mode: ViMode,
    ) -> Option<Option<ViCommand>> {
        match ch {
            'd' | 'x' => {
                let reg = self.selected_register.take();
                self.reset();
                Some(Some(ViCommand::VisualOperate {
                    op: Operator::Delete,
                    register: reg,
                }))
            }
            'y' => {
                let reg = self.selected_register.take();
                self.reset();
                Some(Some(ViCommand::VisualOperate {
                    op: Operator::Yank,
                    register: reg,
                }))
            }
            'c' => {
                let reg = self.selected_register.take();
                self.reset();
                Some(Some(ViCommand::VisualOperate {
                    op: Operator::Change,
                    register: reg,
                }))
            }
            '~' => {
                self.reset();
                Some(Some(ViCommand::VisualSwapCase))
            }
            '>' => {
                self.reset();
                Some(Some(ViCommand::VisualOperate {
                    op: Operator::ShiftRight,
                    register: None,
                }))
            }
            '<' => {
                self.reset();
                Some(Some(ViCommand::VisualOperate {
                    op: Operator::ShiftLeft,
                    register: None,
                }))
            }
            'u' => {
                self.reset();
                Some(Some(ViCommand::VisualOperate {
                    op: Operator::ToLower,
                    register: None,
                }))
            }
            'U' => {
                self.reset();
                Some(Some(ViCommand::VisualOperate {
                    op: Operator::ToUpper,
                    register: None,
                }))
            }
            'i' => {
                self.state = ParserState::WaitingForTextObject {
                    op: None,
                    modifier: TextObjectModifier::Inner,
                    count: 1,
                    register: None,
                };
                Some(None)
            }
            'a' => {
                self.state = ParserState::WaitingForTextObject {
                    op: None,
                    modifier: TextObjectModifier::Around,
                    count: 1,
                    register: None,
                };
                Some(None)
            }
            'r' => {
                self.reset();
                self.state = ParserState::WaitingForVisualReplaceChar;
                Some(None)
            }
            'v' => {
                self.reset();
                if mode == ViMode::Visual {
                    Some(Some(ViCommand::EnterMode(ViMode::Normal)))
                } else {
                    Some(Some(ViCommand::EnterMode(ViMode::Visual)))
                }
            }
            'V' => {
                self.reset();
                if mode == ViMode::VisualLine {
                    Some(Some(ViCommand::EnterMode(ViMode::Normal)))
                } else {
                    Some(Some(ViCommand::EnterMode(ViMode::VisualLine)))
                }
            }
            '\x16' => {
                self.reset();
                if mode == ViMode::VisualBlock {
                    Some(Some(ViCommand::EnterMode(ViMode::Normal)))
                } else {
                    Some(Some(ViCommand::EnterMode(ViMode::VisualBlock)))
                }
            }
            'I' if mode == ViMode::VisualBlock => {
                self.reset();
                Some(Some(ViCommand::VisualBlockInsert { at_end: false }))
            }
            'A' if mode == ViMode::VisualBlock => {
                self.reset();
                Some(Some(ViCommand::VisualBlockInsert { at_end: true }))
            }
            _ => None,
        }
    }

    /// `ParserState::Idle` 状態での1文字キー入力を解釈します。
    pub(super) fn feed_idle_char(&mut self, ch: char) -> Option<ViCommand> {
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

        // マーク設定 (`m`)
        if ch == 'm' {
            self.state = ParserState::WaitingForMarkSet;
            return None;
        }

        // マークジャンプ (`'` 行ジャンプ, `` ` `` 正確位置ジャンプ)
        if ch == '\'' {
            self.state = ParserState::WaitingForMarkJump { line_only: true };
            return None;
        }
        if ch == '`' {
            self.state = ParserState::WaitingForMarkJump { line_only: false };
            return None;
        }

        // f/F/t/T 文字検索ジャンプ
        if matches!(ch, 'f' | 'F' | 't' | 'T') {
            self.state = ParserState::WaitingForFindChar {
                op: None,
                is_forward: matches!(ch, 'f' | 't'),
                is_till: matches!(ch, 't' | 'T'),
                count,
                register: None,
            };
            return None;
        }

        // 直前行文字複製 (`=`)
        if ch == '=' {
            self.state = ParserState::WaitingForEqual;
            return None;
        }

        // ViVi 独自 `#+`, `##`, `#-`, `#!` & 単語後方検索 `#`
        if ch == '#' {
            self.state = ParserState::WaitingForHash { count };
            return None;
        }

        // マクロ記録開始・終了 (`q`)
        if ch == 'q' {
            if self.is_recording {
                self.is_recording = false;
                return Some(ViCommand::StopRecordMacro);
            } else {
                self.state = ParserState::WaitingForMacroRecord;
                return None;
            }
        }

        // マクロ再生 (`@`)
        if ch == '@' {
            self.state = ParserState::WaitingForMacroExecute;
            return None;
        }

        // コード折り畳み (`z`)
        if ch == 'z' {
            self.state = ParserState::WaitingForZ;
            return None;
        }

        // コマンドライン・検索開始
        if ch == ':' || ch == '/' || ch == '?' {
            self.state = ParserState::CommandLine {
                prefix: ch,
                buffer: String::new(),
            };
            return Some(ViCommand::EnterMode(ViMode::CommandLine));
        }

        // オペレータ (`d`, `c`, `y`, `>`, `<`)
        if let Some(op) = match ch {
            'd' => Some(Operator::Delete),
            'c' => Some(Operator::Change),
            'y' => Some(Operator::Yank),
            '>' => Some(Operator::ShiftRight),
            '<' => Some(Operator::ShiftLeft),
            _ => None,
        } {
            self.state = ParserState::WaitingForMotion {
                op,
                count,
                register,
            };
            return None;
        }

        // `g` プレフィックス (`gg`, `g*`, `g#`, `g<`, `g=`)
        if ch == 'g' {
            self.state = ParserState::WaitingForG { count };
            return None;
        }

        // タグジャンプ (`Ctrl-]` = \x1d), タグ復元 (`Ctrl-t` = \x14)
        if ch == '\x1d' {
            return Some(ViCommand::TagJump(None));
        }
        if ch == '\x14' {
            return Some(ViCommand::TagPop);
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
            '\x01' => {
                return Some(ViCommand::Increment {
                    delta: count as i64,
                });
            } // Ctrl-A
            '\x18' => {
                return Some(ViCommand::Increment {
                    delta: -(count as i64),
                });
            } // Ctrl-X
            's' => return Some(ViCommand::SubstituteChar(count)),
            'S' => {
                return Some(ViCommand::OperateLine {
                    op: Operator::Change,
                    count,
                    register,
                });
            }
            'J' => return Some(ViCommand::JoinLines { count }),
            '~' => return Some(ViCommand::SwapCase),
            '.' => return Some(ViCommand::RepeatLastChange),
            'n' => return Some(ViCommand::SearchNext),
            'N' => return Some(ViCommand::SearchPrev),
            '*' => {
                return Some(ViCommand::SearchWordUnderCursor {
                    forward: true,
                    kind: WordSearchKind::ExactWord,
                });
            }
            _ => {}
        }

        // 1文字置換 (`r<char>`)
        if ch == 'r' {
            self.state = ParserState::WaitingForReplaceChar { count };
            return None;
        }

        // 挿入系コマンド
        match ch {
            'i' => return Some(ViCommand::InsertBefore),
            'a' => return Some(ViCommand::InsertAfter),
            'I' => return Some(ViCommand::InsertLineStart),
            'A' => return Some(ViCommand::InsertLineEnd),
            'o' => return Some(ViCommand::OpenLineBelow),
            'O' => return Some(ViCommand::OpenLineAbove),
            'R' => return Some(ViCommand::EnterMode(ViMode::Replace)),
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

    /// 1文字キー入力を [`Motion`] へ変換します。
    pub(super) fn parse_motion(&self, ch: char, count: usize) -> Option<Motion> {
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
            ';' => Some(Motion::RepeatFindChar {
                reverse: false,
                count,
            }),
            ',' => Some(Motion::RepeatFindChar {
                reverse: true,
                count,
            }),
            'G' => {
                let target_line = self
                    .count_buffer
                    .or(if count > 1 { Some(count) } else { None });
                Some(Motion::DocumentEnd(target_line))
            }
            '%' => Some(Motion::MatchingParen),
            '{' => Some(Motion::ParagraphBackward(count)),
            '}' => Some(Motion::ParagraphForward(count)),
            _ => None,
        }
    }
}
