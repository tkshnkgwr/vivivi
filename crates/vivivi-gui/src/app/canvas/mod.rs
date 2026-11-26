//! 中央エディタ分割ビュー・`ui.painter()` 等幅グリッド描画・IME 座標通知・ミニマップ描画モジュール。

mod cursor;
mod editor_pane;
mod helpers;
mod hex;
mod minimap;

use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Frame, Pos2, Rect, Stroke, Vec2};
use vivivi_app::LogicalRect;
use vivivi_vi::ViMode;

use super::ViviviGuiApp;

impl ViviviGuiApp {
    /// 4. 中央エディタ分割ビュー ＆ ミニマップ領域の描画（`ui.painter()` によるカスタム等幅レンダリング）
    pub fn render_central_panel(&mut self, ctx: &egui::Context) {
        let pal = self.palette();
        // 4.1 入力イベント (`Event::Key`, `Event::Text`, `Event::Ime`) の取得とディスパッチ
        let (events, current_modifiers, dropped_files, hovered_files) = ctx.input(|i| {
            (
                i.events.clone(),
                i.modifiers,
                i.raw.dropped_files.clone(),
                i.raw.hovered_files.clone(),
            )
        });
        self.process_egui_events_without_wheel(&events, current_modifiers);
        self.handle_drag_and_drop_files(&dropped_files);
        self.core.sync_treesitter_if_dirty();

        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(pal.bg_editor))
            .show(ctx, |ui| {
                let available_rect = ui.available_rect_before_wrap();

                // プロジェクト可視化グラフ (Visualize / ViVi 4) の描画
                if let Some(ref mut graph) = self.visualize_graph {
                    let file_to_open = graph.ui(ui, available_rect);
                    if let Some(path) = file_to_open {
                        if let Err(e) = self.core.open_file(path) {
                            self.core.status_message = Some(format!("ファイル読込エラー: {e}"));
                        }
                        self.visualize_graph = None;
                        self.core.show_visualize_graph = false;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.visualize_graph = None;
                        self.core.show_visualize_graph = false;
                    }
                    return;
                }

                // バイナリモード (Hex Editor) の描画
                if self.core.is_hex_mode {
                    self.render_hex_editor(ui, available_rect);
                    return;
                }
                let minimap_width = if self.minimap_open {
                    self.minimap_width
                } else {
                    0.0
                };
                let editor_width = (available_rect.width() - minimap_width).max(100.0);

                // エディタ描画前に最新のバッファ状態に基づきガター変更マーカー（Diffキャッシュ）を更新
                self.core.update_change_markers();

                let editor_rect = Rect::from_min_size(
                    available_rect.min,
                    Vec2::new(editor_width, available_rect.height()),
                );
                let minimap_rect_opt = if self.minimap_open {
                    Some(Rect::from_min_size(
                        Pos2::new(editor_rect.max.x, available_rect.min.y),
                        Vec2::new(minimap_width, available_rect.height()),
                    ))
                } else {
                    None
                };
                self.last_editor_rect = Some(editor_rect);
                self.last_minimap_rect = minimap_rect_opt;

                // 等幅フォントメトリクスの算出
                let font_id = FontId::monospace(self.font_size);
                let char_width = ui
                    .fonts(|f| f.glyph_width(&font_id, 'M'))
                    .max(self.font_size * 0.5);
                let line_height = self.line_height;
                let padding = 6.0;

                // 分割ウィンドウの論理矩形算出および App 側キャッシュの更新（Ctrl-W h/j/k/l 用）
                let logical_full_rect = LogicalRect::new(
                    0,
                    0,
                    editor_rect.width().max(1.0) as u16,
                    editor_rect.height().max(1.0) as u16,
                );
                let window_rects = self
                    .core
                    .window_tree
                    .compute_logical_rects(logical_full_rect);
                self.core.window_rects = window_rects.clone();

                // アクティブウィンドウの可視行数・列数に合わせてスクロール位置を自動調整
                if let Some((_, active_lrect)) = window_rects
                    .iter()
                    .find(|(idx, _)| *idx == self.core.active_window)
                {
                    let gutter_cols = if self.core.show_line_numbers {
                        6.5
                    } else {
                        2.0
                    };
                    let gutter_px = gutter_cols * char_width;
                    let view_height = ((active_lrect.height as f32 - padding * 2.0) / line_height)
                        .floor()
                        .max(1.0) as usize;
                    let view_width = ((active_lrect.width as f32 - gutter_px - padding * 2.0)
                        / char_width)
                        .floor()
                        .max(1.0) as usize;
                    self.core.adjust_scroll(view_height, view_width);
                }

                // マウスホイール処理（ポインタがエディタ領域またはミニマップ領域上にある場合のみ有効化）
                if !self.is_any_modal_open()
                    && self.core.lsp_hover_text.is_none()
                    && !self.core.lsp_show_diagnostics_panel
                    && (ui.rect_contains_pointer(editor_rect)
                        || minimap_rect_opt.is_some_and(|mr| ui.rect_contains_pointer(mr)))
                {
                    let pointer_pos =
                        ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
                    let (target_win_idx, target_view_height) = pointer_pos
                        .and_then(|pos| {
                            window_rects.iter().find_map(|(idx, lrect)| {
                                let wrect = Rect::from_min_size(
                                    Pos2::new(
                                        editor_rect.min.x + lrect.x as f32,
                                        editor_rect.min.y + lrect.y as f32,
                                    ),
                                    Vec2::new(lrect.width as f32, lrect.height as f32),
                                );
                                if wrect.contains(pos) {
                                    let vh = ((lrect.height as f32 - padding * 2.0) / line_height)
                                        .floor()
                                        .max(1.0)
                                        as usize;
                                    Some((*idx, vh))
                                } else {
                                    None
                                }
                            })
                        })
                        .unwrap_or_else(|| {
                            let active_h = window_rects
                                .iter()
                                .find(|(idx, _)| *idx == self.core.active_window)
                                .map(|(_, r)| r.height as f32)
                                .unwrap_or(editor_rect.height());
                            let vh = ((active_h - padding * 2.0) / line_height).floor().max(1.0)
                                as usize;
                            (self.core.active_window, vh)
                        });

                    for event in &events {
                        if let egui::Event::MouseWheel { delta, .. } = event {
                            self.apply_editor_mouse_wheel(
                                delta.y,
                                target_win_idx,
                                target_view_height,
                            );
                        }
                    }
                }

                // エディタ中央ペイン領域のマウスホバー連動「Iビーム（テキスト入力）」カーソル変更
                if !self.is_any_modal_open()
                    && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                    && editor_rect.contains(pos)
                {
                    ctx.set_cursor_icon(egui::CursorIcon::Text);
                }

                // マウスクリック・ドラッグ選択によるウィンドウフォーカス＆カーソル/範囲選択判定（モーダル中は無効化）
                let pointer_input = if self.is_any_modal_open() {
                    self.mouse_drag_window = None;
                    super::canvas::editor_pane::EditorPointerInput::default()
                } else {
                    let input = ui.input(|i| {
                        let primary_down = i.pointer.primary_down();
                        let pressed_pos = if i.pointer.primary_pressed() {
                            i.pointer.interact_pos()
                        } else {
                            None
                        };
                        let drag_pos = if primary_down {
                            i.pointer.interact_pos().or_else(|| i.pointer.latest_pos())
                        } else {
                            None
                        };
                        super::canvas::editor_pane::EditorPointerInput {
                            pressed_pos,
                            drag_pos,
                            shift_held: i.modifiers.shift,
                            primary_down,
                        }
                    });
                    if !input.primary_down {
                        self.mouse_drag_window = None;
                    }
                    input
                };

                let mut active_cursor_screen_rect: Option<Rect> = None;

                // 対応括弧ペアの計算
                let total_lines = self.core.buffer.line_count();
                let active_cur_offset = self.core.buffer.cursor_to_position(self.core.cursor_pos());
                let paren_positions =
                    vivivi_core::get_cursor_matching_paren(&self.core.buffer, active_cur_offset)
                        .map(|(p1, p2)| {
                            (
                                self.core.buffer.position_to_cursor(p1),
                                self.core.buffer.position_to_cursor(p2),
                            )
                        });

                // 各分割ウィンドウを Painter で描画
                let window_count = window_rects.len();
                for (win_idx, lrect) in &window_rects {
                    let win_min = Pos2::new(
                        editor_rect.min.x + lrect.x as f32,
                        editor_rect.min.y + lrect.y as f32,
                    );
                    let win_rect = Rect::from_min_size(
                        win_min,
                        Vec2::new(lrect.width as f32, lrect.height as f32),
                    );
                    if let Some(rect) = self.render_window_pane(
                        ui,
                        *win_idx,
                        win_rect,
                        window_count,
                        pointer_input,
                        &font_id,
                        char_width,
                        line_height,
                        padding,
                        total_lines,
                        paren_positions,
                        &pal,
                    ) {
                        active_cursor_screen_rect = Some(rect);
                    }
                }

                // 4.4 削除文字落下パーティクルの物理更新 ＆ オーバーレイ描画
                if self.particles.is_active() {
                    let dt = ctx.input(|i| i.stable_dt);
                    self.particles.update(dt);
                    let overlay_painter = ui.painter().with_clip_rect(editor_rect);
                    self.particles.render(&overlay_painter, &font_id);
                    ctx.request_repaint();
                }

                // 4.5 OS の IME へカーソル矩形座標を毎フレーム通知
                self.last_cursor_rect = active_cursor_screen_rect;
                let mode = self.core.mode();
                let should_enable_ime =
                    matches!(mode, ViMode::Insert | ViMode::Replace | ViMode::CommandLine)
                        || self.core.is_migemo_search;

                if should_enable_ime {
                    let fallback_rect = Rect::from_min_size(
                        self.last_active_text_origin,
                        Vec2::new(2.0, line_height),
                    );
                    let cursor_rect = active_cursor_screen_rect.unwrap_or(fallback_rect);
                    ctx.output_mut(|o| {
                        o.ime = Some(egui::output::IMEOutput {
                            rect: cursor_rect,
                            cursor_rect,
                        });
                    });
                } else {
                    ctx.output_mut(|o| {
                        o.ime = None;
                    });
                }

                // --- 右端ミニマップ領域 ---
                if let Some(minimap_rect) = minimap_rect_opt {
                    self.render_minimap(
                        ui,
                        minimap_rect,
                        editor_rect.height(),
                        &window_rects,
                        padding,
                        line_height,
                        minimap_width,
                        &pal,
                    );
                }

                // 4.6 ファイルホバー中のオーバーレイガイダンス
                if !hovered_files.is_empty() {
                    let overlay_painter = ui.painter_at(available_rect);
                    let ov_c = pal.bg_editor;
                    overlay_painter.rect_filled(
                        available_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(ov_c.r(), ov_c.g(), ov_c.b(), 200),
                    );
                    overlay_painter.rect_stroke(
                        available_rect.shrink(8.0),
                        CornerRadius::same(8),
                        Stroke::new(2.0_f32, pal.accent_blue),
                        egui::StrokeKind::Inside,
                    );
                    overlay_painter.text(
                        available_rect.center(),
                        Align2::CENTER_CENTER,
                        "ここにファイルをドロップして開く",
                        FontId::proportional(20.0),
                        pal.fg_text,
                    );
                }
            });
    }
}
