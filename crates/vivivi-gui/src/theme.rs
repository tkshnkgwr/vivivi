//! Tokyo Night テーマ設定モジュール（ステップ 1）。
//!
//! エディタ背景（`#1a1b26`）、パネル背景（`#16161e`）、ステータスライン（`#1f2335`）で
//! 3段階の明淡コントラストを適用し、余計な枠線（Stroke）を排除した角丸 4px のモダンなダークテーマを構成します。

use egui::{Color32, CornerRadius, Stroke, Visuals};

/// Tokyo Night パレット定数群。
///
/// エディタ本文、サイドバー、ステータスバー、および各 Vi モードバッジの配色を一元管理します。
#[allow(dead_code)]
pub mod colors {
    use egui::Color32;

    /// エディタ中央・本文背景 (`#1a1b26`)
    pub const BG_EDITOR: Color32 = Color32::from_rgb(26, 27, 38);
    /// サイドバー・メニューバー・タブバー背景 (`#16161e`)
    pub const BG_PANEL: Color32 = Color32::from_rgb(22, 22, 30);
    /// ステータスライン・非アクティブ枠背景 (`#1f2335`)
    pub const BG_STATUSLINE: Color32 = Color32::from_rgb(31, 35, 53);
    /// ビジュアル選択範囲のハイライト背景 (`#283457`)
    pub const BG_SELECTION: Color32 = Color32::from_rgb(40, 52, 87);
    /// ウィジェットホバー時背景 (`#292e42`)
    pub const BG_HOVER: Color32 = Color32::from_rgb(41, 46, 66);
    /// ウィジェットアクティブ・クリック時背景 (`#414868`)
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(65, 72, 104);

    /// 標準テキスト前景色 (`#c0caf5`)
    pub const FG_TEXT: Color32 = Color32::from_rgb(192, 202, 245);
    /// コメント・非アクティブ要素前景色 (`#565f89`)
    pub const FG_DIMMED: Color32 = Color32::from_rgb(86, 95, 137);
    /// 行番号・ミニマップライン前景色 (`#3b4261`)
    pub const FG_LINENUMBER: Color32 = Color32::from_rgb(59, 66, 97);

    /// NORMAL モード・情報アクセント (`#7aa2f7`)
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(122, 162, 247);
    /// INSERT モード・文字列リテラルアクセント (`#9ece6a`)
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(158, 206, 106);
    /// VISUAL モード・予約語キーワードアクセント (`#bb9af7`)
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(187, 154, 247);
    /// COMMAND モード・数値・警告アクセント (`#e0af68`)
    pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(224, 175, 104);
    /// REPLACE モード・削除パーティクル・エラーアクセント (`#f7768e`)
    pub const ACCENT_RED: Color32 = Color32::from_rgb(247, 118, 142);
    /// 型名・IME 変換下線・罫線バッジアクセント (`#7dcfff`)
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(125, 207, 255);
}

/// Tokyo Night を基調としたエディタのビジュアルスタイル（[`egui::Visuals`]）を構築して [`egui::Context`] に適用します。
///
/// 完全な黒（`#000000`）を避けて眼精疲労を軽減する3段階の背景明淡を設定し、
/// ウィジェットの枠線（`Stroke`）を排除したフラットかつ角丸 `4px` の外観へ更新します。
///
/// # Examples
/// ```
/// use vivivi_gui::setup_visuals;
///
/// let ctx = egui::Context::default();
/// setup_visuals(&ctx);
/// assert_eq!(ctx.style().visuals.panel_fill, vivivi_gui::colors::BG_PANEL);
/// ```
pub fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // 1. 3段階の明淡コントラスト背景
    // - パネル背景（サイドバー、メニューバー、タブバー）: 最も暗い #16161e
    visuals.panel_fill = colors::BG_PANEL;
    // - エディタ背景・ウィンドウ背景: 標準背景 #1a1b26
    visuals.window_fill = colors::BG_EDITOR;
    // - 極低明度背景: #101016
    visuals.extreme_bg_color = Color32::from_rgb(16, 16, 22);

    // 2. 余計な枠線（Stroke）を排除
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    visuals.window_stroke = Stroke::NONE;

    // 3. 角丸は 4px で統一
    let corner_4 = CornerRadius::same(4);
    visuals.widgets.noninteractive.corner_radius = corner_4;
    visuals.widgets.inactive.corner_radius = corner_4;
    visuals.widgets.hovered.corner_radius = corner_4;
    visuals.widgets.active.corner_radius = corner_4;
    visuals.widgets.open.corner_radius = corner_4;
    visuals.window_corner_radius = corner_4;
    visuals.menu_corner_radius = corner_4;

    // 4. ウィジェット背景色
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_fill = colors::BG_STATUSLINE;
    visuals.widgets.hovered.bg_fill = colors::BG_HOVER;
    visuals.widgets.active.bg_fill = colors::BG_ACTIVE;

    // 5. テキスト色および選択領域
    visuals.override_text_color = Some(colors::FG_TEXT);
    visuals.selection.bg_fill = colors::BG_SELECTION;
    visuals.selection.stroke = Stroke::NONE;

    ctx.set_visuals(visuals);
}
