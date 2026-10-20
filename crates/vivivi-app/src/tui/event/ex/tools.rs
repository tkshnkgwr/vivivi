//! 外部ツール、プレビュー、検索およびナビゲーションコマンド処理モジュール (`:grep`, `:diff`, `:tags`, `:todo` など)。

use crate::tui::app::App;

/// ツール・プレビュー・検索関連の ex コマンドを処理します。
///
/// コマンドを処理した場合は `true`、合致しなかった場合は `false` を返します。
pub(crate) fn handle_tools_command(app: &mut App, trimmed: &str) -> bool {
    match trimmed {
        // --- ToDo抽出・一覧プレビュー (:todo, :undone) ---
        "todo" | "undone" => {
            let content = app.buffer.to_string();
            let file_name = app.file_path.as_ref().and_then(|p| p.to_str());
            let todos = vivivi_modes::scan_buffer_todos(&content, file_name);
            if todos.is_empty() {
                app.status_message =
                    Some("バッファ内に ToDo タグは見つかりませんでした".to_string());
            } else {
                let mut lines = vec![
                    format!("# バッファ内 ToDo 一覧 (全 {} 件)", todos.len()),
                    "".to_string(),
                ];
                for t in &todos {
                    lines.push(format!("* 行 {:>4} [{:?}]: {}", t.lineno, t.tag, t.message));
                }
                app.preview_lines = lines;
                app.show_preview = true;
                app.preview_scroll = 0;
                app.status_message = Some(format!(
                    "ToDo: {} 件を抽出しました (Escで閉じる)",
                    todos.len()
                ));
            }
            true
        }

        // --- 差分比較プレビュー (:diff [path]) ---
        "diff" => {
            if let Some(ref path) = app.file_path {
                if let Ok(orig) = std::fs::read_to_string(path) {
                    let cur = app.buffer.to_string();
                    let diff_text = vivivi_modes::create_unified_diff(
                        &orig,
                        &cur,
                        &format!("a/{}", path.display()),
                        &format!("b/{}", path.display()),
                    );
                    app.preview_lines = diff_text.lines().map(|s| s.to_string()).collect();
                    app.show_preview = true;
                    app.preview_scroll = 0;
                    app.status_message = Some("保存前との差分を表示中 (Escで閉じる)".to_string());
                } else {
                    app.status_message = Some("ディスク上の元ファイルが読み込めません".to_string());
                }
            } else {
                app.status_message = Some("ファイル名のないバッファはdiffできません".to_string());
            }
            true
        }
        _ if trimmed.starts_with("diff ") => {
            let target_str = trimmed.strip_prefix("diff ").unwrap_or("").trim();
            let target_path = std::path::Path::new(target_str);
            if let Ok(other_content) = std::fs::read_to_string(target_path) {
                let cur = app.buffer.to_string();
                let cur_name = app
                    .file_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "current".to_string());
                let diff_text = vivivi_modes::create_unified_diff(
                    &other_content,
                    &cur,
                    &format!("a/{}", target_path.display()),
                    &format!("b/{cur_name}"),
                );
                app.preview_lines = diff_text.lines().map(|s| s.to_string()).collect();
                app.show_preview = true;
                app.preview_scroll = 0;
                app.status_message = Some(format!(
                    "'{}' との差分を表示中 (Escで閉じる)",
                    target_path.display()
                ));
            } else {
                app.status_message = Some(format!("指定ファイルが読み込めません: {target_str}"));
            }
            true
        }

        // --- 差分ログ世代一覧 (:difflog) ---
        "difflog" => {
            if let Some(ref path) = app.file_path {
                let log_res = vivivi_modes::DiffLog::load_from_file(path);
                if let Ok(difflog) = log_res {
                    let mut lines = vec![
                        format!("# 差分ログ世代履歴 (全 {} 世代)", difflog.entries.len()),
                        "".to_string(),
                    ];
                    for e in &difflog.entries {
                        lines.push(format!(
                            "* [世代 #{}] {} - {}",
                            e.generation, e.timestamp, e.comment
                        ));
                    }
                    app.preview_lines = lines;
                    app.show_preview = true;
                    app.preview_scroll = 0;
                    app.status_message = Some(format!(
                        "差分ログ: {} 世代 (Escで閉じる)",
                        difflog.entries.len()
                    ));
                } else {
                    app.status_message = Some("差分ログ (.difflog) が存在しません".to_string());
                }
            } else {
                app.status_message =
                    Some("ファイル名のないバッファはdifflogがありません".to_string());
            }
            true
        }

        // --- 2ペイン インラインファイラー (:filer [path], :explore [path]) ---
        "filer" | "explore" => {
            app.open_filer(None);
            true
        }
        _ if trimmed.starts_with("filer ") || trimmed.starts_with("explore ") => {
            let path_str = trimmed
                .strip_prefix("filer ")
                .or_else(|| trimmed.strip_prefix("explore "))
                .unwrap_or("")
                .trim();
            let path = std::path::Path::new(path_str);
            app.open_filer(Some(path));
            true
        }

        // --- 印刷プレビュー (:preview, :print) ---
        "preview" | "print" => {
            let content = app.buffer.to_string();
            let file_name = app
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled");
            let date_str = crate::tui::app::current_date_str();
            let settings = vivivi_modes::PrintSettings::default();
            let pages = vivivi_modes::paginate_text(&content, file_name, &date_str, &settings);
            let preview_text = vivivi_modes::format_preview_text(&pages);
            app.preview_lines = preview_text.lines().map(|s| s.to_string()).collect();
            app.show_preview = true;
            app.preview_scroll = 0;
            app.status_message =
                Some("印刷プレビュー [p:PDF保存, j/k:スクロール, q/Esc:閉じる]".to_string());
            true
        }

        // --- PDF直接出力 (:pdf [path]) ---
        cmd if cmd == "pdf" || cmd.starts_with("pdf ") => {
            let path_arg = trimmed.strip_prefix("pdf").unwrap_or("").trim();
            let target = if path_arg.is_empty() {
                None
            } else {
                Some(path_arg)
            };
            if let Err(e) = app.export_pdf(target) {
                app.status_message = Some(format!("PDF出力エラー: {e}"));
            }
            true
        }

        // --- プロジェクト内Grep検索 (:grep <pattern> [path], :cnext, :cn, :cprev, :cp) ---
        _ if trimmed.starts_with("grep ") => {
            let args = trimmed.strip_prefix("grep ").unwrap_or("").trim();
            let parts: Vec<&str> = args.split_whitespace().collect();
            if parts.is_empty() {
                app.status_message = Some("使用法: :grep <pattern> [path]".to_string());
            } else {
                let pat = parts[0];
                let search_dir = parts.get(1).copied().unwrap_or(".");
                match vivivi_modes::execute_grep(search_dir, pat, false, &[]) {
                    Ok(session) => {
                        let count = session.matches.len();
                        if count == 0 {
                            app.status_message = Some(format!(
                                "Grep: 一致は見つかりませんでした (pattern: '{pat}')"
                            ));
                        } else {
                            if let Some(first) = session.current_match()
                                && let Some(ref cur_path) = app.file_path
                                && first.file_path == *cur_path
                            {
                                app.cursor.pos.line = first.lineno.saturating_sub(1);
                                app.cursor.pos.col = first.col.saturating_sub(1);
                                app.clamp_cursor();
                            }
                            let mut lines = vec![
                                format!("# Grep 検索結果: '{pat}' (全 {count} 件)"),
                                "".to_string(),
                            ];
                            for (idx, m) in session.matches.iter().enumerate() {
                                lines.push(format!(
                                    "{:>3}: {}:{} | {}",
                                    idx + 1,
                                    m.file_path.display(),
                                    m.lineno,
                                    m.line_text.trim()
                                ));
                            }
                            app.preview_lines = lines;
                            app.show_preview = true;
                            app.preview_scroll = 0;
                            app.status_message = Some(format!(
                                "Grep完了: {count} 件の一致 (pattern: '{pat}', :cnext/:cprev で巡回)"
                            ));
                        }
                        app.vi.grep_session = Some(session);
                    }
                    Err(e) => {
                        app.status_message = Some(format!("Grepエラー: {e}"));
                    }
                }
            }
            true
        }
        "cnext" | "cn" => {
            let info = if let Some(session) = &mut app.vi.grep_session {
                session
                    .next()
                    .cloned()
                    .map(|m| (m, session.current_idx + 1, session.matches.len()))
            } else {
                None
            };
            if let Some((m, cur_idx, total)) = info {
                let need_open = app
                    .file_path
                    .as_ref()
                    .map(|p| *p != m.file_path)
                    .unwrap_or(true);
                if need_open {
                    let _ = app.open_file(m.file_path.clone());
                }
                app.cursor.pos.line = m.lineno.saturating_sub(1);
                app.cursor.pos.col = m.col.saturating_sub(1);
                app.clamp_cursor();
                app.status_message = Some(format!(
                    "[{cur_idx}/{total}] {}:{} | {}",
                    m.file_path.display(),
                    m.lineno,
                    m.line_text.trim()
                ));
            } else {
                app.status_message =
                    Some("有効な Grep 結果がありません (:grep で検索してください)".to_string());
            }
            true
        }
        "cprev" | "cp" => {
            let info = if let Some(session) = &mut app.vi.grep_session {
                session
                    .prev()
                    .cloned()
                    .map(|m| (m, session.current_idx + 1, session.matches.len()))
            } else {
                None
            };
            if let Some((m, cur_idx, total)) = info {
                let need_open = app
                    .file_path
                    .as_ref()
                    .map(|p| *p != m.file_path)
                    .unwrap_or(true);
                if need_open {
                    let _ = app.open_file(m.file_path.clone());
                }
                app.cursor.pos.line = m.lineno.saturating_sub(1);
                app.cursor.pos.col = m.col.saturating_sub(1);
                app.clamp_cursor();
                app.status_message = Some(format!(
                    "[{cur_idx}/{total}] {}:{} | {}",
                    m.file_path.display(),
                    m.lineno,
                    m.line_text.trim()
                ));
            } else {
                app.status_message =
                    Some("有効な Grep 結果がありません (:grep で検索してください)".to_string());
            }
            true
        }

        // --- タグジャンプ & タグ生成 (:tag <name>, :tags, :maketags, :tags gen) ---
        _ if trimmed.starts_with("tag ") => {
            let tag = trimmed
                .strip_prefix("tag ")
                .unwrap_or("")
                .trim()
                .to_string();
            let res = app.vi.execute(
                &vivivi_vi::ViCommand::TagJump(Some(tag)),
                &mut app.buffer,
                &mut app.cursor,
            );
            app.status_message = res.message;
            true
        }
        "tags" => {
            let res = app.vi.execute(
                &vivivi_vi::ViCommand::ListTags,
                &mut app.buffer,
                &mut app.cursor,
            );
            app.status_message = res.message;
            true
        }
        "maketags" | "tags gen" | "tags generate" => {
            let pm = vivivi_modes::ProjectManager::new(".");
            match pm.generate_tags() {
                Ok(count) => {
                    app.status_message = Some(format!(
                        "tags ファイルを生成しました ({count} 件のシンボル)"
                    ));
                }
                Err(e) => {
                    app.status_message = Some(format!("tags 生成エラー: {e}"));
                }
            }
            true
        }

        // --- マーク一覧表示 (:marks) ---
        "marks" => {
            let res = app.vi.execute(
                &vivivi_vi::ViCommand::ListMarks,
                &mut app.buffer,
                &mut app.cursor,
            );
            app.status_message = res.message;
            true
        }

        // --- 永続Undo手動保存 (:undofile) ---
        "undofile" => {
            if let Some(ref path) = app.file_path {
                match app.buffer.save_undofile(path) {
                    Ok(()) => {
                        app.status_message = Some(format!(
                            "永続 undofile を保存しました: .<filename>.undofile ({})",
                            path.display()
                        ));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("undofile 保存エラー: {e}"));
                    }
                }
            } else {
                app.status_message =
                    Some("ファイル名のないバッファは undofile を保存できません".to_string());
            }
            true
        }

        // --- コード折り畳み自動検出 (:fold, :foldindent) ---
        "fold" | "foldindent" => {
            let content = app.buffer.to_string();
            app.vi.fold_manager.compute_indent_folds(&content, 4);
            let count = app.vi.fold_manager.folds.len();
            app.status_message = Some(format!(
                "コード折り畳み: {count} 箇所のブロックを検出しました (zc:閉じる, zo:開く, za:トグル)"
            ));
            true
        }

        _ => false,
    }
}
