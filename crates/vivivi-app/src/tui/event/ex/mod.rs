//! exコマンドのパースおよび実行ディスパッチモジュール。

pub(crate) mod settings;
pub(crate) mod substitute;
pub(crate) mod text;
pub(crate) mod tools;

use super::*;
use substitute::execute_substitute_command;

/// exコマンドの実行処理 (:w, :q, :wq, :s, etc.)
pub fn execute_ex_command(app: &mut App, cmd: &str) {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return;
    }

    // 1. 各専門サブモジュールにディスパッチ
    if tools::handle_tools_command(app, trimmed)
        || settings::handle_settings_command(app, trimmed)
        || text::handle_text_command(app, trimmed)
    {
        return;
    }

    // 2. コアエディタ操作・ウィンドウ・ファイル管理コマンド
    match trimmed {
        "w" => {
            if let Err(e) = app.save_file() {
                app.status_message = Some(format!("保存エラー: {e}"));
            }
        }
        "q" | "qa" | "qall" => {
            // カレントバッファおよび他タブのいずれかに未保存変更があれば終了を中断
            let has_modified_tab =
                app.buffer.is_modified() || app.tabs.iter().any(|t| t.buffer.is_modified());
            if has_modified_tab {
                app.status_message = Some(
                    "E37: 前回の変更が保存されていません (! を追加して :q! または :qa! で強制終了)"
                        .to_string(),
                );
            } else {
                app.remove_swap_file();
                app.should_quit = true;
            }
        }
        "q!" | "qa!" | "qall!" => {
            app.remove_swap_file();
            app.should_quit = true;
        }
        "wq" | "x" => {
            if let Err(e) = app.save_file() {
                app.status_message = Some(format!("保存エラー: {e}"));
            } else {
                app.remove_swap_file();
                app.should_quit = true;
            }
        }

        // --- クラッシュリカバリ・スワップファイル操作 (:recover, :swap) ---
        "recover" => {
            if let Err(e) = app.recover_from_swap() {
                app.status_message = Some(format!("復元エラー: {e}"));
            }
        }
        "swap" => {
            if let Some(ref path) = app.file_path {
                let swp = vivivi_core::SwapFile::swap_path_for(path);
                if swp.exists() {
                    app.status_message = Some(format!("スワップファイル検出: {}", swp.display()));
                } else {
                    app.save_swap_file();
                    app.status_message =
                        Some(format!("スワップファイルを作成しました: {}", swp.display()));
                }
            } else {
                app.status_message =
                    Some("無名バッファにはスワップファイルがありません".to_string());
            }
        }

        // --- マルチタブ操作 (:tabnew, :tabedit, :tabclose, :tabnext, :tabprev, :tabonly) ---
        "tabnew" | "tabe" | "tabedit" => {
            app.new_tab(None);
        }
        _ if trimmed.starts_with("tabnew ")
            || trimmed.starts_with("tabe ")
            || trimmed.starts_with("tabedit ") =>
        {
            let path_str = trimmed
                .strip_prefix("tabnew ")
                .or_else(|| trimmed.strip_prefix("tabe "))
                .or_else(|| trimmed.strip_prefix("tabedit "))
                .unwrap_or("")
                .trim();
            let path = std::path::PathBuf::from(path_str);
            app.new_tab(Some(path));
        }
        "tabclose" | "tabc" => {
            if let Err(e) = app.close_tab_forced(false) {
                app.status_message = Some(e.to_string());
            }
        }
        "tabclose!" | "tabc!" => {
            if let Err(e) = app.close_tab_forced(true) {
                app.status_message = Some(e.to_string());
            }
        }
        "tabnext" | "tabn" => {
            app.next_tab();
        }
        "tabprev" | "tabp" | "tabprevious" => {
            app.prev_tab();
        }
        "tabonly" | "tabo" => {
            if let Err(e) = app.close_other_tabs_forced(false) {
                app.status_message = Some(e.to_string());
            }
        }
        "tabonly!" | "tabo!" => {
            if let Err(e) = app.close_other_tabs_forced(true) {
                app.status_message = Some(e.to_string());
            }
        }
        _ if trimmed.starts_with("w ") => {
            let path_str = trimmed[2..].trim();
            let path = std::path::PathBuf::from(path_str);
            app.syntax_lang = vivivi_modes::SyntaxLang::from_path(&path);
            app.file_path = Some(path);
            if let Err(e) = app.save_file() {
                app.status_message = Some(format!("保存エラー: {e}"));
            } else {
                app.refresh_outline();
            }
        }

        // --- 文字列置換 & 式置換 (:%s/before/after/[flags], :s/before/after/[flags]) ---
        _ if trimmed.starts_with("%s/")
            || trimmed.starts_with("s/")
            || trimmed.starts_with("%s") =>
        {
            execute_substitute_command(app, trimmed);
        }

        // --- 指定行ジャンプ (:行番号 例: :42, :1) ---
        _ if trimmed.chars().all(|c| c.is_ascii_digit()) => {
            if let Ok(line_num) = trimmed.parse::<usize>() {
                let total = app.buffer.line_count();
                let target_line = line_num.saturating_sub(1).min(total.saturating_sub(1));
                app.cursor.pos.line = target_line;
                app.cursor.pos.col = 0;
                app.clamp_cursor();
                // ターゲット行が画面中央に来るようスクロールを同期
                app.scroll.0 = target_line.saturating_sub(12);
                if let Some(win) = app.windows.get_mut(app.active_window) {
                    win.cursor = app.cursor.clone();
                    win.scroll = app.scroll;
                }
                app.status_message = Some(format!("{} 行目へ移動しました", target_line + 1));
            }
        }

        // --- AI連携 (:ai [instruction]) ---
        _ if trimmed.starts_with("ai ") || trimmed == "ai" => {
            execute_ai_command(app, trimmed);
        }

        // --- Undo / Redo (:undo, :u, :redo) ---
        "undo" | "u" => {
            app.feed_vi_char('u');
        }
        "redo" => {
            app.feed_vi_char('\x12');
        }

        // --- 画面分割 (:split, :vsplit, :close, :only) ---
        "split" | "sp" => {
            app.split_horizontal();
        }
        "vsplit" | "vsp" | "vs" => {
            app.split_vertical();
        }
        "close" | "c" => {
            app.close_window();
        }
        "only" => {
            app.only_window();
        }

        // --- アウトライン (:outline) ---
        "outline" => {
            app.toggle_outline();
        }

        // --- ヘルプ (:help, :h) ---
        "help" | "h" => {
            app.open_help(None);
        }
        _ if trimmed.starts_with("help ") || trimmed.starts_with("h ") => {
            let topic = trimmed.split_whitespace().nth(1);
            app.open_help(topic);
        }

        // --- 最近使ったファイル & お気に入り (:recent, :mru, :fav, :favorite, :e) ---
        "recent" | "mru" | "favs" => {
            app.show_recent_modal = true;
            app.selected_recent_idx = 0;
            app.status_message = Some(
                "最近使ったファイル (j/k:選択, Enter:開く, f:お気に入り, d:削除, Esc:閉じる)"
                    .to_string(),
            );
        }
        "fav" | "favorite" | "addfav" => {
            app.toggle_favorite_current_file();
        }
        _ if trimmed.starts_with("e ")
            || trimmed.starts_with("e! ")
            || trimmed.starts_with("edit ")
            || trimmed.starts_with("edit! ")
            || trimmed == "e"
            || trimmed == "e!"
            || trimmed == "edit"
            || trimmed == "edit!" =>
        {
            let is_forced = trimmed.starts_with("e! ")
                || trimmed.starts_with("edit! ")
                || trimmed == "e!"
                || trimmed == "edit!";

            if !is_forced && app.buffer.is_modified() {
                app.status_message = Some(
                    "E37: 前回の変更が保存されていません (! を追加して :e! で強制オープン)"
                        .to_string(),
                );
                return;
            }

            let rest = if is_forced {
                trimmed
                    .strip_prefix("edit!")
                    .or_else(|| trimmed.strip_prefix("e!"))
                    .unwrap_or("")
                    .trim()
            } else {
                trimmed
                    .strip_prefix("edit")
                    .or_else(|| trimmed.strip_prefix("e"))
                    .unwrap_or("")
                    .trim()
            };

            let mut forced_enc = None;
            let mut file_parts = Vec::new();

            for part in rest.split_whitespace() {
                if let Some(enc_name) = part.strip_prefix("++enc=") {
                    if let Some(enc) = vivivi_core::CharEncoding::parse_enc(enc_name) {
                        forced_enc = Some(enc);
                    } else {
                        app.status_message =
                            Some(format!("未対応のエンコーディングです: {enc_name}"));
                        return;
                    }
                } else if !part.is_empty() {
                    file_parts.push(part);
                }
            }

            let file_target = if file_parts.is_empty() {
                None
            } else {
                Some(file_parts.join(" "))
            };

            if let Some(file_path_str) = file_target {
                let clean_path = file_path_str.trim_matches('"').trim_matches('\'');
                let path = std::path::PathBuf::from(clean_path);
                if let Err(e) = app.open_file_with_encoding(path, forced_enc) {
                    app.status_message = Some(format!("ファイル読込エラー: {e}"));
                }
            } else if let Some(enc) = forced_enc {
                if let Err(e) = app.reload_with_encoding(enc) {
                    app.status_message = Some(format!("再読込エラー: {e}"));
                }
            } else if let Some(ref path) = app.file_path.clone()
                && let Err(e) = app.open_file(path.clone())
            {
                app.status_message = Some(format!("再読込エラー: {e}"));
            }
        }

        _ => {
            app.status_message = Some(format!("未対応のexコマンド: {trimmed}"));
        }
    }
}

/// AI連携exコマンド (`:ai <instruction>`) の実行
fn execute_ai_command(app: &mut App, cmd: &str) {
    let instruction = cmd.strip_prefix("ai").unwrap_or("").trim();
    if instruction.is_empty() {
        app.status_message = Some("使用法: :ai <編集指示プロンプト>".to_string());
        return;
    }

    let full_text = app.buffer.to_string();
    let prompt = format!(
        "以下のソースコードに対して指示を実行し、変更後の完全なコードのみを出力してください。\n\n【指示】\n{}\n\n【コード】\n{}",
        instruction, full_text
    );

    // AI呼び出し用プレースホルダーメッセージ（外部API未設定時は案内を表示）
    app.status_message = Some(format!(
        "AIリクエスト送信中: '{}' (全 {} 文字)",
        instruction,
        prompt.len()
    ));
}
