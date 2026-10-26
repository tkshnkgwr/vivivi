use super::super::App;
use super::super::*;

impl App {
    // --- テキストアライン (:align <target>) ---

    /// 指定した区切り文字列の位置で対象行範囲のテキストを垂直整列します (`:align <target>` 相当)。
    ///
    /// 選択範囲内の各行で最初に出現する `target` 文字列の位置を揃えるよう空白を挿入し、
    /// 実行後はノーマルモードへ戻ります。操作全体は単一のアンドゥブロックとして記録されます。
    ///
    /// # 引数
    /// - `target`: 整列基準となる区切り文字列（例: `"="`, `":"`, `","`）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("a = 1\nbbb = 2", None, Default::default(), Default::default());
    /// app.align_selection_or_buffer("=");
    /// ```
    pub fn align_selection_or_buffer(&mut self, target: &str) {
        if target.is_empty() || self.buffer.line_count() == 0 {
            return;
        }
        let (start_line, end_line) = self.get_target_line_range();
        let mut lines = Vec::new();
        for l in start_line..=end_line {
            lines.push(self.buffer.line_text(l).unwrap_or_default());
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let aligned = align_lines(&line_refs, target);

        self.buffer.open_undo_block();
        for (i, new_line) in aligned.into_iter().enumerate() {
            let cur_line = start_line + i;
            let old_line_len = self
                .buffer
                .line_text(cur_line)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line, 0));
            let end_pos = start_pos + old_line_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &new_line);
        }
        self.buffer.close_undo_block();

        self.vi.set_mode(ViMode::Normal);
        self.cursor.anchor = None;
        self.status_message = Some(format!(
            "テキストアライン実行: '{}' ({}〜{} 行)",
            target,
            start_line + 1,
            end_line + 1
        ));
    }

    // --- リナンバ (:renumber [start] [step]) ---

    /// 対象行範囲の箇条書き・リスト連番を自動検出し、連番を再計算・更新します (`:renumber` 相当)。
    ///
    /// `1.`, `2)`, `[3]` などの連番パターンを検出し、開始番号 `start` と増分 `step` に応じて連番を振り直します。
    ///
    /// # 引数
    /// - `start`: 開始番号
    /// - `step`: 加算ステップ数
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("1. first\n1. second", None, Default::default(), Default::default());
    /// app.renumber_selection_or_buffer(1, 1);
    /// ```
    pub fn renumber_selection_or_buffer(&mut self, start: usize, step: usize) {
        if self.buffer.line_count() == 0 {
            return;
        }
        let (start_line, end_line) = self.get_target_line_range();
        let mut lines = Vec::new();
        for l in start_line..=end_line {
            lines.push(self.buffer.line_text(l).unwrap_or_default());
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let renumbered = renumber_lines(&line_refs, start, step);

        self.buffer.open_undo_block();
        for (i, new_line) in renumbered.into_iter().enumerate() {
            let cur_line = start_line + i;
            let old_line_len = self
                .buffer
                .line_text(cur_line)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line, 0));
            let end_pos = start_pos + old_line_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &new_line);
        }
        self.buffer.close_undo_block();

        self.vi.set_mode(ViMode::Normal);
        self.cursor.anchor = None;
        self.status_message = Some(format!(
            "リナンバ実行: 開始={}, 増分={} ({}〜{} 行)",
            start,
            step,
            start_line + 1,
            end_line + 1
        ));
    }

    // --- 行頭行末一括挿入 (:insbegend, :insbeg, :insend) ---

    /// 対象行範囲の行頭および行末に文字列を一括挿入します (`:insbegend` 相当)。
    ///
    /// インデント維持フラグ（`keep_indent`）や空行スキップフラグ（`skip_empty`）をサポートし、
    /// コメントアウトやマークダウン箇条書き化などを高速に行えます。
    ///
    /// # 引数
    /// - `beg`: 行頭に挿入する文字列
    /// - `end`: 行末に挿入する文字列
    /// - `keep_indent`: `true` の場合、既存インデントの直後に `beg` を挿入
    /// - `skip_empty`: `true` の場合、空行をスキップ
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("item 1\nitem 2", None, Default::default(), Default::default());
    /// app.insert_beg_end_in_range("- ", "", true, true);
    /// ```
    pub fn insert_beg_end_in_range(
        &mut self,
        beg: &str,
        end: &str,
        keep_indent: bool,
        skip_empty: bool,
    ) {
        if self.buffer.line_count() == 0 {
            return;
        }
        let (start_line, end_line) = self.get_target_line_range();
        let mut lines = Vec::new();
        for l in start_line..=end_line {
            lines.push(self.buffer.line_text(l).unwrap_or_default());
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let edited = insert_beg_end_lines(&line_refs, beg, end, keep_indent, skip_empty);

        self.buffer.open_undo_block();
        for (i, new_line) in edited.into_iter().enumerate() {
            let cur_line = start_line + i;
            let old_line_len = self
                .buffer
                .line_text(cur_line)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line, 0));
            let end_pos = start_pos + old_line_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &new_line);
        }
        self.buffer.close_undo_block();

        self.vi.set_mode(ViMode::Normal);
        self.cursor.anchor = None;
        self.status_message = Some(format!(
            "行頭行末挿入実行: beg='{}', end='{}' ({}〜{} 行)",
            beg,
            end,
            start_line + 1,
            end_line + 1
        ));
    }

    // --- 動的補完 (dabbrev / Ctrl-L, Ctrl-M) ---

    /// 動的単語補完（dabbrev）を実行します (`Ctrl-L` で次候補、`Ctrl-M` で前候補)。
    ///
    /// 初回呼び出し時はカーソル直前の単語プレフィックスからバッファ全体を走査して候補一覧を生成し、
    /// 2回目以降は補完文字列を循環的に差し替えます。
    ///
    /// # 引数
    /// - `is_forward`: `true` で次の候補へ進む、`false` で前の候補へ戻る
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("function_apple function_ap", None, Default::default(), Default::default());
    /// app.cursor.pos.col = 26;
    /// app.trigger_dabbrev(true);
    /// ```
    pub fn trigger_dabbrev(&mut self, is_forward: bool) {
        let cur = self.cursor.pos;
        if !self.dabbrev.is_active() {
            // 初回開始 (前方に検索)
            let opt_candidate = self
                .dabbrev
                .start(&self.buffer, cur.line, cur.col)
                .map(|s| s.to_string());
            if let Some(candidate) = opt_candidate {
                // プレフィックスの後に続く残りの文字列を挿入
                let prefix_len = self.dabbrev.prefix.chars().count();
                let candidate_chars: Vec<char> = candidate.chars().collect();
                let remaining: String = candidate_chars[prefix_len..].iter().collect();

                let insert_pos = self.buffer.cursor_to_position(cur);
                self.buffer.insert_text(insert_pos, &remaining);
                self.cursor.pos.col += remaining.chars().count();

                let total = self.dabbrev.candidates.len();
                self.status_message = Some(format!("[動的補完 1/{total}] {candidate}"));
            } else {
                self.status_message = Some("動的補完の候補が見つかりません".to_string());
            }
        } else {
            // 2回目以降: 既存の補完部分を差し替え
            let old_inserted_len = self.dabbrev.last_inserted_len;
            let next_cand = if is_forward {
                self.dabbrev.next_candidate().map(|s| s.to_string())
            } else {
                self.dabbrev.prev_candidate().map(|s| s.to_string())
            };

            if let Some(candidate) = next_cand {
                let prefix_len = self.dabbrev.prefix.chars().count();
                let candidate_chars: Vec<char> = candidate.chars().collect();
                let new_remaining: String = candidate_chars[prefix_len..].iter().collect();

                // 直前に挿入した文字を削除して新しい残りを挿入
                let cur_pos = self.buffer.cursor_to_position(self.cursor.pos);
                let del_start = cur_pos.saturating_sub(old_inserted_len);
                self.buffer.delete_text(del_start, old_inserted_len, false);
                self.buffer.insert_text(del_start, &new_remaining);

                self.cursor.pos.col = self.cursor.pos.col.saturating_sub(old_inserted_len)
                    + new_remaining.chars().count();

                let curr_idx = self.dabbrev.current_index.unwrap_or(0) + 1;
                let total = self.dabbrev.candidates.len();
                self.status_message = Some(format!("[動的補完 {curr_idx}/{total}] {candidate}"));
            }
        }
    }

    // --- 静的略語展開 (abbrev / Ctrl-;) ---

    /// カーソル直前の単語を検出し、略語辞書から定型コード・スニペットを展開します (`Ctrl-;` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("fnm", None, Default::default(), Default::default());
    /// app.cursor.pos.col = 3;
    /// app.trigger_abbrev_expansion();
    /// ```
    pub fn trigger_abbrev_expansion(&mut self) {
        let cur_line_idx = self.cursor.pos.line;
        let line_text = match self.buffer.line_text(cur_line_idx) {
            Some(t) => t,
            None => return,
        };
        let chars: Vec<char> = line_text.chars().collect();
        let col = self.cursor.pos.col.min(chars.len());

        let mut word_start = col;
        while word_start > 0 {
            let c = chars[word_start - 1];
            if c.is_alphanumeric() || c == '_' {
                word_start -= 1;
            } else {
                break;
            }
        }

        let word: String = chars[word_start..col].iter().collect();
        if word.is_empty() {
            self.status_message = Some("略語展開: カーソル直前に単語がありません".to_string());
            return;
        }

        if let Some(snippet) = self.abbrev.expand(&word) {
            let snippet = snippet.to_string();
            let word_len = word.chars().count();
            self.buffer.open_undo_block();

            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line_idx, word_start));
            let end_pos = start_pos + word_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &snippet);
            self.buffer.close_undo_block();

            let lines: Vec<&str> = snippet.lines().collect();
            if lines.len() <= 1 {
                self.cursor.pos.col = word_start + snippet.chars().count();
            } else {
                self.cursor.pos.line = cur_line_idx + lines.len() - 1;
                let last_line = lines.last().unwrap_or(&"");
                self.cursor.pos.col = last_line.chars().count();
            }

            self.status_message = Some(format!("略語展開成功: '{}'", word));
        } else {
            self.status_message = Some(format!("略語が見つかりません: '{}'", word));
        }
    }

    // --- Zen-Coding (Emmet) 展開 (Ctrl-,) ---

    /// カーソル直前の短縮記法（`ul>li*3` 等）を検出し、HTMLタグ構造へ展開します (`Ctrl-,` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("div>p", None, Default::default(), Default::default());
    /// app.cursor.pos.col = 5;
    /// app.trigger_zen_coding();
    /// ```
    pub fn trigger_zen_coding(&mut self) {
        let cur_line_idx = self.cursor.pos.line;
        let line_text = match self.buffer.line_text(cur_line_idx) {
            Some(t) => t,
            None => return,
        };
        let chars: Vec<char> = line_text.chars().collect();
        let col = self.cursor.pos.col.min(chars.len());

        let mut zen_start = col;
        while zen_start > 0 {
            let c = chars[zen_start - 1];
            if c.is_whitespace() {
                break;
            }
            zen_start -= 1;
        }

        let zen_str: String = chars[zen_start..col].iter().collect();
        if zen_str.is_empty() {
            self.status_message = Some("Zen-Coding: 展開対象の記述がありません".to_string());
            return;
        }

        if let Some(expanded) = expand_zen_coding(&zen_str) {
            let query_len = zen_str.chars().count();
            self.buffer.open_undo_block();

            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line_idx, zen_start));
            let end_pos = start_pos + query_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, &expanded);
            self.buffer.close_undo_block();

            let lines: Vec<&str> = expanded.lines().collect();
            if lines.len() <= 1 {
                self.cursor.pos.col = zen_start + expanded.chars().count();
            } else {
                self.cursor.pos.line = cur_line_idx + lines.len() - 1;
                let last_line = lines.last().unwrap_or(&"");
                self.cursor.pos.col = last_line.chars().count();
            }

            self.status_message = Some(format!("Zen-Coding展開: '{}'", zen_str));
        } else {
            self.status_message = Some(format!("Zen-Coding構文エラー: '{}'", zen_str));
        }
    }

    // --- ダイナミックマクロ (Dynamic Macro / Alt-M) ---

    /// 直前の入力・編集履歴から増井式反復規則を検出し、マクロを自動補完・実行します (`Alt-M` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.trigger_dynamic_macro();
    /// ```
    pub fn trigger_dynamic_macro(&mut self) {
        if let Some(chars) = self.dynamic_macro.detect_repeat() {
            let seq_str: String = chars.iter().collect();
            self.buffer.open_undo_block();
            for &ch in &chars {
                let pos = self.buffer.cursor_to_position(self.cursor.pos);
                self.buffer.insert_char(pos, ch);
                if ch == '\n' {
                    self.cursor.pos.line += 1;
                    self.cursor.pos.col = 0;
                } else {
                    self.cursor.pos.col += 1;
                }
            }
            self.buffer.close_undo_block();
            self.status_message = Some(format!("ダイナミックマクロ実行: '{}'", seq_str));
        } else {
            self.status_message =
                Some("ダイナミックマクロ: 反復規則を検出できませんでした".to_string());
        }
    }

    // --- 字句編集予測 (LexEdit / Ctrl-J) ---

    /// カーソル位置の単語に対して過去の置換履歴から次の編集パターンを予測し適用します (`Ctrl-J` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("old_fn", None, Default::default(), Default::default());
    /// app.trigger_lex_edit();
    /// ```
    pub fn trigger_lex_edit(&mut self) {
        let cur_line_idx = self.cursor.pos.line;
        let line_text = match self.buffer.line_text(cur_line_idx) {
            Some(t) => t,
            None => return,
        };
        let chars: Vec<char> = line_text.chars().collect();
        let col = self.cursor.pos.col.min(chars.len());

        let mut word_start = col;
        while word_start > 0
            && (chars[word_start - 1].is_alphanumeric() || chars[word_start - 1] == '_')
        {
            word_start -= 1;
        }
        let mut word_end = col;
        while word_end < chars.len()
            && (chars[word_end].is_alphanumeric() || chars[word_end] == '_')
        {
            word_end += 1;
        }

        let word: String = chars[word_start..word_end].iter().collect();
        if word.is_empty() {
            self.status_message = Some("字句編集予測: 対象単語がありません".to_string());
            return;
        }

        let candidates = self.lex_edit.predict(&word);
        if let Some(target) = candidates.first() {
            let word_len = word.chars().count();
            self.buffer.open_undo_block();

            let start_pos = self
                .buffer
                .cursor_to_position(CursorPosition::new(cur_line_idx, word_start));
            let end_pos = start_pos + word_len;
            self.buffer.delete_range(start_pos, end_pos);
            self.buffer.insert_text(start_pos, target);
            self.buffer.close_undo_block();

            self.cursor.pos.col = word_start + target.chars().count();
            self.status_message = Some(format!("字句編集予測適用: '{}' -> '{}'", word, target));
        } else {
            self.status_message = Some(format!("字句編集予測候補なし: '{}'", word));
        }
    }

    // --- キーボードマクロ (Ctrl-Alt-R / Ctrl-Alt-P) ---

    /// キーボードマクロの記録開始／停止をトグル（切り替え）します (`F9` または `Ctrl+Alt+R` 相当)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.toggle_macro_recording(); // 記録開始
    /// assert!(app.is_recording_macro);
    /// app.toggle_macro_recording(); // 記録終了
    /// assert!(!app.is_recording_macro);
    /// ```
    pub fn toggle_macro_recording(&mut self) {
        if self.is_recording_macro {
            self.is_recording_macro = false;
            self.last_macro_events = Some(self.recorded_macro_events.clone());
            let count = self.recorded_macro_events.len();
            self.status_message = Some(format!("マクロ記録終了 ({count} 件のキーイベントを保存)"));
        } else {
            self.is_recording_macro = true;
            self.recorded_macro_events.clear();
            self.status_message =
                Some("● マクロ記録中... (Ctrl+Alt+R または F9 で終了)".to_string());
        }
    }
}
