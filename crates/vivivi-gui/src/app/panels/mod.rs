//! メニューバー・マルチタブバー・左サイドバー・下部ステータスバー＆コマンドライン描画モジュール。

mod bottom_bar;
mod sidebar;
mod top_bar;

use eframe::egui;
use egui::Vec2;

/// Zed 風のメニューポップアップ共通スタイル（最小幅・余白・セパレータ境界線）を適用します。
pub(super) fn apply_menu_popup_style(ui: &mut egui::Ui, min_width: f32) {
    ui.set_min_width(min_width);
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
    ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);
    ui.visuals_mut().popup_shadow = egui::epaint::Shadow::NONE;
    let border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
    ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, border_color);
}

/// Zed 風の左寄せラベル＋右寄せショートカット付きメニュー項目を描画します。
pub(super) fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: &str) -> bool {
    let shortcut_color = ui.visuals().weak_text_color();
    let mut btn = egui::Button::new(label);
    if !shortcut.is_empty() {
        btn = btn.shortcut_text(egui::RichText::new(shortcut).color(shortcut_color));
    }
    ui.add(btn).clicked()
}

/// Zed 風のチェック状態付き左寄せラベル＋右寄せショートカット付きメニュー項目を描画します。
pub(super) fn menu_toggle_item(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: &str,
    shortcut: &str,
) -> bool {
    let shortcut_color = ui.visuals().weak_text_color();
    let prefix = if *checked { "✓ " } else { "  " };
    let mut btn = egui::Button::new(format!("{prefix}{label}")).selected(*checked);
    if !shortcut.is_empty() {
        btn = btn.shortcut_text(egui::RichText::new(shortcut).color(shortcut_color));
    }
    if ui.add(btn).clicked() {
        *checked = !*checked;
        true
    } else {
        false
    }
}
