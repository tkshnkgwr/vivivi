//! 中央エディタ分割ビュー・`ui.painter()` 等幅グリッド描画・IME 座標通知・ミニマップ描画モジュール。

use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Frame, Pos2, Rect, Stroke, Vec2};
use unicode_width::UnicodeWidthChar;
use vivivi_app::LogicalRect;
use vivivi_modes::{LineChangeState, TokenType};
use vivivi_vi::ViMode;

use super::ViviviGuiApp;

#[inline]
fn compute_line_syntax_key(text: &str, lang: vivivi_modes::SyntaxLang) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    (lang as u8).hash(&mut hasher);
    hasher.finish()
}

/// 文字の視覚幅を計算します（タブは `tab_width` 境界まで展開）。
#[inline]
fn char_display_width(ch: char, visual_col: usize, tab_width: usize) -> usize {
    if ch == '\t' {
        tab_width - (visual_col % tab_width)
    } else {
        UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
    }
}

/// 折り返し幅 `wrap_cols` に基づき、現在位置 `(sub_row, local_col)` を必要に応じて次サブ行へ折り返します。
///
/// 折り返しが発生した場合は `true` を返します。
#[inline]
fn advance_wrap_cursor(
    sub_row: &mut usize,
    local_col: &mut usize,
    ch_cols: usize,
    wrap_cols: Option<usize>,
) -> bool {
    if let Some(max_cols) = wrap_cols
        && *local_col > 0
        && *local_col + ch_cols > max_cols
    {
        *sub_row += 1;
        *local_col = 0;
        true
    } else {
        false
    }
}

/// 1 論理行が指定された折り返し幅で占める視覚サブ行数（最低 1 行）を計算します。
fn count_line_visual_rows(chars: &[char], tab_width: usize, wrap_cols: Option<usize>) -> usize {
    let Some(max_cols) = wrap_cols else {
        return 1;
    };
    let mut sub_row = 0usize;
    let mut local_col = 0usize;
    let mut abs_col = 0usize;
    for &ch in chars {
        let w = char_display_width(ch, abs_col, tab_width);
        advance_wrap_cursor(&mut sub_row, &mut local_col, w, Some(max_cols));
        local_col += w;
        abs_col += w;
    }
    sub_row + 1
}

/// ソフトラップ有効時の視覚行内をクリックされた位置から、対象論理行内の文字インデックスを解決します。
fn resolve_wrapped_char_col(
    line_str: &str,
    tab_width: usize,
    wrap_cols: usize,
    target_sub_row: usize,
    target_local_col: usize,
) -> usize {
    let mut sub_row = 0usize;
    let mut local_col = 0usize;
    let mut abs_col = 0usize;
    let mut last_idx_in_target_row = 0usize;
    let mut saw_target_row = target_sub_row == 0;

    for (c_idx, ch) in line_str.chars().enumerate() {
        let w = char_display_width(ch, abs_col, tab_width);
        let wrapped = advance_wrap_cursor(&mut sub_row, &mut local_col, w, Some(wrap_cols));
        if wrapped && sub_row == target_sub_row {
            saw_target_row = true;
            last_idx_in_target_row = c_idx;
        }
        if sub_row > target_sub_row {
            return last_idx_in_target_row;
        }
        if sub_row == target_sub_row {
            if local_col + w > target_local_col {
                return c_idx;
            }
            last_idx_in_target_row = c_idx + 1;
        }
        local_col += w;
        abs_col += w;
    }

    if saw_target_row {
        last_idx_in_target_row
    } else {
        line_str.chars().count()
    }
}

