//! GUI 設定パネル（Preferences Dialog: 左右2ペイン・モーダルダイアログ）モジュール。
//!
//! ViVi 3/4 の多機能プロパティシートを踏襲・洗練させ、6大カテゴリの設定項目を
//! リアルタイムプレビューおよび TOML 永続化可能なモーダルウィンドウとして提供します。

use eframe::egui;
use vivivi_app::config::AppConfig;
use vivivi_app::theme::Theme;

use super::ViviviGuiApp;
use crate::theme::colors;

/// 設定ダイアログの 6 大カテゴリ定義
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    /// 1. 一般 (General: 文字コード、改行、クリップボード、スワップ、更新)
    General,
    /// 2. 表示・外観 (Appearance: カラーテーマ、行番号、折り返し、フォント、ミニマップ)
    Appearance,
    /// 3. 不可視文字・空白 (Invisibles: 全角スペース、タブ、改行記号)
    Invisibles,
    /// 4. 入力・編集 (Editing: タブ幅、展開、自動インデント、検索オプション)
    Editing,
    /// 5. 独自ツール (Tools: 重力落下アニメーション、省電力制御、Migemo、マクロ)
    Tools,
    /// 6. キーマップ (Keymap: 主要キーバインド確認リファレンス)
    Keymap,
}

impl SettingsCategory {
    /// 全カテゴリ一覧
    pub fn all() -> &'static [SettingsCategory] {
        &[
            SettingsCategory::General,
            SettingsCategory::Appearance,
            SettingsCategory::Invisibles,
            SettingsCategory::Editing,
            SettingsCategory::Tools,
            SettingsCategory::Keymap,
        ]
    }

    /// 日本語表示ラベル
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsCategory::General => "1. 一般 (General)",
            SettingsCategory::Appearance => "2. 表示・外観 (Appearance)",
            SettingsCategory::Invisibles => "3. 不可視文字 (Invisibles)",
            SettingsCategory::Editing => "4. 入力・編集 (Editing)",
            SettingsCategory::Tools => "5. 独自ツール (Tools)",
            SettingsCategory::Keymap => "6. キーマップ (Keymap)",
        }
    }

    /// アイコン
    pub fn icon(&self) -> &'static str {
        match self {
            SettingsCategory::General => "⚙",
            SettingsCategory::Appearance => "☼",
            SettingsCategory::Invisibles => "👁",
            SettingsCategory::Editing => "✏",
            SettingsCategory::Tools => "⚒",
            SettingsCategory::Keymap => "⌨",
        }
    }
}

/// 設定ダイアログの状態管理
#[derive(Debug, Clone)]
pub struct SettingsDialogState {
    /// ダイアログが開いているかどうか
    pub is_open: bool,
    /// 現在選択中のカテゴリ
    pub active_category: SettingsCategory,
    /// 検索フィルタ文字列
    pub search_query: String,
    /// ダイアログ内で編集中の一時ドラフト設定
    pub draft_config: AppConfig,
    /// ダイアログを開いた時点のバックアップ設定（キャンセル復元用）
    pub backup_config: AppConfig,
    /// 不可視文字記号編集用バッファ (char / String)
    pub space_char_buf: String,
    pub eol_char_buf: String,
}

impl SettingsDialogState {
    /// 初期状態を生成します。
    pub fn new(current_config: &AppConfig) -> Self {
        Self {
            is_open: false,
            active_category: SettingsCategory::General,
            search_query: String::new(),
            draft_config: current_config.clone(),
            backup_config: current_config.clone(),
            space_char_buf: current_config.invisibles.space.to_string(),
            eol_char_buf: current_config.invisibles.eol.to_string(),
        }
    }

    /// 設定ダイアログを開きます。
    pub fn open(&mut self, current_config: &AppConfig) {
        self.draft_config = current_config.clone();
        self.backup_config = current_config.clone();
        self.space_char_buf = current_config.invisibles.space.to_string();
        self.eol_char_buf = current_config.invisibles.eol.to_string();
        self.is_open = true;
    }

    /// 設定ダイアログを閉じます。
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

impl ViviviGuiApp {
    /// 設定モーダルダイアログを描画します。
    pub fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.settings_dialog.is_open {
            return;
        }

        let screen_rect = ctx.screen_rect();

