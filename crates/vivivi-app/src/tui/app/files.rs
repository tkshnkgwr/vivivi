use super::App;
use super::*;

impl App {
    /// 編集中バッファをスワップファイル（`.<filename>.swp`）に保存します。
    pub fn save_swap_file(&self) {
        if let Some(ref path) = self.file_path
            && self.buffer.is_modified()
        {
            let swp_path = vivivi_core::SwapFile::swap_path_for(path);
            let _ = vivivi_core::SwapFile::save(
                &swp_path,
                Some(path),
                &self.buffer.to_string(),
                self.encoding,
                self.newline,
            );
        }
    }

    /// スワップファイルを安全に削除します。
    pub fn remove_swap_file(&self) {
        if let Some(ref path) = self.file_path {
            let swp_path = vivivi_core::SwapFile::swap_path_for(path);
            let _ = vivivi_core::SwapFile::remove(&swp_path);
        }
    }

    /// スワップファイルからバッファ内容を復元します。
    pub fn recover_from_swap(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.file_path {
            let swp_path = vivivi_core::SwapFile::swap_path_for(path);
            if swp_path.exists() {
                let rec = vivivi_core::SwapFile::load(&swp_path)?;
                self.buffer = vivivi_core::TextBuffer::from_text(&rec.content);
                self.encoding = rec.encoding;
                self.newline = rec.newline;
                self.has_swap_warning = false;
                self.status_message = Some(format!(
                    "スワップファイルから復元しました ({})",
                    swp_path.display()
                ));
                return Ok(());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "復元対象のスワップファイルが見つかりません",
        ))
    }

