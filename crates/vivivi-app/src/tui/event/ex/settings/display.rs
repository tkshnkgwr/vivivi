//! 表示・外観・GUI・ファイルタイプ関連の `:set` コマンド処理モジュール。

use crate::tui::app::App;

/// 表示・外観・構文関連の `:set` オプションを処理します。
pub(super) fn handle_display_set_command(app: &mut App, trimmed: &str) -> bool {
    match trimmed {
        // --- 行番号表示切替 (:set number / :set nonumber / :set nu / :set nonu / :se nu / :se nonu) ---
        "number" | "set number" | "se number" | "nu" | "set nu" | "se nu" => {
            app.show_line_numbers = true;
            app.status_message = Some("行番号表示: ON (:set number)".to_string());
            true
        }
        "nonumber" | "set nonumber" | "se nonumber" | "nonu" | "set nonu" | "se nonu" => {
            app.show_line_numbers = false;
            app.status_message = Some("行番号表示: OFF (:set nonumber)".to_string());
            true
        }
        "number?" | "set number?" | "se number?" | "nu?" | "set nu?" | "se nu?" => {
            let state = if app.show_line_numbers {
                "number"
            } else {
                "nonumber"
            };
            app.status_message = Some(format!(
                "{state} (行番号表示: {})",
                if app.show_line_numbers { "ON" } else { "OFF" }
            ));
            true
        }

        // --- テキスト折り返し切替 (:set wrap / :set nowrap / :se wrap / :se nowrap) ---
        "wrap" | "set wrap" | "se wrap" => {
            app.wrap = true;
            app.scroll.1 = 0;
            app.status_message = Some("テキスト折り返し: ON (:set wrap)".to_string());
            true
        }
        "nowrap" | "set nowrap" | "se nowrap" => {
            app.wrap = false;
            app.status_message = Some("テキスト折り返し: OFF (:set nowrap)".to_string());
            true
        }
        "wrap?" | "set wrap?" | "se wrap?" => {
            let state = if app.wrap { "wrap" } else { "nowrap" };
            app.status_message = Some(format!(
                "{state} (テキスト折り返し: {})",
                if app.wrap { "ON" } else { "OFF" }
            ));
            true
        }

        // --- 相対行番号表示切替 (:set relativenumber / :set norelativenumber / :set rnu / :set nornu / :se rnu / :se nornu) ---
        "relativenumber" | "set relativenumber" | "se relativenumber" | "rnu" | "set rnu"
        | "se rnu" => {
            app.relative_number = true;
            app.status_message = Some("相対行番号表示: ON (:set relativenumber)".to_string());
            true
        }
        "norelativenumber"
        | "set norelativenumber"
        | "se norelativenumber"
        | "nornu"
        | "set nornu"
        | "se nornu" => {
            app.relative_number = false;
            app.status_message = Some("相対行番号表示: OFF (:set norelativenumber)".to_string());
            true
        }
        "relativenumber?"
        | "set relativenumber?"
        | "se relativenumber?"
        | "rnu?"
        | "set rnu?"
        | "se rnu?" => {
            let state = if app.relative_number {
                "relativenumber"
            } else {
                "norelativenumber"
            };
            app.status_message = Some(format!(
                "{state} (相対行番号表示: {})",
                if app.relative_number { "ON" } else { "OFF" }
            ));
            true
        }

        // --- 不可視文字可視化 (:set list / :set nolist / :se list / :se nolist / :list / :nolist / :set list?) ---
        "list" | "set list" | "se list" => {
            app.list_chars_config.enabled = true;
            app.status_message =
                Some("不可視文字表示 (全角スペース□ / タブ▸--- / 改行$): ON".to_string());
            true
        }
        "nolist" | "set nolist" | "se nolist" => {
            app.list_chars_config.enabled = false;
            app.status_message = Some("不可視文字表示: OFF".to_string());
            true
        }
        "list?" | "set list?" | "se list?" => {
            let state = if app.list_chars_config.enabled {
                "list"
            } else {
                "nolist"
            };
            app.status_message = Some(format!(
                "{state} (不可視文字表示: {})",
                if app.list_chars_config.enabled {
                    "ON"
                } else {
                    "OFF"
                }
            ));
            true
        }

        // --- 不可視文字記号設定 (:set listchars=... / :set lcs=... / :set listchars? / :set lcs?) ---
        "listchars?" | "set listchars?" | "se listchars?" | "lcs?" | "set lcs?" | "se lcs?" => {
            app.status_message = Some(format!(
                "listchars=zenkaku:{},tab:{},space:{},eol:{}",
                app.list_chars_config.zenkaku_space,
                app.config.invisibles.tab,
                app.list_chars_config.space,
                app.list_chars_config.eol
            ));
            true
        }
        _ if trimmed.starts_with("set listchars=")
            || trimmed.starts_with("se listchars=")
            || trimmed.starts_with("set lcs=")
            || trimmed.starts_with("se lcs=") =>
        {
            let val = trimmed
                .strip_prefix("set listchars=")
                .or_else(|| trimmed.strip_prefix("se listchars="))
                .or_else(|| trimmed.strip_prefix("set lcs="))
                .or_else(|| trimmed.strip_prefix("se lcs="))
                .unwrap_or("")
                .trim();
            for part in val.split(',') {
                if let Some((k, v)) = part.split_once(':') {
                    match k.trim() {
                        "tab" => {
                            app.config.invisibles.tab = v.to_string();
                        }
                        "zenkaku" => {
                            app.config.invisibles.zenkaku_space = v.to_string();
                            app.list_chars_config.zenkaku_space = v.to_string().into();
                        }
                        "space" => {
                            if let Some(ch) = v.chars().next() {
                                app.config.invisibles.space = ch;
                                app.list_chars_config.space = ch;
                            }
                        }
                        "eol" => {
                            if let Some(ch) = v.chars().next() {
                                app.config.invisibles.eol = ch;
                                app.list_chars_config.eol = ch;
                            }
                        }
                        _ => {}
                    }
                }
            }
            app.status_message = Some(format!("不可視文字記号 (listchars) を設定しました: {val}"));
            true
        }

        // --- 変更行ガター強調 (:set gutter / :set nogutter / :se gutter / :se nogutter / :gutter / :nogutter / :set gutter?) ---
        "gutter" | "set gutter" | "se gutter" => {
            app.change_tracker.enabled = true;
            app.status_message =
                Some("変更行ガター強調表示 (黄:未保存, 緑:保存済): ON".to_string());
            true
        }
        "nogutter" | "set nogutter" | "se nogutter" => {
            app.change_tracker.enabled = false;
            app.status_message = Some("変更行ガター強調表示: OFF".to_string());
            true
        }
        "gutter?" | "set gutter?" | "se gutter?" => {
            let state = if app.change_tracker.enabled {
                "gutter"
            } else {
                "nogutter"
            };
            app.status_message = Some(format!(
                "{state} (変更行ガター強調: {})",
                if app.change_tracker.enabled {
                    "ON"
                } else {
                    "OFF"
                }
            ));
            true
        }

        // --- ミニマップ表示切替 (:set minimap / :set nominimap / :set minimap? / :se minimap / :minimap) ---
        "minimap" | "set minimap!" | "se minimap!" | "set invminimap" | "se invminimap" => {
            app.config.appearance.gui.show_minimap = !app.config.appearance.gui.show_minimap;
            let state = if app.config.appearance.gui.show_minimap {
                "ON"
            } else {
                "OFF"
            };
            app.status_message = Some(format!("ミニマップ表示: {state}"));
            true
        }
        "set minimap" | "se minimap" => {
            app.config.appearance.gui.show_minimap = true;
            app.status_message = Some("ミニマップ表示: ON (:set minimap)".to_string());
            true
        }
        "nominimap" | "set nominimap" | "se nominimap" => {
            app.config.appearance.gui.show_minimap = false;
            app.status_message = Some("ミニマップ表示: OFF (:set nominimap)".to_string());
            true
        }
        "minimap?" | "set minimap?" | "se minimap?" => {
            let state = if app.config.appearance.gui.show_minimap {
                "minimap"
            } else {
                "nominimap"
            };
            app.status_message = Some(format!(
                "{state} (ミニマップ表示: {})",
                if app.config.appearance.gui.show_minimap {
                    "ON"
                } else {
                    "OFF"
                }
            ));
            true
        }

        // --- 文字削除重力落下アニメーション切替 (:set particles / :set noparticles / :set particles?) ---
        "particles" | "set particles" | "se particles" => {
            app.config.appearance.gui.enable_gravity_drop = true;
            app.status_message =
                Some("文字削除重力落下アニメーション: ON (:set particles)".to_string());
            true
        }
        "noparticles" | "set noparticles" | "se noparticles" => {
            app.config.appearance.gui.enable_gravity_drop = false;
            app.status_message =
                Some("文字削除重力落下アニメーション: OFF (:set noparticles)".to_string());
            true
        }
        "particles?" | "set particles?" | "se particles?" => {
            let state = if app.config.appearance.gui.enable_gravity_drop {
                "particles"
            } else {
                "noparticles"
            };
            app.status_message = Some(format!(
                "{state} (重力落下アニメーション: {})",
                if app.config.appearance.gui.enable_gravity_drop {
                    "ON"
                } else {
                    "OFF"
                }
            ));
            true
        }

        // --- 背景明暗切替 (:set bg=dark / :set bg=light / :set background=dark / :set bg?) ---
        "bg?" | "set bg?" | "se bg?" | "background?" | "set background?" | "se background?" => {
            let current_is_light = app.config.appearance.theme.to_lowercase().contains("light");
            let bg_name = if current_is_light { "light" } else { "dark" };
            app.status_message = Some(format!(
                "background={bg_name} ({})",
                app.config.appearance.theme
            ));
            true
        }
        "set bg=dark" | "se bg=dark" | "set background=dark" | "se background=dark" => {
            app.config.appearance.theme = "tokyo-night".to_string();
            app.status_message = Some("背景明暗を設定しました: dark (tokyo-night)".to_string());
            true
        }
        "set bg=light" | "se bg=light" | "set background=light" | "se background=light" => {
            app.config.appearance.theme = "vivivi-light".to_string();
            app.status_message = Some("背景明暗を設定しました: light (vivivi-light)".to_string());
            true
        }

        // --- カーソル形状・角丸設定 (:set cursor_shape=<auto|block|beam|underline> / :set guicursor=... / :set cursor_radius=<px>) ---
        "cursor_shape?" | "set cursor_shape?" | "se cursor_shape?" | "guicursor?"
        | "set guicursor?" | "se guicursor?" => {
            app.status_message = Some(format!(
                "cursor_shape={} ({}) / cursor_radius={:.1}px",
                app.config.appearance.gui.cursor_shape.as_str(),
                app.config.appearance.gui.cursor_shape.label(),
                app.config.appearance.gui.cursor_corner_radius,
            ));
            true
        }
        _ if trimmed.starts_with("set cursor_shape=")
            || trimmed.starts_with("se cursor_shape=")
            || trimmed.starts_with("set guicursor=")
            || trimmed.starts_with("se guicursor=")
            || trimmed.starts_with("set cs=")
            || trimmed.starts_with("se cs=") =>
        {
            let val = trimmed.split_once('=').map(|(_, v)| v.trim()).unwrap_or("");
            if let Some(shape) = crate::config::CursorShape::parse(val) {
                app.config.appearance.gui.cursor_shape = shape;
                app.status_message = Some(format!(
                    "カーソル形状を設定しました: {} ({})",
                    shape.as_str(),
                    shape.label()
                ));
            } else {
                app.status_message = Some(format!(
                    "未対応のカーソル形状です: {val} (指定可能: auto, block, beam, underline)"
                ));
            }
            true
        }
        "cursor_radius?" | "set cursor_radius?" | "se cursor_radius?" => {
            app.status_message = Some(format!(
                "cursor_radius={:.1}px (0.0 で直角)",
                app.config.appearance.gui.cursor_corner_radius
            ));
            true
        }
        _ if trimmed.starts_with("set cursor_radius=")
            || trimmed.starts_with("se cursor_radius=") =>
        {
            let val = trimmed.split_once('=').map(|(_, v)| v.trim()).unwrap_or("");
            if let Ok(radius) = val.parse::<f32>() {
                let clamped = radius.clamp(0.0, 8.0);
                app.config.appearance.gui.cursor_corner_radius = clamped;
                app.status_message = Some(format!(
                    "カーソル角丸を設定しました: {clamped:.1}px{}",
                    if clamped == 0.0 { " (直角)" } else { "" }
                ));
            } else {
                app.status_message = Some(format!(
                    "無効なカーソル角丸値です: {val} (0.0〜8.0 の数値を指定)"
                ));
            }
            true
        }

        // --- 全設定一覧確認 (:set all / :se all) ---
        "set all" | "se all" => {
            let nu = if app.show_line_numbers {
                "number"
            } else {
                "nonumber"
            };
            let rnu = if app.relative_number {
                "relativenumber"
            } else {
                "norelativenumber"
            };
            let wr = if app.wrap { "wrap" } else { "nowrap" };
            let mm = if app.config.appearance.gui.show_minimap {
                "minimap"
            } else {
                "nominimap"
            };
            let pt = if app.config.appearance.gui.enable_gravity_drop {
                "particles"
            } else {
                "noparticles"
            };
            let lc = if app.list_chars_config.enabled {
                "list"
            } else {
                "nolist"
            };
            let gt = if app.change_tracker.enabled {
                "gutter"
            } else {
                "nogutter"
            };
            let ic = if app.config.editing.ignorecase {
                "ic"
            } else {
                "noic"
            };
            let sc = if app.config.editing.smartcase {
                "sc"
            } else {
                "nosc"
            };
            app.status_message = Some(format!(
                "{nu} {rnu} {wr} ts={} {lc} {gt} {mm} {pt} {ic} {sc} theme={} fenc={} ff={}",
                app.tab_width,
                app.config.appearance.theme,
                app.encoding.name(),
                app.newline.name()
            ));
            true
        }

        // --- 言語構文・ファイルタイプ切替 (:set ft=<lang>, :se ft=<lang>, :set filetype=<lang>, :se filetype=<lang>, :set ft?) ---
        "set ft?" | "se ft?" | "set filetype?" | "se filetype?" | "ft?" => {
            app.status_message = Some(format!(
                "現在のファイルタイプ (filetype): {:?}",
                app.syntax_lang
            ));
            true
        }
        _ if trimmed.starts_with("set ft=")
            || trimmed.starts_with("se ft=")
            || trimmed.starts_with("set filetype=")
            || trimmed.starts_with("se filetype=") =>
        {
            let lang_str = trimmed
                .strip_prefix("set ft=")
                .or_else(|| trimmed.strip_prefix("se ft="))
                .or_else(|| trimmed.strip_prefix("set filetype="))
                .or_else(|| trimmed.strip_prefix("se filetype="))
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            app.syntax_lang = match lang_str.as_str() {
                "rs" | "rust" => vivivi_modes::SyntaxLang::Rust,
                "md" | "markdown" => vivivi_modes::SyntaxLang::Markdown,
                "py" | "python" => vivivi_modes::SyntaxLang::Python,
                "c" | "cpp" | "cxx" | "cc" | "h" | "hpp" => vivivi_modes::SyntaxLang::Cpp,
                "js" | "ts" | "jsx" | "tsx" | "javascript" | "typescript" => {
                    vivivi_modes::SyntaxLang::JavaScript
                }
                "json" => vivivi_modes::SyntaxLang::Json,
                "sql" => vivivi_modes::SyntaxLang::Sql,
                "plsql" | "pls" | "plb" | "pks" | "pkb" => vivivi_modes::SyntaxLang::PlSql,
                "java" => vivivi_modes::SyntaxLang::Java,
                "html" | "htm" => vivivi_modes::SyntaxLang::Html,
                "sh" | "bash" | "zsh" => vivivi_modes::SyntaxLang::Bash,
                "go" => vivivi_modes::SyntaxLang::Go,
                "rb" | "ruby" => vivivi_modes::SyntaxLang::Ruby,
                "toml" => vivivi_modes::SyntaxLang::Toml,
                "ps1" | "psm1" | "psd1" | "powershell" => vivivi_modes::SyntaxLang::PowerShell,
                "yaml" | "yml" => vivivi_modes::SyntaxLang::Yaml,
                "xml" | "svg" | "xsd" | "xsl" => vivivi_modes::SyntaxLang::Xml,
                "bat" | "cmd" | "batch" => vivivi_modes::SyntaxLang::Batch,
                "hosts" => vivivi_modes::SyntaxLang::Hosts,
                "cron" | "crontab" => vivivi_modes::SyntaxLang::Cron,
                _ => vivivi_modes::SyntaxLang::PlainText,
            };
            app.reinit_treesitter();
            app.refresh_outline();
            app.status_message = Some(format!(
                "ファイルタイプを設定しました: {:?}",
                app.syntax_lang
            ));
            true
        }

        _ => false,
    }
}
