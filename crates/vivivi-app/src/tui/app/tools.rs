use super::App;
use super::*;

impl App {
    // --- アウトライン解析操作 ---

    /// アウトラインサイドペインの表示状態をトグル（切り替え）します (`:outline` または `F8` 相当)。
    ///
    /// 表示を有効化した場合はバッファ内容を自動再解析します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("# Heading", None, Default::default(), Default::default());
    /// app.toggle_outline();
    /// assert!(app.show_outline);
    /// ```
    pub fn toggle_outline(&mut self) {
        self.show_outline = !self.show_outline;
        if self.show_outline {
            self.refresh_outline();
            self.outline_focus = true;
            self.status_message = Some(
                "アウトライン: ON (j/k:選択, Enter:ジャンプ, Tab/Esc:エディタ復帰, F2/F8:閉じる)"
                    .to_string(),
            );
        } else {
            self.outline_focus = false;
            self.status_message = Some("アウトライン: OFF".to_string());
        }
    }

    /// 現在のバッファ内容をシンタックス言語に基づいて再解析し、アウトライン項目一覧を更新します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("fn main() {}", None, Default::default(), Default::default());
    /// app.refresh_outline();
    /// ```
    pub fn refresh_outline(&mut self) {
        let lines: Vec<String> = (0..self.buffer.line_count())
            .filter_map(|i| self.buffer.line_text(i))
            .collect();
        // 無名バッファ等でPlainTextの場合、内容から言語を自動検出してsyntax_langも同期
        if self.file_path.is_none()
            && self.syntax_lang == SyntaxLang::PlainText
            && let Some(detected) = vivivi_modes::detect_language_from_content(&lines)
        {
            self.syntax_lang = detected;
        }
        self.outline_items = parse_outline(&lines, self.syntax_lang);
        if self.selected_outline_idx >= self.outline_items.len() {
            self.selected_outline_idx = 0;
        }
    }

    /// サイドペインで現在選択されているアウトライン項目の定義行へカーソルをジャンプさせます。
    ///
    /// ジャンプ完了後、アウトラインペインのフォーカスを解除してエディタバッファへ戻します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("fn main() {}", None, Default::default(), Default::default());
    /// app.refresh_outline();
    /// app.jump_to_selected_outline();
    /// ```
    pub fn jump_to_selected_outline(&mut self) {
        if let Some(item) = self.outline_items.get(self.selected_outline_idx) {
            let target_line = item
                .line_idx
                .min(self.buffer.line_count().saturating_sub(1));
            self.cursor.pos.line = target_line;
            self.cursor.pos.col = 0;
            self.outline_focus = false;
            self.status_message =
                Some(format!("ジャンプ: 行 {} - {}", target_line + 1, item.title));
        }
    }

    // --- 文字変換操作 ---

    /// バッファ全文に対して指定された文字種変換（全角/半角、ひらがな/カタカナ等）を実行します (`:zenkaku`, `:hankaku` 等)。
    ///
    /// 操作全体は単一のアンドゥブロックとして記録され、1回のアンドゥ操作で復元できます。
    ///
    /// # 引数
    /// - `conv_type`: 適用する変換ルール（[`ConversionType`]）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    /// use vivivi_modes::ConversionType;
    ///
    /// let mut app = App::new("hello", None, Default::default(), Default::default());
    /// app.convert_buffer_text(ConversionType::ToUppercase);
    /// assert_eq!(app.buffer.line_text(0).unwrap(), "HELLO");
    /// ```
    pub fn convert_buffer_text(&mut self, conv_type: ConversionType) {
        self.buffer.open_undo_block();
        let mut count = 0;
        for line in 0..self.buffer.line_count() {
            if let Some(line_str) = self.buffer.line_text(line) {
                let converted = convert_text(&line_str, conv_type);
                if converted != line_str {
                    let start_pos = self.buffer.line_start_position(line);
                    let old_len = line_str.chars().count();
                    self.buffer.delete_text(start_pos, old_len, false);
                    self.buffer.insert_text(start_pos, &converted);
                    count += 1;
                }
            }
        }
        self.buffer.close_undo_block();
        self.status_message = Some(format!("文字変換完了 ({count} 行変更)"));
    }

    // --- BOX選択数値集計 (:calc / :sum) ---

