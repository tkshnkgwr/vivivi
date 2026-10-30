//! 中央エディタ分割ビュー・`ui.painter()` 等幅グリッド描画・IME 座標通知・ミニマップ描画モジュール。

use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Frame, Pos2, Rect, Stroke, Vec2};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use vivivi_app::LogicalRect;
use vivivi_modes::{LineChangeState, TokenType};
use vivivi_vi::ViMode;

use super::ViviviGuiApp;
use crate::theme::colors;

#[inline]
fn compute_line_syntax_key(text: &str, lang: vivivi_modes::SyntaxLang) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    (lang as u8).hash(&mut hasher);
    hasher.finish()
}

impl ViviviGuiApp {
    /// 4. 中央エディタ分割ビュー ＆ ミニマップ領域の描画（`ui.painter()` によるカスタム等幅レンダリング）
    pub(crate) fn render_central_panel(&mut self, ctx: &egui::Context) {
        // 4.1 入力イベント (`Event::Key`, `Event::Text`, `Event::Ime`) の取得とディスパッチ
        let (events, current_modifiers, dropped_files, hovered_files) = ctx.input(|i| {
            (
                i.events.clone(),
                i.modifiers,
                i.raw.dropped_files.clone(),
                i.raw.hovered_files.clone(),
            )
        });
        self.process_egui_events(&events, current_modifiers);
        self.handle_drag_and_drop_files(&dropped_files);

        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(colors::BG_EDITOR))
            .show(ctx, |ui| {
                let available_rect = ui.available_rect_before_wrap();
                let minimap_width = if self.minimap_open {
                    self.minimap_width
                } else {
                    0.0
                };
                let editor_width = (available_rect.width() - minimap_width).max(100.0);

                let editor_rect = Rect::from_min_size(
                    available_rect.min,
                    Vec2::new(editor_width, available_rect.height()),
                );

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

                // マウスクリックによるウィンドウフォーカス＆カーソル位置移動判定
                let pointer_click_pos = ui.input(|i| {
                    if i.pointer.primary_clicked() {
                        i.pointer.interact_pos()
                    } else {
                        None
                    }
                });

                let mut active_cursor_screen_rect: Option<Rect> = None;
                let active_search_re = self.core.active_search_regex();

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
                for (win_idx, lrect) in &window_rects {
                    let win_min = Pos2::new(
                        editor_rect.min.x + lrect.x as f32,
                        editor_rect.min.y + lrect.y as f32,
                    );
                    let win_rect = Rect::from_min_size(
                        win_min,
                        Vec2::new(lrect.width as f32, lrect.height as f32),
                    );

                    let gutter_cols = if self.core.show_line_numbers {
                        6.5
                    } else {
                        2.0
                    };
                    let gutter_px = gutter_cols * char_width;
                    let text_start_x = win_rect.min.x + padding + gutter_px;
                    let text_start_y = win_rect.min.y + padding;

                    // クリックされた場合、そのウィンドウへフォーカスを切り替えカーソルを移動
                    if let Some(click_pos) = pointer_click_pos
                        && win_rect.contains(click_pos)
                    {
                        self.core.switch_to_window(*win_idx);
                        let clicked_vis_row = ((click_pos.y - text_start_y) / line_height)
                            .floor()
                            .max(0.0) as usize;
                        let clicked_vis_col =
                            ((click_pos.x - text_start_x) / char_width).floor().max(0.0) as usize;

                        let target_line = (self.core.scroll.0 + clicked_vis_row)
                            .min(self.core.buffer.line_count().saturating_sub(1));

                        // 水平スクロールと全角/タブ文字幅を考慮して正確な文字インデックス（Char Index）へ解決
                        let scroll_col = self.core.scroll.1;
                        let target_vis_col = scroll_col + clicked_vis_col;
                        let resolved_col =
                            if let Some(line_str) = self.core.buffer.get_line(target_line) {
                                let tab_w = self.core.tab_width.max(1);
                                let mut cur_vcol = 0;
                                let mut found_idx = 0;
                                for (c_idx, ch) in line_str.chars().enumerate() {
                                    let w = if ch == '\t' {
                                        tab_w - (cur_vcol % tab_w)
                                    } else {
                                        UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
                                    };
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

                        self.core.cursor.pos.line = target_line;
                        self.core.cursor.pos.col = resolved_col;
                        self.core.clamp_cursor();
                    }

                    let is_active_window = *win_idx == self.core.active_window;
                    if is_active_window {
                        self.last_active_text_origin = Pos2::new(text_start_x, text_start_y);
                        self.last_char_width = char_width;
                    }
                    let (cur, scroll) = if is_active_window {
                        (self.core.cursor_pos(), self.core.scroll)
                    } else if let Some(win) = self.core.windows.get(*win_idx) {
                        (win.cursor.pos, win.scroll)
                    } else {
                        (self.core.cursor_pos(), self.core.scroll)
                    };

                    let painter = ui.painter().with_clip_rect(win_rect);

                    // ウィンドウ背景
                    painter.rect_filled(win_rect, 0.0, colors::BG_EDITOR);

                    // ガター（行番号 ＆ 変更マーカー領域）の背景と境界線
                    let gutter_rect = Rect::from_min_size(
                        win_rect.min,
                        Vec2::new(padding + gutter_px - 4.0, win_rect.height()),
                    );
                    painter.rect_filled(gutter_rect, 0.0, Color32::from_rgb(24, 25, 36));
                    painter.line_segment(
                        [gutter_rect.right_top(), gutter_rect.right_bottom()],
                        Stroke::new(1.0_f32, colors::BG_STATUSLINE),
                    );

                    let visible_lines = ((win_rect.height() - padding * 2.0) / line_height)
                        .floor()
                        .max(1.0) as usize;
                    let tab_width = self.core.tab_width.max(1);

                    // 描画ループ用スクラッチバッファ（毎フレーム・各行でのヒープアロケーションをゼロ化）
                    let mut chars_buf: Vec<char> = Vec::with_capacity(256);
                    let mut char_colors_buf: Vec<Color32> = Vec::with_capacity(256);
                    let mut search_matched_buf: Vec<bool> = Vec::with_capacity(256);
                    let mut visual_selected_buf: Vec<bool> = Vec::with_capacity(256);

                    let mut line_idx = scroll.0;
                    let mut vis_row = 0;

                    while vis_row < visible_lines {
                        let row_y = text_start_y + (vis_row as f32) * line_height;

                        if line_idx >= total_lines {
                            // バッファ末尾以降のチルダ (~) 描画
                            painter.text(
                                Pos2::new(win_rect.min.x + padding, row_y),
                                Align2::LEFT_TOP,
                                "~",
                                font_id.clone(),
                                colors::FG_DIMMED,
                            );
                            vis_row += 1;
                            line_idx += 1;
                            continue;
                        }

                        // コード折り畳みによって隠されている行はスキップ
                        if self.core.vi.fold_manager.is_line_hidden(line_idx + 1) {
                            line_idx += 1;
                            continue;
                        }

                        let is_current_line = line_idx == cur.line;

                        // カレント行のハイライト背景
                        if is_current_line && is_active_window {
                            let cur_line_rect = Rect::from_min_size(
                                Pos2::new(gutter_rect.right(), row_y),
                                Vec2::new(win_rect.right() - gutter_rect.right(), line_height),
                            );
                            painter.rect_filled(cur_line_rect, 0.0, Color32::from_rgb(31, 35, 53));
                        }

                        // 行番号の描画
                        if self.core.show_line_numbers {
                            let line_num_val = if self.core.relative_number && !is_current_line {
                                (line_idx as isize - cur.line as isize).unsigned_abs()
                            } else {
                                line_idx + 1
                            };
                            let line_num_str = format!("{:>4}", line_num_val);
                            let num_color = if is_current_line && is_active_window {
                                colors::ACCENT_YELLOW
                            } else {
                                colors::FG_LINENUMBER
                            };
                            painter.text(
                                Pos2::new(win_rect.min.x + padding, row_y),
                                Align2::LEFT_TOP,
                                line_num_str,
                                font_id.clone(),
                                num_color,
                            );
                        }

                        // 行テキストの取得と折り畳み・CSV・不可視文字フォーマット
                        let raw_line = self.core.buffer.get_line(line_idx).unwrap_or_default();

                        // 変更追跡ガターマーカー (Added / Modified / SavedModified) の描画（O(1) アロケーションフリー判定）
                        let marker = self.core.change_tracker.marker_at(line_idx, &raw_line);
                        let marker_color = match marker {
                            LineChangeState::Added | LineChangeState::Modified => {
                                Some(colors::ACCENT_YELLOW)
                            }
                            LineChangeState::SavedModified => Some(colors::ACCENT_GREEN),
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
                        } else {
                            let key = compute_line_syntax_key(&display_line, self.core.syntax_lang);
                            if let Some(cached) = self.syntax_cache.get(&key) {
                                cached.clone()
                            } else {
                                let t = vivivi_modes::tokenize_line(
                                    &display_line,
                                    self.core.syntax_lang,
                                );
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
                            let c = Self::syntax_token_color(token.token_type);
                            for _ in token.text.chars() {
                                char_colors.push(c);
                            }
                        }
                        char_colors.resize(chars.len(), colors::FG_TEXT);

                        if let Some(ref inv) = inv_info {
                            for (idx, item) in inv.iter().enumerate() {
                                if idx < char_colors.len() && item.is_invisible {
                                    char_colors[idx] = colors::FG_DIMMED;
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
                        let mut visual_col: usize = 0;
                        let scroll_col = scroll.1;
                        let mut text_batch: Option<(String, f32, Color32)> = None;

                        for idx in 0..=chars.len() {
                            // アクティブカーソル位置に到達した場合、IME Preedit（未確定文字列）をインライン描画
                            if is_active_window
                                && is_current_line
                                && idx == cur.col.min(chars.len())
                                && !self.ime_preedit.is_empty()
                            {
                                if let Some((b_text, b_x, b_fg)) = text_batch.take() {
                                    painter.text(
                                        Pos2::new(b_x, row_y),
                                        Align2::LEFT_TOP,
                                        b_text,
                                        font_id.clone(),
                                        b_fg,
                                    );
                                }
                                let preedit_start_col = visual_col;
                                for pch in self.ime_preedit.chars() {
                                    let pw = UnicodeWidthChar::width(pch).unwrap_or(1).max(1);
                                    if visual_col >= scroll_col {
                                        let px = text_start_x
                                            + ((visual_col - scroll_col) as f32) * char_width;
                                        let cell_rect = Rect::from_min_size(
                                            Pos2::new(px, row_y),
                                            Vec2::new((pw as f32) * char_width, line_height),
                                        );
                                        painter.rect_filled(
                                            cell_rect,
                                            0.0,
                                            Color32::from_rgb(45, 58, 92),
                                        );
                                        painter.text(
                                            Pos2::new(px, row_y),
                                            Align2::LEFT_TOP,
                                            pch.to_string(),
                                            font_id.clone(),
                                            colors::ACCENT_CYAN,
                                        );
                                    }
                                    visual_col += pw;
                                }
                                // IME 未確定文字列の下線（Preedit Underline）を描画
                                if visual_col > scroll_col {
                                    let ul_start_col = preedit_start_col.max(scroll_col);
                                    let ul_x1 = text_start_x
                                        + ((ul_start_col - scroll_col) as f32) * char_width;
                                    let ul_x2 = text_start_x
                                        + ((visual_col - scroll_col) as f32) * char_width;
                                    let ul_y = row_y + line_height - 2.0;
                                    painter.line_segment(
                                        [Pos2::new(ul_x1, ul_y), Pos2::new(ul_x2, ul_y)],
                                        Stroke::new(2.0_f32, colors::ACCENT_CYAN),
                                    );
                                }
                            }

                            if idx == chars.len() {
                                break;
                            }

                            let ch = chars[idx];
                            let ch_cols = if ch == '\t' {
                                tab_width - (visual_col % tab_width)
                            } else {
                                UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
                            };

                            if visual_col >= scroll_col {
                                let char_x =
                                    text_start_x + ((visual_col - scroll_col) as f32) * char_width;
                                let cell_rect = Rect::from_min_size(
                                    Pos2::new(char_x, row_y),
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

                                // ビジュアル選択・検索ハイライト・対応括弧・全角スペース背景描画
                                if *visual_selected.get(idx).unwrap_or(&false) {
                                    painter.rect_filled(cell_rect, 0.0, colors::BG_SELECTION);
                                } else if *search_matched.get(idx).unwrap_or(&false) {
                                    painter.rect_filled(
                                        cell_rect,
                                        2.0,
                                        Color32::from_rgba_unmultiplied(224, 175, 104, 90),
                                    );
                                } else if is_paren_match {
                                    painter.rect_filled(
                                        cell_rect,
                                        2.0,
                                        Color32::from_rgb(65, 85, 130),
                                    );
                                } else if is_zenkaku_space {
                                    painter.rect_filled(
                                        cell_rect.shrink(1.0),
                                        2.0,
                                        Color32::from_rgba_unmultiplied(247, 118, 142, 60),
                                    );
                                }

                                if ch != '\t' {
                                    let fg =
                                        char_colors.get(idx).copied().unwrap_or(colors::FG_TEXT);
                                    if ch_cols == 1 {
                                        // 半角文字（ASCII/等幅1桁）は同色連続バッチにまとめてアロケーションと描画回数を削減
                                        if let Some((ref mut text, _, cur_fg)) = text_batch {
                                            if cur_fg == fg {
                                                text.push(ch);
                                            } else {
                                                if let Some((b_text, b_x, b_fg)) = text_batch.take()
                                                {
                                                    painter.text(
                                                        Pos2::new(b_x, row_y),
                                                        Align2::LEFT_TOP,
                                                        b_text,
                                                        font_id.clone(),
                                                        b_fg,
                                                    );
                                                }
                                                text_batch = Some((String::from(ch), char_x, fg));
                                            }
                                        } else {
                                            text_batch = Some((String::from(ch), char_x, fg));
                                        }
                                    } else {
                                        // 全角文字はバッチをフラッシュして個別描画
                                        if let Some((b_text, b_x, b_fg)) = text_batch.take() {
                                            painter.text(
                                                Pos2::new(b_x, row_y),
                                                Align2::LEFT_TOP,
                                                b_text,
                                                font_id.clone(),
                                                b_fg,
                                            );
                                        }
                                        if ch != ' ' {
                                            painter.text(
                                                Pos2::new(char_x, row_y),
                                                Align2::LEFT_TOP,
                                                ch.encode_utf8(&mut [0u8; 4]),
                                                font_id.clone(),
                                                fg,
                                            );
                                        }
                                    }
                                } else if let Some((b_text, b_x, b_fg)) = text_batch.take() {
                                    painter.text(
                                        Pos2::new(b_x, row_y),
                                        Align2::LEFT_TOP,
                                        b_text,
                                        font_id.clone(),
                                        b_fg,
                                    );
                                }
                            }

                            visual_col += ch_cols;
                        }

                        // 行末に残った半角テキストバッチを一括描画
                        if let Some((b_text, b_x, b_fg)) = text_batch {
                            painter.text(
                                Pos2::new(b_x, row_y),
                                Align2::LEFT_TOP,
                                b_text,
                                font_id.clone(),
                                b_fg,
                            );
                        }

                        // 空行での VisualLine 選択表示
                        if chars.is_empty() && *visual_selected.first().unwrap_or(&false) {
                            let cell_rect = Rect::from_min_size(
                                Pos2::new(text_start_x, row_y),
                                Vec2::new(char_width, line_height),
                            );
                            painter.rect_filled(cell_rect, 0.0, colors::BG_SELECTION);
                        }

                        // 4.3 現在のモードに応じたカーソル（ブロック／Iビーム／下線）の描画
                        if is_active_window && is_current_line {
                            let base_vis_col = self.core.visual_col_at(cur.line, cur.col);
                            let preedit_cols = if !self.ime_preedit.is_empty() {
                                UnicodeWidthStr::width(self.ime_preedit.as_str())
                            } else {
                                0
                            };
                            let effective_vis_col = base_vis_col + preedit_cols;

                            if effective_vis_col >= scroll_col {
                                let cursor_x = text_start_x
                                    + ((effective_vis_col - scroll_col) as f32) * char_width;
                                let char_under_cursor = chars.get(cur.col).copied();
                                let cursor_cell_cols = char_under_cursor
                                    .and_then(|c| {
                                        if c == '\t' {
                                            Some(tab_width - (base_vis_col % tab_width))
                                        } else {
                                            UnicodeWidthChar::width(c)
                                        }
                                    })
                                    .unwrap_or(1)
                                    .max(1);
                                let block_w = (cursor_cell_cols as f32) * char_width;

                                let mode = self.core.mode();
                                match mode {
                                    ViMode::Insert => {
                                        // Iビーム（縦バー）カーソル
                                        let ibeam_rect = Rect::from_min_size(
                                            Pos2::new(cursor_x, row_y),
                                            Vec2::new(2.0, line_height),
                                        );
                                        painter.rect_filled(ibeam_rect, 1.0, colors::ACCENT_GREEN);
                                        active_cursor_screen_rect = Some(ibeam_rect);
                                    }
                                    ViMode::Replace => {
                                        // アンダースコア（下線）カーソル
                                        let under_rect = Rect::from_min_size(
                                            Pos2::new(cursor_x, row_y + line_height - 3.0),
                                            Vec2::new(block_w, 3.0),
                                        );
                                        painter.rect_filled(under_rect, 1.0, colors::ACCENT_RED);
                                        active_cursor_screen_rect = Some(Rect::from_min_size(
                                            Pos2::new(cursor_x, row_y),
                                            Vec2::new(block_w, line_height),
                                        ));
                                    }
                                    _ => {
                                        // ブロックカーソル（Normal / Visual / CommandLine）
                                        let block_color = if mode.is_visual() {
                                            colors::ACCENT_PURPLE
                                        } else {
                                            colors::ACCENT_BLUE
                                        };
                                        let block_rect = Rect::from_min_size(
                                            Pos2::new(cursor_x, row_y),
                                            Vec2::new(block_w, line_height),
                                        );
                                        painter.rect_filled(block_rect, 2.0, block_color);

                                        // ブロックカーソル上の文字を反転色（背景色）でくっきり再描画
                                        if let Some(ch) = char_under_cursor
                                            && ch != '\t'
                                            && ch != ' '
                                        {
                                            painter.text(
                                                Pos2::new(cursor_x, row_y),
                                                Align2::LEFT_TOP,
                                                ch.encode_utf8(&mut [0u8; 4]),
                                                font_id.clone(),
                                                colors::BG_EDITOR,
                                            );
                                        }
                                        active_cursor_screen_rect = Some(block_rect);
                                    }
                                }
                            }
                        }

                        vis_row += 1;
                        line_idx += 1;
                    }

                    // ウィンドウ枠線（複数分割時またはアクティブ強調）
                    let stroke_color = if is_active_window && window_rects.len() > 1 {
                        colors::ACCENT_BLUE
                    } else {
                        colors::BG_STATUSLINE
                    };
                    painter.rect_stroke(
                        win_rect,
                        0.0,
                        Stroke::new(1.0_f32, stroke_color),
                        egui::StrokeKind::Inside,
                    );
                }

                // 4.4 削除文字落下パーティクルの物理更新 ＆ オーバーレイ描画（ステップ 4）
                if self.particles.is_active() {
                    let dt = ctx.input(|i| i.stable_dt);
                    self.particles.update(dt);
                    let overlay_painter = ui.painter().with_clip_rect(editor_rect);
                    self.particles.render(&overlay_painter, &font_id);
                    // パーティクルが存在する間のみ再描画を要求し、消滅後は 0% CPU リアクティブ待機へ戻す
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

                // --- 右端ミニマップ領域（可視行 Viewport Culling 描画） ---
                if self.minimap_open {
                    let minimap_rect = Rect::from_min_size(
                        Pos2::new(editor_rect.max.x, available_rect.min.y),
                        Vec2::new(minimap_width, available_rect.height()),
                    );

                    // ミニマップクリックで画面中央へスクロールジャンプ
                    if let Some(click_pos) = pointer_click_pos
                        && minimap_rect.contains(click_pos)
                    {
                        let ratio = ((click_pos.y - minimap_rect.top()) / minimap_rect.height())
                            .clamp(0.0, 1.0);
                        let total_cnt = self.core.buffer.line_count().max(1);
                        let target_line =
                            (ratio * (total_cnt.saturating_sub(1)) as f32).round() as usize;
                        let visible_rows = (editor_rect.height() / line_height).max(1.0) as usize;
                        let half_screen = visible_rows / 2;
                        self.core.scroll.0 = target_line.saturating_sub(half_screen);
                        self.core.cursor.pos.line = target_line;
                        self.core.clamp_cursor();
                    }

                    let mm_painter = ui.painter().with_clip_rect(minimap_rect);
                    mm_painter.rect_filled(minimap_rect, 0.0, colors::BG_PANEL);
                    mm_painter.line_segment(
                        [minimap_rect.left_top(), minimap_rect.left_bottom()],
                        Stroke::new(1.0_f32, colors::BG_STATUSLINE),
                    );

                    let total_lines_mm = self.core.buffer.line_count().max(1);
                    let scroll_row = self.core.scroll.0;
                    let visible_rows = (editor_rect.height() / line_height).max(1.0) as usize;

                    // 可視範囲周辺のコード構造ミニライン描画（バッファ内容キーによるキャッシュ化）
                    let max_mm_rows = (minimap_rect.height() / 3.0) as usize;
                    let current_key = (self.core.buffer.len(), total_lines_mm);
                    if self.minimap_cache_key != current_key
                        || self.minimap_cache.len() != max_mm_rows
                    {
                        self.minimap_cache.clear();
                        let step = (total_lines_mm / max_mm_rows.max(1)).max(1);
                        for i in 0..max_mm_rows {
                            let l_idx = i * step;
                            if l_idx >= total_lines_mm {
                                break;
                            }
                            if let Some(line_str) = self.core.buffer.get_line(l_idx) {
                                let trimmed = line_str.trim_start();
                                if !trimmed.is_empty() {
                                    let indent = (line_str.len() - trimmed.len()).min(20) as f32;
                                    let len = (trimmed.len().min(40) as f32) * 0.9;
                                    self.minimap_cache.push((indent, len));
                                } else {
                                    self.minimap_cache.push((0.0, 0.0));
                                }
                            } else {
                                self.minimap_cache.push((0.0, 0.0));
                            }
                        }
                        self.minimap_cache_key = current_key;
                    }

                    for (i, &(indent, len)) in self.minimap_cache.iter().enumerate() {
                        if len > 0.0 {
                            let y = minimap_rect.top() + (i as f32) * 3.0 + 2.0;
                            let x1 = minimap_rect.left() + 6.0 + indent * 1.2;
                            let x2 = (x1 + len).min(minimap_rect.right() - 4.0);
                            if x2 > x1 {
                                mm_painter.line_segment(
                                    [Pos2::new(x1, y), Pos2::new(x2, y)],
                                    Stroke::new(1.5_f32, colors::FG_LINENUMBER),
                                );
                            }
                        }
                    }

                    // ビューポートハイライト矩形
                    let norm_start = (scroll_row as f32 / total_lines_mm as f32).clamp(0.0, 1.0);
                    let norm_len = (visible_rows as f32 / total_lines_mm as f32).clamp(0.05, 1.0);

                    let vp_top = minimap_rect.top() + norm_start * minimap_rect.height();
                    let vp_height =
                        (norm_len * minimap_rect.height()).min(minimap_rect.bottom() - vp_top);

                    let vp_rect = Rect::from_min_size(
                        Pos2::new(minimap_rect.left() + 2.0, vp_top),
                        Vec2::new(minimap_width - 4.0, vp_height),
                    );

                    mm_painter.rect_filled(
                        vp_rect,
                        CornerRadius::same(2),
                        Color32::from_rgba_unmultiplied(122, 162, 247, 40),
                    );
                    mm_painter.rect_stroke(
                        vp_rect,
                        CornerRadius::same(2),
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(122, 162, 247, 100)),
                        egui::StrokeKind::Inside,
                    );
                }

                // 4.6 ファイルホバー中のオーバーレイガイダンス（ドラッグ＆ドロップ受け入れ視覚フィードバック）
                if !hovered_files.is_empty() {
                    let overlay_painter = ui.painter_at(available_rect);
                    overlay_painter.rect_filled(
                        available_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(26, 27, 38, 200),
                    );
                    overlay_painter.rect_stroke(
                        available_rect.shrink(8.0),
                        CornerRadius::same(8),
                        Stroke::new(2.0, colors::ACCENT_BLUE),
                        egui::StrokeKind::Inside,
                    );
                    overlay_painter.text(
                        available_rect.center(),
                        Align2::CENTER_CENTER,
                        "ここにファイルをドロップして開く",
                        FontId::proportional(20.0),
                        colors::FG_TEXT,
                    );
                }
            });
    }
}
