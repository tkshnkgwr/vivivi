//! エディタ設定およびオプション切替コマンド処理モジュール (`:set`, `:colorscheme`, `:mkvivivirc`, `:source` など)。

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

    // --- 4. 既存および拡張の :set コマンド ---
    match trimmed {
        // --- 検索ハイライト消去 (:noh, :nohl, :nohlsearch, :set nohls, :set hls) ---
        "noh" | "nohl" | "nohlsearch" => {
            app.clear_search_highlight();
            app.status_message = Some("検索ハイライトを消去しました (:nohlsearch)".to_string());
            true
        }
        "set nohls" | "se nohls" | "set nohlsearch" | "se nohlsearch" => {
            app.clear_search_highlight();
            app.status_message = Some("検索ハイライト: OFF (:set nohlsearch)".to_string());
            true
        }
        "set hls" | "se hls" | "set hlsearch" | "se hlsearch" => {
            app.is_search_highlight_visible = true;
            app.status_message = Some("検索ハイライト: ON (:set hlsearch)".to_string());
            true
        }
        "set hls?" | "se hls?" | "set hlsearch?" | "se hlsearch?" => {
            let state = if app.is_search_highlight_visible {
                "hlsearch"
            } else {
                "nohlsearch"
            };
            app.status_message = Some(format!(
                "{state} (検索ハイライト: {})",
                if app.is_search_highlight_visible {
                    "ON"
                } else {
                    "OFF"
                }
            ));
            true
        }

        // --- 保存時文字コード切替 (:set fenc=<enc>, :se fenc=<enc>, :set fileencoding=<enc>, :set fenc?, :set fenc) ---
        "set fenc?" | "se fenc?" | "set fileencoding?" | "se fileencoding?" | "set fenc"
        | "se fenc" | "set fileencoding" | "se fileencoding" => {
            app.status_message = Some(format!(
                "現在のファイルエンコーディング (fenc): {}",
                app.encoding.name()
            ));
            true
        }
        _ if trimmed.starts_with("set fenc=")
            || trimmed.starts_with("se fenc=")
            || trimmed.starts_with("set fileencoding=")
            || trimmed.starts_with("se fileencoding=") =>
        {
            let enc_str = trimmed
                .strip_prefix("set fenc=")
                .or_else(|| trimmed.strip_prefix("se fenc="))
                .or_else(|| trimmed.strip_prefix("set fileencoding="))
                .or_else(|| trimmed.strip_prefix("se fileencoding="))
                .unwrap_or("")
                .trim();
            if let Some(enc) = vivivi_core::CharEncoding::parse_enc(enc_str) {
                app.encoding = enc;
                app.status_message = Some(format!(
                    "保存エンコーディング (fenc) を設定しました: {}",
                    enc.name()
                ));
            } else {
                app.status_message = Some(format!(
                    "不明なエンコーディングです: {enc_str} (使用可能: utf-8, sjis, euc-jp, utf-16le, utf-16be)"
                ));
            }
            true
        }

        // --- 保存時改行コード切替 (:set ff=<format>, :se ff=<format>, :set fileformat=<format>, :set ff?, :set ff) ---
        "set ff?" | "se ff?" | "set fileformat?" | "se fileformat?" | "set ff" | "se ff"
        | "set fileformat" | "se fileformat" => {
            app.status_message = Some(format!("現在の改行形式 (ff): {}", app.newline.name()));
            true
        }
        _ if trimmed.starts_with("set ff=")
            || trimmed.starts_with("se ff=")
            || trimmed.starts_with("set fileformat=")
            || trimmed.starts_with("se fileformat=") =>
        {
            let ff_str = trimmed
                .strip_prefix("set ff=")
                .or_else(|| trimmed.strip_prefix("se ff="))
                .or_else(|| trimmed.strip_prefix("set fileformat="))
                .or_else(|| trimmed.strip_prefix("se fileformat="))
                .unwrap_or("")
                .trim();
            if let Some(nl) = vivivi_core::NewlineType::parse_ff(ff_str) {
                app.newline = nl;
                app.status_message = Some(format!("改行形式 (ff) を設定しました: {}", nl.name()));
            } else {
                app.status_message = Some(format!(
                    "不明な改行形式です: {ff_str} (使用可能: unix/lf, dos/crlf, mac/cr)"
                ));
            }
            true
        }

        // --- システムクリップボード同期 (:set clipboard=<opt>, :se cb=<opt>, :set clipboard?, :set cb?) ---
        "set clipboard?" | "se clipboard?" | "set cb?" | "se cb?" | "set clipboard"
        | "se clipboard" | "set cb" | "se cb" => {
            let current = match app.vi.clipboard_option() {
                vivivi_vi::ClipboardOption::None => "none",
                vivivi_vi::ClipboardOption::Unnamed => "unnamed",
                vivivi_vi::ClipboardOption::UnnamedPlus => "unnamedplus",
            };
            app.status_message = Some(format!("現在のクリップボード設定 (clipboard): {current}"));
            true
        }
        _ if trimmed.starts_with("set clipboard=")
            || trimmed.starts_with("se clipboard=")
            || trimmed.starts_with("set cb=")
            || trimmed.starts_with("se cb=") =>
        {
            let cb_str = trimmed
                .strip_prefix("set clipboard=")
                .or_else(|| trimmed.strip_prefix("se clipboard="))
                .or_else(|| trimmed.strip_prefix("set cb="))
                .or_else(|| trimmed.strip_prefix("se cb="))
                .unwrap_or("")
                .trim();
            let opt = vivivi_vi::ClipboardOption::parse(cb_str);
            app.vi.set_clipboard_option(opt);
            let opt_name = match opt {
                vivivi_vi::ClipboardOption::None => "none (同期無効)",
                vivivi_vi::ClipboardOption::Unnamed => "unnamed (* レジスタ同期)",
                vivivi_vi::ClipboardOption::UnnamedPlus => "unnamedplus (+ レジスタ同期)",
            };
            app.status_message = Some(format!(
                "クリップボード同期 (clipboard) を設定しました: {opt_name}"
            ));
            true
        }

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

        // --- タブ幅切替 (:set tabstop=<N>, :se tabstop=<N>, :set ts=<N>, :se ts=<N>, :set ts?, :set tabstop?) ---
        "tabstop?" | "set tabstop?" | "se tabstop?" | "ts?" | "set ts?" | "se ts?" => {
            app.status_message = Some(format!(
                "tabstop={} (タブ幅: {})",
                app.tab_width, app.tab_width
            ));
            true
        }
        _ if trimmed.starts_with("set tabstop=")
            || trimmed.starts_with("se tabstop=")
            || trimmed.starts_with("set ts=")
            || trimmed.starts_with("se ts=") =>
        {
            let val = trimmed
                .strip_prefix("set tabstop=")
                .or_else(|| trimmed.strip_prefix("se tabstop="))
                .or_else(|| trimmed.strip_prefix("set ts="))
                .or_else(|| trimmed.strip_prefix("se ts="))
                .unwrap_or("")
                .trim();
            if let Ok(w) = val.parse::<usize>() {
                app.tab_width = w.max(1);
                app.status_message = Some(format!(
                    "タブ幅 (tabstop) を設定しました: {}",
                    app.tab_width
                ));
            } else {
                app.status_message = Some(format!("無効なタブ幅です: {val}"));
            }
            true
        }

        // --- 設定ファイル再読込・パス確認 (:config, :set config, :se config) ---
        "config" | "set config" | "se config" => {
            let path = crate::config::AppConfig::default_config_path();
            let new_config = crate::config::AppConfig::load_or_default(Some(&path));
            app.apply_config(&new_config);
            app.status_message = Some(format!("設定を再読込しました: {}", path.display()));
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
                            app.list_chars_config.zenkaku_space =
                                Box::leak(v.to_string().into_boxed_str());
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

        // --- ミニマップ表示切替 (:set minimap / :set nominimap / :set minimap? / :se minimap) ---
        "minimap" | "set minimap" | "se minimap" => {
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
            app.status_message = Some(format!(
                "{nu} {rnu} {wr} ts={} {lc} {gt} {mm} {pt} theme={} fenc={} ff={}",
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
                "c" | "cpp" | "cxx" => vivivi_modes::SyntaxLang::Cpp,
                "js" | "ts" | "javascript" | "typescript" => vivivi_modes::SyntaxLang::JavaScript,
                "json" => vivivi_modes::SyntaxLang::Json,
                "sql" => vivivi_modes::SyntaxLang::Sql,
                "html" => vivivi_modes::SyntaxLang::Html,
                "sh" | "bash" => vivivi_modes::SyntaxLang::Bash,
                "go" => vivivi_modes::SyntaxLang::Go,
                "rb" | "ruby" => vivivi_modes::SyntaxLang::Ruby,
                "toml" => vivivi_modes::SyntaxLang::Toml,
                "ps1" | "powershell" => vivivi_modes::SyntaxLang::PowerShell,
                "yaml" | "yml" => vivivi_modes::SyntaxLang::Yaml,
                "xml" => vivivi_modes::SyntaxLang::Xml,
                "bat" | "cmd" | "batch" => vivivi_modes::SyntaxLang::Batch,
                _ => vivivi_modes::SyntaxLang::PlainText,
            };
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
