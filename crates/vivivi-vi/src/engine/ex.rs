//! exコマンド（`:w`, `:q`, `:%s` など）およびバッファ内検索処理モジュール。

use super::{ViEngine, ViExecutionResult};
use crate::command::ViCommand;
use vivivi_core::{TextBuffer, TextCursor};

impl ViEngine {
    /// 文字列検索 (`/`, `?`, `n`, `N`)
    pub(crate) fn search_pattern(
        &mut self,
        pat: &str,
        forward: bool,
        buffer: &TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        if pat.is_empty() {
            return;
        }

        let full_text = buffer.to_string();
        let cur_pos = buffer.cursor_to_position(cursor.pos);
        let cur_byte_offset = full_text
            .char_indices()
            .nth(cur_pos)
            .map(|(i, _)| i)
            .unwrap_or(full_text.len());

        let re = regex::Regex::new(pat).ok();

        if forward {
            let next_byte_offset = full_text
                .char_indices()
                .nth(cur_pos + 1)
                .map(|(i, _)| i)
                .unwrap_or(0);

            let found_byte = if let Some(ref r) = re {
                r.find_at(&full_text, next_byte_offset)
                    .map(|m| m.start())
                    .or_else(|| r.find(&full_text).map(|m| m.start()))
            } else {
                full_text
                    .char_indices()
                    .filter(|(idx, _)| *idx >= next_byte_offset)
                    .find(|(idx, _)| full_text[*idx..].starts_with(pat))
                    .map(|(idx, _)| idx)
                    .or_else(|| {
                        full_text
                            .char_indices()
                            .find(|(idx, _)| full_text[*idx..].starts_with(pat))
                            .map(|(idx, _)| idx)
                    })
            };

            if let Some(byte_idx) = found_byte {
                let char_idx = full_text[..byte_idx].chars().count();
                cursor.pos = buffer.position_to_cursor(char_idx);
                result.need_redraw = true;
            } else {
                result.message = Some(format!("パターンが見つかりません: {pat}"));
            }
        } else {
            let found_byte = if let Some(ref r) = re {
                let mut matches_before = r
                    .find_iter(&full_text[..cur_byte_offset])
                    .map(|m| m.start())
                    .collect::<Vec<_>>();
                if let Some(last) = matches_before.pop() {
                    Some(last)
                } else {
                    r.find_iter(&full_text).map(|m| m.start()).last()
                }
            } else {
                full_text
                    .char_indices()
                    .filter(|(idx, _)| *idx < cur_byte_offset)
                    .rfind(|(idx, _)| full_text[*idx..].starts_with(pat))
                    .map(|(idx, _)| idx)
                    .or_else(|| {
                        full_text
                            .char_indices()
                            .rfind(|(idx, _)| full_text[*idx..].starts_with(pat))
                            .map(|(idx, _)| idx)
                    })
            };

            if let Some(byte_idx) = found_byte {
                let char_idx = full_text[..byte_idx].chars().count();
                cursor.pos = buffer.position_to_cursor(char_idx);
                result.need_redraw = true;
            } else {
                result.message = Some(format!("パターンが見つかりません: {pat}"));
            }
        }
    }

    /// exコマンドのパース・実行 (`:w`, `:q`, `:%s/before/after/g`, `:行番号`)
    pub(crate) fn execute_ex(
        &mut self,
        cmd_str: &str,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        let cmd = cmd_str.trim();

        if cmd == "w" {
            result.save_requested = true;
            result.message = Some("ファイルを保存しました".to_string());
        } else if cmd == "q" {
            result.quit_requested = true;
        } else if cmd == "q!" {
            result.quit_requested = true;
            result.force_quit = true;
        } else if cmd == "wq" || cmd == "x" {
            result.save_requested = true;
            result.quit_requested = true;
        } else if let Ok(line_num) = cmd.parse::<usize>() {
            // 指定行へのジャンプ
            cursor.pos.line = line_num
                .saturating_sub(1)
                .min(buffer.line_count().saturating_sub(1));
            cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            result.need_redraw = true;
        } else if cmd.starts_with("%s/") || cmd.starts_with("s/") {
            // 置換コマンド
            self.execute_substitute(cmd, buffer, cursor, result);
        } else if cmd == "marks" {
            let res = self.execute(&ViCommand::ListMarks, buffer, cursor);
            result.message = res.message;
        } else if cmd == "tags" {
            let res = self.execute(&ViCommand::ListTags, buffer, cursor);
            result.message = res.message;
        } else if let Some(tag_str) = cmd.strip_prefix("tag ") {
            let tag = tag_str.trim().to_string();
            let res = self.execute(&ViCommand::TagJump(Some(tag)), buffer, cursor);
            result.message = res.message;
            result.need_redraw = res.need_redraw;
        } else {
            result.message = Some(format!("未定義のコマンドです: :{cmd}"));
        }
    }