    /// 現在のバッファ内容を設定されたエンコーディングおよび改行コードでファイルに保存します (`:w` 相当)。
    ///
    /// 既存ファイルの上書き時は世代差分バックアップ（[`vivivi_modes::DiffLog`]）および
    /// 永続化アンドゥログ（`.undodir`）を自動更新し、変更行ガター追跡のベースラインを再設定します。
    ///
    /// # Errors
    /// ファイルの書き込み権限不足やディスク容量不足など、I/O エラーが発生した場合に [`std::io::Error`] を返します。
    /// ファイルパスが設定されていない無名バッファの場合は何もしません。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("test", None, Default::default(), Default::default());
    /// let _ = app.save_file();
    /// ```
    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = self.file_path.clone() {
            let content = self.buffer.to_string();

            // 既存ファイルとの差分を DiffLog に記録
            if path.exists()
                && let Ok(old_content) = std::fs::read_to_string(&path)
            {
                let ts = crate::tui::app::current_datetime_str();
                let _ = vivivi_modes::DiffLog::record_generation(
                    &path,
                    &old_content,
                    &content,
                    &ts,
                    "auto-save",
                );
            }

            let bytes = vivivi_core::encode_text(&content, self.encoding, self.newline);
            std::fs::write(&path, bytes)?;
            self.buffer.on_saved();

            // 変更行ガター追跡の保存状態を更新
            let current_lines: Vec<String> = (0..self.buffer.line_count())
                .filter_map(|i| self.buffer.get_line(i))
                .collect();
            self.change_tracker.on_save(&current_lines);

            // 最近使ったファイルリストに追加・永続化
            self.recent_manager.add_recent(path.clone());
            let recent_file_path = get_recent_file_path();
            let _ = self.recent_manager.save_to_file(&recent_file_path);

            // 永続Undoを保存
            let _ = self.buffer.save_undofile(&path);

            // 保存完了に伴いスワップファイルをクリーンアップ
            self.remove_swap_file();

            self.syntax_lang = SyntaxLang::from_path(&path);
            self.refresh_outline();

            self.status_message = Some(format!(
                "\"{path:?}\" 書込完了 [{}/{}] (DiffLog & Undo永続化)",
                self.encoding.name(),
                self.newline.name()
            ));
            Ok(())
        } else {
            self.status_message =
                Some("ファイル名が指定されていません (:w <ファイル名>)".to_string());
            Ok(())
        }
    }

    /// 指定されたパスのファイルを読み込み、現在のエディタバッファを切り替えます。
    ///
    /// 文字コードおよび改行コードを自動判別し、拡張子に応じたシンタックスハイライト、
    /// CSV整列ビュー、永続Undoログ（`.undodir`）の復元、最近使ったファイル履歴の追加を行います。
    ///
    /// # 引数
    /// - `path`: 開くファイルのパス
    ///
    /// # Errors
    /// ファイルの読み込み権限不足など、I/O エラーが発生した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use std::path::PathBuf;
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// let _ = app.open_file(PathBuf::from("src/main.rs"));
    /// ```
    pub fn open_file(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        self.open_file_with_encoding(path, None)
    }

    /// 指定された文字エンコーディング（省略時は自動判別）でファイルを開きます。
    pub fn open_file_with_encoding(
        &mut self,
        path: PathBuf,
        forced_encoding: Option<CharEncoding>,
    ) -> Result<(), std::io::Error> {
        let (content, enc, nl) = if path.exists() {
            let bytes = std::fs::read(&path)?;
            if let Some(forced) = forced_encoding {
                vivivi_core::decode_bytes_with_encoding(&bytes, forced)
            } else {
                vivivi_core::decode_bytes(&bytes)
            }
        } else {
            let tm = vivivi_modes::TemplateManager::new();
            let date_str = crate::tui::app::current_date_str();
            let dt_str = crate::tui::app::current_datetime_str();
            let time_str = dt_str.split_whitespace().nth(1).unwrap_or("00:00:00");
            let templated = if let Some(res) = tm.load_and_expand(&path, &date_str, time_str) {
                res.text
            } else {
                String::new()
            };
            (
                templated,
                forced_encoding.unwrap_or(CharEncoding::Utf8),
                NewlineType::default(),
            )
        };

        self.buffer = TextBuffer::from_text(&content);
        let _ = self.buffer.load_undofile(&path);

        let lines: Vec<String> = (0..self.buffer.line_count())
            .filter_map(|i| self.buffer.get_line(i))
            .collect();
        self.change_tracker = ChangeTracker::new(&lines);

        let is_csv = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "csv" | "tsv"))
            .unwrap_or(false);
        self.is_csv_active = is_csv;

        self.syntax_lang = SyntaxLang::from_path(&path);
        self.vi.current_file_path = path.to_string_lossy().to_string();
        self.file_path = Some(path.clone());
        self.encoding = enc;
        self.newline = nl;
        self.cursor = TextCursor::new(0, 0);
        self.scroll = (0, 0);

        self.recent_manager.add_recent(path);
        let recent_file_path = get_recent_file_path();
        let _ = self.recent_manager.save_to_file(&recent_file_path);

        self.status_message = Some(format!(
            "ファイルを開きました [{}/{}]: {:?}",
            self.encoding.name(),
            self.newline.name(),
            self.file_path
        ));
        Ok(())
    }

    /// 現在開いているファイルを、指定された文字エンコーディングで再読み込みします。
    pub fn reload_with_encoding(&mut self, enc: CharEncoding) -> Result<(), std::io::Error> {
        if let Some(path) = self.file_path.clone() {
            self.open_file_with_encoding(path, Some(enc))
        } else {
            self.encoding = enc;
            self.status_message = Some(format!(
                "文字エンコーディングを設定しました: {}",
                enc.name()
            ));
            Ok(())
        }
    }

    /// 現在開いているファイルのお気に入り登録／解除をトグル（切り替え）します。
    ///
    /// 無名バッファの場合はエラーメッセージをステータスラインに表示します。
    ///
    /// # Examples
    /// ```rust,ignore
    /// use vivivi_app::tui::App;
    ///
    /// let mut app = App::new("", None, Default::default(), Default::default());
    /// app.toggle_favorite_current_file();
    /// ```
    pub fn toggle_favorite_current_file(&mut self) {
        if let Some(ref path) = self.file_path {
            let is_fav = self.recent_manager.toggle_favorite(path.clone());
            let recent_file_path = get_recent_file_path();
            let _ = self.recent_manager.save_to_file(&recent_file_path);
            if is_fav {
                self.status_message = Some(format!("お気に入りに追加: {:?}", path));
            } else {
                self.status_message = Some(format!("お気に入りを解除: {:?}", path));
            }
        } else {
            self.status_message =
                Some("名前のないバッファはお気に入りに登録できません".to_string());
        }
    }
}
