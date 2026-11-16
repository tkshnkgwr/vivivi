//! 設定ダイアログ前半カテゴリ（1. 一般 / 2. 表示・外観 / 3. 不可視文字）描画モジュール。

use eframe::egui;
use vivivi_app::theme::Theme;

use super::super::ViviviGuiApp;

impl ViviviGuiApp {
    /// 1. 一般 (General)
    pub(super) fn render_category_general(&mut self, ui: &mut egui::Ui) {
        let pal = self.palette();
        ui.heading("⚙ 1. 一般設定 (General)");
        ui.label(
            egui::RichText::new(
                "エディタの基本動作、エンコーディング、および安全制御を設定します。",
            )
            .color(pal.fg_dimmed),
        );
        ui.separator();

        egui::Grid::new("general_grid")
            .num_columns(2)
            .spacing([24.0, 10.0])
            .show(ui, |ui| {
                Self::nowrap_label(ui, "デフォルト文字コード (Encoding):");
                egui::ComboBox::from_id_salt("general_encoding")
                    .selected_text(&self.settings_dialog.draft_config.general.default_encoding)
                    .show_ui(ui, |ui| {
                        for enc in &["utf-8", "sjis", "euc-jp"] {
                            ui.selectable_value(
                                &mut self.settings_dialog.draft_config.general.default_encoding,
                                enc.to_string(),
                                *enc,
                            );
                        }
                    });
                ui.end_row();

                Self::nowrap_label(ui, "デフォルト改行形式 (Fileformat):");
                egui::ComboBox::from_id_salt("general_ff")
                    .selected_text(
                        match self
                            .settings_dialog
                            .draft_config
                            .general
                            .default_fileformat
                            .as_str()
                        {
                            "dos" => "Windows (CRLF / dos)",
                            _ => "Unix / Linux / macOS (LF / unix)",
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.settings_dialog.draft_config.general.default_fileformat,
                            "unix".to_string(),
                            "Unix / Linux / macOS (LF / unix)",
                        );
                        ui.selectable_value(
                            &mut self.settings_dialog.draft_config.general.default_fileformat,
                            "dos".to_string(),
                            "Windows (CRLF / dos)",
                        );
                    });
                ui.end_row();

                Self::nowrap_label(ui, "クリップボード同期 (Clipboard):");
                egui::ComboBox::from_id_salt("general_clipboard")
                    .selected_text(
                        match self.settings_dialog.draft_config.general.clipboard.as_str() {
                            "unnamed" => "unnamed (* レジスタ同期)",
                            "unnamedplus" => "unnamedplus (+ レジスタ同期)",
                            _ => "none (同期無効 / 内部独立)",
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.settings_dialog.draft_config.general.clipboard,
                            "none".to_string(),
                            "none (同期無効 / 内部独立)",
                        );
                        ui.selectable_value(
                            &mut self.settings_dialog.draft_config.general.clipboard,
                            "unnamed".to_string(),
                            "unnamed (* レジスタ同期)",
                        );
                        ui.selectable_value(
                            &mut self.settings_dialog.draft_config.general.clipboard,
                            "unnamedplus".to_string(),
                            "unnamedplus (+ レジスタ同期)",
                        );
                    });
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.checkbox(
            &mut self.settings_dialog.draft_config.general.crash_recovery,
            "異常終了・クラッシュ時の自動スワップ保存 (:recover 有効化)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.general.check_update,
            "起動時の更新確認 (Check for Updates)",
        );

        ui.add_space(8.0);
        let status = self.core.update_status();
        ui.horizontal_wrapped(|ui| {
            if ui.button("🔄 今すぐ更新を確認 (:checkupdate)").clicked() {
                self.core.check_for_updates(true);
            }

            match status {
                moult_update::UpdateStatus::UpdateAvailable(ref rel) => {
                    Self::nowrap_label(
                        ui,
                        egui::RichText::new(format!("⬆ v{} が利用可能です", rel.version))
                            .color(pal.accent_cyan)
                            .strong(),
                    );
                    if ui.button("ダウンロード").clicked() {
                        self.core.download_update();
                    }
                }
                moult_update::UpdateStatus::Downloading { downloaded, total } => {
                    let text = if let Some(tot) = total {
                        let pct = (downloaded * 100).checked_div(tot).unwrap_or(0);
                        format!("ダウンロード中... {pct}%")
                    } else {
                        "ダウンロード中...".to_string()
                    };
                    Self::nowrap_label(ui, egui::RichText::new(text).color(pal.accent_yellow));
                }
                moult_update::UpdateStatus::ReadyToApply { ref release, .. } => {
                    Self::nowrap_label(
                        ui,
                        egui::RichText::new(format!("✨ v{} 準備完了", release.version))
                            .color(pal.accent_green)
                            .strong(),
                    );
                    if ui.button("反映して再起動").clicked() {
                        let _ = self.core.apply_update_and_restart();
                    }
                }
                moult_update::UpdateStatus::UpToDate => {
                    Self::nowrap_label(
                        ui,
                        egui::RichText::new("✓ 最新バージョンを利用中です").color(pal.fg_dimmed),
                    );
                }
                moult_update::UpdateStatus::Checking => {
                    Self::nowrap_label(ui, egui::RichText::new("照会中...").color(pal.accent_blue));
                }
                moult_update::UpdateStatus::Failed(_) => {}
                _ => {}
            }
        });

        // 長いネットワーク/TLS エラー文は別行で確実に折り返し表示し、ウィンドウ幅の拡張を防止
        if let moult_update::UpdateStatus::Failed(ref err) = status {
            ui.add_space(4.0);
            ui.add(
                egui::Label::new(egui::RichText::new(format!("⚠️ {err}")).color(pal.accent_red))
                    .wrap_mode(egui::TextWrapMode::Wrap),
            );
        }
    }

