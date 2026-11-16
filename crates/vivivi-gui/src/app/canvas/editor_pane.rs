//! 分割エディタウィンドウペインのクリック判定・行/テキストグリッド描画モジュール。

use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};
use unicode_width::UnicodeWidthChar;
use vivivi_core::CursorPosition;
use vivivi_modes::{LineChangeState, TokenType};
use vivivi_vi::ViMode;

use super::super::ViviviGuiApp;
use super::helpers::{
    advance_wrap_cursor, char_display_width, compute_line_syntax_key, count_line_visual_rows,
    draw_keisen_glyph, resolve_wrapped_char_col,
};

impl ViviviGuiApp {
    /// ウィンドウクリック時のフォーカス切替およびクリック座標からのカーソル位置解決を行います。
    #[allow(clippy::too_many_arguments)]
    fn handle_window_click(
        &mut self,
        win_idx: usize,
        click_pos: Pos2,
        text_start_x: f32,
        text_start_y: f32,
        char_width: f32,
        line_height: f32,
        wrap_cols: Option<usize>,
    ) {
        self.core.switch_to_window(win_idx);
        let clicked_vis_row = ((click_pos.y - text_start_y) / line_height)
            .floor()
            .max(0.0) as usize;
        let clicked_vis_col = ((click_pos.x - text_start_x) / char_width).floor().max(0.0) as usize;
        let tab_w = self.core.tab_width.max(1);

        let (target_line, resolved_col) = if let Some(max_cols) = wrap_cols {
            let mut cur_line = self.core.scroll.0;
            let mut rem_rows = clicked_vis_row;
            let max_line_idx = self.core.buffer.line_count().saturating_sub(1);
            let mut target_sub_row = 0usize;
            while cur_line < max_line_idx {
                if self.core.vi.fold_manager.is_line_hidden(cur_line + 1) {
                    cur_line += 1;
                    continue;
                }
                let line_chars: Vec<char> = self
                    .core
                    .buffer
                    .get_line(cur_line)
                    .unwrap_or_default()
                    .chars()
                    .collect();
                let row_span = count_line_visual_rows(&line_chars, tab_w, Some(max_cols));
                if rem_rows < row_span {
                    target_sub_row = rem_rows;
                    break;
                }
                rem_rows -= row_span;
                cur_line += 1;
            }
            if cur_line >= max_line_idx {
                cur_line = max_line_idx;
                target_sub_row = rem_rows;
            }
            let resolved = self
                .core
                .buffer
                .get_line(cur_line)
                .map(|s| {
                    resolve_wrapped_char_col(&s, tab_w, max_cols, target_sub_row, clicked_vis_col)
                })
                .unwrap_or(0);
            (cur_line, resolved)
        } else {
            let target_line = (self.core.scroll.0 + clicked_vis_row)
                .min(self.core.buffer.line_count().saturating_sub(1));

            let scroll_col = self.core.scroll.1;
            let target_vis_col = scroll_col + clicked_vis_col;
            let resolved_col = if let Some(line_str) = self.core.buffer.get_line(target_line) {
                let mut cur_vcol = 0;
                let mut found_idx = 0;
                for (c_idx, ch) in line_str.chars().enumerate() {
                    let w = char_display_width(ch, cur_vcol, tab_w);
                    if cur_vcol + w > target_vis_col {
                        found_idx = c_idx;
                        break;
                    }
                    cur_vcol += w;
                    found_idx = c_idx + 1;
                }
                found_idx
            } else {
                0
            };
            (target_line, resolved_col)
        };

        self.core.cursor.pos.line = target_line;
        self.core.cursor.pos.col = resolved_col;
        self.core.clamp_cursor();
    }

