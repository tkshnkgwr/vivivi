//! ViEngine コマンド実行・ディスパッチ処理。
//!
//! 各種 Vi コマンドのディスパッチ、数値/真偽値トグル、タグジャンプ、
//! 上行からの文字複製等の実行ロジックを統括します。

use crate::command::{Operator, ViCommand, WordSearchKind};
use crate::mode::ViMode;
use crate::register::RegisterContent;
use vivivi_core::{TextBuffer, TextCursor};

use super::{ViEngine, ViExecutionResult};

impl ViEngine {
    /// パースされた [`ViCommand`] をバッファとカーソルに適用して実行します。
    ///
    /// 移動、編集、Undo/Redo、exコマンドなどを実行し、画面更新要否や終了要求を含む [`ViExecutionResult`] を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{TextBuffer, TextCursor};
    /// use vivivi_vi::{Motion, ViCommand, ViEngine};
    ///
    /// let mut engine = ViEngine::new();
    /// let mut buffer = TextBuffer::from_text("abcdef");
    /// let mut cursor = TextCursor::new(0, 0);
    ///
    /// let res = engine.execute(&ViCommand::Move(Motion::Right(3)), &mut buffer, &mut cursor);
    /// assert_eq!(cursor.pos.col, 3);
    /// assert!(res.need_redraw);
    /// ```
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