    /// 2. 表示・外観 (Appearance)
    pub(super) fn render_category_appearance(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let pal = self.palette();
        ui.heading("☼ 2. 表示・外観設定 (Appearance)");
        ui.label(
            egui::RichText::new("カラーテーマ、行番号、ガター、およびGUIフォントを設定します。")
                .color(pal.fg_dimmed),
        );
        ui.separator();

        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("【エディタ外観・テーマ】").strong());
            egui::Grid::new("appearance_grid")
                .num_columns(2)
                .spacing([24.0, 10.0])
                .show(ui, |ui| {
                    Self::nowrap_label(ui, "カラーテーマ (Color Theme):");
                    let current_theme = self.settings_dialog.draft_config.appearance.theme.clone();
                    egui::ComboBox::from_id_salt("appearance_theme_combo")
                        .selected_text(&current_theme)
                        .show_ui(ui, |ui| {
                            let available = Theme::list_available_themes();
                            for t in available {
                                ui.selectable_value(
                                    &mut self.settings_dialog.draft_config.appearance.theme,
                                    t.clone(),
                                    t,
                                );
                            }
                        });
                    ui.end_row();
                });

            ui.add_space(5.0);
            ui.checkbox(
                &mut self.settings_dialog.draft_config.appearance.line_number,
                "行番号を表示する (Line Numbers / :set number)",
            );
            ui.checkbox(
                &mut self.settings_dialog.draft_config.appearance.relative_number,
                "相対行番号を表示する (Relative Numbers / :set relativenumber)",
            );
            ui.checkbox(
                &mut self.settings_dialog.draft_config.appearance.wrap,
                "画面端で折り返す (Word Wrap / :set wrap)",
            );
            ui.checkbox(
                &mut self.settings_dialog.draft_config.appearance.gutter_markers,
                "変更行ガター強調を表示する (Change Markers: 黄=未保存, 緑=保存済)",
            );
        });

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("【GUI・カーソル・フォント・ミニマップ】").strong());
            ui.spacing_mut().slider_width = 180.0;
            egui::Grid::new("gui_appearance_grid")
                .num_columns(2)
                .spacing([24.0, 10.0])
                .show(ui, |ui| {
                    Self::nowrap_label(ui, "カーソル形状 (Cursor Shape):");
                    ui.horizontal_wrapped(|ui| {
                        for shape in [
                            vivivi_app::CursorShape::Auto,
                            vivivi_app::CursorShape::Block,
                            vivivi_app::CursorShape::Beam,
                            vivivi_app::CursorShape::Underline,
                        ] {
                            ui.selectable_value(
                                &mut self
                                    .settings_dialog
                                    .draft_config
                                    .appearance
                                    .gui
                                    .cursor_shape,
                                shape,
                                shape.label(),
                            );
                        }
                    });
                    ui.end_row();

                    Self::nowrap_label(ui, "カーソル角丸 (Corner Radius):");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut self
                                    .settings_dialog
                                    .draft_config
                                    .appearance
                                    .gui
                                    .cursor_corner_radius,
                                0.0..=6.0,
                            )
                            .step_by(0.5)
                            .text("px"),
                        );
                        Self::nowrap_label(
                            ui,
                            egui::RichText::new("（0.0px = 直角）").color(pal.fg_dimmed),
                        );
                    });
                    ui.end_row();

                    Self::nowrap_label(ui, "フォントサイズ (Font Size):");
                    ui.add(
                        egui::Slider::new(
                            &mut self.settings_dialog.draft_config.appearance.gui.font_size,
                            10.0..=28.0,
                        )
                        .text("pt"),
                    );
                    ui.end_row();

                    Self::nowrap_label(ui, "行間倍率 (Line Height):");
                    ui.add(
                        egui::Slider::new(
                            &mut self
                                .settings_dialog
                                .draft_config
                                .appearance
                                .gui
                                .line_height_multiplier,
                            1.0..=2.0,
                        )
                        .text("x"),
                    );
                    ui.end_row();

                    Self::nowrap_label(ui, "ミニマップ幅 (Minimap Width):");
                    ui.add(
                        egui::Slider::new(
                            &mut self
                                .settings_dialog
                                .draft_config
                                .appearance
                                .gui
                                .minimap_width,
                            40.0..=140.0,
                        )
                        .text("px"),
                    );
                    ui.end_row();
                });

            ui.add_space(5.0);
            ui.checkbox(
                &mut self
                    .settings_dialog
                    .draft_config
                    .appearance
                    .gui
                    .show_minimap,
                "ミニマップを表示する (Show Minimap)",
            );
        });
    }

    /// 3. 不可視文字・空白 (Invisibles)
    pub(super) fn render_category_invisibles(&mut self, ui: &mut egui::Ui) {
        let pal = self.palette();
        ui.heading("👁 3. 不可視文字・空白設定 (Invisibles)");
        ui.label(
            egui::RichText::new("全角スペース、タブ文字、改行記号の可視化スタイルを設定します。")
                .color(pal.fg_dimmed),
        );
        ui.separator();

        ui.checkbox(
            &mut self.settings_dialog.draft_config.invisibles.enabled,
            "不可視文字の可視化を有効にする (:set list)",
        );

        ui.add_space(10.0);
        egui::Grid::new("invisibles_grid")
            .num_columns(2)
            .spacing([24.0, 10.0])
            .show(ui, |ui| {
                Self::nowrap_label(ui, "全角スペース記号:");
                ui.add(
                    egui::TextEdit::singleline(
                        &mut self.settings_dialog.draft_config.invisibles.zenkaku_space,
                    )
                    .desired_width(60.0),
                );
                ui.end_row();

                Self::nowrap_label(ui, "半角スペース記号:");
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.settings_dialog.space_char_buf)
                                .desired_width(40.0),
                        )
                        .changed()
                        && let Some(ch) = self.settings_dialog.space_char_buf.chars().next()
                    {
                        self.settings_dialog.draft_config.invisibles.space = ch;
                    }
                    Self::nowrap_label(
                        ui,
                        egui::RichText::new("（例: · や 空白）").color(pal.fg_dimmed),
                    );
                });
                ui.end_row();

                Self::nowrap_label(ui, "タブ文字記号 (Tab):");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut self.settings_dialog.draft_config.invisibles.tab,
                        )
                        .desired_width(60.0),
                    );
                    Self::nowrap_label(
                        ui,
                        egui::RichText::new("（例: ▸--- や >---）").color(pal.fg_dimmed),
                    );
                });
                ui.end_row();

                Self::nowrap_label(ui, "改行記号 (EOL):");
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.settings_dialog.eol_char_buf)
                                .desired_width(40.0),
                        )
                        .changed()
                        && let Some(ch) = self.settings_dialog.eol_char_buf.chars().next()
                    {
                        self.settings_dialog.draft_config.invisibles.eol = ch;
                    }
                    Self::nowrap_label(
                        ui,
                        egui::RichText::new("（例: $ や ↵）").color(pal.fg_dimmed),
                    );
                });
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("※ 全角スペースは2マス幅・ワインレッド背景で強調表示されます。")
                .color(pal.fg_dimmed),
        );
    }
}