/// Unicode 罫線文字（`U+2500..=U+254B`, `╭╮╯╰`）を 1 セル矩形内のベクター線分として描画します。
///
/// 日本語フォールバックフォントでは罫線グリフが全角幅として描画され列ズレや行間隙間が生じるため、
/// 文字グリフではなくセル境界いっぱいのベクター線分で描画します。
fn draw_keisen_glyph(painter: &egui::Painter, cell_rect: Rect, ch: char, color: Color32) -> bool {
    let state = match vivivi_modes::keisen::char_to_keisen_state(ch) {
        Some(s) => s,
        None => match ch {
            '╭' => vivivi_modes::keisen::KT_RIGHT_THIN | vivivi_modes::keisen::KT_DOWN_THIN,
            '╮' => vivivi_modes::keisen::KT_LEFT_THIN | vivivi_modes::keisen::KT_DOWN_THIN,
            '╯' => vivivi_modes::keisen::KT_LEFT_THIN | vivivi_modes::keisen::KT_UP_THIN,
            '╰' => vivivi_modes::keisen::KT_RIGHT_THIN | vivivi_modes::keisen::KT_UP_THIN,
            _ => return false,
        },
    };
    let cx = cell_rect.center().x;
    let cy = cell_rect.center().y;
    let thin_w = (cell_rect.width() * 0.14).clamp(1.2, 2.0);
    let thick_w = (thin_w * 2.1).clamp(2.4, 4.0);

    let arm_width = |mask: u8, thin: u8, thick: u8| -> f32 {
        let bits = state & mask;
        if bits == thick {
            thick_w
        } else if bits == thin || bits != 0 {
            thin_w
        } else {
            0.0
        }
    };

    let up_w = arm_width(
        vivivi_modes::keisen::KT_UP_MASK,
        vivivi_modes::keisen::KT_UP_THIN,
        vivivi_modes::keisen::KT_UP_THICK,
    );
    let left_w = arm_width(
        vivivi_modes::keisen::KT_LEFT_MASK,
        vivivi_modes::keisen::KT_LEFT_THIN,
        vivivi_modes::keisen::KT_LEFT_THICK,
    );
    let down_w = arm_width(
        vivivi_modes::keisen::KT_DOWN_MASK,
        vivivi_modes::keisen::KT_DOWN_THIN,
        vivivi_modes::keisen::KT_DOWN_THICK,
    );
    let right_w = arm_width(
        vivivi_modes::keisen::KT_RIGHT_MASK,
        vivivi_modes::keisen::KT_RIGHT_THIN,
        vivivi_modes::keisen::KT_RIGHT_THICK,
    );

    let h_half = left_w.max(right_w) * 0.5;
    let v_half = up_w.max(down_w) * 0.5;

    if up_w > 0.0 {
        painter.line_segment(
            [Pos2::new(cx, cell_rect.top()), Pos2::new(cx, cy + h_half)],
            Stroke::new(up_w, color),
        );
    }
    if down_w > 0.0 {
        painter.line_segment(
            [
                Pos2::new(cx, cy - h_half),
                Pos2::new(cx, cell_rect.bottom()),
            ],
            Stroke::new(down_w, color),
        );
    }
    if left_w > 0.0 {
        painter.line_segment(
            [Pos2::new(cell_rect.left(), cy), Pos2::new(cx + v_half, cy)],
            Stroke::new(left_w, color),
        );
    }
    if right_w > 0.0 {
        painter.line_segment(
            [Pos2::new(cx - v_half, cy), Pos2::new(cell_rect.right(), cy)],
            Stroke::new(right_w, color),
        );
    }
    true
}

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
        self.process_egui_events(&events, current_modifiers);
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

                // エディタ中央ペイン領域のマウスホバー連動「Iビーム（テキスト入力）」カーソル変更
                // モーダル表示中はモーダル側のカーソル優先のためスキップ
                if !self.is_any_modal_open()
                    && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                    && editor_rect.contains(pos)
                {
                    ctx.set_cursor_icon(egui::CursorIcon::Text);
                }

                // マウスクリックによるウィンドウフォーカス＆カーソル位置移動判定（モーダル中は無効化）
                let pointer_click_pos = if self.is_any_modal_open() {
                    None
                } else {
                    ui.input(|i| {
                        if i.pointer.primary_clicked() {
                            i.pointer.interact_pos()
                        } else {
                            None
                        }
                    })
                };

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
                    let wrap_cols = if self.core.wrap {
                        let text_area_w =
                            (win_rect.right() - text_start_x - padding).max(char_width * 2.0);
                        Some((text_area_w / char_width).floor().max(2.0) as usize)
                    } else {
                        None
                    };

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
                                let row_span =
                                    count_line_visual_rows(&line_chars, tab_w, Some(max_cols));
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
                                    resolve_wrapped_char_col(
                                        &s,
                                        tab_w,
                                        max_cols,
                                        target_sub_row,
                                        clicked_vis_col,
                                    )
                                })
                                .unwrap_or(0);
                            (cur_line, resolved)
                        } else {
                            let target_line = (self.core.scroll.0 + clicked_vis_row)
                                .min(self.core.buffer.line_count().saturating_sub(1));

                            // 水平スクロールと全角/タブ文字幅を考慮して正確な文字インデックス（Char Index）へ解決
                            let scroll_col = self.core.scroll.1;
                            let target_vis_col = scroll_col + clicked_vis_col;
                            let resolved_col =
                                if let Some(line_str) = self.core.buffer.get_line(target_line) {
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
                                pal.fg_dimmed,
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

                        // 行テキストの取得と折り畳み・CSV・不可視文字フォーマット
                        let raw_line = self.core.buffer.get_line(line_idx).unwrap_or_default();

                        // 変更追跡ガターマーカー (Added / Modified / SavedModified) の描画（O(1) アロケーションフリー判定）
                        let marker = self.core.change_tracker.marker_at(line_idx, &raw_line);
                        let marker_color = match marker {
                            LineChangeState::Added | LineChangeState::Modified => {
                                Some(pal.gutter_modified)
                            }
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

                        // カレント行のハイライト背景（折り返し時は占有する全視覚サブ行にまたがって描画）
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

                        // 文字単位のグリッド整列描画 ＆ IME Preedit インライン挿入（ソフトラップ対応）
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
                            // アクティブカーソル位置に到達した場合、IME Preedit（未確定文字列）をインライン描画
                            if is_active_window
                                && is_current_line
                                && idx == cur.col.min(chars.len())
                            {
                                if !self.ime_preedit.is_empty() {
                                    flush_text_batch(&mut text_batch);
                                    let mut ul_sub_row = sub_row;
                                    let mut ul_start_col = local_col;
                                    for pch in self.ime_preedit.chars() {
                                        let pw = UnicodeWidthChar::width(pch).unwrap_or(1).max(1);
                                        if advance_wrap_cursor(
                                            &mut sub_row,
                                            &mut local_col,
                                            pw,
                                            wrap_cols,
                                        ) {
                                            // 折り返し前の行に下線を描画して次行へリセット
                                            let prev_max_col = wrap_cols.unwrap_or(usize::MAX);
                                            if vis_row + ul_sub_row < visible_lines
                                                && prev_max_col > scroll_col
                                            {
                                                let ul_y = text_start_y
                                                    + ((vis_row + ul_sub_row) as f32) * line_height
                                                    + line_height
                                                    - 2.0;
                                                let x1 = text_start_x
                                                    + ((ul_start_col.max(scroll_col) - scroll_col)
                                                        as f32)
                                                        * char_width;
                                                let x2 = text_start_x
                                                    + ((prev_max_col - scroll_col) as f32)
                                                        * char_width;
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
                                        let sub_y = text_start_y
                                            + ((vis_row + sub_row) as f32) * line_height;
                                        if vis_row + sub_row < visible_lines
                                            && local_col >= scroll_col
                                        {
                                            let px = text_start_x
                                                + ((local_col - scroll_col) as f32) * char_width;
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
                                    // IME 未確定文字列の下線（Preedit Underline）を描画
                                    if vis_row + ul_sub_row < visible_lines
                                        && local_col > scroll_col
                                    {
                                        let ul_s = ul_start_col.max(scroll_col);
                                        let ul_x1 = text_start_x
                                            + ((ul_s - scroll_col) as f32) * char_width;
                                        let ul_x2 = text_start_x
                                            + ((local_col - scroll_col) as f32) * char_width;
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
                            if advance_wrap_cursor(&mut sub_row, &mut local_col, ch_cols, wrap_cols)
                            {
                                flush_text_batch(&mut text_batch);
                            }

                            let sub_y = text_start_y + ((vis_row + sub_row) as f32) * line_height;
                            if vis_row + sub_row < visible_lines && local_col >= scroll_col {
                                let char_x =
                                    text_start_x + ((local_col - scroll_col) as f32) * char_width;
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

                                // ビジュアル選択・検索ハイライト・対応括弧・全角スペース背景描画
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
                                        // 罫線文字はフォールバックフォントの幅ズレ・隙間を防ぐためベクター線分で直接描画
                                        flush_text_batch(&mut text_batch);
                                        draw_keisen_glyph(&painter, cell_rect, ch, fg);
                                    } else if ch_cols == 1 {
                                        // 半角文字（ASCII/等幅1桁）は同色連続バッチにまとめてアロケーションと描画回数を削減
                                        if let Some((ref mut text, _, _, cur_fg)) = text_batch {
                                            if cur_fg == fg {
                                                text.push(ch);
                                            } else {
                                                flush_text_batch(&mut text_batch);
                                                text_batch =
                                                    Some((String::from(ch), char_x, sub_y, fg));
                                            }
                                        } else {
                                            text_batch =
                                                Some((String::from(ch), char_x, sub_y, fg));
                                        }
                                    } else {
                                        // 全角文字はバッチをフラッシュして個別描画
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

                        // 行末に残った半角テキストバッチを一括描画
                        flush_text_batch(&mut text_batch);

                        // 空行での VisualLine 選択表示
                        if chars.is_empty() && *visual_selected.first().unwrap_or(&false) {
                            let cell_rect = Rect::from_min_size(
                                Pos2::new(text_start_x, row_y),
                                Vec2::new(char_width, line_height),
                            );
                            painter.rect_filled(cell_rect, 0.0, pal.bg_selection);
                        }

                        // 4.3 現在のモードに応じたカーソル（ブロック／Iビーム／下線）の描画
                        if is_active_window && is_current_line {
                            let (c_sub_row, c_local_col) = cursor_vis_pos.unwrap_or((0, 0));
                            let cursor_row_y =
                                text_start_y + ((vis_row + c_sub_row) as f32) * line_height;

                            if vis_row + c_sub_row < visible_lines && c_local_col >= scroll_col {
                                let cursor_x =
                                    text_start_x + ((c_local_col - scroll_col) as f32) * char_width;
                                let char_under_cursor = chars.get(cur.col).copied();
                                let cursor_cell_cols = char_under_cursor
                                    .and_then(|c| {
                                        if c == '\t' {
                                            let base_vis_col =
                                                self.core.visual_col_at(cur.line, cur.col);
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
                                            Pos2::new(cursor_x, cursor_row_y),
                                            Vec2::new(2.0, line_height),
                                        );
                                        painter.rect_filled(ibeam_rect, 1.0, pal.accent_green);
                                        active_cursor_screen_rect = Some(ibeam_rect);
                                    }
                                    ViMode::Replace => {
                                        // アンダースコア（下線）カーソル
                                        let under_rect = Rect::from_min_size(
                                            Pos2::new(cursor_x, cursor_row_y + line_height - 3.0),
                                            Vec2::new(block_w, 3.0),
                                        );
                                        painter.rect_filled(under_rect, 1.0, pal.accent_red);
                                        active_cursor_screen_rect = Some(Rect::from_min_size(
                                            Pos2::new(cursor_x, cursor_row_y),
                                            Vec2::new(block_w, line_height),
                                        ));
                                    }
                                    _ => {
                                        // ブロックカーソル（Normal / Visual / CommandLine）
                                        let block_color = if mode.is_visual() {
                                            pal.accent_purple
                                        } else {
                                            pal.accent_blue
                                        };
                                        let block_rect = Rect::from_min_size(
                                            Pos2::new(cursor_x, cursor_row_y),
                                            Vec2::new(block_w, line_height),
                                        );
                                        painter.rect_filled(block_rect, 2.0, block_color);

                                        // ブロックカーソル上の文字を反転色（背景色）でくっきり再描画
                                        if let Some(ch) = char_under_cursor
                                            && ch != '\t'
                                            && ch != ' '
                                            && !draw_keisen_glyph(
                                                &painter,
                                                block_rect,
                                                ch,
                                                pal.bg_editor,
                                            )
                                        {
                                            painter.text(
                                                Pos2::new(cursor_x, cursor_row_y),
                                                Align2::LEFT_TOP,
                                                ch.encode_utf8(&mut [0u8; 4]),
                                                font_id.clone(),
                                                pal.bg_editor,
                                            );
                                        }
                                        active_cursor_screen_rect = Some(block_rect);
                                    }
                                }
                            }
                        }

                        vis_row += (sub_row + 1).max(line_sub_rows);
                        line_idx += 1;
                    }

                    // ウィンドウ枠線（複数分割時またはアクティブ強調）
                    let stroke_color = if is_active_window && window_rects.len() > 1 {
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

                // --- 右端ミニマップ領域（完全幾何学的線形対応 ＆ ドラッグ追従） ---
                if self.minimap_open {
                    let minimap_rect = Rect::from_min_size(
                        Pos2::new(editor_rect.max.x, available_rect.min.y),
                        Vec2::new(minimap_width, available_rect.height()),
                    );

                    let total_lines_mm = self.core.buffer.line_count().max(1);
                    let visible_rows = (editor_rect.height() / line_height).max(1.0) as usize;
                    let max_scroll = total_lines_mm.saturating_sub(visible_rows);
                    let mm_h = minimap_rect.height();

                    // ビューポートハイライト矩形の高さ（画面表示行数と総行数の比率、最小24px）
                    let vp_height = if total_lines_mm <= visible_rows {
                        mm_h
                    } else {
                        ((visible_rows as f32 / total_lines_mm as f32) * mm_h).clamp(24.0, mm_h)
                    };
                    let scrollable_h = (mm_h - vp_height).max(0.0);

                    // ミニマップのマウスクリックおよびドラッグ追従処理
                    let is_pointer_down = ui.input(|i| i.pointer.primary_down());
                    let pointer_pos =
                        ui.input(|i| i.pointer.interact_pos().or(i.pointer.latest_pos()));

                    if is_pointer_down
                        && let Some(pos) = pointer_pos
                        && minimap_rect.contains(pos)
                        && max_scroll > 0
                        && scrollable_h > 0.0
                    {
                        // クリック・ドラッグ位置がビューポート矩形の中心になるよう吸着
                        let target_vp_top = (pos.y - vp_height / 2.0)
                            .clamp(minimap_rect.top(), minimap_rect.top() + scrollable_h);
                        let ratio = (target_vp_top - minimap_rect.top()) / scrollable_h;
                        let new_scroll = (ratio * max_scroll as f32).round() as usize;
                        self.core.scroll.0 = new_scroll.min(max_scroll);

                        // カーソル位置も可視画面中央へ同期
                        let center_line =
                            (new_scroll + visible_rows / 2).min(total_lines_mm.saturating_sub(1));
                        self.core.cursor.pos.line = center_line;
                        self.core.clamp_cursor();
                    }

                    let mm_painter = ui.painter().with_clip_rect(minimap_rect);
                    mm_painter.rect_filled(minimap_rect, 0.0, pal.bg_panel);
                    mm_painter.line_segment(
                        [minimap_rect.left_top(), minimap_rect.left_bottom()],
                        Stroke::new(1.0_f32, pal.bg_statusline),
                    );

                    // ミニライン構造描画（0行目から最終行まで均等にサンプリングして端数の切り捨てを防止）
                    let max_mm_rows = (mm_h / 3.0).max(1.0) as usize;
                    let current_key = (self.core.buffer.len(), total_lines_mm);
                    if self.minimap_cache_key != current_key
                        || self.minimap_cache.len() != max_mm_rows
                    {
                        self.minimap_cache.clear();
                        let num_samples = if total_lines_mm <= max_mm_rows {
                            total_lines_mm
                        } else {
                            max_mm_rows
                        };

                        for i in 0..num_samples {
                            let l_idx = if total_lines_mm <= max_mm_rows {
                                i
                            } else {
                                (i as f32 * (total_lines_mm - 1) as f32
                                    / (max_mm_rows - 1).max(1) as f32)
                                    .round() as usize
                            };

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

                    let cache_len = self.minimap_cache.len().max(1);
                    for (i, &(indent, len)) in self.minimap_cache.iter().enumerate() {
                        if len > 0.0 {
                            let y = if total_lines_mm <= max_mm_rows {
                                minimap_rect.top() + (i as f32) * 3.0 + 2.0
                            } else {
                                minimap_rect.top()
                                    + (i as f32 / (cache_len - 1).max(1) as f32) * (mm_h - 4.0)
                                    + 2.0
                            };
                            let x1 = minimap_rect.left() + 6.0 + indent * 1.2;
                            let x2 = (x1 + len).min(minimap_rect.right() - 4.0);
                            if x2 > x1 {
                                mm_painter.line_segment(
                                    [Pos2::new(x1, y), Pos2::new(x2, y)],
                                    Stroke::new(1.5_f32, pal.fg_linenumber),
                                );
                            }
                        }
                    }

                    // ビューポートハイライト矩形（現在スクロール位置と100%線形同期）
                    let current_scroll = self.core.scroll.0.min(max_scroll);
                    let vp_top = if max_scroll == 0 || scrollable_h <= 0.0 {
                        minimap_rect.top()
                    } else {
                        minimap_rect.top()
                            + (current_scroll as f32 / max_scroll as f32) * scrollable_h
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

                // 4.6 ファイルホバー中のオーバーレイガイダンス（ドラッグ＆ドロップ受け入れ視覚フィードバック）
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
                        pal.accent_blue, // 青
                    );

                    // 2. 16進ダンプ列
                    let hex_x = area.left() + 110.0;
                    painter.text(
                        Pos2::new(hex_x, y),
                        Align2::LEFT_TOP,
                        &row_data.hex_dump,
                        font_id.clone(),
                        pal.fg_text, // 本文色
                    );

                    // 3. ASCII列
                    let ascii_x = hex_x + 390.0;
                    painter.text(
                        Pos2::new(ascii_x, y),
                        Align2::LEFT_TOP,
                        &row_data.ascii_dump,
                        font_id.clone(),
                        pal.accent_green, // 緑
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_wrap_row_counting_and_click_resolution() {
        // wrap なし（None）では常に 1 視覚行
        let chars: Vec<char> = "1234567890".chars().collect();
        assert_eq!(count_line_visual_rows(&chars, 4, None), 1);

        // wrap 幅 5 桁では 10 文字は 2 行になる
        assert_eq!(count_line_visual_rows(&chars, 4, Some(5)), 2);
        assert_eq!(resolve_wrapped_char_col("1234567890", 4, 5, 0, 2), 2);
        assert_eq!(resolve_wrapped_char_col("1234567890", 4, 5, 1, 0), 5);
        assert_eq!(resolve_wrapped_char_col("1234567890", 4, 5, 1, 3), 8);

        // 全角文字（幅 2）が残り 1 桁に収まらない場合は手前で次行へ折り返す
        let jp_chars: Vec<char> = "aあb".chars().collect();
        assert_eq!(count_line_visual_rows(&jp_chars, 4, Some(2)), 3);
        assert_eq!(resolve_wrapped_char_col("aあb", 4, 2, 1, 0), 1);
        assert_eq!(resolve_wrapped_char_col("aあb", 4, 2, 2, 0), 2);
    }
}
