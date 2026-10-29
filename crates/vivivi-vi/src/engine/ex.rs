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
        } else if let Some(grep_args) = cmd.strip_prefix("grep ") {
            // プロジェクト内Grep検索 (:grep <pattern> [path])
            let parts: Vec<&str> = grep_args.split_whitespace().collect();
            if parts.is_empty() {
                result.message = Some("使用法: :grep <pattern> [path]".to_string());
            } else {
                let pat = parts[0];
                let search_dir = parts.get(1).copied().unwrap_or(".");
                match vivivi_modes::execute_grep(search_dir, pat, false, &[]) {
                    Ok(session) => {
                        let count = session.matches.len();
                        if let Some(first) = session.current_match() {
                            cursor.pos.line = first.lineno.saturating_sub(1);
                            cursor.pos.col = first.col.saturating_sub(1);
                            result.need_redraw = true;
                        }
                        self.grep_session = Some(session);
                        result.message =
                            Some(format!("Grep完了: {count} 件の一致 (pattern: '{pat}')"));
                    }
                    Err(e) => {
                        result.message = Some(format!("Grepエラー: {e}"));
                    }
                }
            }
        } else if cmd == "cnext" || cmd == "cn" {
            // Grep結果の次へジャンプ
            if let Some(session) = &mut self.grep_session {
                if let Some(m) = session.next().cloned() {
                    cursor.pos.line = m.lineno.saturating_sub(1);
                    cursor.pos.col = m.col.saturating_sub(1);
                    result.need_redraw = true;
                    result.message = Some(format!(
                        "[{}/{}] {}:{} {}",
                        session.current_idx + 1,
                        session.matches.len(),
                        m.file_path.display(),
                        m.lineno,
                        m.line_text.trim()
                    ));
                }
            } else {
                result.message =
                    Some("有効な Grep 結果がありません (:grep で検索してください)".to_string());
            }
        } else if cmd == "cprev" || cmd == "cp" {
            // Grep結果の前へジャンプ
            if let Some(session) = &mut self.grep_session {
                if let Some(m) = session.prev().cloned() {
                    cursor.pos.line = m.lineno.saturating_sub(1);
                    cursor.pos.col = m.col.saturating_sub(1);
                    result.need_redraw = true;
                    result.message = Some(format!(
                        "[{}/{}] {}:{} {}",
                        session.current_idx + 1,
                        session.matches.len(),
                        m.file_path.display(),
                        m.lineno,
                        m.line_text.trim()
                    ));
                }
            } else {
                result.message =
                    Some("有効な Grep 結果がありません (:grep で検索してください)".to_string());
            }
        } else if cmd == "todo" || cmd == "undone" {
            // コード内 ToDo 抽出 (:todo / :undone)
            let items =
                vivivi_modes::scan_buffer_todos(&buffer.to_string(), Some(&self.current_file_path));
            let count = items.len();
            if let Some(first) = items.first() {
                cursor.pos.line = first.lineno.saturating_sub(1);
                cursor.pos.col = first.col.saturating_sub(1);
                result.need_redraw = true;
                result.message = Some(format!(
                    "ToDo: {count} 件見つかりました (先頭: [{}] {})",
                    first.tag.as_str(),
                    first.message
                ));
            } else {
                result.message = Some("ToDo タグは見つかりませんでした".to_string());
            }
            self.todo_items = items;
        } else if cmd == "fold" || cmd == "foldindent" {
            // インデント折り畳み自動計算
            self.fold_manager
                .compute_indent_folds(&buffer.to_string(), 4);
            result.need_redraw = true;
            result.message = Some(format!(
                "コード折り畳み: {} 箇所のブロックを検出しました",
                self.fold_manager.folds.len()
            ));
        } else if cmd == "filer" || cmd == "explore" {
            // 2ペインファイラーの初期化
            let dir = if self.current_file_path.is_empty() {
                "."
            } else {
                std::path::Path::new(&self.current_file_path)
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or(".")
            };
            self.filer = Some(vivivi_modes::TwoPaneFiler::new(dir));
            result.need_redraw = true;
            result.message = Some("2ペインファイラーを起動しました (Tabでペイン切替)".to_string());
        } else if cmd == "difflog" || cmd == "blame" {
            // 世代バックアップログの確認
            let log_path = vivivi_modes::DiffLog::difflog_path(&self.current_file_path);
            match vivivi_modes::DiffLog::load_from_file(&log_path) {
                Ok(log) => {
                    let gen_count = log.entries.len();
                    let blame = log.compute_blame(&buffer.to_string());
                    let cur_line = cursor.pos.line + 1;
                    let line_info = blame.get(cursor.pos.line);
                    let info_str = line_info
                        .map(|b| format!("第{}世代 ({})", b.generation, b.timestamp))
                        .unwrap_or_else(|| "未記録".to_string());
                    result.message = Some(format!(
                        "DiffLog: 蓄積世代数: {gen_count} 件 / {cur_line}行目の更新世代: {info_str}"
                    ));
                    self.difflog = Some(log);
                }
                Err(e) => {
                    result.message = Some(format!("DiffLog 読み込みエラー: {e}"));
                }
            }
        } else if cmd == "preview" || cmd == "print" {
            // 印刷プレビュー生成
            let settings = vivivi_modes::PrintSettings::default();
            let pages = vivivi_modes::paginate_text(
                &buffer.to_string(),
                &self.current_file_path,
                "2026-10-14",
                &settings,
            );
            let preview_text = vivivi_modes::format_preview_text(&pages);
            result.preview_requested = Some(preview_text);
            result.message = Some(format!(
                "印刷プレビューを生成しました (全 {} ページ)",
                pages.len()
            ));
        } else if cmd == "maketags" || cmd == "tags gen" {
            // プロジェクト内tags一括生成
            let pm = vivivi_modes::ProjectManager::new(".");
            match pm.generate_tags() {
                Ok(count) => {
                    result.message = Some(format!(
                        "tags ファイルを生成しました ({count} 件のシンボル)"
                    ));
                }
                Err(e) => {
                    result.message = Some(format!("tags 生成エラー: {e}"));
                }
            }
        } else if cmd == "undofile" {
            // 永続Undoの手動保存
            if self.current_file_path.is_empty() {
                result.message = Some("ファイル名が未指定です".to_string());
            } else {
                match buffer.save_undofile(&self.current_file_path) {
                    Ok(()) => {
                        result.message = Some("永続 undofile を保存しました".to_string());
                    }
                    Err(e) => {
                        result.message = Some(format!("undofile 保存エラー: {e}"));
                    }
                }
            }
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

            let line_range = if is_global {
                0..buffer.line_count()
            } else {
                cursor.pos.line..(cursor.pos.line + 1).min(buffer.line_count())
            };

            for line_idx in line_range {
                if let Some(line_str) = buffer.line_text(line_idx) {
                    let matches_in_line = line_str.matches(before).count();
                    if matches_in_line == 0 {
                        continue;
                    }

                    let (new_line, replaced) = if replace_all {
                        (line_str.replace(before, after), matches_in_line)
                    } else {
                        // 行内で最初の1箇所のみ置換
                        (line_str.replacen(before, after, 1), 1)
                    };

                    if new_line != line_str {
                        let start_pos = buffer.line_start_position(line_idx);
                        buffer.delete_text(start_pos, line_str.chars().count(), false);
                        buffer.insert_text(start_pos, &new_line);
                        replace_count += replaced;
                    }
                }
            }

            buffer.close_undo_block();
            self.clamp_cursor(buffer, cursor);
            result.need_redraw = true;
            result.message = Some(format!("{replace_count} 箇所の置換を行いました"));
        }
    }
}