    /// 現在のBOX選択（矩形選択）範囲内の数値を抽出し、合計・平均・最大・最小・標準偏差を集計します (`:calc` / `:sum`)。
    ///
    /// 集計結果はステータスラインに表示され、合計値（Sum）は自動的に無名レジスタ（`"`）にコピーされます。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("10\n20\n30", None, Default::default(), Default::default());
    /// app.calculate_current_box_selection();
    /// ```
    pub fn calculate_current_box_selection(&mut self) {
        let anchor = self.cursor.anchor.unwrap_or(self.cursor.pos);
        let pos = self.cursor.pos;

        let min_line = anchor.line.min(pos.line);
        let max_line = anchor.line.max(pos.line);
        let min_col = anchor.col.min(pos.col);
        let max_col = anchor.col.max(pos.col);

        if let Some(result) =
            calculate_box_selection(&self.buffer, min_line, max_line, min_col, max_col)
        {
            let summary = result.formatted_summary();
            // 合計値を無名レジスタにコピー
            let sum_str = format!("{}", result.sum);
            self.vi
                .registers_mut()
                .set(None, RegisterContent::CharacterWise(sum_str));
            self.status_message = Some(format!("{summary} [合計値をレジスタにコピーしました]"));
        } else {
            self.status_message =
                Some("BOX選択範囲から有効な数値を検出できませんでした".to_string());
        }
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

    // --- クリップボード履歴 (cbHist) 操作 ---

    /// クリップボード履歴ダイアログから指定インデックスのテキストをカーソル位置に貼り付けます。
    ///
    /// 文字単位、行単位、ブロック単位（矩形）のレジスタ種別に応じた適切なペースト処理を行い、
    /// モーダルダイアログを自動的に閉じます。
    ///
    /// # 引数
    /// - `index`: クリップボード履歴リスト内のインデックス（0-based）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.paste_clipboard_item(0);
    /// ```
    pub fn paste_clipboard_item(&mut self, index: usize) {
        if let Some(item) = self.vi.registers().history().get(index).cloned() {
            let pos = self.buffer.cursor_to_position(self.cursor.pos);
            match item.content {
                RegisterContent::CharacterWise(s) => {
                    let char_count = s.chars().count();
                    self.buffer.insert_text(pos, &s);
                    self.cursor.pos.col += char_count;
                }
                RegisterContent::LineWise(s) => {
                    let next_line_start = self.buffer.line_start_position(self.cursor.pos.line + 1);
                    self.buffer.insert_text(next_line_start, &s);
                    self.cursor.pos.line += 1;
                    self.cursor.pos.col = 0;
                }
                RegisterContent::BlockWise(lines) => {
                    for (i, line) in lines.iter().enumerate() {
                        let l = self.cursor.pos.line + i;
                        if l >= self.buffer.line_count() {
                            self.buffer.insert_char(self.buffer.len(), '\n');
                        }
                        let insert_pos = self.buffer.line_start_position(l) + self.cursor.pos.col;
                        self.buffer.insert_text(insert_pos, line);
                    }
                }
            }
            self.show_clipboard_hist = false;
            self.status_message = Some(format!("履歴から貼り付けました: {}", item.preview));
        }
    }

    // --- ローマ字日本語検索 (Migemo / g/) ---

    /// ローマ字クエリを日本語正規表現パターンにコンパイルし、インクリメンタル検索を実行します (`g/` 相当)。
    ///
    /// マッチ箇所が存在する場合はカーソルを移動させ、全画面検索ハイライトを自動有効化します。
    ///
    /// # 引数
    /// - `romaji`: 入力されたローマ字文字列（例: `"kensaku"`）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("日本語の検索テスト", None, Default::default(), Default::default());
    /// app.execute_migemo_search("kensaku");
    /// ```
    pub fn execute_migemo_search(&mut self, romaji: &str) {
        match self.migemo.compile_query(romaji) {
            Ok(regex) => {
                self.migemo_last_regex = Some(regex.clone());
                self.is_search_highlight_visible = true;
                let full_text = self.buffer.to_string();
                let cur_pos = self.buffer.cursor_to_position(self.cursor.pos);
                let next_byte_offset = full_text
                    .char_indices()
                    .nth(cur_pos + 1)
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                // カーソル以降を検索、なければ先頭から検索（ラップアラウンド）
                let found_match = regex
                    .find_at(&full_text, next_byte_offset)
                    .or_else(|| regex.find(&full_text));

                if let Some(m) = found_match {
                    let char_idx = full_text[..m.start()].chars().count();
                    self.cursor.pos = self.buffer.position_to_cursor(char_idx);
                    if let Some(win) = self.windows.get_mut(self.active_window) {
                        win.cursor = self.cursor.clone();
                        win.scroll = self.scroll;
                    }
                    self.status_message = Some(format!("g/{romaji} [Migemo一致: {}]", m.as_str()));
                } else {
                    self.status_message = Some(format!("g/{romaji} [Migemo: 一致なし]"));
                }
            }
            Err(e) => {
                self.status_message = Some(format!("Migemo正規表現エラー: {e}"));
            }
        }
    }

    /// Migemo 検索結果の次の一致箇所（または前の一致箇所）へカーソルをジャンプさせます (`n` / `N` 相当)。
    ///
    /// # 引数
    /// - `is_forward`: `true` で正方向（次の一致）、`false` で逆方向（前の一致）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("検索 検索", None, Default::default(), Default::default());
    /// app.execute_migemo_search("kensaku");
    /// app.jump_next_migemo(true);
    /// ```
    pub fn jump_next_migemo(&mut self, is_forward: bool) {
        let regex = match &self.migemo_last_regex {
            Some(r) => r.clone(),
            None => return,
        };

        self.is_search_highlight_visible = true;
        let full_text = self.buffer.to_string();
        let cur_pos = self.buffer.cursor_to_position(self.cursor.pos);

        if is_forward {
            let next_byte_offset = full_text
                .char_indices()
                .nth(cur_pos + 1)
                .map(|(i, _)| i)
                .unwrap_or(0);

            if let Some(m) = regex
                .find_at(&full_text, next_byte_offset)
                .or_else(|| regex.find(&full_text))
            {
                let char_idx = full_text[..m.start()].chars().count();
                self.cursor.pos = self.buffer.position_to_cursor(char_idx);
                if let Some(win) = self.windows.get_mut(self.active_window) {
                    win.cursor = self.cursor.clone();
                    win.scroll = self.scroll;
                }
                self.status_message = Some(format!("[Migemo] 一致: {}", m.as_str()));
            }
        } else {
            // 逆方向検索: cur_byte_offset より手前の最後の一致を探索
            let cur_byte_offset = full_text
                .char_indices()
                .nth(cur_pos)
                .map(|(i, _)| i)
                .unwrap_or(full_text.len());

            let matches: Vec<_> = regex.find_iter(&full_text).collect();
            if let Some(m) = matches
                .iter()
                .rev()
                .find(|m| m.start() < cur_byte_offset)
                .or_else(|| matches.last())
            {
                let char_idx = full_text[..m.start()].chars().count();
                self.cursor.pos = self.buffer.position_to_cursor(char_idx);
                if let Some(win) = self.windows.get_mut(self.active_window) {
                    win.cursor = self.cursor.clone();
                    win.scroll = self.scroll;
                }
                self.status_message = Some(format!("[Migemo] 一致: {}", m.as_str()));
            }
        }
    }

    /// 現在有効な検索正規表現オブジェクトを取得します。
    ///
    /// Migemo検索の正規表現が優先され、存在しない場合は通常のVi検索パターンが使用されます。
    /// `:nohlsearch` 等でハイライトが非表示化されている場合は `None` を返します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let app = App::new("test", None, Default::default(), Default::default());
    /// assert!(app.active_search_regex().is_none());
    /// ```
    pub fn active_search_regex(&self) -> Option<regex::Regex> {
        if !self.is_search_highlight_visible {
            return None;
        }
        if let Some(ref r) = self.migemo_last_regex {
            return Some(r.clone());
        }
        if let Some(pat) = self.vi.last_search_pattern()
            && !pat.is_empty()
        {
            return regex::Regex::new(pat)
                .ok()
                .or_else(|| regex::Regex::new(&regex::escape(pat)).ok());
        }
        None
    }

    /// 画面上の検索マッチハイライト表示を一時的に消去します (`:noh` / `:nohlsearch` 相当)。
    ///
    /// 次に検索を実行するか `n` / `N` を押すと自動的にハイライトが再開されます。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// app.clear_search_highlight();
    /// assert!(!app.is_search_highlight_visible);
    /// ```
    pub fn clear_search_highlight(&mut self) {
        self.is_search_highlight_visible = false;
    }

    // --- ヘルプビューア (:help / F1) ---

    /// ヘルプドキュメント画面をモーダルビューとして開きます (`:help` または `F1` 相当)。
    ///
    /// 指定されたトピック名（例: `"keisen"`, `"csv"`）に対応する Markdown ヘルプを探索・ロードします。
    /// トピックが省略された場合は総合インデックス（`index.md`）を表示します。
    ///
    /// # 引数
    /// - `topic`: 閲覧したいヘルプトピック名（オプション）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.open_help(Some("keisen"));
    /// assert!(app.show_help);
    /// ```
    pub fn open_help(&mut self, topic: Option<&str>) {
        let topic_name = topic.unwrap_or("index").trim();
        let file_name = if topic_name.ends_with(".md") {
            topic_name.to_string()
        } else {
            format!("{topic_name}.md")
        };

        // 探索候補パス（doc/help, docs/help, help, ../../doc/help 等）
        let mut candidates = vec![
            PathBuf::from("doc").join("help").join(&file_name),
            PathBuf::from("docs").join("help").join(&file_name),
            PathBuf::from("help").join(&file_name),
            PathBuf::from("..")
                .join("..")
                .join("doc")
                .join("help")
                .join(&file_name),
            PathBuf::from("..")
                .join("..")
                .join("docs")
                .join("help")
                .join(&file_name),
        ];
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            candidates.push(
                PathBuf::from(manifest_dir)
                    .join("..")
                    .join("..")
                    .join("doc")
                    .join("help")
                    .join(&file_name),
            );
        }

        let mut content = None;
        let mut loaded_path = None;
        for path in &candidates {
            if let Ok(text) = std::fs::read_to_string(path) {
                content = Some(text);
                loaded_path = Some(path.display().to_string());
                break;
            }
        }

        let text = content.unwrap_or_else(|| {
            // 外部ファイルが見つからない場合のフォールバック（埋め込みインデックス）
            include_str!("../../../../../doc/help/index.md").to_string()
        });

        self.help_title = loaded_path.unwrap_or_else(|| format!("ヘルプ: {topic_name}"));
        self.help_lines = text.lines().map(|s| s.to_string()).collect();
        self.help_scroll = 0;
        self.show_help = true;
        self.status_message = Some(format!(
            "ヘルプを開きました (j/k:スクロール, q/Esc:閉じる): {topic_name}"
        ));
    }

