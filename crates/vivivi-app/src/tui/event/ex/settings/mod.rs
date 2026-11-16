//! エディタ設定およびオプション切替コマンド処理モジュール (`:set`, `:colorscheme`, `:mkvivivirc`, `:source` など)。

mod display;
mod editing;

use crate::tui::app::App;
use std::path::PathBuf;

/// チルダや相対パスを安全に解決します。
fn resolve_path(path_str: &str) -> PathBuf {
    let trimmed = path_str.trim();
    if trimmed.starts_with('~') {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        let rest = trimmed
            .strip_prefix('~')
            .unwrap_or("")
            .trim_start_matches(['/', '\\']);
        PathBuf::from(home).join(rest)
    } else {
        PathBuf::from(trimmed)
    }
}

/// 設定・オプション関連の ex コマンドを処理します。
///
/// コマンドを処理した場合は `true`、合致しなかった場合は `false` を返します。
pub(crate) fn handle_settings_command(app: &mut App, trimmed: &str) -> bool {
    // --- 1. カラーテーマ操作 (:colorscheme, :colo) ---
    if trimmed == "colo"
        || trimmed == "colo?"
        || trimmed == "colorscheme"
        || trimmed == "colorscheme?"
    {
        app.status_message = Some(format!(
            "現在のカラーテーマ (colorscheme): {}",
            app.config.appearance.theme
        ));
        return true;
    }
    if trimmed.starts_with("colorscheme ") || trimmed.starts_with("colo ") {
        let theme_name = trimmed
            .strip_prefix("colorscheme ")
            .or_else(|| trimmed.strip_prefix("colo "))
            .unwrap_or("")
            .trim();
        if theme_name.is_empty() {
            app.status_message = Some(format!(
                "現在のカラーテーマ (colorscheme): {}",
                app.config.appearance.theme
            ));
        } else if let Some(loaded) = crate::theme::Theme::find_by_name(theme_name) {
            app.config.appearance.theme = loaded.meta.name.clone();
            app.active_theme = loaded.clone();
            app.status_message = Some(format!(
                "カラーテーマを変更しました: {} ({})",
                loaded.meta.name, loaded.meta.author
            ));
        } else {
            app.status_message = Some(format!(
                "E185: テーマ '{theme_name}' が見つかりません (使用可能: tokyo-night, openvivi, vivi-classic, vivivi-dark, vivivi-light, または ~/.vivivi/themes/ 配下)"
            ));
        }
        return true;
    }

    // --- 2. 設定ファイル永続化 (:mkvivivirc, :wconfig) ---
    if trimmed == "mkvivivirc"
        || trimmed == "wconfig"
        || trimmed.starts_with("mkvivivirc ")
        || trimmed.starts_with("wconfig ")
    {
        app.sync_runtime_to_config();
        let target_path = if trimmed == "mkvivivirc" || trimmed == "wconfig" {
            crate::config::AppConfig::default_config_path()
        } else {
            let arg = trimmed
                .strip_prefix("mkvivivirc ")
                .or_else(|| trimmed.strip_prefix("wconfig "))
                .unwrap_or("")
                .trim();
            resolve_path(arg)
        };

        match app.config.save_to(&target_path) {
            Ok(()) => {
                app.status_message = Some(format!(
                    "設定ファイルを保存しました: {}",
                    target_path.display()
                ));
            }
            Err(e) => {
                app.status_message = Some(format!("E190: 設定ファイルの保存に失敗しました: {e}"));
            }
        }
        return true;
    }

    // --- 3. 外部ファイル再読込・スクリプト実行 (:source, :so) ---
    if trimmed == "source"
        || trimmed == "so"
        || trimmed.starts_with("source ")
        || trimmed.starts_with("so ")
    {
        let target_path = if trimmed == "source" || trimmed == "so" {
            crate::config::AppConfig::default_config_path()
        } else {
            let arg = trimmed
                .strip_prefix("source ")
                .or_else(|| trimmed.strip_prefix("so "))
                .unwrap_or("")
                .trim();
            resolve_path(arg)
        };

        if !target_path.exists() {
            app.status_message = Some(format!(
                "E484: ファイルを開けません: {}",
                target_path.display()
            ));
            return true;
        }

        match std::fs::read_to_string(&target_path) {
            Ok(content) => {
                let is_toml_ext = target_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("toml"))
                    .unwrap_or(false);

                // TOML 設定ファイルとしてパース可能なら設定適用
                if (is_toml_ext || content.trim_start().starts_with('['))
                    && let Ok(new_config) = crate::config::AppConfig::from_toml(&content)
                {
                    app.apply_config(&new_config);
                    app.status_message = Some(format!(
                        "設定ファイルを再読込しました: {}",
                        target_path.display()
                    ));
                    return true;
                }

                // スクリプト形式: 1行ずつ Ex コマンドを実行
                let mut count = 0;
                for line in content.lines() {
                    let l = line.trim();
                    if l.is_empty() || l.starts_with('"') || l.starts_with('#') {
                        continue;
                    }
                    crate::tui::event::execute_ex_command(app, l);
                    count += 1;
                }
                app.status_message = Some(format!(
                    "設定スクリプトを実行しました ({} 行): {}",
                    count,
                    target_path.display()
                ));
            }
            Err(e) => {
                app.status_message = Some(format!(
                    "E484: ファイル読込エラー: {e} ({})",
                    target_path.display()
                ));
            }
        }
        return true;
    }

    // --- 4. 設定画面・設定再読込 ---
    match trimmed {
        "preferences" | "pref" | "settings" | "dialog" => {
            app.open_preferences_dialog();
            app.status_message = Some("設定画面を開きます".to_string());
            return true;
        }
        "config" | "set config" | "se config" => {
            let path = crate::config::AppConfig::default_config_path();
            let new_config = crate::config::AppConfig::load_or_default(Some(&path));
            app.apply_config(&new_config);
            app.status_message = Some(format!("設定を再読込しました: {}", path.display()));
            return true;
        }
        _ => {}
    }

    // --- 5. サブハンドラへ委譲 ---
    editing::handle_editing_set_command(app, trimmed)
        || display::handle_display_set_command(app, trimmed)
}