    /// 置換コマンド (`:s/old/new/`, `:%s/old/new/g`)
    pub(crate) fn execute_substitute(
        &mut self,
        cmd: &str,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        let is_global = cmd.starts_with("%s/");
        let spec = if is_global { &cmd[3..] } else { &cmd[2..] };
        let parts: Vec<&str> = spec.split('/').collect();

        if parts.len() >= 2 {
            let before = parts[0];
            let after = parts[1];
            let flags = parts.get(2).copied().unwrap_or("");
            let replace_all = flags.contains('g');
            let is_expr = flags.contains('e');
            let ignore_case = flags.contains('i');

            if before.is_empty() {
                return;
            }

            if is_expr {
                // ViVi 3 / ViVi 4 式置換 (:s/pattern/expression/e)
                buffer.open_undo_block();
                if is_global {
                    let full_text = buffer.to_string();
                    match vivivi_modes::execute_expr_substitute(
                        &full_text,
                        before,
                        after,
                        replace_all,
                        ignore_case,
                        1,
                    ) {
                        Ok((new_text, count)) => {
                            if new_text != full_text {
                                let len = buffer.len();
                                buffer.delete_text(0, len, false);
                                buffer.insert_text(0, &new_text);
                            }
                            buffer.close_undo_block();
                            self.clamp_cursor(buffer, cursor);
                            result.need_redraw = true;
                            result.message = Some(format!("{count} 箇所の式置換を行いました"));
                        }
                        Err(e) => {
                            buffer.close_undo_block();
                            result.message = Some(format!("式置換エラー: {e}"));
                        }
                    }
                } else if let Some(line_str) = buffer.line_text(cursor.pos.line) {
                    let line_no = cursor.pos.line + 1;
                    match vivivi_modes::execute_expr_substitute(
                        &line_str,
                        before,
                        after,
                        replace_all,
                        ignore_case,
                        line_no,
                    ) {
                        Ok((new_line, count)) => {
                            if new_line != line_str {
                                let start_pos = buffer.line_start_position(cursor.pos.line);
                                buffer.delete_text(start_pos, line_str.chars().count(), false);
                                buffer.insert_text(start_pos, &new_line);
                            }
                            buffer.close_undo_block();
                            self.clamp_cursor(buffer, cursor);
                            result.need_redraw = true;
                            result.message = Some(format!("{count} 箇所の式置換を行いました"));
                        }
                        Err(e) => {
                            buffer.close_undo_block();
                            result.message = Some(format!("式置換エラー: {e}"));
                        }
                    }
                }
                return;
            }

            buffer.open_undo_block();
            let mut replace_count = 0;

            if is_global {
                let full_text = buffer.to_string();
                let new_text = if replace_all {
                    full_text.replace(before, after)
                } else {
                    full_text.replacen(before, after, 1)
                };

                if new_text != full_text {
                    let len = buffer.len();
                    buffer.delete_text(0, len, false);
                    buffer.insert_text(0, &new_text);
                    replace_count += 1;
                }
            } else if let Some(line_str) = buffer.line_text(cursor.pos.line) {
                let new_line = if replace_all {
                    line_str.replace(before, after)
                } else {
                    line_str.replacen(before, after, 1)
                };

                if new_line != line_str {
                    let start_pos = buffer.line_start_position(cursor.pos.line);
                    buffer.delete_text(start_pos, line_str.chars().count(), false);
                    buffer.insert_text(start_pos, &new_line);
                    replace_count += 1;
                }
            }

            buffer.close_undo_block();
            self.clamp_cursor(buffer, cursor);
            result.need_redraw = true;
            result.message = Some(format!("{replace_count} 箇所の置換を行いました"));
        }
    }
}
