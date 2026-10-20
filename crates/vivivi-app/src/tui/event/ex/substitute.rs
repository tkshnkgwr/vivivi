//! 文字列置換および式置換コマンド処理モジュール (`:s`, `:%s`)。

use crate::tui::app::App;

/// 置換コマンド (`:s/before/after/[flags]`, `:%s/before/after/[flags]`) を実行します。
pub(crate) fn execute_substitute_command(app: &mut App, cmd: &str) {
    let (is_global, spec) = if let Some(rest) = cmd.strip_prefix("%s") {
        (true, rest)
    } else if let Some(rest) = cmd.strip_prefix('s') {
        (false, rest)
    } else {
        return;
    };

    let delim = match spec.chars().next() {
        Some(c) => c,
        None => {
            app.status_message = Some(format!("置換構文エラー (例: :%s/old/new/g): {cmd}"));
            return;
        }
    };
    let body = &spec[delim.len_utf8()..];
    let parts: Vec<&str> = body.split(delim).collect();

    if parts.len() < 2 {
        app.status_message = Some(format!("置換構文エラー (例: :%s/old/new/g): {cmd}"));
        return;
    }

    let before = parts[0];
    let after = parts[1];
    let flags = parts.get(2).copied().unwrap_or("");
    let replace_all = flags.contains('g');
    let is_expr = flags.contains('e');
    let ignore_case = flags.contains('i');

    if before.is_empty() {
        app.status_message = Some("置換対象パターンが空です".to_string());
        return;
    }

    if is_expr {
        // ViVi 3 / ViVi 4 式置換 (:s/.../.../e)
        app.buffer.open_undo_block();
        if is_global {
            let full_text = app.buffer.to_string();
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
                        let len = app.buffer.len();
                        app.buffer.delete_text(0, len, false);
                        app.buffer.insert_text(0, &new_text);
                    }
                    app.buffer.close_undo_block();
                    app.clamp_cursor();
                    app.status_message = Some(format!("{count} 箇所の式置換を行いました"));
                }
                Err(e) => {
                    app.buffer.close_undo_block();
                    app.status_message = Some(format!("式置換エラー: {e}"));
                }
            }
        } else {
            let line_idx = app.cursor.pos.line;
            if let Some(line_str) = app.buffer.get_line(line_idx) {
                let line_no = line_idx + 1;
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
                            let start_pos = app.buffer.line_start_position(line_idx);
                            app.buffer
                                .delete_text(start_pos, line_str.chars().count(), false);
                            app.buffer.insert_text(start_pos, &new_line);
                        }
                        app.buffer.close_undo_block();
                        app.clamp_cursor();
                        app.status_message = Some(format!("{count} 箇所の式置換を行いました"));
                    }
                    Err(e) => {
                        app.buffer.close_undo_block();
                        app.status_message = Some(format!("式置換エラー: {e}"));
                    }
                }
            } else {
                app.buffer.close_undo_block();
                app.status_message = Some("行の取得に失敗しました".to_string());
            }
        }
        return;
    }

    // 通常の正規表現 / 文字列置換
    let regex_result = regex::RegexBuilder::new(before)
        .case_insensitive(ignore_case)
        .build();

    let re = match regex_result {
        Ok(r) => r,
        Err(_) => {
            let escaped = regex::escape(before);
            match regex::RegexBuilder::new(&escaped)
                .case_insensitive(ignore_case)
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    app.status_message = Some(format!("正規表現エラー: {e}"));
                    return;
                }
            }
        }
    };

    app.buffer.open_undo_block();
    if is_global {
        let full_text = app.buffer.to_string();
        let mut count = 0;
        let new_text = if replace_all {
            count = re.find_iter(&full_text).count();
            re.replace_all(&full_text, after).to_string()
        } else {
            if re.is_match(&full_text) {
                count = 1;
            }
            re.replace(&full_text, after).to_string()
        };

        if new_text != full_text {
            let len = app.buffer.len();
            app.buffer.delete_text(0, len, false);
            app.buffer.insert_text(0, &new_text);
        }
        app.buffer.close_undo_block();
        app.clamp_cursor();
        app.status_message = Some(format!("{count} 箇所の置換を行いました"));
    } else {
        let line_idx = app.cursor.pos.line;
        if let Some(line_str) = app.buffer.get_line(line_idx) {
            let mut count = 0;
            let new_line = if replace_all {
                count = re.find_iter(&line_str).count();
                re.replace_all(&line_str, after).to_string()
            } else {
                if re.is_match(&line_str) {
                    count = 1;
                }
                re.replace(&line_str, after).to_string()
            };

            if new_line != line_str {
                let start_pos = app.buffer.line_start_position(line_idx);
                app.buffer
                    .delete_text(start_pos, line_str.chars().count(), false);
                app.buffer.insert_text(start_pos, &new_line);
            }
            app.buffer.close_undo_block();
            app.clamp_cursor();
            app.status_message = Some(format!("{count} 箇所の置換を行いました"));
        } else {
            app.buffer.close_undo_block();
            app.status_message = Some("行の取得に失敗しました".to_string());
        }
    }
}
