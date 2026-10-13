//! ViEngine 状態機械およびコマンド実行エンジンモジュール。
//!
//! OpenViVi の `ViEngine.cpp` に基づき、キーボード入力の解釈と、
//! `TextBuffer` および `TextCursor` に対する Vi コマンドの完全な実行・適用を行います。

mod ex;
mod motion;
mod operate;
mod visual;

#[cfg(test)]
mod tests;

pub use visual::BlockInsertSession;

use crate::command::{Operator, ViCommand};
use crate::mode::ViMode;
use crate::parser::CommandParser;
use crate::register::{RegisterContent, RegisterMgr};
use vivivi_core::{TextBuffer, TextCursor};

/// exコマンドやエディタ制御の実行結果。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViExecutionResult {
    /// 画面再描画が必要かどうか
    pub need_redraw: bool,
    /// ファイル保存が要求されたか (`:w`)
    pub save_requested: bool,
    /// エディタ終了が要求されたか (`:q`)
    pub quit_requested: bool,
    /// 強制終了フラグ (`:q!`)
    pub force_quit: bool,
    /// ステータスライン等に表示するメッセージ
    pub message: Option<String>,
}

/// Vi コマンドエンジン本体。
#[derive(Debug, Default)]
pub struct ViEngine {
    mode: ViMode,
    registers: RegisterMgr,
    parser: CommandParser,
    /// 直前の検索文字列
    last_search_pattern: Option<String>,
    /// 直前の検索方向 (true: 前方 `/`, false: 後方 `?`)
    last_search_forward: bool,
    /// 矩形一括挿入セッション
    pub block_insert_session: Option<BlockInsertSession>,
}

impl ViEngine {
    /// 新しい ViEngine を生成します。
    pub fn new() -> Self {
        Self {
            mode: ViMode::Normal,
            registers: RegisterMgr::new(),
            parser: CommandParser::new(),
            last_search_pattern: None,
            last_search_forward: true,
            block_insert_session: None,
        }
    }

    /// 現在のViモードを取得します。
    pub fn mode(&self) -> ViMode {
        self.mode
    }

    /// Viモードを直接変更します。
    pub fn set_mode(&mut self, mode: ViMode) {
        self.mode = mode;
    }

    /// レジスタマネージャへの参照を取得します。
    pub fn registers(&self) -> &RegisterMgr {
        &self.registers
    }

    /// レジスタマネージャへの可変参照を取得します。
    pub fn registers_mut(&mut self) -> &mut RegisterMgr {
        &mut self.registers
    }

    /// パーサーの状態を取得します。
    pub fn parser(&self) -> &CommandParser {
        &self.parser
    }

    /// 単一文字のキー入力を状態機械に供給し、確定した ViCommand があれば返します。
    pub fn feed_char(&mut self, ch: char) -> Option<ViCommand> {
        let cmd = self.parser.feed_char(ch, self.mode);
        if let Some(ref c) = cmd {
            match c {
                ViCommand::EnterMode(new_mode) => self.mode = *new_mode,
                ViCommand::InsertBefore
                | ViCommand::InsertAfter
                | ViCommand::InsertLineStart
                | ViCommand::InsertLineEnd
                | ViCommand::OpenLineBelow
                | ViCommand::OpenLineAbove
                | ViCommand::VisualBlockInsert { .. } => {
                    self.mode = ViMode::Insert;
                }
                ViCommand::Operate {
                    op: Operator::Change,
                    ..
                }
                | ViCommand::OperateLine {
                    op: Operator::Change,
                    ..
                }
                | ViCommand::ChangeToEol(_)
                | ViCommand::VisualOperate {
                    op: Operator::Change,
                    ..
                } => {
                    self.mode = ViMode::Insert;
                }
                ViCommand::VisualOperate { .. } | ViCommand::VisualSwapCase => {
                    self.mode = ViMode::Normal;
                }
                _ => {}
            }
        }
        cmd
    }

    /// 挿入モード中にタイプされた文字を記録します（矩形一括挿入の追跡用）。
    pub fn record_inserted_char(&mut self, ch: char) {
        if let Some(session) = &mut self.block_insert_session {
            session.typed_chars.push(ch);
        }
    }

    /// パースされた ViCommand をバッファとカーソルに適用して実行します。
    pub fn execute(
        &mut self,
        cmd: &ViCommand,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) -> ViExecutionResult {
        let mut result = ViExecutionResult::default();

        match cmd {
            ViCommand::EnterMode(mode) => {
                let prev_mode = self.mode;
                self.mode = *mode;
                buffer.prohibit_merge_undo();

                if matches!(
                    mode,
                    ViMode::Visual | ViMode::VisualLine | ViMode::VisualBlock
                ) {
                    cursor.anchor = Some(cursor.pos);
                } else if *mode == ViMode::Normal {
                    cursor.clear_selection();

                    // 矩形一括挿入セッションが完了した場合
                    if prev_mode == ViMode::Insert && self.block_insert_session.is_some() {
                        self.finish_block_insert(buffer);
                    }
                }
                result.need_redraw = true;
            }

            ViCommand::Move(motion) => {
                self.execute_motion(motion, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::InsertBefore => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                result.need_redraw = true;
            }

            ViCommand::InsertAfter => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                if cursor.pos.col < line_len {
                    cursor.pos.col += 1;
                }
                result.need_redraw = true;
            }

            ViCommand::InsertLineStart => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                result.need_redraw = true;
            }

            ViCommand::InsertLineEnd => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                cursor.pos.col = line_len;
                result.need_redraw = true;
            }

