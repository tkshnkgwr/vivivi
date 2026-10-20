//! エディタ設定およびオプション切替コマンド処理モジュール (`:set`, `:config` など)。

use crate::tui::app::App;

/// 設定・オプション関連の ex コマンドを処理します。
///
/// コマンドを処理した場合は `true`、合致しなかった場合は `false` を返します。
pub(crate) fn handle_settings_command(app: &mut App, trimmed: &str) -> bool {
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

        // --- タブ幅切替 (:set tabstop=<N>, :se tabstop=<N>, :set ts=<N>, :se ts=<N>) ---
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
            app.show_line_numbers = new_config.editor.line_number;
            app.relative_number = new_config.editor.relative_number;
            app.tab_width = new_config.editor.tab_width;
            app.wrap = new_config.editor.wrap;
            app.change_tracker.enabled = new_config.gutter.enabled;
            app.list_chars_config.enabled = new_config.invisible.enabled;
            app.list_chars_config.zenkaku_space =
                Box::leak(new_config.invisible.zenkaku_space.clone().into_boxed_str());
            app.list_chars_config.space = new_config.invisible.space;
            app.list_chars_config.eol = new_config.invisible.eol;
            let cb_opt = vivivi_vi::ClipboardOption::parse(&new_config.editor.clipboard);
            app.vi.set_clipboard_option(cb_opt);
            app.config = new_config;
            app.status_message = Some(format!("設定を再読込しました: {}", path.display()));
            true
        }

        // --- 不可視文字可視化 (:set list / :set nolist / :se list / :se nolist / :list / :nolist) ---
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

        // --- 変更行ガター強調 (:set gutter / :set nogutter / :se gutter / :se nogutter / :gutter / :nogutter) ---
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

        // --- 言語構文・ファイルタイプ切替 (:set ft=<lang>, :se ft=<lang>, :set filetype=<lang>, :se filetype=<lang>) ---
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
