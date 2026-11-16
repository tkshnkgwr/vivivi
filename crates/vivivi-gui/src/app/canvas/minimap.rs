//! 右端ミニマップ（俯瞰描画・ビューポート同期・クリック/ドラッグ操作）モジュール。

use eframe::egui;
use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};
use vivivi_app::LogicalRect;

use super::super::ViviviGuiApp;
use super::helpers::char_display_width;

impl ViviviGuiApp {
    /// 右端ミニマップ領域の描画とクリック／ドラッグ操作を処理します。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_minimap(
        &mut self,
        ui: &mut egui::Ui,
        minimap_rect: Rect,
        editor_height: f32,
        window_rects: &[(usize, LogicalRect)],
        padding: f32,
        line_height: f32,
        minimap_width: f32,
        pal: &crate::theme::GuiPalette,
    ) {
        let active_win_h = window_rects
            .iter()
            .find(|(idx, _)| *idx == self.core.active_window)
            .map(|(_, r)| r.height as f32)
            .unwrap_or(editor_height);
        let active_visible_rows = ((active_win_h - padding * 2.0) / line_height)
            .floor()
            .max(1.0) as usize;

        let total_lines_mm = self.core.buffer.line_count().max(1);
        let max_scroll = total_lines_mm.saturating_sub(active_visible_rows);
        let mm_h = minimap_rect.height();
        let content_top = minimap_rect.top() + 2.0;
        let usable_mm_h = (mm_h - 4.0).max(3.0);
        let max_mm_rows = (usable_mm_h / 3.0).floor().max(1.0) as usize;

        // 短いファイルでは実描画高さ (行数×3px)、長いファイルではミニマップ全高を基準にする
        let content_h = if total_lines_mm <= max_mm_rows {
            (total_lines_mm as f32 * 3.0).min(usable_mm_h)
        } else {
            usable_mm_h
        };

        // ビューポートハイライト矩形の高さ（アクティブウィンドウの可視行数と総行数の比率）
        let vp_height = if total_lines_mm <= active_visible_rows {
            content_h
        } else {
            ((active_visible_rows as f32 / total_lines_mm as f32) * content_h)
                .clamp(16.0_f32.min(content_h), content_h)
        };
        let scrollable_h = (content_h - vp_height).max(0.0);

        // egui の allocate_rect でクリック＆ドラッグを捕捉（ポインタが左右にはみ出しても追従継続）
        let mm_response = ui
            .allocate_rect(minimap_rect, egui::Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        if !self.is_any_modal_open()
            && (mm_response.clicked() || mm_response.dragged())
            && max_scroll > 0
            && scrollable_h > 0.0
            && let Some(pos) = mm_response.interact_pointer_pos()
        {
            // クリック・ドラッグ位置がビューポート矩形の中心になるよう吸着
            let target_vp_top =
                (pos.y - vp_height / 2.0).clamp(content_top, content_top + scrollable_h);
            let ratio = (target_vp_top - content_top) / scrollable_h;
            let new_scroll = (ratio * max_scroll as f32).round() as usize;
            self.core.scroll.0 = new_scroll.min(max_scroll);

            // カーソル位置もアクティブウィンドウ可視範囲の中央へ同期（次フレームの adjust_scroll と整合）
            let center_line = (self.core.scroll.0 + active_visible_rows / 2)
                .min(total_lines_mm.saturating_sub(1));
            self.core.cursor.pos.line = center_line;
            self.core.clamp_cursor();
            if let Some(win) = self.core.windows.get_mut(self.core.active_window) {
                win.cursor = self.core.cursor.clone();
                win.scroll = self.core.scroll;
            }
        }

        let mm_painter = ui.painter().with_clip_rect(minimap_rect);
        mm_painter.rect_filled(minimap_rect, 0.0, pal.bg_panel);
        mm_painter.line_segment(
            [minimap_rect.left_top(), minimap_rect.left_bottom()],
            Stroke::new(1.0_f32, pal.bg_statusline),
        );

        // ミニライン構造描画（0行目から最終行まで均等サンプリング・タブおよび全角文字幅対応）
        let expected_samples = total_lines_mm.min(max_mm_rows);
        let buffer_rev = self.core.buffer.revision();
        let current_key = (self.core.buffer.len(), total_lines_mm);
        let tab_w = self.core.tab_width.max(1);
        if self.minimap_cache_revision != buffer_rev
            || self.minimap_cache_key != current_key
            || self.minimap_cache.len() != expected_samples
        {
            self.minimap_cache.clear();
            for i in 0..expected_samples {
                let l_idx = if total_lines_mm <= max_mm_rows {
                    i
                } else if expected_samples <= 1 {
                    0
                } else {
                    (i as f32 * (total_lines_mm - 1) as f32 / (expected_samples - 1) as f32).round()
                        as usize
                };

                if let Some(line_str) = self.core.buffer.get_line(l_idx) {
                    let mut indent_cols = 0usize;
                    let mut total_cols = 0usize;
                    let mut seen_non_ws = false;
                    for ch in line_str.chars() {
                        let w = char_display_width(ch, total_cols, tab_w);
                        if !seen_non_ws && (ch == ' ' || ch == '\t' || ch == '　') {
                            indent_cols += w;
                        } else {
                            seen_non_ws = true;
                        }
                        total_cols += w;
                    }
                    let content_cols = total_cols.saturating_sub(indent_cols);
                    if content_cols > 0 {
                        let indent = (indent_cols.min(24) as f32) * 1.1;
                        let len = (content_cols.min(48) as f32) * 1.05;
                        self.minimap_cache.push((indent, len));
                    } else {
                        self.minimap_cache.push((0.0, 0.0));
                    }
                } else {
                    self.minimap_cache.push((0.0, 0.0));
                }
            }
            self.minimap_cache_revision = buffer_rev;
            self.minimap_cache_key = current_key;
        }

        let cache_len = self.minimap_cache.len().max(1);
        for (i, &(indent, len)) in self.minimap_cache.iter().enumerate() {
            if len > 0.0 {
                let y = if total_lines_mm <= max_mm_rows {
                    content_top + (i as f32) * 3.0 + 1.0
                } else if cache_len <= 1 {
                    content_top + 1.0
                } else {
                    content_top + (i as f32 / (cache_len - 1) as f32) * (content_h - 2.0) + 1.0
                };
                let x1 = minimap_rect.left() + 6.0 + indent;
                let x2 = (x1 + len).min(minimap_rect.right() - 4.0);
                if x2 > x1 {
                    mm_painter.line_segment(
                        [Pos2::new(x1, y), Pos2::new(x2, y)],
                        Stroke::new(1.5_f32, pal.fg_linenumber),
                    );
                }
            }
        }

        // 複数分割時は非アクティブウィンドウのビューポート位置も控えめに補助表示
        if window_rects.len() > 1 {
            let inactive_c = pal.fg_dimmed;
            for (win_idx, lrect) in window_rects {
                if *win_idx == self.core.active_window {
                    continue;
                }
                if let Some(win) = self.core.windows.get(*win_idx) {
                    let win_vis_rows = ((lrect.height as f32 - padding * 2.0) / line_height)
                        .floor()
                        .max(1.0) as usize;
                    let win_max_scroll = total_lines_mm.saturating_sub(win_vis_rows);
                    let win_vp_h = if total_lines_mm <= win_vis_rows {
                        content_h
                    } else {
                        ((win_vis_rows as f32 / total_lines_mm as f32) * content_h)
                            .clamp(16.0_f32.min(content_h), content_h)
                    };
                    let win_scrollable_h = (content_h - win_vp_h).max(0.0);
                    let win_scroll = win.scroll.0.min(win_max_scroll);
                    let win_vp_top = if win_max_scroll == 0 || win_scrollable_h <= 0.0 {
                        content_top
                    } else {
                        content_top + (win_scroll as f32 / win_max_scroll as f32) * win_scrollable_h
                    };
                    let inactive_rect = Rect::from_min_size(
                        Pos2::new(minimap_rect.left() + 2.0, win_vp_top),
                        Vec2::new(minimap_width - 4.0, win_vp_h),
                    );
                    mm_painter.rect_filled(
                        inactive_rect,
                        CornerRadius::same(2),
                        Color32::from_rgba_unmultiplied(
                            inactive_c.r(),
                            inactive_c.g(),
                            inactive_c.b(),
                            18,
                        ),
                    );
                }
            }
        }

        // アクティブウィンドウのビューポートハイライト矩形（現在スクロール位置と100%線形同期）
        let current_scroll = self.core.scroll.0.min(max_scroll);
        let vp_top = if max_scroll == 0 || scrollable_h <= 0.0 {
            content_top
        } else {
            content_top + (current_scroll as f32 / max_scroll as f32) * scrollable_h
        };

        let vp_rect = Rect::from_min_size(
            Pos2::new(minimap_rect.left() + 2.0, vp_top),
            Vec2::new(minimap_width - 4.0, vp_height),
        );

        let vp_c = pal.accent_blue;
        mm_painter.rect_filled(
            vp_rect,
            CornerRadius::same(2),
            Color32::from_rgba_unmultiplied(vp_c.r(), vp_c.g(), vp_c.b(), 40),
        );
        mm_painter.rect_stroke(
            vp_rect,
            CornerRadius::same(2),
            Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(vp_c.r(), vp_c.g(), vp_c.b(), 100),
            ),
            egui::StrokeKind::Inside,
        );
    }
}