            ViCommand::OpenLineBelow => {
                self.mode = ViMode::Insert;
                buffer.open_undo_block();
                let line_end_pos = buffer.line_start_position(cursor.pos.line + 1);
                // 最終行の場合は末尾に改行を追加
                if cursor.pos.line + 1 >= buffer.line_count() {
                    let end_pos = buffer.len();
                    buffer.insert_text(end_pos, "\n");
                } else {
                    buffer.insert_text(line_end_pos.saturating_sub(1), "\n");
                }
                buffer.close_undo_block();
                cursor.pos.line += 1;
                cursor.pos.col = 0;
                result.need_redraw = true;
            }

            ViCommand::OpenLineAbove => {
                self.mode = ViMode::Insert;
                buffer.open_undo_block();
                let line_start_pos = buffer.line_start_position(cursor.pos.line);
                buffer.insert_text(line_start_pos, "\n");
                buffer.close_undo_block();
                cursor.pos.col = 0;
                result.need_redraw = true;
            }

            ViCommand::DeleteChar(count) => {
                buffer.open_undo_block();
                let pos = buffer.cursor_to_position(cursor.pos);
                let deleted = buffer.delete_text(pos, *count, false);
                buffer.close_undo_block();
                if !deleted.is_empty() {
                    self.registers
                        .set(None, RegisterContent::CharacterWise(deleted));
                }
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::DeletePrevChar(count) => {
                let pos = buffer.cursor_to_position(cursor.pos);
                let delete_len = (*count).min(cursor.pos.col);
                if delete_len > 0 {
                    buffer.open_undo_block();
                    let start_pos = pos.saturating_sub(delete_len);
                    let deleted = buffer.delete_text(start_pos, delete_len, true);
                    buffer.close_undo_block();
                    cursor.pos.col = cursor.pos.col.saturating_sub(delete_len);
                    if !deleted.is_empty() {
                        self.registers
                            .set(None, RegisterContent::CharacterWise(deleted));
                    }
                }
                result.need_redraw = true;
            }

            ViCommand::DeleteToEol(reg) => {
                let pos = buffer.cursor_to_position(cursor.pos);
                let next_line_start = buffer.line_start_position(cursor.pos.line + 1);
                let end_pos = if cursor.pos.line + 1 < buffer.line_count() {
                    next_line_start.saturating_sub(1)
                } else {
                    buffer.len()
                };
                if end_pos > pos {
                    buffer.open_undo_block();
                    let deleted = buffer.delete_text(pos, end_pos - pos, false);
                    buffer.close_undo_block();
                    self.registers
                        .set(*reg, RegisterContent::CharacterWise(deleted));
                }
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::ChangeToEol(reg) => {
                self.execute(&ViCommand::DeleteToEol(*reg), buffer, cursor);
                self.mode = ViMode::Insert;
                result.need_redraw = true;
            }

            ViCommand::OperateLine {
                op,
                count,
                register,
            } => {
                self.execute_operate_line(*op, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::Operate {
                op,
                motion,
                register,
            } => {
                self.execute_operate(*op, motion, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::PasteAfter { count, register } => {
                self.execute_paste(true, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::PasteBefore { count, register } => {
                self.execute_paste(false, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::Undo => {
                buffer.undo();
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::Redo => {
                buffer.redo();
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::SearchForward(pat) => {
                self.last_search_pattern = Some(pat.clone());
                self.last_search_forward = true;
                self.search_pattern(pat, true, buffer, cursor, &mut result);
            }

            ViCommand::SearchBackward(pat) => {
                self.last_search_pattern = Some(pat.clone());
                self.last_search_forward = false;
                self.search_pattern(pat, false, buffer, cursor, &mut result);
            }

            ViCommand::SearchNext => {
                if let Some(ref pat) = self.last_search_pattern.clone() {
                    self.search_pattern(pat, self.last_search_forward, buffer, cursor, &mut result);
                }
            }

            ViCommand::SearchPrev => {
                if let Some(ref pat) = self.last_search_pattern.clone() {
                    self.search_pattern(
                        pat,
                        !self.last_search_forward,
                        buffer,
                        cursor,
                        &mut result,
                    );
                }
            }

            ViCommand::SearchWordUnderCursor { forward } => {
                if let Some(word) = self.word_under_cursor(buffer, cursor) {
                    self.last_search_pattern = Some(word.clone());
                    self.last_search_forward = *forward;
                    self.search_pattern(&word, *forward, buffer, cursor, &mut result);
                }
            }

            ViCommand::ExCommand(cmd_str) => {
                self.execute_ex(cmd_str, buffer, cursor, &mut result);
            }

            ViCommand::VisualOperate { op, register } => {
                self.execute_visual_operate(*op, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::VisualSwapCase => {
                self.execute_visual_swap_case(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::VisualBlockInsert { at_end } => {
                self.execute_visual_block_insert(*at_end, buffer, cursor);
                result.need_redraw = true;
            }
        }

        result
    }

    /// カーソルを現在の行の有効範囲内にクランプします。
    pub(crate) fn clamp_cursor(&self, buffer: &TextBuffer, cursor: &mut TextCursor) {
        if cursor.pos.line >= buffer.line_count() {
            cursor.pos.line = buffer.line_count().saturating_sub(1);
        }
        let line_len = buffer
            .line_text(cursor.pos.line)
            .map(|s| s.chars().count())
            .unwrap_or(0);
        let max_col = if self.mode == ViMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        cursor.pos.col = cursor.pos.col.min(max_col);
    }

    /// 行内の最初の非空白文字の列番号を返します。
    pub(crate) fn find_first_non_blank(&self, buffer: &TextBuffer, line: usize) -> usize {
        buffer
            .line_text(line)
            .and_then(|text| text.chars().position(|c| !c.is_whitespace()))
            .unwrap_or(0)
    }
}
