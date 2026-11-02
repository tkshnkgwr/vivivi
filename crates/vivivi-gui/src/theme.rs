//! テーマ設定・ビジュアルスタイルモジュール。
//!
//! `vivivi-app` の共通テーマ（[`Theme`]）から `egui::Visuals` を動的に構築・適用し、
//! 3段階の背景明淡、余計な枠線（Stroke）の排除、および統一された角丸スタイルを提供します。

use egui::{Color32, CornerRadius, Stroke, Visuals};
use vivivi_app::theme::{Theme, ThemeColor};

/// `ThemeColor` を `egui::Color32` へ変換します。
pub fn to_color32(c: ThemeColor) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
}

/// Tokyo Night パレット定数群（互換用）。
#[allow(dead_code)]
pub mod colors {
    use egui::Color32;

    pub const BG_EDITOR: Color32 = Color32::from_rgb(26, 27, 38);
    pub const BG_PANEL: Color32 = Color32::from_rgb(22, 22, 30);
    pub const BG_STATUSLINE: Color32 = Color32::from_rgb(31, 35, 53);
    pub const BG_SELECTION: Color32 = Color32::from_rgb(40, 52, 87);
    pub const BG_HOVER: Color32 = Color32::from_rgb(41, 46, 66);
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(65, 72, 104);

    pub const FG_TEXT: Color32 = Color32::from_rgb(192, 202, 245);
    pub const FG_DIMMED: Color32 = Color32::from_rgb(86, 95, 137);
    pub const FG_LINENUMBER: Color32 = Color32::from_rgb(59, 66, 97);

    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(122, 162, 247);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(158, 206, 106);
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(187, 154, 247);
    pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(224, 175, 104);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(247, 118, 142);
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(125, 207, 255);
}

/// テーマ定義（[`Theme`]）から `egui::Visuals` を構築して適用します。
pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let is_light = theme.meta.variant.eq_ignore_ascii_case("light");
    let mut visuals = if is_light {
        Visuals::light()
    } else {
        Visuals::dark()
    };

    // 1. 3段階の背景コントラスト
    visuals.panel_fill = to_color32(theme.ui.bg_panel);
    visuals.window_fill = to_color32(theme.ui.bg_editor);
    visuals.extreme_bg_color = to_color32(theme.ui.bg_panel);

    // 2. 余計な枠線（Stroke）を排除
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    visuals.window_stroke = Stroke::NONE;

    // 3. フラットデザイン統一：ボタン・タブ・ウィンドウ角丸は 2.0px
    let corner_2 = CornerRadius::same(2);
    visuals.widgets.noninteractive.corner_radius = corner_2;
    visuals.widgets.inactive.corner_radius = corner_2;
    visuals.widgets.hovered.corner_radius = corner_2;
    visuals.widgets.active.corner_radius = corner_2;
    visuals.widgets.open.corner_radius = corner_2;
    visuals.window_corner_radius = corner_2;
    visuals.menu_corner_radius = corner_2;

    // 4. ウィジェット背景色
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_fill = to_color32(theme.ui.bg_statusline);
    visuals.widgets.hovered.bg_fill = to_color32(theme.ui.bg_hover);
    visuals.widgets.active.bg_fill = to_color32(theme.ui.bg_active);

    // 5. テキスト色および選択領域
    visuals.override_text_color = Some(to_color32(theme.ui.fg_text));
    visuals.selection.bg_fill = to_color32(theme.ui.bg_selection);
    visuals.selection.stroke = Stroke::NONE;

    ctx.set_visuals(visuals);
}

/// Tokyo Night をデフォルトスタイルとして初期化します。
pub fn setup_visuals(ctx: &egui::Context) {
    apply_theme(ctx, &Theme::builtin_tokyo_night());
}
