mod macros;

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
            include_str!("../../../../../../doc/help/index.md").to_string()
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
