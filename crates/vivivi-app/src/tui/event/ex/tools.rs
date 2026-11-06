//! 外部ツール、プレビュー、検索およびナビゲーションコマンド処理モジュール (`:grep`, `:diff`, `:tags`, `:todo` など)。

use crate::tui::app::App;

/// ツール・プレビュー・検索関連の ex コマンドを処理します。
///
/// コマンドを処理した場合は `true`、合致しなかった場合は `false` を返します。
pub(crate) fn handle_tools_command(app: &mut App, trimmed: &str) -> bool {
    // :r !<cmd> または :read !<cmd>
    if let Some(cmd) = trimmed
        .strip_prefix("r !")
        .or_else(|| trimmed.strip_prefix("read !"))
    {
        let cmd = cmd.trim();
        match app.read_command_into_buffer(cmd, None) {
            Ok(count) => {
                app.status_message = Some(format!(":r !{cmd} ➔ {count} 行を挿入しました"));
            }
            Err(e) => {
                app.status_message = Some(format!("コマンド読込エラー: {e}"));
            }
        }
        return true;
    }

    // :%!sort などのバッファ全体外部コマンドフィルター置換
    if let Some(cmd) = trimmed.strip_prefix("%!") {
        let cmd = cmd.trim();
        let line_count = app.buffer.line_count();
        match app.filter_range_with_command(cmd, 0, line_count.saturating_sub(1)) {
            Ok(count) => {
                app.status_message = Some(format!(":%!{cmd} ➔ {count} 行に置換しました"));
            }
            Err(e) => {
                app.status_message = Some(format!("フィルターエラー: {e}"));
            }
        }
        return true;
    }

    // :'<,'>!sort などの選択範囲外部コマンドフィルター置換
    if let Some(cmd) = trimmed.strip_prefix("'<,'>!") {
        let cmd = cmd.trim();
        let (start, end) = match app.cursor.anchor {
            Some(anchor) => (
                anchor.line.min(app.cursor.pos.line),
                anchor.line.max(app.cursor.pos.line),
            ),
            None => (app.cursor.pos.line, app.cursor.pos.line),
        };
        match app.filter_range_with_command(cmd, start, end) {
            Ok(count) => {
                app.status_message = Some(format!(":'<,'>!{cmd} ➔ {count} 行に置換しました"));
            }
            Err(e) => {
                app.status_message = Some(format!("フィルターエラー: {e}"));
            }
        }
        return true;
    }

    // :cmdexec / :cmd (外部コマンド実行ダイアログ起動)
    if trimmed == "cmdexec" || trimmed == "cmd" {
        app.open_cmd_exec_dialog();
        app.status_message = Some("外部コマンド実行ダイアログを開きました".to_string());
        return true;
    }

    // --- 末尾空白一括削除 (:trim, :%trim, :'<,'>trim, :10,20trim) ---
    if trimmed.starts_with("trim")
        || trimmed.starts_with("%trim")
        || trimmed.starts_with("'<,'>trim")
        || (trimmed.ends_with("trim") && trimmed.contains(','))
    {
        let (range_opt, rest) = if let Some(r) = trimmed.strip_prefix("%trim") {
            (None, r.trim())
        } else if let Some(r) = trimmed.strip_prefix("'<,'>trim") {
            let (start, end) = match app.cursor.anchor {
                Some(anchor) => (
                    anchor.line.min(app.cursor.pos.line),
                    anchor.line.max(app.cursor.pos.line),
                ),
                None => (app.cursor.pos.line, app.cursor.pos.line),
            };
            (Some((start, end)), r.trim())
        } else if let Some(r) = trimmed.strip_prefix("trim") {
            (None, r.trim())
        } else if let Some((range_part, r)) = trimmed.split_once("trim") {
            if let Some((s_str, e_str)) = range_part.split_once(',') {
                let s = s_str.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
                let e = e_str
                    .trim()
                    .parse::<usize>()
                    .unwrap_or(s + 1)
                    .saturating_sub(1);
                (Some((s, e)), r.trim())
            } else {
                (None, r.trim())
            }
        } else {
            (None, "")
        };

        let kind = match rest {
            "half" => vivivi_modes::trim::TrimKind::Half,
            "zenkaku" => vivivi_modes::trim::TrimKind::Zenkaku,
            _ => vivivi_modes::trim::TrimKind::All,
        };

        let total_lines = app.buffer.line_count();
        let modified = app.trim_trailing_whitespace(range_opt, kind);
        if modified > 0 {
            app.status_message = Some(format!(
                "Trim: {} 行の末尾空白を削除しました (全 {} 行)",
                modified, total_lines
            ));
        } else {
            app.status_message = Some("Trim: 末尾空白はありませんでした".to_string());
        }
        return true;
    }
    // --- 外部コマンド実行・選択テキストパイプ連携 (:!, :w !) ---
    if let Some(cmd) = trimmed.strip_prefix("w !") {
        let cmd = cmd.trim();
        let input_text = if let Some(sel) = app.get_selected_text() {
            sel
        } else {
            app.buffer.to_string()
        };
        match app.pipe_command_with_input(cmd, &input_text) {
            Ok(output) => {
                let first_line = output.lines().next().unwrap_or("[出力なし]");
                app.status_message = Some(format!(":w !{cmd} ➔ {first_line}"));
            }
            Err(e) => {
                app.status_message = Some(format!("コマンド実行エラー: {e}"));
            }
        }
        return true;
    }

    if trimmed == "!" {
        match app.execute_selected_text() {
            Ok(output) => {
                let first_line = output.lines().next().unwrap_or("[完了]");
                app.status_message = Some(format!("選択テキスト実行結果: {first_line}"));
            }
            Err(e) => {
                app.status_message = Some(format!("実行エラー: {e}"));
            }
        }
        return true;
    }

    if let Some(cmd) = trimmed.strip_prefix('!') {
        let cmd = cmd.trim();
        if !cmd.is_empty() {
            let res = if app.get_selected_text().is_some() {
                app.pipe_selected_text(cmd)
            } else {
                app.execute_shell_command(cmd)
            };
            match res {
                Ok(output) => {
                    let first_line = output.lines().next().unwrap_or("[完了]");
                    app.status_message = Some(format!("!{cmd} ➔ {first_line}"));
                }
                Err(e) => {
                    app.status_message = Some(format!("コマンド実行エラー: {e}"));
                }
            }
            return true;
        }
    }

    match trimmed {
        // --- インラインCSV/TSVモードトグル (:csv, :tsv) ---
        "csv" | "tsv" => {
            app.is_csv_active = !app.is_csv_active;
            let status = if app.is_csv_active {
                "CSV モード: 有効 [CSV] (カンマ/タブ列自動整列)"
            } else {
                "CSV モード: 無効 (通常テキスト表示)"
            };
            app.status_message = Some(status.to_string());
            true
        }

        // --- 罫線モードトグル (:keisen) ---
        "keisen" => {
            app.keisen_mode.toggle();
            let status = if app.keisen_mode.is_active {
                "罫線モード: 有効 [罫線] (Ctrl+矢印で描画・十字自動結合, F5で終了)"
            } else {
                "罫線モード: 無効"
            };
            app.status_message = Some(status.to_string());
            true
        }

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
                    if app.buffer.is_modified() {
                        app.status_message = Some(
                            "E37: 前回の変更が保存されていません (:w で保存するか :tabnew で開いてください)"
                                .to_string(),
                        );
                        return true;
                    }
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
                    if app.buffer.is_modified() {
                        app.status_message = Some(
                            "E37: 前回の変更が保存されていません (:w で保存するか :tabnew で開いてください)"
                                .to_string(),
                        );
                        return true;
                    }
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

        // --- サイドバー / ファイルツリー (:tree, :Explore) ---
        "tree" | "Explore" => {
            app.toggle_tree_request = true;
            app.status_message = Some("ファイルツリーの表示をトグルしました".to_string());
            true
        }

        // --- プロジェクト可視化グラフ (:visualize, :graph) ---
        "visualize" | "graph" => {
            app.show_visualize_graph = !app.show_visualize_graph;
            let status = if app.show_visualize_graph {
                "プロジェクト可視化グラフを開きました (Esc でエディタに戻る)"
            } else {
                "プロジェクト可視化グラフを閉じました"
            };
            app.status_message = Some(status.to_string());
            true
        }

        _ => false,
    }
}
