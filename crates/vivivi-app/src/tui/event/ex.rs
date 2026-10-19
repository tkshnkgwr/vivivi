use super::*;

/// exコマンドの実行処理 (:w, :q, :wq, :s, etc.)
pub(crate) fn execute_ex_command(app: &mut App, cmd: &str) {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return;
    }

    match trimmed {
        "w" => {
            if let Err(e) = app.save_file() {
                app.status_message = Some(format!("保存エラー: {e}"));
            }
        }
        "q" | "qa" | "qall" => {
            if app.buffer.is_modified() {
                app.status_message = Some(
                    "E37: 前回の変更が保存されていません (! を追加して :q! で強制終了)".to_string(),
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
            if let Err(e) = app.close_tab() {
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
            app.close_other_tabs();
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
        _ if trimmed.starts_with("%s/") || trimmed.starts_with("s/") => {
            app.status_message = Some(format!("置換実行: {trimmed}"));
        }

        // --- 検索ハイライト消去 (:noh, :nohl, :nohlsearch, :set nohls, :set hls) ---
        "noh" | "nohl" | "nohlsearch" => {
            app.clear_search_highlight();
            app.status_message = Some("検索ハイライトを消去しました (:nohlsearch)".to_string());
        }
        "set nohls" | "se nohls" | "set nohlsearch" | "se nohlsearch" => {
            app.clear_search_highlight();
            app.status_message = Some("検索ハイライト: OFF (:set nohlsearch)".to_string());
        }
        "set hls" | "se hls" | "set hlsearch" | "se hlsearch" => {
            app.is_search_highlight_visible = true;
            app.status_message = Some("検索ハイライト: ON (:set hlsearch)".to_string());
        }
        // --- テキストアライン (:align <target>, :align left/center/right) ---
        "align left" | "left" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Left);
        }
        "align center" | "center" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Center);
        }
        "align right" | "right" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Right);
        }
        _ if trimmed.starts_with("align ") => {
            let target = trimmed[6..].trim();
            match target {
                "left" => app.align_keisen_box(vivivi_modes::BoxAlignment::Left),
                "center" => app.align_keisen_box(vivivi_modes::BoxAlignment::Center),
                "right" => app.align_keisen_box(vivivi_modes::BoxAlignment::Right),
                _ => app.align_selection_or_buffer(target),
            }
        }

        // --- リナンバ (:renumber [start] [step], :renum [start] [step]) ---
        _ if trimmed.starts_with("renumber") || trimmed.starts_with("renum") => {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let start = parts
                .get(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);
            let step = parts
                .get(2)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);
            app.renumber_selection_or_buffer(start, step);
        }

        // --- 行頭行末一括挿入 (:insbeg, :insend, :begend, :insbegend) ---
        _ if trimmed.starts_with("insbeg ") => {
            let beg = &trimmed[7..];
            app.insert_beg_end_in_range(beg, "", true, true);
        }
        _ if trimmed.starts_with("insend ") => {
            let end = &trimmed[7..];
            app.insert_beg_end_in_range("", end, false, true);
        }
        _ if trimmed.starts_with("begend ") || trimmed.starts_with("insbegend ") => {
            let rest = trimmed
                .strip_prefix("begend ")
                .or_else(|| trimmed.strip_prefix("insbegend "))
                .unwrap_or(trimmed)
                .trim();
            if let Some((b, e)) = rest.split_once(' ') {
                app.insert_beg_end_in_range(b.trim(), e.trim(), true, true);
            } else {
                app.insert_beg_end_in_range(rest, rest, true, true);
            }
        }

        // --- 静的略語展開 (:abbrev [key val], :ab [key val]) ---
        _ if trimmed.starts_with("abbrev ") || trimmed.starts_with("ab ") => {
            let rest = trimmed
                .strip_prefix("abbrev ")
                .or_else(|| trimmed.strip_prefix("ab "))
                .unwrap_or(trimmed)
                .trim();
            if let Some((k, v)) = rest.split_once(' ') {
                app.abbrev.register(k.trim(), v.trim());
                app.status_message = Some(format!("略語登録: '{}' -> '{}'", k.trim(), v.trim()));
            } else {
                app.status_message = Some(format!("略語登録数: {} 件", app.abbrev.len()));
            }
        }
        "abbrev" | "ab" => {
            app.status_message = Some(format!(
                "略語登録数: {} 件 (挿入モードで Ctrl-; / Alt-; / F4 で展開)",
                app.abbrev.len()
            ));
        }

        // --- ダイナミックマクロ (:dmacro) ---
        "dmacro" => {
            app.trigger_dynamic_macro();
        }

        // --- 字句編集予測 (:lexedit) ---
        "lexedit" => {
            app.trigger_lex_edit();
        }

        // --- Zen-Coding展開 (:zen [expr], :zencoding [expr]) ---
        _ if trimmed.starts_with("zen ") || trimmed.starts_with("zencoding ") => {
            let query = trimmed
                .strip_prefix("zen ")
                .or_else(|| trimmed.strip_prefix("zencoding "))
                .unwrap_or(trimmed)
                .trim();
            if let Some(expanded) = vivivi_modes::expand_zen_coding(query) {
                let cur = app.cursor_pos();
                let pos = app.buffer.cursor_to_position(cur);
                app.buffer.open_undo_block();
                app.buffer.insert_text(pos, &expanded);
                app.buffer.close_undo_block();
                app.status_message = Some(format!("Zen-Coding展開: '{query}'"));
            } else {
                app.status_message = Some(format!("Zen-Coding構文エラー: '{query}'"));
            }
        }
        "zen" | "zencoding" => {
            app.trigger_zen_coding();
        }

        // --- キーボードマクロ (:macro record / stop / play) ---
        "macro record" => {
            if !app.is_recording_macro {
                app.toggle_macro_recording();
            }
        }
        "macro stop" => {
            if app.is_recording_macro {
                app.toggle_macro_recording();
            }
        }
        "macro play" => {
            play_macro(app);
        }

        // --- AI連携 (:ai [instruction]) ---
        _ if trimmed.starts_with("ai ") || trimmed == "ai" => {
            execute_ai_command(app, trimmed);
        }

        // --- 画面分割 (:split, :vsplit, :close, :only) ---
        "split" | "sp" => {
            app.split_horizontal();
        }
        "vsplit" | "vs" => {
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

        // --- ローマ字日本語検索 (:migemo <query>, :m <query>) ---
        _ if trimmed.starts_with("migemo ") || trimmed.starts_with("m ") => {
            let query = if let Some(q) = trimmed.strip_prefix("migemo ") {
                q.trim()
            } else if let Some(q) = trimmed.strip_prefix("m ") {
                q.trim()
            } else {
                ""
            };
            if !query.is_empty() {
                app.execute_migemo_search(query);
            } else {
                app.is_migemo_search = true;
                app.migemo_query.clear();
                app.status_message = Some(
                    "g/ (ローマ字入力で日本語検索 - Enterで確定, Escでキャンセル)".to_string(),
                );
            }
        }
        "migemo" => {
            app.is_migemo_search = true;
            app.migemo_query.clear();
            app.status_message =
                Some("g/ (ローマ字入力で日本語検索 - Enterで確定, Escでキャンセル)".to_string());
        }

        // --- 文字変換 (:zenkaku, :hankaku, :hiragana, :katakana, etc.) ---
        "zenkaku" | "%zenkaku" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToZenkakuAlphanumeric);
        }
        "hankaku" | "%hankaku" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHankakuAlphanumeric);
        }
        "hiragana" | "%hiragana" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHiragana);
        }
        "katakana" | "%katakana" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToKatakana);
        }
        "hankata" | "%hankata" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHankakuKatakana);
        }
        "zenkata" | "%zenkata" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToZenkakuKatakana);
        }
        "toupper" | "%toupper" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToUppercase);
        }
        "tolower" | "%tolower" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToLowercase);
        }

        // --- BOX選択数値集計 (:calc, :sum, :total) ---
        "calc" | "sum" | "total" => {
            app.calculate_current_box_selection();
        }

        // --- クリップボード履歴 (:cbhist, :history) ---
        "cbhist" | "history" | "clipboard" => {
            app.show_clipboard_hist = true;
            app.selected_cb_idx = 0;
            app.status_message =
                Some("クリップボード履歴 (j/k:選択, Enter:貼付, Esc:閉じる)".to_string());
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
            || trimmed.starts_with("edit ")
            || trimmed == "e"
            || trimmed == "edit" =>
        {
            let rest = trimmed
                .strip_prefix("edit")
                .or_else(|| trimmed.strip_prefix("e"))
                .unwrap_or("")
                .trim();

            let mut forced_enc = None;
            let mut file_target = None;

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
                    file_target = Some(part);
                }
            }

            if let Some(file_path_str) = file_target {
                let path = std::path::PathBuf::from(file_path_str);
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

        // --- 保存時文字コード切替 (:set fenc=<enc>, :se fenc=<enc>, :set fileencoding=<enc>, :set fenc?, :set fenc) ---
        "set fenc?" | "se fenc?" | "set fileencoding?" | "se fileencoding?" | "set fenc"
        | "se fenc" | "set fileencoding" | "se fileencoding" => {
            app.status_message = Some(format!(
                "現在のファイルエンコーディング (fenc): {}",
                app.encoding.name()
            ));
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
        }

        // --- 保存時改行コード切替 (:set ff=<format>, :se ff=<format>, :set fileformat=<format>, :set ff?, :set ff) ---
        "set ff?" | "se ff?" | "set fileformat?" | "se fileformat?" | "set ff" | "se ff"
        | "set fileformat" | "se fileformat" => {
            app.status_message = Some(format!("現在の改行形式 (ff): {}", app.newline.name()));
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
        }

        // --- 行番号表示切替 (:set number / :set nonumber / :set nu / :set nonu / :se nu / :se nonu) ---
        "number" | "set number" | "se number" | "nu" | "set nu" | "se nu" => {
            app.show_line_numbers = true;
            app.status_message = Some("行番号表示: ON (:set number)".to_string());
        }
        "nonumber" | "set nonumber" | "se nonumber" | "nonu" | "set nonu" | "se nonu" => {
            app.show_line_numbers = false;
            app.status_message = Some("行番号表示: OFF (:set nonumber)".to_string());
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
        }

        // --- テキスト折り返し切替 (:set wrap / :set nowrap / :se wrap / :se nowrap) ---
        "wrap" | "set wrap" | "se wrap" => {
            app.wrap = true;
            app.scroll.1 = 0;
            app.status_message = Some("テキスト折り返し: ON (:set wrap)".to_string());
        }
        "nowrap" | "set nowrap" | "se nowrap" => {
            app.wrap = false;
            app.status_message = Some("テキスト折り返し: OFF (:set nowrap)".to_string());
        }
        "wrap?" | "set wrap?" | "se wrap?" => {
            let state = if app.wrap { "wrap" } else { "nowrap" };
            app.status_message = Some(format!(
                "{state} (テキスト折り返し: {})",
                if app.wrap { "ON" } else { "OFF" }
            ));
        }

        // --- 相対行番号表示切替 (:set relativenumber / :set norelativenumber / :set rnu / :set nornu / :se rnu / :se nornu) ---
        "relativenumber" | "set relativenumber" | "se relativenumber" | "rnu" | "set rnu"
        | "se rnu" => {
            app.relative_number = true;
            app.status_message = Some("相対行番号表示: ON (:set relativenumber)".to_string());
        }
        "norelativenumber"
        | "set norelativenumber"
        | "se norelativenumber"
        | "nornu"
        | "set nornu"
        | "se nornu" => {
            app.relative_number = false;
            app.status_message = Some("相対行番号表示: OFF (:set norelativenumber)".to_string());
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
        }

        // --- 不可視文字可視化 (:set list / :set nolist / :se list / :se nolist / :list / :nolist) ---
        "list" | "set list" | "se list" => {
            app.list_chars_config.enabled = true;
            app.status_message =
                Some("不可視文字表示 (全角スペース□ / タブ▸--- / 改行$): ON".to_string());
        }
        "nolist" | "set nolist" | "se nolist" => {
            app.list_chars_config.enabled = false;
            app.status_message = Some("不可視文字表示: OFF".to_string());
        }

        // --- 変更行ガター強調 (:set gutter / :set nogutter / :se gutter / :se nogutter / :gutter / :nogutter) ---
        "gutter" | "set gutter" | "se gutter" => {
            app.change_tracker.enabled = true;
            app.status_message =
                Some("変更行ガター強調表示 (黄:未保存, 緑:保存済): ON".to_string());
        }
        "nogutter" | "set nogutter" | "se nogutter" => {
            app.change_tracker.enabled = false;
            app.status_message = Some("変更行ガター強調表示: OFF".to_string());
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
        }

        // --- 差分比較プレビュー (:diff) ---
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
        }

        // --- 2ペイン インラインファイラー (:filer [path], :explore [path]) ---
        "filer" | "explore" => {
            app.open_filer(None);
        }
        _ if trimmed.starts_with("filer ") || trimmed.starts_with("explore ") => {
            let path_str = trimmed
                .strip_prefix("filer ")
                .or_else(|| trimmed.strip_prefix("explore "))
                .unwrap_or("")
                .trim();
            let path = std::path::Path::new(path_str);
            app.open_filer(Some(path));
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
        }

        _ => {
            app.status_message = Some(format!("未対応のexコマンド: {trimmed}"));
        }
    }
}

/// AI連携exコマンド (`:ai <instruction>`) の実行
fn execute_ai_command(app: &mut App, cmd: &str) {
    let instruction = if cmd == "ai" { "help" } else { cmd[3..].trim() };

    match instruction {
        "help" => {
            app.status_message = Some(
                "AI連携: :ai <指示> (例: :ai doc, :ai refactor, :ai commit, :ai inline)"
                    .to_string(),
            );
        }
        "commit" => {
            app.status_message =
                Some("AI連携 [コミット支援]: git diff からメッセージ生成準備".to_string());
        }
        _ => {
            let (start, end) = app.get_target_line_range();
            app.status_message = Some(format!(
                "AI連携 [指示受付]: '{instruction}' (対象行: {}〜{} 行)",
                start + 1,
                end + 1
            ));
        }
    }
}
