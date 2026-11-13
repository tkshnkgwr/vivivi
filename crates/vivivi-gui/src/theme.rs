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

/// アクティブな [`Theme`] から導出した GUI 描画用のカラーパレット。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuiPalette {
    /// エディタ背景色
    pub bg_editor: Color32,
    /// パネル・サイドバー背景色
    pub bg_panel: Color32,
    /// ステータスライン基準背景色
    pub bg_statusline: Color32,
    /// 選択範囲背景色
    pub bg_selection: Color32,
    /// ホバー時背景色
    pub bg_hover: Color32,
    /// アクティブ時背景色
    pub bg_active: Color32,
    /// ガター領域背景色
    pub bg_gutter: Color32,
    /// 現在行ハイライト背景色
    pub bg_cur_line: Color32,
    /// パネル境界・セパレータ色
    pub border: Color32,
    /// 標準テキスト色
    pub fg_text: Color32,
    /// 控えめな補助テキスト色
    pub fg_dimmed: Color32,
    /// 行番号テキスト色
    pub fg_linenumber: Color32,
    /// カーソル内テキスト色
    pub fg_cursor: Color32,
    /// カーソル背景色
    pub bg_cursor: Color32,
    /// アクセントカラー（青 / ノーマルモード）
    pub accent_blue: Color32,
    /// アクセントカラー（緑 / 挿入モード）
    pub accent_green: Color32,
    /// アクセントカラー（紫 / ビジュアルモード）
    pub accent_purple: Color32,
    /// アクセントカラー（黄 / コマンド・警告）
    pub accent_yellow: Color32,
    /// アクセントカラー（赤 / 置換・エラー）
    pub accent_red: Color32,
    /// アクセントカラー（シアン / 罫線・情報）
    pub accent_cyan: Color32,
    /// ガター変更マーカー（未保存）
    pub gutter_modified: Color32,
    /// ガター変更マーカー（保存済み）
    pub gutter_saved: Color32,
    /// ダーク系テーマかどうか
    pub is_dark: bool,
}

impl GuiPalette {
    /// 指定された [`Theme`] から GUI 描画用カラーパレットを構築します。
    pub fn from_theme(theme: &Theme) -> Self {
        let bg_editor = to_color32(theme.ui.bg_editor);
        let bg_panel = to_color32(theme.ui.bg_panel);
        let bg_statusline = to_color32(theme.ui.bg_statusline);
        let bg_selection = to_color32(theme.ui.bg_selection);
        let bg_hover = to_color32(theme.ui.bg_hover);
        let bg_active = to_color32(theme.ui.bg_active);
        let fg_text = to_color32(theme.ui.fg_text);
        let fg_dimmed = to_color32(theme.ui.fg_dimmed);
        let fg_linenumber = to_color32(theme.ui.fg_linenumber);
        let fg_cursor = to_color32(theme.ui.fg_cursor);
        let bg_cursor = to_color32(theme.ui.bg_cursor);

        let luminance = 0.299 * bg_editor.r() as f32
            + 0.587 * bg_editor.g() as f32
            + 0.114 * bg_editor.b() as f32;
        let is_dark = luminance < 128.0 && !theme.meta.variant.eq_ignore_ascii_case("light");

        Self {
            bg_editor,
            bg_panel,
            bg_statusline,
            bg_selection,
            bg_hover,
            bg_active,
            bg_gutter: blend_color32(bg_panel, bg_editor, 0.35),
            bg_cur_line: blend_color32(bg_editor, bg_statusline, 0.75),
            border: blend_color32(bg_statusline, fg_dimmed, 0.28),
            fg_text,
            fg_dimmed,
            fg_linenumber,
            fg_cursor,
            bg_cursor,
            accent_blue: to_color32(theme.modes.normal),
            accent_green: to_color32(theme.modes.insert),
            accent_purple: to_color32(theme.modes.visual),
            accent_yellow: to_color32(theme.modes.command),
            accent_red: to_color32(theme.modes.replace),
            accent_cyan: to_color32(theme.modes.keisen),
            gutter_modified: to_color32(theme.gutter.change_modified),
            gutter_saved: to_color32(theme.gutter.change_saved),
            is_dark,
        }
    }

    /// 2色を線形補間します（`t = 0.0` で `a`、`1.0` で `b`）。
    pub fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
        blend_color32(a, b, t)
    }
}

fn blend_color32(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    Color32::from_rgb(
        (a.r() as f32 * inv + b.r() as f32 * t).round() as u8,
        (a.g() as f32 * inv + b.g() as f32 * t).round() as u8,
        (a.b() as f32 * inv + b.b() as f32 * t).round() as u8,
    )
}

/// テーマ定義（[`Theme`]）から `egui::Visuals` を構築して適用します。
pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let palette = GuiPalette::from_theme(theme);
    let mut visuals = if !palette.is_dark {
        Visuals::light()
    } else {
        Visuals::dark()
    };

    // 1. 3段階の背景コントラスト
    visuals.panel_fill = palette.bg_panel;
    visuals.window_fill = palette.bg_editor;
    visuals.extreme_bg_color = palette.bg_panel;
    visuals.faint_bg_color = palette.bg_statusline;

    // 2. 枠線・セパレータ統一
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, palette.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, palette.fg_dimmed);
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, palette.fg_text);
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, palette.fg_text);
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, palette.fg_text);
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, palette.fg_text);
    visuals.window_stroke = Stroke::new(1.0_f32, palette.border);

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
    visuals.widgets.noninteractive.bg_fill = palette.bg_panel;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_fill = palette.bg_statusline;
    visuals.widgets.hovered.weak_bg_fill = palette.bg_hover;
    visuals.widgets.hovered.bg_fill = palette.bg_hover;
    visuals.widgets.active.weak_bg_fill = palette.bg_active;
    visuals.widgets.active.bg_fill = palette.bg_active;
    visuals.widgets.open.weak_bg_fill = palette.bg_hover;
    visuals.widgets.open.bg_fill = palette.bg_hover;

    // 5. テキスト色および選択領域
    visuals.override_text_color = Some(palette.fg_text);
    visuals.selection.bg_fill = palette.bg_selection;
    visuals.selection.stroke = Stroke::new(1.0_f32, palette.accent_blue);

    ctx.set_visuals(visuals);
}

/// Tokyo Night をデフォルトスタイルとして初期化します。
pub fn setup_visuals(ctx: &egui::Context) {
    apply_theme(ctx, &Theme::builtin_tokyo_night());
}
