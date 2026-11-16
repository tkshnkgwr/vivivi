//! エディタカーソル描画（ブロック / Iビーム / 下線 / 角丸カスタマイズ）モジュール。

use eframe::egui;
use egui::{Align2, CornerRadius, FontId, Pos2, Rect, Vec2};
use unicode_width::UnicodeWidthChar;
use vivivi_app::CursorShape;
use vivivi_core::CursorPosition;
use vivivi_vi::ViMode;

use super::super::ViviviGuiApp;
use super::helpers::draw_keisen_glyph;

impl ViviviGuiApp {
    /// 現在設定と操作モードに応じたカーソル矩形を描画し、スクリーン矩形を返します。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_editor_cursor(
        &self,
        painter: &egui::Painter,
        pal: &crate::theme::GuiPalette,
        font_id: &FontId,
        chars: &[char],
        cur: CursorPosition,
        tab_width: usize,
        cursor_x: f32,
        cursor_row_y: f32,
        char_width: f32,
        line_height: f32,
    ) -> Rect {
        let char_under_cursor = chars.get(cur.col).copied();
        let cursor_cell_cols = char_under_cursor
            .and_then(|c| {
                if c == '\t' {
                    let base_vis_col = self.core.visual_col_at(cur.line, cur.col);
                    Some(tab_width - (base_vis_col % tab_width))
                } else {
                    UnicodeWidthChar::width(c)
                }
            })
            .unwrap_or(1)
            .max(1);
        let block_w = (cursor_cell_cols as f32) * char_width;

        let mode = self.core.mode();
        let cursor_color = match mode {
            ViMode::Insert => pal.accent_green,
            ViMode::Replace => pal.accent_red,
            _ if mode.is_visual() => pal.accent_purple,
            _ => pal.accent_blue,
        };
        let effective_shape = match self.core.config.appearance.gui.cursor_shape {
            CursorShape::Auto => match mode {
                ViMode::Insert => CursorShape::Beam,
                ViMode::Replace => CursorShape::Underline,
                _ => CursorShape::Block,
            },
            custom => custom,
        };
        let corner_radius = CornerRadius::same(
            self.core
                .config
                .appearance
                .gui
                .cursor_corner_radius
                .clamp(0.0, 8.0)
                .round() as u8,
        );

        match effective_shape {
            CursorShape::Beam => {
                let ibeam_rect = Rect::from_min_size(
                    Pos2::new(cursor_x, cursor_row_y),
                    Vec2::new(2.0, line_height),
                );
                painter.rect_filled(ibeam_rect, corner_radius, cursor_color);
                ibeam_rect
            }
            CursorShape::Underline => {
                let under_rect = Rect::from_min_size(
                    Pos2::new(cursor_x, cursor_row_y + line_height - 3.0),
                    Vec2::new(block_w, 3.0),
                );
                painter.rect_filled(under_rect, corner_radius, cursor_color);
                Rect::from_min_size(
                    Pos2::new(cursor_x, cursor_row_y),
                    Vec2::new(block_w, line_height),
                )
            }
            CursorShape::Block | CursorShape::Auto => {
                let block_rect = Rect::from_min_size(
                    Pos2::new(cursor_x, cursor_row_y),
                    Vec2::new(block_w, line_height),
                );
                painter.rect_filled(block_rect, corner_radius, cursor_color);

                // ブロックカーソル上の文字を反転色（背景色）でくっきり再描画
                if let Some(ch) = char_under_cursor
                    && ch != '\t'
                    && ch != ' '
                    && !draw_keisen_glyph(painter, block_rect, ch, pal.bg_editor)
                {
                    painter.text(
                        Pos2::new(cursor_x, cursor_row_y),
                        Align2::LEFT_TOP,
                        ch.encode_utf8(&mut [0u8; 4]),
                        font_id.clone(),
                        pal.bg_editor,
                    );
                }
                block_rect
            }
        }
    }
}
