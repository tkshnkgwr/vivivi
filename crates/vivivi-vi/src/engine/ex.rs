//! exコマンド（`:w`, `:q`, `:%s` など）およびバッファ内検索処理モジュール。

use super::{ViEngine, ViExecutionResult};
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

        if forward {
            // 現在位置の後方を検索
            let target = if cur_pos + 1 < full_text.chars().count() {
                cur_pos + 1
            } else {
                0
            };

            // 全テキストの char インデックスで検索
            let found = full_text
                .char_indices()
                .filter(|(idx, _)| *idx >= target)
                .find(|(idx, _)| full_text[*idx..].starts_with(pat))
                .or_else(|| {
                    // ラップアラウンドして先頭から検索
                    full_text
                        .char_indices()
                        .find(|(idx, _)| full_text[*idx..].starts_with(pat))
                });

            if let Some((byte_idx, _)) = found {
                let char_idx = full_text[..byte_idx].chars().count();
                cursor.pos = buffer.position_to_cursor(char_idx);
                result.need_redraw = true;
            } else {
                result.message = Some(format!("パターンが見つかりません: {pat}"));
            }
        } else {
            // 現在位置より前方を後方検索
            let found = full_text
                .char_indices()
                .filter(|(idx, _)| *idx < cur_pos)
                .rfind(|(idx, _)| full_text[*idx..].starts_with(pat))
                .or_else(|| {
                    // ラップアラウンドして末尾から検索
                    full_text
                        .char_indices()
                        .rfind(|(idx, _)| full_text[*idx..].starts_with(pat))
                });

            if let Some((byte_idx, _)) = found {
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

            if before.is_empty() {
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
