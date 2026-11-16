//! GUI 設定パネル（Preferences Dialog: 左右2ペイン・モーダルダイアログ）モジュール。
//!
//! ViVi 3/4 の多機能プロパティシートを踏襲・洗練させ、6大カテゴリの設定項目を
//! リアルタイムプレビューおよび TOML 永続化可能なモーダルウィンドウとして提供します。

mod tabs_advanced;
mod tabs_basic;

use eframe::egui;
use vivivi_app::config::AppConfig;
use vivivi_app::theme::Theme;

use super::ViviviGuiApp;

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
        let pal = self.palette();

        // 1. エディタ全体を暗転（Dim）し、背後へのマウス操作を遮断するモーダルバックドロップ（Order::Middle）
        let backdrop_alpha = if pal.is_dark { 150 } else { 110 };
        egui::Area::new(egui::Id::new("settings_modal_backdrop_area"))
            .order(egui::Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_black_alpha(backdrop_alpha),
                );
                ui.allocate_response(screen_rect.size(), egui::Sense::click_and_drag());
            });

        let mut is_open = self.settings_dialog.is_open;
        let active_theme_name = self.active_theme.meta.name.clone();

        let mut should_cancel = false;
        let mut should_save = false;

        const SETTINGS_DIALOG_WIDTH: f32 = 920.0;
        const SETTINGS_DIALOG_HEIGHT: f32 = 640.0;
        const SETTINGS_LEFT_PANE_WIDTH: f32 = 236.0;
        let dialog_width = (screen_rect.width() - 32.0).clamp(480.0, SETTINGS_DIALOG_WIDTH);
        let dialog_height = (screen_rect.height() - 32.0).clamp(420.0, SETTINGS_DIALOG_HEIGHT);

        let default_pos = egui::pos2(
            (screen_rect.center().x - dialog_width * 0.5).max(16.0),
            (screen_rect.center().y - dialog_height * 0.5).max(16.0),
        );

        egui::Window::new(format!("設定 (Preferences) - [{active_theme_name}]"))
            .id(egui::Id::new("vivivi_preferences_modal_v2"))
            .order(egui::Order::Foreground)
            .default_pos(default_pos)
            .open(&mut is_open)
            .fixed_size([dialog_width, dialog_height])
            .min_width(dialog_width)
            .max_width(dialog_width)
            .min_height(dialog_height)
            .max_height(dialog_height)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // --- 上部・中央: 左右 2 ペイン領域 ---
                    let available_height = (ui.available_height() - 44.0).max(300.0);
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), available_height),
                        egui::Layout::left_to_right(egui::Align::TOP),
                        |ui| {
                            // [左ペイン: カテゴリツリー] (幅固定・単一行表示)
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(SETTINGS_LEFT_PANE_WIDTH, available_height),
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    ui.set_width(SETTINGS_LEFT_PANE_WIDTH);
                                    self.render_settings_left_pane(ui);
                                },
                            );

                            ui.separator();

                            // [右ペイン: カテゴリ別詳細設定] (固定残余幅、長文は右ペイン幅で折り返し)
                            let right_pane_width =
                                (dialog_width - SETTINGS_LEFT_PANE_WIDTH - 44.0).max(280.0);
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(right_pane_width, available_height),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.set_width(right_pane_width);
                                    ui.set_max_width(right_pane_width);
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                    egui::ScrollArea::vertical()
                                        .id_salt("settings_right_pane_scroll")
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            let inner_w = (right_pane_width - 14.0).max(260.0);
                                            ui.set_width(inner_w);
                                            ui.set_max_width(inner_w);
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

    /// 折り返しを禁止した単一行ラベル（項目名・コマンド列の文字途切れ防止用）
    pub(super) fn nowrap_label(
        ui: &mut egui::Ui,
        text: impl Into<egui::WidgetText>,
    ) -> egui::Response {
        ui.add(egui::Label::new(text).wrap_mode(egui::TextWrapMode::Extend))
    }

    /// 左ペイン: カテゴリ選択ボタン一覧
    fn render_settings_left_pane(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .id_salt("settings_category_list")
            .show(ui, |ui| {
                // 左ペインのカテゴリ項目名は途中改行させず均一幅の選択行として表示
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
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

    /// 最下部: 4 大アクションボタンバー
    fn render_settings_bottom_bar(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        should_save: &mut bool,
        should_cancel: &mut bool,
    ) {
        let pal = self.palette();
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
                let save_fg = if pal.is_dark {
                    pal.bg_panel
                } else {
                    egui::Color32::WHITE
                };
                let save_btn = ui
                    .add(
                        egui::Button::new(egui::RichText::new("💾 保存して閉じる").color(save_fg))
                            .fill(pal.accent_blue),
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
        if !Self::theme_name_matches(&self.active_theme.meta.name, &backup.appearance.theme) {
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
        if !Self::theme_name_matches(&self.active_theme.meta.name, &draft.appearance.theme) {
            self.active_theme = Theme::load_by_name(&draft.appearance.theme);
            crate::theme::apply_theme(ctx, &self.active_theme);
            self.syntax_cache.clear();
        }
    }
}