                    // 挿入モードを抜けた際にドットリピート用変更操作として確定
                    if prev_mode == ViMode::Insert
                        && let Some(insert_cmd) = self.current_insert_cmd.take()
                    {
                        let text: String = self.current_insert_chars.drain(..).collect();
                        self.last_change = Some(crate::engine::RepeatableChange::Insert {
                            cmd: insert_cmd,
                            text,
                        });
                    }

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
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                result.need_redraw = true;
            }

            ViCommand::InsertAfter => {
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
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
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                result.need_redraw = true;
            }

            ViCommand::InsertLineEnd => {
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
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
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
                self.mode = ViMode::Insert;
                buffer.open_undo_block();
                if cursor.pos.line + 1 >= buffer.line_count() {
                    let end_pos = buffer.len();
                    buffer.insert_text(end_pos, "\n");
                } else {
                    let next_line_start = buffer.line_start_position(cursor.pos.line + 1);
                    buffer.insert_text(next_line_start, "\n");
                }
                buffer.close_undo_block();
                cursor.pos.line += 1;
                cursor.pos.col = 0;
                result.need_redraw = true;
            }

            ViCommand::OpenLineAbove => {
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
                self.mode = ViMode::Insert;
                buffer.open_undo_block();
                let line_start_pos = buffer.line_start_position(cursor.pos.line);
                buffer.insert_text(line_start_pos, "\n");
                buffer.close_undo_block();
                cursor.pos.col = 0;
                result.need_redraw = true;
            }

            ViCommand::ReplaceChar { ch, count } => {
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
                buffer.open_undo_block();
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                if line_len > 0 && cursor.pos.col < line_len {
                    let start_col = cursor.pos.col;
                    let replace_count = (*count).min(line_len - start_col);
                    let pos = buffer.cursor_to_position(cursor.pos);
                    buffer.delete_text(pos, replace_count, false);
                    let replace_str: String = std::iter::repeat_n(*ch, replace_count).collect();
                    buffer.insert_text(pos, &replace_str);
                    // カーソルは置換した最後の文字へ
                    cursor.pos.col = start_col + replace_count.saturating_sub(1);
                }
                buffer.close_undo_block();
                result.need_redraw = true;
            }

            ViCommand::SubstituteChar(count) => {
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
                self.mode = ViMode::Insert;
                buffer.open_undo_block();
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                if line_len > 0 && cursor.pos.col < line_len {
                    let del_count = (*count).min(line_len - cursor.pos.col);
                    let pos = buffer.cursor_to_position(cursor.pos);
                    buffer.delete_text(pos, del_count, false);
                }
                buffer.close_undo_block();
                result.need_redraw = true;
            }

            ViCommand::JoinLines { count } => {
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
                let times = (*count).max(1);
                buffer.open_undo_block();
                for _ in 0..times {
                    if cursor.pos.line + 1 < buffer.line_count() {
                        let cur_line_len = buffer
                            .line_text(cursor.pos.line)
                            .map(|s| s.chars().count())
                            .unwrap_or(0);
                        let next_line_start = buffer.line_start_position(cursor.pos.line + 1);
                        let next_line_text =
                            buffer.line_text(cursor.pos.line + 1).unwrap_or_default();
                        let indent_len = next_line_text
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .count();

                        // カレント行末の改行（1文字）＋次行のインデント文字を削除し、半角スペース1文字に置換
                        let del_start = next_line_start.saturating_sub(1);
                        let del_len = 1 + indent_len;
                        buffer.delete_text(del_start, del_len, false);
                        buffer.insert_text(del_start, " ");

                        cursor.pos.col = cur_line_len;
                    }
                }
                buffer.close_undo_block();
                result.need_redraw = true;
            }

            ViCommand::SwapCase => {
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
                let pos = buffer.cursor_to_position(cursor.pos);
                if let Some(ch) = buffer.char_at(pos)
                    && ch != '\n'
                {
                    let swapped = if ch.is_uppercase() {
                        ch.to_lowercase().next().unwrap_or(ch)
                    } else {
                        ch.to_uppercase().next().unwrap_or(ch)
                    };
                    buffer.open_undo_block();
                    buffer.delete_text(pos, 1, false);
                    buffer.insert_char(pos, swapped);
                    buffer.close_undo_block();

                    let line_len = buffer
                        .line_text(cursor.pos.line)
                        .map(|s| s.chars().count())
                        .unwrap_or(0);
                    if cursor.pos.col + 1 < line_len {
                        cursor.pos.col += 1;
                    }
                }
                result.need_redraw = true;
            }

            ViCommand::DeleteChar(count) => {
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
                buffer.open_undo_block();
                let pos = buffer.cursor_to_position(cursor.pos);
                let deleted = buffer.delete_text(pos, *count, false);
                buffer.close_undo_block();
                if !deleted.is_empty() {
                    self.registers
                        .set(None, RegisterContent::CharacterWise(deleted));
                }
                self.last_edit_pos = Some(cursor.pos);
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::DeletePrevChar(count) => {
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
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
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
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
                self.current_insert_cmd = Some(cmd.clone());
                self.current_insert_chars.clear();
                self.execute(&ViCommand::DeleteToEol(*reg), buffer, cursor);
                self.mode = ViMode::Insert;
                result.need_redraw = true;
            }

            ViCommand::OperateLine {
                op,
                count,
                register,
            } => {
                if op == &Operator::Change {
                    self.current_insert_cmd = Some(cmd.clone());
                    self.current_insert_chars.clear();
                } else if op != &Operator::Yank {
                    self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
                }
                self.execute_operate_line(*op, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::Operate {
                op,
                motion,
                register,
            } => {
                if op == &Operator::Change {
                    self.current_insert_cmd = Some(cmd.clone());
                    self.current_insert_chars.clear();
                } else if op != &Operator::Yank {
                    self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
                }
                self.execute_operate(*op, motion, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::PasteAfter { count, register } => {
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
                self.execute_paste(true, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::PasteBefore { count, register } => {
                self.last_change = Some(crate::engine::RepeatableChange::Command(cmd.clone()));
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

            ViCommand::SearchWordUnderCursor { forward, kind } => {
                if let Some(word) = self.word_under_cursor(buffer, cursor) {
                    let escaped = regex::escape(&word);
                    let pat = match kind {
                        WordSearchKind::ExactWord => format!(r"\b{escaped}\b"),
                        WordSearchKind::Partial => escaped,
                        WordSearchKind::Prefix => format!(r"\b{escaped}"),
                        WordSearchKind::Suffix => format!(r"{escaped}\b"),
                    };
                    self.last_search_pattern = Some(pat.clone());
                    self.last_search_forward = *forward;
                    self.search_pattern(&pat, *forward, buffer, cursor, &mut result);
                }
            }

            ViCommand::Increment { delta } => {
                self.execute_increment(*delta, buffer, cursor, &mut result);
            }

            ViCommand::ToggleBoolean => {
                self.execute_toggle_boolean(buffer, cursor, &mut result);
            }

            ViCommand::JumpToLastEdit { line_only } => {
                if let Some(pos) = self.last_edit_pos {
                    cursor.pos.line = pos.line.min(buffer.line_count().saturating_sub(1));
                    if *line_only {
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    } else {
                        cursor.pos.col = pos.col;
                        self.clamp_cursor(buffer, cursor);
                    }
                    result.need_redraw = true;
                } else {
                    result.message = Some("直前の編集位置が記録されていません".to_string());
                }
            }

            ViCommand::CopyFromAboveLine { target } => {
                self.execute_copy_from_above(*target, buffer, cursor, &mut result);
            }

            ViCommand::SetMark(mark) => {
                self.marks
                    .insert(*mark, (self.current_file_path.clone(), cursor.pos));
                result.message = Some(format!("マーク '{mark}' を設定しました"));
            }

            ViCommand::JumpMark { mark, line_only } => {
                if let Some((_file, pos)) = self.marks.get(mark).cloned() {
                    cursor.pos.line = pos.line.min(buffer.line_count().saturating_sub(1));
                    if *line_only {
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    } else {
                        cursor.pos.col = pos.col;
                        self.clamp_cursor(buffer, cursor);
                    }
                    result.need_redraw = true;
                } else {
                    result.message = Some(format!("マーク '{mark}' は設定されていません"));
                }
            }

            ViCommand::ListMarks => {
                if self.marks.is_empty() {
                    result.message = Some("登録されているマークはありません".to_string());
                } else {
                    let mut sorted: Vec<_> = self.marks.iter().collect();
                    sorted.sort_by_key(|&(k, _)| *k);
                    let list: Vec<String> = sorted
                        .iter()
                        .map(|(k, (file, pos))| {
                            let f = if file.is_empty() { "<current>" } else { file };
                            format!("'{k}': {}行{}列 ({f})", pos.line + 1, pos.col + 1)
                        })
                        .collect();
                    result.message = Some(list.join("; "));
                }
            }

            ViCommand::TagJump(target_opt) => {
                let tag = target_opt
                    .clone()
                    .or_else(|| self.word_under_cursor(buffer, cursor));
                if let Some(tag_name) = tag {
                    self.execute_tag_jump(&tag_name, buffer, cursor, &mut result);
                } else {
                    result.message = Some("カーソル位置にタグがありません".to_string());
                }
            }

            ViCommand::TagPop => {
                if let Some((_file, pos, tag)) = self.tag_stack.pop() {
                    cursor.pos = pos;
                    self.clamp_cursor(buffer, cursor);
                    result.need_redraw = true;
                    result.message = Some(format!("タグ '{tag}' のジャンプ元へ復帰しました"));
                } else {
                    result.message = Some("タグスタックが空です".to_string());
                }
            }

            ViCommand::ListTags => {
                if self.tag_stack.is_empty() {
                    result.message = Some("タグスタックは空です".to_string());
                } else {
                    let list: Vec<String> = self
                        .tag_stack
                        .iter()
                        .enumerate()
                        .map(|(i, (_f, pos, tag))| format!("{}: {tag} ({}行)", i + 1, pos.line + 1))
                        .collect();
                    result.message = Some(list.join("; "));
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

            ViCommand::VisualReplaceChar(c) => {
                self.execute_visual_replace_char(*c, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::RecordMacro(reg) => {
                self.recording_register = Some(*reg);
                self.recorded_macro_keys.clear();
                result.message = Some(format!("recording @{reg}"));
                result.need_redraw = true;
            }

            ViCommand::StopRecordMacro => {
                if let Some(reg) = self.recording_register.take() {
                    self.macro_storage
                        .insert(reg, self.recorded_macro_keys.clone());
                    result.message = Some(format!("recorded @{reg}"));
                    result.need_redraw = true;
                }
            }

            ViCommand::ExecuteMacro(target_reg) => {
                let reg = if *target_reg == '@' {
                    match self.last_macro_reg {
                        Some(last) => last,
                        None => return result,
                    }
                } else {
                    *target_reg
                };
                self.last_macro_reg = Some(reg);

                if let Some(keys) = self.macro_storage.get(&reg).cloned() {
                    for ch in keys {
                        if let Some(cmd) = self.feed_char(ch) {
                            self.execute(&cmd, buffer, cursor);
                        } else if self.mode == ViMode::Insert {
                            let pos = buffer.cursor_to_position(cursor.pos);
                            buffer.insert_char(pos, ch);
                            cursor.pos.col += 1;
                        }
                    }
                    result.need_redraw = true;
                }
            }

            ViCommand::Fold(action) => {
                result.need_redraw = true;
                if self.fold_manager.folds.is_empty() {
                    self.fold_manager
                        .compute_indent_folds(&buffer.to_string(), 4);
                }
                let line_1based = cursor.pos.line + 1;
                match action {
                    crate::command::FoldAction::Close => {
                        let ok = self.fold_manager.close_fold(line_1based);
                        result.message = if ok {
                            Some("折り畳みを閉じました (zc)".to_string())
                        } else {
                            Some("この行に閉じられる折り畳みはありません (zc)".to_string())
                        };
                    }
                    crate::command::FoldAction::Open => {
                        let ok = self.fold_manager.open_fold(line_1based);
                        result.message = if ok {
                            Some("折り畳みを開きました (zo)".to_string())
                        } else {
                            Some("この行に開く折り畳みはありません (zo)".to_string())
                        };
                    }
                    crate::command::FoldAction::Toggle => {
                        let ok = self.fold_manager.toggle_fold(line_1based);
                        result.message = if ok {
                            Some("折り畳みをトグルしました (za)".to_string())
                        } else {
                            Some("この行に折り畳みはありません (za)".to_string())
                        };
                    }
                    crate::command::FoldAction::CloseAll => {
                        self.fold_manager.close_all();
                        result.message = Some("すべての折り畳みを閉じました (zM)".to_string());
                    }
                    crate::command::FoldAction::OpenAll => {
                        self.fold_manager.open_all();
                        result.message = Some("すべての折り畳みを開きました (zR)".to_string());
                    }
                    crate::command::FoldAction::CloseLevel(lvl) => {
                        self.fold_manager.close_to_level(*lvl);
                        result.message = Some(format!("レベル {lvl} 以上の折り畳みを閉じました"));
                    }
                }
            }

            ViCommand::ScrollAdjust(target) => {
                result.scroll_adjust = Some(*target);
                result.need_redraw = true;
            }

            ViCommand::RepeatLastChange => {
                if let Some(change) = self.last_change.clone() {
                    match change {
                        crate::engine::RepeatableChange::Command(c) => {
                            self.execute(&c, buffer, cursor);
                        }
                        crate::engine::RepeatableChange::Insert { cmd, text } => {
                            self.execute(&cmd, buffer, cursor);
                            if !text.is_empty() {
                                buffer.open_undo_block();
                                let cur = cursor.pos;
                                let pos = buffer.cursor_to_position(cur);
                                buffer.insert_text(pos, &text);
                                let char_cnt = text.chars().count();
                                cursor.pos =
                                    buffer.position_to_cursor(pos + char_cnt.saturating_sub(1));
                                buffer.close_undo_block();
                            }
                            self.mode = ViMode::Normal;
                        }
                    }
                    result.need_redraw = true;
                }
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
