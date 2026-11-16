//! モーダルダイアログ・オーバーレイ（ヘルプ・プレビュー・設定・ファイラー・履歴等）のキー処理モジュール。

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::hex::handle_hex_mode_key;
use crate::tui::app::App;

/// 優先モーダル（ヘルプ、プレビュー、外部コマンド、TUI設定、ファイラー、LSPパネル、Hexモード）のキー入力を処理します。
///
/// 処理した場合は `true` を返します。
pub(super) fn handle_primary_modals(app: &mut App, key: KeyEvent, is_esc: bool) -> bool {
    // ヘルプ表示中のキーハンドリング
    if app.show_help {
        if is_esc || key.code == KeyCode::Char('q') {
            app.show_help = false;
            app.status_message = Some("ヘルプを閉じました".to_string());
            return true;
        }
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                app.help_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            KeyCode::PageDown | KeyCode::Char('d') => {
                app.help_scroll += 10;
            }
            KeyCode::PageUp | KeyCode::Char('u') => {
                app.help_scroll = app.help_scroll.saturating_sub(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                app.help_scroll = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                app.help_scroll = app.help_lines.len().saturating_sub(1);
            }
            _ => {}
        }
        return true;
    }

    // 印刷プレビュー表示中のキーハンドリング
    if app.show_preview {
        if is_esc || key.code == KeyCode::Char('q') {
            app.show_preview = false;
            app.status_message = Some("印刷プレビューを閉じました".to_string());
            return true;
        }
        match key.code {
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if let Err(e) = app.export_pdf(None) {
                    app.status_message = Some(format!("PDF出力エラー: {e}"));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.preview_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.preview_scroll = app.preview_scroll.saturating_sub(1);
            }
            KeyCode::PageDown | KeyCode::Char('d') => {
                app.preview_scroll += 10;
            }
            KeyCode::PageUp | KeyCode::Char('u') => {
                app.preview_scroll = app.preview_scroll.saturating_sub(10);
            }
            _ => {}
        }
        return true;
    }

    // 外部コマンド実行ダイアログ (cmdExecDlg) 表示中のキーハンドリング
    if app.show_cmd_exec_dialog {
        if is_esc {
            app.close_cmd_exec_dialog();
            return true;
        }

        match key.code {
            KeyCode::Tab => {
                app.cmd_exec_mode = (app.cmd_exec_mode + 1) % 3;
            }
            KeyCode::Backspace => {
                app.cmd_exec_input.pop();
            }
            KeyCode::Enter => {
                app.execute_cmd_exec_dialog();
            }
            KeyCode::Char(c) => {
                app.cmd_exec_input.push(c);
            }
            _ => {}
        }
        return true;
    }

    // TUI 設定ダイアログ表示中のキーハンドリング
    if app.show_preferences_dialog {
        if is_esc || key.code == KeyCode::Char('q') {
            app.cancel_preferences_dialog();
            return true;
        }

        match key.code {
            KeyCode::Char('s') => {
                app.sync_runtime_to_config();
                let path = crate::config::AppConfig::default_config_path();
                match app.config.save_to(&path) {
                    Ok(()) => {
                        app.status_message =
                            Some(format!("設定を保存しました: {}", path.display()));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("設定の保存に失敗しました: {e}"));
                    }
                }
                app.tui_settings_backup = None;
                app.show_preferences_dialog = false;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.tui_settings_in_items = false;
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                app.tui_settings_in_items = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.tui_settings_in_items {
                    app.tui_settings_category = (app.tui_settings_category + 1).min(5);
                    app.tui_settings_item = 0;
                } else {
                    let max_items = crate::tui::ui::settings::get_category_items(
                        app,
                        app.tui_settings_category,
                    )
                    .len();
                    if max_items > 0 {
                        app.tui_settings_item = (app.tui_settings_item + 1).min(max_items - 1);
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.tui_settings_in_items {
                    app.tui_settings_category = app.tui_settings_category.saturating_sub(1);
                    app.tui_settings_item = 0;
                } else {
                    app.tui_settings_item = app.tui_settings_item.saturating_sub(1);
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if !app.tui_settings_in_items {
                    app.tui_settings_in_items = true;
                } else {
                    toggle_tui_settings_item(app);
                }
            }
            _ => {}
        }
        return true;
    }

    // 2ペインファイラー表示中のキーハンドリング
    if app.show_filer {
        handle_filer_key(app, key, is_esc);
        return true;
    }

    // LSP ホバー表示中のキーハンドリング
    if app.lsp_show_hover && (is_esc || key.code == KeyCode::Char('q')) {
        app.lsp_show_hover = false;
        app.lsp_hover_text = None;
        return true;
    }

    // LSP 診断情報パネル表示中のキーハンドリング
    if app.lsp_show_diagnostics_panel && (is_esc || key.code == KeyCode::Char('q')) {
        app.lsp_show_diagnostics_panel = false;
        return true;
    }

    // バイナリモード (Hex Editor) のキーハンドリング
    if app.is_hex_mode {
        handle_hex_mode_key(app, key, is_esc);
        return true;
    }

    false
}

/// 二次モーダル（最近使ったファイル履歴、クリップボード履歴）のキー入力を処理します。
pub(super) fn handle_secondary_modals(app: &mut App, key: KeyEvent, is_esc: bool) -> bool {
    // 最近使ったファイル & お気に入りモーダル表示中のキーハンドリング
    if app.show_recent_modal {
        let mut all_entries = Vec::new();
        for fav in &app.recent_manager.favorite_files {
            all_entries.push(fav.clone());
        }
        for rec in &app.recent_manager.recent_files {
            if !app.recent_manager.favorite_files.contains(rec) {
                all_entries.push(rec.clone());
            }
        }
        let total_count = all_entries.len();

        if is_esc || key.code == KeyCode::Char('q') {
            app.show_recent_modal = false;
            app.status_message = Some("履歴モーダルを閉じました".to_string());
            return true;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected_recent_idx = app.selected_recent_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if total_count > 0 => {
                app.selected_recent_idx = (app.selected_recent_idx + 1).min(total_count - 1);
            }
            KeyCode::Enter if total_count > 0 => {
                if let Some(target_path) = all_entries.get(app.selected_recent_idx).cloned() {
                    if app.buffer.is_modified() {
                        app.status_message = Some(
                            "E37: 前回の変更が保存されていません。先に保存するか新しいタブ (:tabnew) で開いてください"
                                .to_string(),
                        );
                        return true;
                    }
                    app.show_recent_modal = false;
                    if let Err(e) = app.open_file(target_path) {
                        app.status_message = Some(format!("ファイル読込エラー: {e}"));
                    }
                }
            }
            KeyCode::Char('f') if total_count > 0 => {
                if let Some(target_path) = all_entries.get(app.selected_recent_idx).cloned() {
                    let is_fav = app.recent_manager.toggle_favorite(target_path.clone());
                    let p = crate::tui::app::get_recent_file_path();
                    let _ = app.recent_manager.save_to_file(&p);
                    if is_fav {
                        app.status_message = Some(format!("お気に入りに追加: {:?}", target_path));
                    } else {
                        app.status_message = Some(format!("お気に入りを解除: {:?}", target_path));
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Delete if total_count > 0 => {
                if let Some(target_path) = all_entries.get(app.selected_recent_idx).cloned() {
                    app.recent_manager.remove_recent(&target_path);
                    if app.recent_manager.is_favorite(&target_path) {
                        app.recent_manager.toggle_favorite(target_path.clone());
                    }
                    let p = crate::tui::app::get_recent_file_path();
                    let _ = app.recent_manager.save_to_file(&p);
                    app.status_message = Some(format!("履歴から削除: {:?}", target_path));
                }
            }
            _ => {}
        }
        return true;
    }

    // クリップボード履歴ダイアログ表示中のキーハンドリング
    if app.show_clipboard_hist {
        if is_esc || key.code == KeyCode::Char('q') {
            app.show_clipboard_hist = false;
            return true;
        }
        let history_len = app.vi.registers().history().len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected_cb_idx = app.selected_cb_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if history_len > 0 => {
                app.selected_cb_idx = (app.selected_cb_idx + 1).min(history_len - 1);
            }
            KeyCode::Enter if history_len > 0 => {
                app.paste_clipboard_item(app.selected_cb_idx);
            }
            _ => {}
        }
        return true;
    }

    false
}

/// 2ペインファイラー表示中のキーハンドリング
fn handle_filer_key(app: &mut App, key: KeyEvent, is_esc: bool) {
    if is_esc || key.code == KeyCode::Char('q') {
        app.close_filer();
        app.status_message = Some("ファイラーを閉じました".to_string());
        return;
    }
    let mut to_open = None;
    if let Some(ref mut filer) = app.filer {
        match key.code {
            KeyCode::Tab => {
                filer.switch_pane();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                filer.active_pane_mut().cursor_down();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                filer.active_pane_mut().cursor_up();
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    filer.active_pane_mut().cursor_down();
                }
            }
            KeyCode::PageUp => {
                for _ in 0..10 {
                    filer.active_pane_mut().cursor_up();
                }
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                for _ in 0..10 {
                    filer.active_pane_mut().cursor_down();
                }
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                for _ in 0..10 {
                    filer.active_pane_mut().cursor_up();
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                for _ in 0..10 {
                    filer.active_pane_mut().cursor_down();
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                for _ in 0..10 {
                    filer.active_pane_mut().cursor_up();
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                filer.active_pane_mut().cursor = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                let total = filer.active_pane().entries.len();
                filer.active_pane_mut().cursor = total.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                filer.active_pane_mut().toggle_select();
            }
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
                filer.active_pane_mut().up_dir();
            }
            KeyCode::Enter | KeyCode::Right => match filer.active_pane_mut().enter() {
                Ok(Some(path)) => {
                    to_open = Some(path);
                }
                Ok(None) => {}
                Err(e) => {
                    app.status_message = Some(format!("エラー: {e}"));
                }
            },
            KeyCode::Char('c') => match filer.copy_to_other() {
                Ok(cnt) => {
                    app.status_message =
                        Some(format!("{cnt} 件のファイルを反対画面へコピーしました"));
                }
                Err(e) => {
                    app.status_message = Some(format!("コピー失敗: {e}"));
                }
            },
            KeyCode::Char('m') => match filer.move_to_other() {
                Ok(cnt) => {
                    app.status_message =
                        Some(format!("{cnt} 件のファイルを反対画面へ移動しました"));
                }
                Err(e) => {
                    app.status_message = Some(format!("移動失敗: {e}"));
                }
            },
            KeyCode::Char('d') | KeyCode::Delete | KeyCode::Char('x') => {
                match filer.delete_selected() {
                    Ok(0) => {
                        app.status_message = Some(
                            "削除対象がありません（親ディレクトリ「..」は削除できません）"
                                .to_string(),
                        );
                    }
                    Ok(cnt) => {
                        app.status_message = Some(format!("{cnt} 件のファイルを削除しました"));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("削除失敗: {e}"));
                    }
                }
            }
            _ => {}
        }
    } else {
        app.show_filer = false;
        return;
    }

    if let Some(path) = to_open {
        if app.buffer.is_modified() {
            app.status_message = Some(
                "E37: 前回の変更が保存されていません。先に保存するか新しいタブ (:tabnew) で開いてください"
                    .to_string(),
            );
            return;
        }
        app.close_filer();
        if let Err(e) = app.open_file(path.clone()) {
            app.status_message = Some(format!("ファイル読込エラー: {e}"));
        } else {
            app.status_message = Some(format!("ファイルを開きました: {}", path.display()));
        }
    }
}

/// TUI 設定ダイアログで Space または Enter が押されたときの項目トグル処理
fn toggle_tui_settings_item(app: &mut App) {
    let cat = app.tui_settings_category;
    let item = app.tui_settings_item;

    match cat {
        0 => match item {
            // 文字エンコーディング
            0 => {
                app.config.general.default_encoding =
                    match app.config.general.default_encoding.as_str() {
                        "utf-8" => "sjis".to_string(),
                        "sjis" => "euc-jp".to_string(),
                        _ => "utf-8".to_string(),
                    };
            }
            // 改行形式
            1 => {
                app.config.general.default_fileformat =
                    if app.config.general.default_fileformat == "dos" {
                        "unix".to_string()
                    } else {
                        "dos".to_string()
                    };
            }
            // クリップボード
            2 => {
                app.config.general.clipboard = match app.config.general.clipboard.as_str() {
                    "none" => "unnamed".to_string(),
                    "unnamed" => "unnamedplus".to_string(),
                    _ => "none".to_string(),
                };
                let opt = vivivi_vi::ClipboardOption::parse(&app.config.general.clipboard);
                app.vi.set_clipboard_option(opt);
            }
            // スワップ
            3 => {
                app.config.general.crash_recovery = !app.config.general.crash_recovery;
            }
            // 更新チェック
            4 => {
                app.config.general.check_update = !app.config.general.check_update;
            }
            _ => {}
        },
        1 => match item {
            // カラーテーマ
            0 => {
                let themes = crate::theme::Theme::list_available_themes();
                let cur = app.config.appearance.theme.to_lowercase().replace(' ', "-");
                let mut next_theme = "tokyo-night".to_string();
                if let Some(pos) = themes
                    .iter()
                    .position(|t| t == &cur || t.replace('-', " ") == cur)
                {
                    next_theme = themes[(pos + 1) % themes.len()].clone();
                }
                let loaded = crate::theme::Theme::load_by_name(&next_theme);
                app.config.appearance.theme = loaded.meta.name.clone();
                app.active_theme = loaded;
            }
            // 行番号
            1 => {
                app.show_line_numbers = !app.show_line_numbers;
                app.config.appearance.line_number = app.show_line_numbers;
            }
            // 相対行番号
            2 => {
                app.relative_number = !app.relative_number;
                app.config.appearance.relative_number = app.relative_number;
            }
            // 折り返し
            3 => {
                app.wrap = !app.wrap;
                app.config.appearance.wrap = app.wrap;
            }
            // ガター強調
            4 => {
                app.change_tracker.enabled = !app.change_tracker.enabled;
                app.config.appearance.gutter_markers = app.change_tracker.enabled;
            }
            // ミニマップ
            5 => {
                app.config.appearance.gui.show_minimap = !app.config.appearance.gui.show_minimap;
            }
            _ => {}
        },
        2 => match item {
            // 不可視文字
            0 => {
                app.list_chars_config.enabled = !app.list_chars_config.enabled;
                app.config.invisibles.enabled = app.list_chars_config.enabled;
            }
            // 全角スペース
            1 => {
                if app.config.invisibles.zenkaku_space == "□" {
                    app.config.invisibles.zenkaku_space = "　".to_string();
                } else {
                    app.config.invisibles.zenkaku_space = "□".to_string();
                }
                app.list_chars_config.zenkaku_space =
                    app.config.invisibles.zenkaku_space.clone().into();
            }
            // 半角スペース
            2 => {
                if app.config.invisibles.space == '·' {
                    app.config.invisibles.space = ' ';
                } else {
                    app.config.invisibles.space = '·';
                }
                app.list_chars_config.space = app.config.invisibles.space;
            }
            // タブ
            3 => {
                if app.config.invisibles.tab == ">---" {
                    app.config.invisibles.tab = "▸---".to_string();
                } else {
                    app.config.invisibles.tab = ">---".to_string();
                }
            }
            // 改行
            4 => {
                if app.config.invisibles.eol == '↵' {
                    app.config.invisibles.eol = '$';
                } else {
                    app.config.invisibles.eol = '↵';
                }
                app.list_chars_config.eol = app.config.invisibles.eol;
            }
            _ => {}
        },
        3 => match item {
            // タブ幅
            0 => {
                app.tab_width = if app.tab_width >= 16 {
                    2
                } else {
                    app.tab_width + 2
                };
                app.config.editing.tab_width = app.tab_width;
            }
            // スペース展開
            1 => {
                app.config.editing.expand_tab = !app.config.editing.expand_tab;
            }
            // 罫線保護
            2 => {
                app.config.editing.keisen_protect = !app.config.editing.keisen_protect;
            }
            // 括弧強調
            3 => {
                app.config.editing.match_paren = !app.config.editing.match_paren;
            }
            // dabbrev大文字無視
            4 => {
                app.config.editing.dabbrev_ignorecase = !app.config.editing.dabbrev_ignorecase;
            }
            // ignorecase
            5 => {
                app.config.editing.ignorecase = !app.config.editing.ignorecase;
            }
            // smartcase
            6 => {
                app.config.editing.smartcase = !app.config.editing.smartcase;
            }
            _ => {}
        },
        4 => match item {
            0 => {
                app.config.appearance.gui.enable_gravity_drop =
                    !app.config.appearance.gui.enable_gravity_drop;
            }
            1 => {
                app.config.appearance.gui.reactive_idle_sleep =
                    !app.config.appearance.gui.reactive_idle_sleep;
            }
            _ => {}
        },
        _ => {}
    }
}
