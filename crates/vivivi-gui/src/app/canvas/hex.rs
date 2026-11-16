//! GUI バイナリモード (Hex Editor) レンダリングモジュール。

use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Stroke};

use super::super::ViviviGuiApp;

impl ViviviGuiApp {
    /// GUI バイナリモード (Hex Editor) のレンダリング
    pub fn render_hex_editor(&mut self, ui: &mut egui::Ui, area: Rect) {
        let pal = self.palette();
        let hex_doc = match &self.core.hex_doc {
            Some(doc) => doc,
            None => return,
        };

        let row_height = 20.0_f32;
        let font_id = FontId::monospace(14.0);
        let total_rows = vivivi_modes::hex::calculate_row_count(hex_doc.len());

        let cursor_offset = self.core.hex_cursor.offset;
        let cursor_row = cursor_offset / vivivi_modes::hex::BYTES_PER_ROW;
        let cursor_col = cursor_offset % vivivi_modes::hex::BYTES_PER_ROW;

        // キーボード操作
        let ctx = ui.ctx();
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.core.toggle_hex_mode();
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.core.hex_cursor.toggle_pane();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.core.hex_cursor.offset = self.core.hex_cursor.offset.saturating_sub(1);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight))
            && self.core.hex_cursor.offset + 1 < hex_doc.len()
        {
            self.core.hex_cursor.offset += 1;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.core.hex_cursor.offset = self.core.hex_cursor.offset.saturating_sub(16);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if self.core.hex_cursor.offset + 16 < hex_doc.len() {
                self.core.hex_cursor.offset += 16;
            } else {
                self.core.hex_cursor.offset = hex_doc.len().saturating_sub(1);
            }
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                let painter = ui.painter();
                let start_y = ui.cursor().top();

                for r in row_range {
                    let y = start_y + (r as f32) * row_height;
                    let row_data = match vivivi_modes::hex::format_hex_row(hex_doc.data(), r) {
                        Some(row) => row,
                        None => continue,
                    };

                    let is_cursor_row = r == cursor_row;

                    // 1. アドレス列
                    let addr_text = format!("{:08X}:", row_data.offset);
                    painter.text(
                        Pos2::new(area.left() + 10.0, y),
                        Align2::LEFT_TOP,
                        addr_text,
                        font_id.clone(),
                        pal.accent_blue,
                    );

                    // 2. 16進ダンプ列
                    let hex_x = area.left() + 110.0;
                    painter.text(
                        Pos2::new(hex_x, y),
                        Align2::LEFT_TOP,
                        &row_data.hex_dump,
                        font_id.clone(),
                        pal.fg_text,
                    );

                    // 3. ASCII列
                    let ascii_x = hex_x + 390.0;
                    painter.text(
                        Pos2::new(ascii_x, y),
                        Align2::LEFT_TOP,
                        &row_data.ascii_dump,
                        font_id.clone(),
                        pal.accent_green,
                    );

                    // カーソルハイライト
                    if is_cursor_row && cursor_col < row_data.byte_count {
                        let cursor_rect = Rect::from_min_size(
                            Pos2::new(
                                hex_x
                                    + (cursor_col as f32) * 23.0
                                    + if cursor_col >= 8 { 8.0 } else { 0.0 },
                                y,
                            ),
                            egui::Vec2::new(18.0, row_height),
                        );
                        painter.rect_stroke(
                            cursor_rect,
                            CornerRadius::ZERO,
                            Stroke::new(1.5_f32, Color32::YELLOW),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            });
    }
}
