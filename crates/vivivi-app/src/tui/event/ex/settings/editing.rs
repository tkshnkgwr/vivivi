//! 編集・検索・エンコーディング・保存関連の `:set` コマンド処理モジュール。

use crate::tui::app::App;

/// 編集・エンコーディング・検索関連の `:set` オプションを処理します。
pub(super) fn handle_editing_set_command(app: &mut App, trimmed: &str) -> bool {
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

        // --- 保存時末尾空白自動削除切替 (:set trim / :set notrim / :set trim_on_save / :set notrim_on_save) ---
        "trim" | "set trim" | "se trim" | "trim_on_save" | "set trim_on_save"
        | "se trim_on_save" => {
            app.config.editing.trim_on_save = true;
            app.status_message = Some("保存時行末空白自動削除: ON (:set trim)".to_string());
            true
        }
        "notrim" | "set notrim" | "se notrim" | "notrim_on_save" | "set notrim_on_save"
        | "se notrim_on_save" => {
            app.config.editing.trim_on_save = false;
            app.status_message = Some("保存時行末空白自動削除: OFF (:set notrim)".to_string());
            true
        }
        "trim?" | "set trim?" | "se trim?" | "trim_on_save?" | "set trim_on_save?"
        | "se trim_on_save?" => {
            let state = if app.config.editing.trim_on_save {
                "trim"
            } else {
                "notrim"
            };
            app.status_message = Some(format!(
                "{state} (保存時行末空白自動削除: {})",
                if app.config.editing.trim_on_save {
                    "ON"
                } else {
                    "OFF"
                }
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

        // --- 検索時大文字小文字無視 (:set ignorecase / :set noignorecase / :set ic / :set noic / :set ic? / :set invic / :set ic!) ---
        "ic" | "set ic" | "se ic" | "ignorecase" | "set ignorecase" | "se ignorecase" => {
            app.config.editing.ignorecase = true;
            app.vi.ignorecase = true;
            app.status_message = Some("大文字小文字同一視 (ignorecase): ON (:set ic)".to_string());
            true
        }
        "noic" | "set noic" | "se noic" | "noignorecase" | "set noignorecase"
        | "se noignorecase" => {
            app.config.editing.ignorecase = false;
            app.vi.ignorecase = false;
            app.status_message =
                Some("大文字小文字同一視 (ignorecase): OFF (:set noic)".to_string());
            true
        }
        "set ic!" | "se ic!" | "set invic" | "se invic" | "set ignorecase!" | "se ignorecase!" => {
            app.config.editing.ignorecase = !app.config.editing.ignorecase;
            app.vi.ignorecase = app.config.editing.ignorecase;
            let state = if app.config.editing.ignorecase {
                "ON"
            } else {
                "OFF"
            };
            app.status_message = Some(format!("大文字小文字同一視 (ignorecase): {state}"));
            true
        }
        "ic?" | "set ic?" | "se ic?" | "ignorecase?" | "set ignorecase?" | "se ignorecase?" => {
            let state = if app.config.editing.ignorecase {
                "ignorecase"
            } else {
                "noignorecase"
            };
            app.status_message = Some(format!(
                "{state} (大文字小文字同一視: {})",
                if app.config.editing.ignorecase {
                    "ON"
                } else {
                    "OFF"
                }
            ));
            true
        }

        // --- 大文字混在時厳密区別 (:set smartcase / :set nosmartcase / :set sc / :set nosc / :set sc? / :set invsc / :set sc!) ---
        "sc" | "set sc" | "se sc" | "smartcase" | "set smartcase" | "se smartcase" => {
            app.config.editing.smartcase = true;
            app.vi.smartcase = true;
            app.status_message = Some("スマートケース (smartcase): ON (:set sc)".to_string());
            true
        }
        "nosc" | "set nosc" | "se nosc" | "nosmartcase" | "set nosmartcase" | "se nosmartcase" => {
            app.config.editing.smartcase = false;
            app.vi.smartcase = false;
            app.status_message = Some("スマートケース (smartcase): OFF (:set nosc)".to_string());
            true
        }
        "set sc!" | "se sc!" | "set invsc" | "se invsc" | "set smartcase!" | "se smartcase!" => {
            app.config.editing.smartcase = !app.config.editing.smartcase;
            app.vi.smartcase = app.config.editing.smartcase;
            let state = if app.config.editing.smartcase {
                "ON"
            } else {
                "OFF"
            };
            app.status_message = Some(format!("スマートケース (smartcase): {state}"));
            true
        }
        "sc?" | "set sc?" | "se sc?" | "smartcase?" | "set smartcase?" | "se smartcase?" => {
            let state = if app.config.editing.smartcase {
                "smartcase"
            } else {
                "nosmartcase"
            };
            app.status_message = Some(format!(
                "{state} (スマートケース: {})",
                if app.config.editing.smartcase {
                    "ON"
                } else {
                    "OFF"
                }
            ));
            true
        }

        _ => false,
    }
}