    // --- 編集支援: 対象行範囲取得 ---

    /// ビジュアル選択範囲またはバッファ全体の対象行インデックス範囲 `(start_line, end_line)` を取得します。
    ///
    /// ビジュアルモード中の場合はアンカー位置と現在カーソル位置から選択範囲の開始行・終了行を算出し、
    /// それ以外の場合はバッファの先頭行（0）から末尾行までの全行範囲を返します。
    ///
    /// # 戻り値
    /// `(開始行, 終了行)` （0-based、inclusive）
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let app = App::new("line 1\nline 2", None, Default::default(), Default::default());
    /// assert_eq!(app.get_target_line_range(), (0, 1));
    /// ```
    pub fn get_target_line_range(&self) -> (usize, usize) {
        if self.vi.mode().is_visual()
            && let Some(anchor) = self.cursor.anchor
        {
            let start = anchor.line.min(self.cursor.pos.line);
            let end = anchor.line.max(self.cursor.pos.line);
            return (start, end);
        }
        let total = self.buffer.line_count();
        if total == 0 { (0, 0) } else { (0, total - 1) }
    }

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

/// 最近使ったファイルおよびお気に入りの永続化保存ファイルパスを取得します。
///
/// Windows では `%USERPROFILE%/.vivivi/recent.txt`、Linux/macOS では `~/.vivivi/recent.txt` を返します。
///
/// # Examples
/// ```rust,ignore
/// use vivivi_app::tui::app::get_recent_file_path;
///
/// let path = get_recent_file_path();
/// assert!(path.ends_with("recent.txt"));
/// ```
pub fn get_recent_file_path() -> PathBuf {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile)
            .join(".vivivi")
            .join("recent.txt")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".vivivi").join("recent.txt")
    } else {
        PathBuf::from(".vivivi_recent.txt")
    }
}
