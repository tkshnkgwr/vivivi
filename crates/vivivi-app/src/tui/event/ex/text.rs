//! テキスト整列、一括挿入、マクロ、文字変換、略語展開コマンド処理モジュール (`:align`, `:abbrev`, `:zenkaku` など)。

use crate::tui::app::App;
use crate::tui::event::play_macro;

/// テキスト編集・変換・マクロ関連の ex コマンドを処理します。
///
/// コマンドを処理した場合は `true`、合致しなかった場合は `false` を返します。
pub(crate) fn handle_text_command(app: &mut App, trimmed: &str) -> bool {
    match trimmed {
        // --- テキストアライン (:align <target>, :align left/center/right) ---
        "align left" | "left" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Left);
            true
        }
        "align center" | "center" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Center);
            true
        }
        "align right" | "right" => {
            app.align_keisen_box(vivivi_modes::BoxAlignment::Right);
            true
        }
        _ if trimmed.starts_with("align ") => {
            let target = trimmed[6..].trim();
            match target {
                "left" => app.align_keisen_box(vivivi_modes::BoxAlignment::Left),
                "center" => app.align_keisen_box(vivivi_modes::BoxAlignment::Center),
                "right" => app.align_keisen_box(vivivi_modes::BoxAlignment::Right),
                _ => app.align_selection_or_buffer(target),
            }
            true
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
            true
        }

        // --- 行頭行末一括挿入 (:insbeg, :insend, :begend, :insbegend) ---
        _ if trimmed.starts_with("insbeg ") => {
            let beg = &trimmed[7..];
            app.insert_beg_end_in_range(beg, "", true, true);
            true
        }
        _ if trimmed.starts_with("insend ") => {
            let end = &trimmed[7..];
            app.insert_beg_end_in_range("", end, false, true);
            true
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
            true
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
            true
        }
        "abbrev" | "ab" => {
            app.status_message = Some(format!(
                "略語登録数: {} 件 (挿入モードで Ctrl-; / Alt-; / F4 で展開)",
                app.abbrev.len()
            ));
            true
        }

        // --- ダイナミックマクロ (:dmacro) ---
        "dmacro" => {
            app.trigger_dynamic_macro();
            true
        }

        // --- 字句編集予測 (:lexedit) ---
        "lexedit" => {
            app.trigger_lex_edit();
            true
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
            true
        }
        "zen" | "zencoding" => {
            app.trigger_zen_coding();
            true
        }

        // --- キーボードマクロ (:macro record / stop / play) ---
        "macro record" => {
            if !app.is_recording_macro {
                app.toggle_macro_recording();
            }
            true
        }
        "macro stop" => {
            if app.is_recording_macro {
                app.toggle_macro_recording();
            }
            true
        }
        "macro play" => {
            play_macro(app);
            true
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
            true
        }
        "migemo" => {
            app.is_migemo_search = true;
            app.migemo_query.clear();
            app.status_message =
                Some("g/ (ローマ字入力で日本語検索 - Enterで確定, Escでキャンセル)".to_string());
            true
        }

        // --- 文字変換 (:zenkaku, :hankaku, :hiragana, :katakana, etc.) ---
        "zenkaku" | "%zenkaku" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToZenkakuAlphanumeric);
            true
        }
        "hankaku" | "%hankaku" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHankakuAlphanumeric);
            true
        }
        "hiragana" | "%hiragana" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHiragana);
            true
        }
        "katakana" | "%katakana" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToKatakana);
            true
        }
        "hankata" | "%hankata" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToHankakuKatakana);
            true
        }
        "zenkata" | "%zenkata" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToZenkakuKatakana);
            true
        }
        "toupper" | "%toupper" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToUppercase);
            true
        }
        "tolower" | "%tolower" => {
            app.convert_buffer_text(vivivi_modes::ConversionType::ToLowercase);
            true
        }

        // --- BOX選択数値集計 (:calc, :sum, :total) ---
        "calc" | "sum" | "total" => {
            app.calculate_current_box_selection();
            true
        }

        // --- クリップボード履歴 (:cbhist, :history) ---
        "cbhist" | "history" | "clipboard" => {
            app.show_clipboard_hist = true;
            app.selected_cb_idx = 0;
            app.status_message =
                Some("クリップボード履歴 (j/k:選択, Enter:貼付, Esc:閉じる)".to_string());
            true
        }

        // --- 行ソート (:sort, :%sort, :'<,'>sort, :1,5sort [!/i/r/u/n]) ---
        _ if try_execute_sort_command(app, trimmed) => true,

        _ => false,
    }
}

/// `:sort` 系コマンド（範囲指定および `!`/`i`/`r`/`u`/`n` フラグ付き）を解析・実行します。
fn try_execute_sort_command(app: &mut App, trimmed: &str) -> bool {
    let parse_line_ref = |token: &str, app: &App| -> Option<usize> {
        let t = token.trim();
        if t == "." {
            Some(app.cursor.pos.line)
        } else if t == "$" {
            let mut last = app.buffer.line_count().saturating_sub(1);
            if last > 0 && app.buffer.get_line(last).as_deref() == Some("") {
                last -= 1;
            }
            Some(last)
        } else {
            t.parse::<usize>().ok().map(|n| n.saturating_sub(1))
        }
    };

    let (range_opt, after_sort) = if let Some(rest) = trimmed.strip_prefix("%sort") {
        (None, rest)
    } else if let Some(rest) = trimmed.strip_prefix("'<,'>sort") {
        let range = match app.cursor.anchor {
            Some(anchor) => (
                anchor.line.min(app.cursor.pos.line),
                anchor.line.max(app.cursor.pos.line),
            ),
            None => (app.cursor.pos.line, app.cursor.pos.line),
        };
        (Some(range), rest)
    } else if let Some(rest) = trimmed.strip_prefix("sort") {
        (None, rest)
    } else if let Some((range_part, after)) = trimmed.split_once("sort")
        && !range_part.is_empty()
        && let Some((s_str, e_str)) = range_part.split_once(',')
        && let (Some(s), Some(e)) = (parse_line_ref(s_str, app), parse_line_ref(e_str, app))
    {
        (Some((s, e)), after)
    } else {
        return false;
    };

    let is_sort_cmd = after_sort.is_empty()
        || after_sort.starts_with('!')
        || after_sort.chars().next().is_some_and(char::is_whitespace);
    if !is_sort_cmd {
        return false;
    }

    match vivivi_modes::parse_sort_flags(after_sort) {
        Ok(options) => {
            let sorted_len = app.sort_buffer_lines(range_opt, options);
            if trimmed.starts_with("'<,'>") {
                app.cursor.anchor = None;
            }
            app.status_message = Some(format!("{sorted_len} 行をソートしました"));
        }
        Err(err) => {
            app.status_message = Some(format!("sort 構文エラー: {err}"));
        }
    }
    true
}