    /// 1つのエディタウィンドウ領域（ガター・行番号・本文・ハイライト・IME・カーソル・枠線）を描画します。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_window_pane(
        &mut self,
        ui: &mut egui::Ui,
        win_idx: usize,
        win_rect: Rect,
        window_count: usize,
        pointer_click_pos: Option<Pos2>,
        font_id: &FontId,
        char_width: f32,
        line_height: f32,
        padding: f32,
        total_lines: usize,
        paren_positions: Option<(CursorPosition, CursorPosition)>,
        pal: &crate::theme::GuiPalette,
    ) -> Option<Rect> {
        let gutter_cols = if self.core.show_line_numbers {
            6.5
        } else {
            2.0
        };
        let gutter_px = gutter_cols * char_width;
        let text_start_x = win_rect.min.x + padding + gutter_px;
        let text_start_y = win_rect.min.y + padding;
        let wrap_cols = if self.core.wrap {
            let text_area_w = (win_rect.right() - text_start_x - padding).max(char_width * 2.0);
            Some((text_area_w / char_width).floor().max(2.0) as usize)
        } else {
            None
        };

        if let Some(click_pos) = pointer_click_pos
            && win_rect.contains(click_pos)
        {
            self.handle_window_click(
                win_idx,
                click_pos,
                text_start_x,
                text_start_y,
                char_width,
                line_height,
                wrap_cols,
            );
        }

        let is_active_window = win_idx == self.core.active_window;
        if is_active_window {
            self.last_active_text_origin = Pos2::new(text_start_x, text_start_y);
            self.last_char_width = char_width;
        }
        let (cur, scroll) = if is_active_window {
            (self.core.cursor_pos(), self.core.scroll)
        } else if let Some(win) = self.core.windows.get(win_idx) {
            (win.cursor.pos, win.scroll)
        } else {
            (self.core.cursor_pos(), self.core.scroll)
        };

        let painter = ui.painter().with_clip_rect(win_rect);

        // ウィンドウ背景
        painter.rect_filled(win_rect, 0.0, pal.bg_editor);

        // ガター（行番号 ＆ 変更マーカー領域）の背景と境界線
        let gutter_rect = Rect::from_min_size(
            win_rect.min,
            Vec2::new(padding + gutter_px - 4.0, win_rect.height()),
        );
        painter.rect_filled(gutter_rect, 0.0, pal.bg_gutter);
        painter.line_segment(
            [gutter_rect.right_top(), gutter_rect.right_bottom()],
            Stroke::new(1.0_f32, pal.bg_statusline),
        );

        let visible_lines = ((win_rect.height() - padding * 2.0) / line_height)
            .floor()
            .max(1.0) as usize;
        let tab_width = self.core.tab_width.max(1);
        let active_search_re = self.core.active_search_regex();

        let mut chars_buf: Vec<char> = Vec::with_capacity(256);
        let mut char_colors_buf: Vec<Color32> = Vec::with_capacity(256);
        let mut search_matched_buf: Vec<bool> = Vec::with_capacity(256);
        let mut visual_selected_buf: Vec<bool> = Vec::with_capacity(256);

        let mut active_cursor_screen_rect: Option<Rect> = None;
        let mut line_idx = scroll.0;
        let mut vis_row = 0;

        while vis_row < visible_lines {
            let row_y = text_start_y + (vis_row as f32) * line_height;

            if line_idx >= total_lines {
                painter.text(
                    Pos2::new(win_rect.min.x + padding, row_y),
                    Align2::LEFT_TOP,
                    "~",
                    font_id.clone(),
                    pal.fg_dimmed,
                );
                vis_row += 1;
                line_idx += 1;
                continue;
            }

            if self.core.vi.fold_manager.is_line_hidden(line_idx + 1) {
                line_idx += 1;
                continue;
            }

            let is_current_line = line_idx == cur.line;

            // 行番号の描画
            if self.core.show_line_numbers {
                let line_num_val = if self.core.relative_number && !is_current_line {
                    (line_idx as isize - cur.line as isize).unsigned_abs()
                } else {
                    line_idx + 1
                };
                let line_num_str = format!("{:>4}", line_num_val);
                let num_color = if is_current_line && is_active_window {
                    pal.accent_yellow
                } else {
                    pal.fg_linenumber
                };
                painter.text(
                    Pos2::new(win_rect.min.x + padding, row_y),
                    Align2::LEFT_TOP,
                    line_num_str,
                    font_id.clone(),
                    num_color,
                );
            }

            let raw_line = self.core.buffer.get_line(line_idx).unwrap_or_default();

            // 変更追跡ガターマーカー
            let marker = self.core.change_tracker.marker_at(line_idx, &raw_line);
            let marker_color = match marker {
                LineChangeState::Added | LineChangeState::Modified => Some(pal.gutter_modified),
                LineChangeState::SavedModified => Some(pal.gutter_saved),
                LineChangeState::Unchanged => None,
            };
            if let Some(mc) = marker_color {
                let marker_rect = Rect::from_min_size(
                    Pos2::new(gutter_rect.right() - 3.0, row_y + 2.0),
                    Vec2::new(2.5, line_height - 4.0),
                );
                painter.rect_filled(marker_rect, 1.0, mc);
            }
            let display_line = if self.core.is_csv_active {
                self.core.csv_mode.format_line(&raw_line, is_current_line)
            } else {
                raw_line
            };

            let fold_placeholder = self
                .core
                .vi
                .fold_manager
                .get_fold_placeholder(line_idx + 1, &display_line);
            let is_fold_start = fold_placeholder.is_some();

            let (processed_line, inv_info) = if let Some(ph) = fold_placeholder {
                (ph, None)
            } else if self.core.list_chars_config.enabled && !self.core.is_csv_active {
                let inv_chars = vivivi_modes::format_line_invisibles(
                    &display_line,
                    tab_width,
                    &self.core.list_chars_config,
                );
                let s: String = inv_chars.iter().map(|c| c.ch).collect();
                (s, Some(inv_chars))
            } else {
                (display_line.clone(), None)
            };

            chars_buf.clear();
            chars_buf.extend(processed_line.chars());
            let chars = &chars_buf;
            let line_sub_rows = count_line_visual_rows(chars, tab_width, wrap_cols);

            if is_current_line && is_active_window {
                let bg_rows = line_sub_rows.min(visible_lines.saturating_sub(vis_row));
                if bg_rows > 0 {
                    let cur_line_rect = Rect::from_min_size(
                        Pos2::new(gutter_rect.right(), row_y),
                        Vec2::new(
                            win_rect.right() - gutter_rect.right(),
                            (bg_rows as f32) * line_height,
                        ),
                    );
                    painter.rect_filled(cur_line_rect, 0.0, pal.bg_cur_line);
                }
            }
            let tokens = if is_fold_start {
                vec![vivivi_modes::SyntaxToken {
                    text: processed_line.clone(),
                    token_type: TokenType::Type,
                }]
            } else if self.core.is_csv_active {
                vec![vivivi_modes::SyntaxToken {
                    text: display_line.clone(),
                    token_type: TokenType::Text,
                }]
            } else if let Some(ref ts) = self.core.treesitter {
                let ast_tokens = ts.highlight_syntax_tokens(line_idx, &display_line);
                if !ast_tokens.is_empty() {
                    ast_tokens
                } else {
                    vivivi_modes::tokenize_line(&display_line, self.core.syntax_lang)
                }
            } else {
                let key = compute_line_syntax_key(&display_line, self.core.syntax_lang);
                if let Some(cached) = self.syntax_cache.get(&key) {
                    cached.clone()
                } else {
                    let t = vivivi_modes::tokenize_line(&display_line, self.core.syntax_lang);
                    if self.syntax_cache.len() > 1024 {
                        self.syntax_cache.clear();
                    }
                    self.syntax_cache.insert(key, t.clone());
                    t
                }
            };

            char_colors_buf.clear();
            let char_colors = &mut char_colors_buf;
            for token in tokens {
                let c = self.syntax_token_color(token.token_type);
                for _ in token.text.chars() {
                    char_colors.push(c);
                }
            }
            char_colors.resize(chars.len(), pal.fg_text);

            if let Some(ref inv) = inv_info {
                for (idx, item) in inv.iter().enumerate() {
                    if idx < char_colors.len() && item.is_invisible {
                        char_colors[idx] = pal.fg_dimmed;
                    }
                }
            }

            // 検索一致箇所のハイライト判定
            search_matched_buf.clear();
            search_matched_buf.resize(chars.len(), false);
            let search_matched = &mut search_matched_buf;
            if let Some(ref re) = active_search_re {
                for m in re.find_iter(&display_line) {
                    let start_c = display_line[..m.start()].chars().count();
                    let end_c = display_line[..m.end()].chars().count();
                    for flag in search_matched
                        .iter_mut()
                        .take(end_c.min(chars.len()))
                        .skip(start_c)
                    {
                        *flag = true;
                    }
                }
            }

            // ビジュアル選択範囲のハイライト判定
            visual_selected_buf.clear();
            visual_selected_buf.resize(chars.len().max(1), false);
            let visual_selected = &mut visual_selected_buf;
            if is_active_window && let Some(anchor) = self.core.cursor.anchor {
                let min_l = anchor.line.min(cur.line);
                let max_l = anchor.line.max(cur.line);
                if line_idx >= min_l && line_idx <= max_l {
                    match self.core.mode() {
                        ViMode::VisualLine => {
                            visual_selected.fill(true);
                        }
                        ViMode::VisualBlock => {
                            let min_c = anchor.col.min(cur.col);
                            let max_c = anchor.col.max(cur.col);
                            for (ci, flag) in visual_selected.iter_mut().enumerate() {
                                if ci >= min_c && ci <= max_c {
                                    *flag = true;
                                }
                            }
                        }
                        ViMode::Visual => {
                            for (ci, flag) in visual_selected.iter_mut().enumerate() {
                                let in_sel = if min_l == max_l {
                                    let min_c = anchor.col.min(cur.col);
                                    let max_c = anchor.col.max(cur.col);
                                    ci >= min_c && ci <= max_c
                                } else if line_idx == min_l {
                                    let sc = if anchor.line <= cur.line {
                                        anchor.col
                                    } else {
                                        cur.col
                                    };
                                    ci >= sc
                                } else if line_idx == max_l {
                                    let ec = if anchor.line >= cur.line {
                                        anchor.col
                                    } else {
                                        cur.col
                                    };
                                    ci <= ec
                                } else {
                                    true
                                };
                                if in_sel {
                                    *flag = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // 文字単位のグリッド整列描画 ＆ IME Preedit インライン挿入
            let mut abs_col: usize = 0;
            let mut sub_row: usize = 0;
            let mut local_col: usize = 0;
            let scroll_col = if wrap_cols.is_some() { 0 } else { scroll.1 };
            let mut text_batch: Option<(String, f32, f32, Color32)> = None;
            let mut cursor_vis_pos: Option<(usize, usize)> = None;
            let flush_text_batch = |batch: &mut Option<(String, f32, f32, Color32)>| {
                if let Some((b_text, b_x, b_y, b_fg)) = batch.take() {
                    painter.text(
                        Pos2::new(b_x, b_y),
                        Align2::LEFT_TOP,
                        b_text,
                        font_id.clone(),
                        b_fg,
                    );
                }
            };

            for idx in 0..=chars.len() {
                if is_active_window && is_current_line && idx == cur.col.min(chars.len()) {
                    if !self.ime_preedit.is_empty() {
                        flush_text_batch(&mut text_batch);
                        let mut ul_sub_row = sub_row;
                        let mut ul_start_col = local_col;
                        for pch in self.ime_preedit.chars() {
                            let pw = UnicodeWidthChar::width(pch).unwrap_or(1).max(1);
                            if advance_wrap_cursor(&mut sub_row, &mut local_col, pw, wrap_cols) {
                                let prev_max_col = wrap_cols.unwrap_or(usize::MAX);
                                if vis_row + ul_sub_row < visible_lines && prev_max_col > scroll_col
                                {
                                    let ul_y = text_start_y
                                        + ((vis_row + ul_sub_row) as f32) * line_height
                                        + line_height
                                        - 2.0;
                                    let x1 = text_start_x
                                        + ((ul_start_col.max(scroll_col) - scroll_col) as f32)
                                            * char_width;
                                    let x2 = text_start_x
                                        + ((prev_max_col - scroll_col) as f32) * char_width;
                                    if x2 > x1 {
                                        painter.line_segment(
                                            [Pos2::new(x1, ul_y), Pos2::new(x2, ul_y)],
                                            Stroke::new(2.0_f32, pal.accent_cyan),
                                        );
                                    }
                                }
                                ul_sub_row = sub_row;
                                ul_start_col = 0;
                            }
                            let sub_y = text_start_y + ((vis_row + sub_row) as f32) * line_height;
                            if vis_row + sub_row < visible_lines && local_col >= scroll_col {
                                let px =
                                    text_start_x + ((local_col - scroll_col) as f32) * char_width;
                                let cell_rect = Rect::from_min_size(
                                    Pos2::new(px, sub_y),
                                    Vec2::new((pw as f32) * char_width, line_height),
                                );
                                painter.rect_filled(cell_rect, 0.0, pal.bg_selection);
                                painter.text(
                                    Pos2::new(px, sub_y),
                                    Align2::LEFT_TOP,
                                    pch.to_string(),
                                    font_id.clone(),
                                    pal.accent_cyan,
                                );
                            }
                            local_col += pw;
                            abs_col += pw;
                        }
                        if vis_row + ul_sub_row < visible_lines && local_col > scroll_col {
                            let ul_s = ul_start_col.max(scroll_col);
                            let ul_x1 = text_start_x + ((ul_s - scroll_col) as f32) * char_width;
                            let ul_x2 =
                                text_start_x + ((local_col - scroll_col) as f32) * char_width;
                            let ul_y = text_start_y
                                + ((vis_row + ul_sub_row) as f32) * line_height
                                + line_height
                                - 2.0;
                            painter.line_segment(
                                [Pos2::new(ul_x1, ul_y), Pos2::new(ul_x2, ul_y)],
                                Stroke::new(2.0_f32, pal.accent_cyan),
                            );
                        }
                    }
                    cursor_vis_pos = Some((sub_row, local_col));
                }

                if idx == chars.len() {
                    break;
                }

                let ch = chars[idx];
                let ch_cols = char_display_width(ch, abs_col, tab_width);
                if advance_wrap_cursor(&mut sub_row, &mut local_col, ch_cols, wrap_cols) {
                    flush_text_batch(&mut text_batch);
                }

                let sub_y = text_start_y + ((vis_row + sub_row) as f32) * line_height;
                if vis_row + sub_row < visible_lines && local_col >= scroll_col {
                    let char_x = text_start_x + ((local_col - scroll_col) as f32) * char_width;
                    let cell_rect = Rect::from_min_size(
                        Pos2::new(char_x, sub_y),
                        Vec2::new((ch_cols as f32) * char_width, line_height),
                    );

                    let is_zenkaku_space = inv_info
                        .as_ref()
                        .and_then(|v| v.get(idx))
                        .map(|item| item.is_zenkaku_space)
                        .unwrap_or(false);

                    let is_paren_match = is_active_window
                        && paren_positions.is_some_and(|(p1, p2)| {
                            (p1.line == line_idx && p1.col == idx)
                                || (p2.line == line_idx && p2.col == idx)
                        });

                    if *visual_selected.get(idx).unwrap_or(&false) {
                        painter.rect_filled(cell_rect, 0.0, pal.bg_selection);
                    } else if *search_matched.get(idx).unwrap_or(&false) {
                        let c = pal.accent_yellow;
                        painter.rect_filled(
                            cell_rect,
                            2.0,
                            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 90),
                        );
                    } else if is_paren_match {
                        painter.rect_filled(cell_rect, 2.0, pal.bg_active);
                    } else if is_zenkaku_space {
                        let c = pal.accent_red;
                        painter.rect_filled(
                            cell_rect.shrink(1.0),
                            2.0,
                            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 60),
                        );
                    }

                    if ch != '\t' {
                        let fg = char_colors.get(idx).copied().unwrap_or(pal.fg_text);
                        if vivivi_modes::keisen::char_to_keisen_state(ch).is_some()
                            || matches!(ch, '╭' | '╮' | '╯' | '╰')
                        {
                            flush_text_batch(&mut text_batch);
                            draw_keisen_glyph(&painter, cell_rect, ch, fg);
                        } else if ch_cols == 1 {
                            if let Some((ref mut text, _, _, cur_fg)) = text_batch {
                                if cur_fg == fg {
                                    text.push(ch);
                                } else {
                                    flush_text_batch(&mut text_batch);
                                    text_batch = Some((String::from(ch), char_x, sub_y, fg));
                                }
                            } else {
                                text_batch = Some((String::from(ch), char_x, sub_y, fg));
                            }
                        } else {
                            flush_text_batch(&mut text_batch);
                            if ch != ' ' {
                                painter.text(
                                    Pos2::new(char_x, sub_y),
                                    Align2::LEFT_TOP,
                                    ch.encode_utf8(&mut [0u8; 4]),
                                    font_id.clone(),
                                    fg,
                                );
                            }
                        }
                    } else {
                        flush_text_batch(&mut text_batch);
                    }
                }

                local_col += ch_cols;
                abs_col += ch_cols;
            }

            flush_text_batch(&mut text_batch);

            if chars.is_empty() && *visual_selected.first().unwrap_or(&false) {
                let cell_rect = Rect::from_min_size(
                    Pos2::new(text_start_x, row_y),
                    Vec2::new(char_width, line_height),
                );
                painter.rect_filled(cell_rect, 0.0, pal.bg_selection);
            }

            if is_active_window && is_current_line {
                let (c_sub_row, c_local_col) = cursor_vis_pos.unwrap_or((0, 0));
                let cursor_row_y = text_start_y + ((vis_row + c_sub_row) as f32) * line_height;

                if vis_row + c_sub_row < visible_lines && c_local_col >= scroll_col {
                    let cursor_x = text_start_x + ((c_local_col - scroll_col) as f32) * char_width;
                    active_cursor_screen_rect = Some(self.render_editor_cursor(
                        &painter,
                        pal,
                        font_id,
                        chars,
                        cur,
                        tab_width,
                        cursor_x,
                        cursor_row_y,
                        char_width,
                        line_height,
                    ));
                }
            }

            vis_row += (sub_row + 1).max(line_sub_rows);
            line_idx += 1;
        }

        // ウィンドウ枠線（複数分割時またはアクティブ強調）
        let stroke_color = if is_active_window && window_count > 1 {
            pal.accent_blue
        } else {
            pal.bg_statusline
        };
        painter.rect_stroke(
            win_rect,
            0.0,
            Stroke::new(1.0_f32, stroke_color),
            egui::StrokeKind::Inside,
        );

        active_cursor_screen_rect
    }
}