        // 1. エディタ全体を暗転（Dim / ぼかし風）し、背後を操作不能にする半透明バックドロップ
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("settings_modal_backdrop_painter"),
        ));
        painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(150));

        // 2. 背後へのマウスクリック・ドラッグを100%遮断する透明ブロッカー
        egui::Area::new(egui::Id::new("settings_modal_backdrop_area"))
            .order(egui::Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                ui.allocate_response(screen_rect.size(), egui::Sense::click_and_drag());
            });

        let mut is_open = self.settings_dialog.is_open;
        let active_theme_name = self.active_theme.meta.name.clone();

        let mut should_cancel = false;
        let mut should_save = false;

        egui::Window::new(format!("設定 (Preferences) - [{active_theme_name}]"))
            .id(egui::Id::new("vivivi_preferences_modal_v2"))
            .open(&mut is_open)
            .default_size([580.0, 620.0])
            .min_size([480.0, 480.0])
            .max_width(660.0)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // --- 上部・中央: 左右 2 ペイン領域 ---
                    let available_height = (ui.available_height() - 44.0).max(300.0);
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), available_height),
                        egui::Layout::left_to_right(egui::Align::TOP),
                        |ui| {
                            // [左ペイン: カテゴリツリー] (幅 180px)
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(180.0, available_height),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    self.render_settings_left_pane(ui);
                                },
                            );

                            ui.separator();

                            // [右ペイン: カテゴリ別詳細設定] (残り幅、縦スクロール & 確実な上から下配置)
                            let right_pane_width = (ui.available_width() - 8.0).max(260.0);
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(right_pane_width, available_height),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    egui::ScrollArea::vertical()
                                        .id_salt("settings_right_pane_scroll")
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                self.render_settings_right_pane(ui, ctx);
                                            });
                                        });
                                },
                            );
                        },
                    );

                    ui.separator();

                    // --- 最下部: 4 大アクションボタンバー ---
                    self.render_settings_bottom_bar(ui, ctx, &mut should_save, &mut should_cancel);
                });
            });

        // 右上の「×」ボタンで閉じられた場合、またはキャンセルが押された場合
        if !is_open || should_cancel {
            self.cancel_settings_dialog(Some(ctx));
        } else if should_save {
            self.save_and_close_settings(ctx);
        }
    }

    /// 左ペイン: カテゴリ選択ボタン一覧
    fn render_settings_left_pane(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .id_salt("settings_category_list")
            .show(ui, |ui| {
                for cat in SettingsCategory::all() {
                    let name = cat.display_name();
                    let icon = cat.icon();
                    let label = format!("{icon} {name}");

                    let is_active = self.settings_dialog.active_category == *cat;
                    let resp = ui
                        .selectable_label(is_active, label)
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if resp.clicked() {
                        self.settings_dialog.active_category = *cat;
                    }
                }
            });
    }

    /// 右ペイン: 選択中カテゴリの設定コントロール群
    fn render_settings_right_pane(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| match self.settings_dialog.active_category {
            SettingsCategory::General => self.render_category_general(ui),
            SettingsCategory::Appearance => self.render_category_appearance(ui, ctx),
            SettingsCategory::Invisibles => self.render_category_invisibles(ui),
            SettingsCategory::Editing => self.render_category_editing(ui),
            SettingsCategory::Tools => self.render_category_tools(ui),
            SettingsCategory::Keymap => self.render_category_keymap(ui),
        });
    }

    /// 1. 一般 (General)
    fn render_category_general(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙ 1. 一般設定 (General)");
        ui.label(
            egui::RichText::new(
                "エディタの基本動作、エンコーディング、および安全制御を設定します。",
            )
            .color(colors::FG_DIMMED),
        );
        ui.separator();

        egui::Grid::new("general_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                ui.label("デフォルト文字コード (Encoding):");
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

                ui.label("デフォルト改行形式 (Fileformat):");
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

                ui.label("クリップボード同期 (Clipboard):");
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
        ui.horizontal(|ui| {
            if ui.button("🔄 今すぐ更新を確認 (:checkupdate)").clicked() {
                self.core.check_for_updates(true);
            }

            match self.core.update_status() {
                moult_update::UpdateStatus::UpdateAvailable(ref rel) => {
                    ui.label(
                        egui::RichText::new(format!("⬆ v{} が利用可能です", rel.version))
                            .color(colors::ACCENT_CYAN)
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
                    ui.label(egui::RichText::new(text).color(colors::ACCENT_YELLOW));
                }
                moult_update::UpdateStatus::ReadyToApply { ref release, .. } => {
                    ui.label(
                        egui::RichText::new(format!("✨ v{} 準備完了", release.version))
                            .color(colors::ACCENT_GREEN)
                            .strong(),
                    );
                    if ui.button("反映して再起動").clicked() {
                        let _ = self.core.apply_update_and_restart();
                    }
                }
                moult_update::UpdateStatus::UpToDate => {
                    ui.label(
                        egui::RichText::new("✓ 最新バージョンを利用中です")
                            .color(colors::FG_DIMMED),
                    );
                }
                moult_update::UpdateStatus::Checking => {
                    ui.label(egui::RichText::new("照会中...").color(colors::ACCENT_BLUE));
                }
                moult_update::UpdateStatus::Failed(ref err) => {
                    ui.label(egui::RichText::new(format!("⚠️ {err}")).color(colors::ACCENT_RED));
                }
                _ => {}
            }
        });
    }

    /// 2. 表示・外観 (Appearance)
    fn render_category_appearance(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("☼ 2. 表示・外観設定 (Appearance)");
        ui.label(
            egui::RichText::new("カラーテーマ、行番号、ガター、およびGUIフォントを設定します。")
                .color(colors::FG_DIMMED),
        );
        ui.separator();

        ui.group(|ui| {
            ui.label(egui::RichText::new("【エディタ外観・テーマ】").strong());
            egui::Grid::new("appearance_grid")
                .num_columns(2)
                .spacing([20.0, 10.0])
                .show(ui, |ui| {
                    ui.label("カラーテーマ (Color Theme):");
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
            ui.label(egui::RichText::new("【GUI・フォント・ミニマップ】").strong());
            egui::Grid::new("gui_appearance_grid")
                .num_columns(2)
                .spacing([20.0, 10.0])
                .show(ui, |ui| {
                    ui.label("フォントサイズ (Font Size):");
                    ui.add(
                        egui::Slider::new(
                            &mut self.settings_dialog.draft_config.appearance.gui.font_size,
                            10.0..=28.0,
                        )
                        .text("pt"),
                    );
                    ui.end_row();

                    ui.label("行間倍率 (Line Height):");
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

                    ui.label("ミニマップ幅 (Minimap Width):");
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
    fn render_category_invisibles(&mut self, ui: &mut egui::Ui) {
        ui.heading("👁 3. 不可視文字・空白設定 (Invisibles)");
        ui.label(
            egui::RichText::new("全角スペース、タブ文字、改行記号の可視化スタイルを設定します。")
                .color(colors::FG_DIMMED),
        );
        ui.separator();

        ui.checkbox(
            &mut self.settings_dialog.draft_config.invisibles.enabled,
            "不可視文字の可視化を有効にする (:set list)",
        );

        ui.add_space(10.0);
        egui::Grid::new("invisibles_grid")
            .num_columns(2)
            .spacing([16.0, 10.0])
            .show(ui, |ui| {
                ui.label("全角スペース記号:");
                ui.add(
                    egui::TextEdit::singleline(
                        &mut self.settings_dialog.draft_config.invisibles.zenkaku_space,
                    )
                    .desired_width(60.0),
                );
                ui.end_row();

                ui.label("半角スペース記号:");
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
                    ui.label(egui::RichText::new("（例: · や 空白）").color(colors::FG_DIMMED));
                });
                ui.end_row();

                ui.label("タブ文字記号 (Tab):");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut self.settings_dialog.draft_config.invisibles.tab,
                        )
                        .desired_width(60.0),
                    );
                    ui.label(egui::RichText::new("（例: ▸--- や >---）").color(colors::FG_DIMMED));
                });
                ui.end_row();

                ui.label("改行記号 (EOL):");
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
                    ui.label(egui::RichText::new("（例: $ や ↵）").color(colors::FG_DIMMED));
                });
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("※ 全角スペースは2マス幅・ワインレッド背景で強調表示されます。")
                .color(colors::FG_DIMMED),
        );
    }

    /// 4. 入力・編集 (Editing)
    fn render_category_editing(&mut self, ui: &mut egui::Ui) {
        ui.heading("✏ 4. 入力・編集設定 (Editing)");
        ui.label(
            egui::RichText::new("タブ幅、インデント、および検索のマッチング挙動を設定します。")
                .color(colors::FG_DIMMED),
        );
        ui.separator();

        egui::Grid::new("editing_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                ui.label("タブ幅 (Tab Stop):");
                ui.add(
                    egui::Slider::new(
                        &mut self.settings_dialog.draft_config.editing.tab_width,
                        1..=16,
                    )
                    .text("マス"),
                );
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.expand_tab,
            "タブ文字入力を半角スペースに展開する (Expand Tab)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.keisen_protect,
            "罫線保護編集を行う (枠線内の文字挿入・削除時に自動空白伸縮)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.match_paren,
            "カーソル位置の対応括弧を強調表示する (%)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.dabbrev_ignorecase,
            "動的補完 (dabbrev / Ctrl-L, Ctrl-M) で大文字小文字を同一視する",
        );

        ui.add_space(5.0);
        ui.separator();
        ui.label(egui::RichText::new("【検索オプション】").strong());
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.ignorecase,
            "検索時に英大文字・小文字を同一視する (ignorecase / :set ic)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.smartcase,
            "大文字が含まれる場合のみ大文字・小文字を厳密に区別する (smartcase / :set scs)",
        );
    }

    /// 5. 独自ツール (Tools)
    fn render_category_tools(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚒ 5. 独自ツール設定 (Tools / OpenViVi)");
        ui.label(
            egui::RichText::new("OpenViVi 特有のビジュアルギミックや入力支援ツールを設定します。")
                .color(colors::FG_DIMMED),
        );
        ui.separator();

        ui.checkbox(
            &mut self
                .settings_dialog
                .draft_config
                .appearance
                .gui
                .enable_gravity_drop,
            "文字削除時の物理重力落下アニメーション (Particles / OpenViVi演出)",
        );
        ui.label(
            egui::RichText::new("  ※Normalモードでの 'x' や削除時に、文字が物理演算で落下します。")
                .color(colors::FG_DIMMED),
        );

        ui.add_space(5.0);
        ui.checkbox(
            &mut self
                .settings_dialog
                .draft_config
                .appearance
                .gui
                .reactive_idle_sleep,
            "アイドル時 CPU 0% 省電力リアクティブ制御 (Reactive Sleep)",
        );
        ui.label(
            egui::RichText::new(
                "  ※キー入力やアニメーションがない待機時のCPU消費をゼロに抑えます。",
            )
            .color(colors::FG_DIMMED),
        );

        ui.add_space(5.0);
        ui.separator();
        ui.label(egui::RichText::new("【インテリジェント入力支援機能】").strong());
        ui.label("• ローマ字日本語検索 (Migemo): 'g/' キーでローマ字のまま漢字バッファを検索");
        ui.label("• ダイナミックマクロ (Dynamic Macro): 'Alt-M' で直前の繰り返し操作を自動再現");
        ui.label("• 字句編集予測 (LexEdit): 'Ctrl-J' でバッファ内の文脈から次の入力を自動予測");
        ui.label("• 静的略語展開 (Abbreviation): 'Ctrl-;' で登録略語を展開");
        ui.label("• Zen-Coding (Emmet): HTML/XMLタグを 'Ctrl-E' で即時展開");
    }

    /// 6. キーマップ (Keymap)
    fn render_category_keymap(&mut self, ui: &mut egui::Ui) {
        ui.heading("⌨ 6. キーマップ・ショートカット (Keymap)");
        ui.label(
            egui::RichText::new("主要な操作モードおよびキーバインドの確認リファレンスです。")
                .color(colors::FG_DIMMED),
        );
        ui.separator();

        egui::Grid::new("keymap_reference_grid")
            .striped(true)
            .num_columns(3)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("モード / キー").strong());
                ui.label(egui::RichText::new("機能・コマンド").strong());
                ui.label(egui::RichText::new("説明").strong());
                ui.end_row();

                let keymaps = [
                    ("Normal", "h, j, k, l", "左・下・上・右カーソル移動"),
                    (
                        "Normal",
                        "i, a, o",
                        "Insertモード移行（カーソル位置・直後・次行）",
                    ),
                    (
                        "Normal",
                        "v, V, Ctrl-v",
                        "Visualモード（文字・行・矩形選択）",
                    ),
                    ("Normal", ":", "Ex コマンドライン入力へ移行"),
                    (
                        "Normal",
                        "u / Ctrl-r",
                        "アンドゥ（元に戻す） / リドゥ（やり直し）",
                    ),
                    ("Normal", "x", "1文字削除（重力落下アニメーション）"),
                    ("Normal", "g/", "Migemo ローマ字日本語検索"),
                    ("Normal", "F1 / :help", "オンラインヘルプビューア起動"),
                    ("Normal", "F6", "クリップボード履歴モーダル表示"),
                    ("Normal", ":filer", "2ペイン・ファイラー起動"),
                    ("Normal", ":preferences", "設定ダイアログ表示 (Ctrl+,)"),
                    ("GUI", "Ctrl+,", "設定ダイアログを開く"),
                    (
                        "GUI",
                        "Ctrl+n / Ctrl+w",
                        "新規タブ作成 / カレントタブを閉じる",
                    ),
                    ("Insert", "Ctrl-[", "Normal モードへ復帰 (Esc互換)"),
                    ("Insert", "Ctrl-j", "字句編集予測 (LexEdit)"),
                    ("Insert", "Ctrl-;", "静的略語展開 (Abbrev)"),
                ];

                for (mode, keys, desc) in keymaps {
                    ui.label(mode);
                    ui.label(
                        egui::RichText::new(keys)
                            .family(egui::FontFamily::Monospace)
                            .color(colors::ACCENT_CYAN),
                    );
                    ui.label(desc);
                    ui.end_row();
                }
            });
    }

    /// 最下部: 4 大アクションボタンバー
    fn render_settings_bottom_bar(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        should_save: &mut bool,
        should_cancel: &mut bool,
    ) {
        ui.horizontal(|ui| {
            // [デフォルトに戻す]
            let reset_btn = ui
                .button("🔄 デフォルトに戻す")
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if reset_btn.clicked() {
                self.settings_dialog.draft_config = AppConfig::default();
                self.settings_dialog.space_char_buf = self
                    .settings_dialog
                    .draft_config
                    .invisibles
                    .space
                    .to_string();
                self.settings_dialog.eol_char_buf =
                    self.settings_dialog.draft_config.invisibles.eol.to_string();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // [保存して閉じる]
                let save_btn = ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("💾 保存して閉じる").color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(45, 120, 200)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if save_btn.clicked() {
                    *should_save = true;
                }

                // [適用] (即時プレビュー)
                let apply_btn = ui
                    .button("⚡ 適用")
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if apply_btn.clicked() {
                    self.apply_dialog_config(ctx);
                    self.core.status_message = Some("設定を一時適用しました".to_string());
                }

                // [キャンセル]
                let cancel_btn = ui
                    .button("❌ キャンセル")
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if cancel_btn.clicked() {
                    *should_cancel = true;
                }
            });
        });
    }

    /// 設定ダイアログの変更をキャンセルし、開いた時点のバックアップ設定に復元して閉じます。
    pub fn cancel_settings_dialog(&mut self, ctx: Option<&egui::Context>) {
        let backup = self.settings_dialog.backup_config.clone();
        self.core.apply_config(&backup);
        self.minimap_open = backup.appearance.gui.show_minimap;
        self.minimap_width = backup.appearance.gui.minimap_width;
        self.particles.enabled = backup.appearance.gui.enable_gravity_drop;
        if self.active_theme.meta.name != backup.appearance.theme {
            self.active_theme = Theme::load_by_name(&backup.appearance.theme);
            if let Some(ctx) = ctx {
                crate::theme::apply_theme(ctx, &self.active_theme);
            }
            self.syntax_cache.clear();
        }
        self.settings_dialog.close();
    }

    /// 設定ダイアログの内容を適用してディスクへ永続化し、閉じます。
    pub fn save_and_close_settings(&mut self, ctx: &egui::Context) {
        self.apply_dialog_config(ctx);
        let path = AppConfig::default_config_path();
        match self.core.config.save_to(&path) {
            Ok(()) => {
                self.core.status_message = Some(format!("設定を保存しました: {}", path.display()));
            }
            Err(e) => {
                self.core.status_message = Some(format!("設定の保存に失敗しました: {e}"));
            }
        }
        self.settings_dialog.close();
    }

    /// ダイアログのドラフト設定を現在の実行時エディタ（core および GUI）へ適用します。
    pub fn apply_dialog_config(&mut self, ctx: &egui::Context) {
        let draft = self.settings_dialog.draft_config.clone();
        self.core.apply_config(&draft);
        self.minimap_open = draft.appearance.gui.show_minimap;
        self.minimap_width = draft.appearance.gui.minimap_width;
        self.particles.enabled = draft.appearance.gui.enable_gravity_drop;
        if self.active_theme.meta.name != draft.appearance.theme {
            self.active_theme = Theme::load_by_name(&draft.appearance.theme);
            crate::theme::apply_theme(ctx, &self.active_theme);
            self.syntax_cache.clear();
        }
    }
}
