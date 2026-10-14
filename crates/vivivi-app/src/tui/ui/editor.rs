//! エディタウィンドウ本体、シンタックスハイライト、選択範囲描画モジュール。

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::App;
use vivivi_vi::ViMode;

/// 個別エディタウィンドウの描画（単一または分割枠線付き）
pub fn render_window(frame: &mut Frame, app: &App, area: Rect, win_idx: usize, is_active: bool) {
    if app.windows.len() > 1 {
        let title = if is_active {
            format!(" [Win {} *Active*] ", win_idx + 1)
        } else {
            format!(" [Win {}] ", win_idx + 1)
        };
        let border_style = if is_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        render_editor(frame, app, inner_area);
    } else {
        render_editor(frame, app, area);
    }
}

/// エディタバッファ（行番号＋テキスト）の描画
pub fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let cur = app.cursor_pos();
    let total_lines = app.buffer.line_count();
    let start_line = app.scroll.0;
    let end_line = (start_line + area.height as usize).min(total_lines);

    // カーソル位置の対応括弧ペアを探索
    let cursor_offset = app.buffer.cursor_to_position(cur);
    let paren_positions =
        vivivi_core::get_cursor_matching_paren(&app.buffer, cursor_offset).map(|(p1, p2)| {
            (
                app.buffer.position_to_cursor(p1),
                app.buffer.position_to_cursor(p2),
            )
        });

    let mut lines_spans = Vec::new();

    // アクティブな検索正規表現 (hlsearch)
    let active_search_re = app.active_search_regex();

    // 変更行マーカーの算出
    let all_lines: Vec<String> = (0..total_lines)
        .filter_map(|i| app.buffer.get_line(i))
        .collect();
    let change_markers = app.change_tracker.compute_markers(&all_lines);

    for line_idx in start_line..end_line {
        let is_current_line = line_idx == cur.line;

        // 行番号のフォーマット
        let line_num_str = format!("{:>4} ", line_idx + 1);
        let num_style = if is_current_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // ガター変更マーカーのスタイル判定
        let marker = change_markers
            .get(line_idx)
            .copied()
            .unwrap_or(vivivi_modes::LineChangeState::Unchanged);
        let (gutter_sym, gutter_style) = match marker {
            vivivi_modes::LineChangeState::Added | vivivi_modes::LineChangeState::Modified => (
                "▌",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            vivivi_modes::LineChangeState::SavedModified => (
                "▌",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            vivivi_modes::LineChangeState::Unchanged => ("│", Style::default().fg(Color::DarkGray)),
        };

        // 行テキストの取得（CSVモード有効時は整列フォーマット適用）
        let raw_line = app.buffer.get_line(line_idx).unwrap_or_default();
        let display_line = if app.is_csv_active {
            app.csv_mode.format_line(&raw_line, is_current_line)
        } else {
            raw_line
        };

        // 不可視文字可視化が有効な場合、専用フォーマットを適用
        let (processed_line, inv_info) = if app.list_chars_config.enabled && !app.is_csv_active {
            let inv_chars =
                vivivi_modes::format_line_invisibles(&display_line, 4, &app.list_chars_config);
            let s: String = inv_chars.iter().map(|c| c.ch).collect();
            (s, Some(inv_chars))
        } else {
            (display_line.clone(), None)
        };

        // 構文解析（シンタックスハイライト）トークンの取得
        let tokens = if app.is_csv_active {
            vec![vivivi_modes::SyntaxToken {
                text: display_line.clone(),
                token_type: vivivi_modes::TokenType::Text,
            }]
        } else {
            vivivi_modes::tokenize_line(&display_line, app.syntax_lang)
        };

        // 各文字のベーススタイルを計算
        let chars: Vec<char> = processed_line.chars().collect();
        let mut styles: Vec<Style> = Vec::with_capacity(chars.len());

        for token in tokens {
            let token_style = match token.token_type {
                vivivi_modes::TokenType::Keyword => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                vivivi_modes::TokenType::Type => Style::default().fg(Color::Yellow),
                vivivi_modes::TokenType::StringLiteral => Style::default().fg(Color::Green),
                vivivi_modes::TokenType::Number => Style::default().fg(Color::Magenta),
                vivivi_modes::TokenType::Comment => Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
                vivivi_modes::TokenType::Punctuation => Style::default().fg(Color::LightCyan),
                vivivi_modes::TokenType::Text => Style::default().fg(Color::White),
            };
            for _ in token.text.chars() {
                styles.push(token_style);
            }
        }

        // 不可視文字で文字数が増減した場合のスタイル長調整
        if let Some(ref inv) = inv_info {
            styles.resize(chars.len(), Style::default().fg(Color::DarkGray));
            for (idx, item) in inv.iter().enumerate() {
                if idx < styles.len() {
                    if item.is_zenkaku_space {
                        styles[idx] = Style::default()
                            .fg(Color::Red)
                            .bg(Color::Rgb(50, 20, 20))
                            .add_modifier(Modifier::BOLD);
                    } else if item.is_invisible {
                        styles[idx] = Style::default().fg(Color::DarkGray);
                    }
                }
            }
        }

        // カレント行の背景色（暗いブルーグレー）
        if is_current_line {
            for s in &mut styles {
                *s = s.bg(Color::Rgb(30, 35, 45));
            }
        }

        // 検索キーワードの一致箇所ハイライト (hlsearch)
        if let Some(ref search_re) = active_search_re {
            let hl_style = Style::default()
                .bg(Color::Rgb(220, 150, 30))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);

            for m in search_re.find_iter(&display_line) {
                let start_col = display_line[..m.start()].chars().count();
                let end_col = display_line[..m.end()].chars().count();
                for col_idx in start_col..end_col {
                    if col_idx < styles.len() {
                        styles[col_idx] = hl_style;
                    }
                }
            }
        }

        // 対応括弧ペアのハイライト（コントラスト背景反転）
        if let Some((pos1, pos2)) = paren_positions {
            let paren_style = Style::default()
                .bg(Color::Rgb(70, 95, 140))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);

            if pos1.line == line_idx && pos1.col < styles.len() {
                styles[pos1.col] = paren_style;
            }
            if pos2.line == line_idx && pos2.col < styles.len() {
                styles[pos2.col] = paren_style;
            }
        }

        // ビジュアル選択範囲（文字/行/矩形ブロック）のハイライト
        if let Some(anchor) = app.cursor.anchor {
            let mode = app.mode();
            let min_line = anchor.line.min(cur.line);
            let max_line = anchor.line.max(cur.line);

            if line_idx >= min_line && line_idx <= max_line {
                let sel_style = Style::default()
                    .bg(Color::Rgb(55, 75, 115))
                    .fg(Color::White);

                match mode {
                    ViMode::VisualBlock => {
                        let min_col = anchor.col.min(cur.col);
                        let max_col = anchor.col.max(cur.col);
                        for (col_idx, st) in styles.iter_mut().enumerate() {
                            if col_idx >= min_col && col_idx <= max_col {
                                *st = sel_style;
                            }
                        }
                    }
                    ViMode::VisualLine => {
                        styles.fill(sel_style);
                    }
                    ViMode::Visual => {
                        for (col_idx, st) in styles.iter_mut().enumerate() {
                            let in_selection = if min_line == max_line {
                                let min_col = anchor.col.min(cur.col);
                                let max_col = anchor.col.max(cur.col);
                                col_idx >= min_col && col_idx <= max_col
                            } else if line_idx == min_line {
                                let start_col = if anchor.line <= cur.line {
                                    anchor.col
                                } else {
                                    cur.col
                                };
                                col_idx >= start_col
                            } else if line_idx == max_line {
                                let end_col = if anchor.line >= cur.line {
                                    anchor.col
                                } else {
                                    cur.col
                                };
                                col_idx <= end_col
                            } else {
                                true
                            };
                            if in_selection {
                                *st = sel_style;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // 行スパンの構築（水平スクロール適用）
        let base_bg = if is_current_line {
            Style::default().bg(Color::Rgb(30, 35, 45))
        } else {
            Style::default()
        };

        let mut row_spans = vec![
            Span::styled(line_num_str, num_style),
            Span::styled(gutter_sym, gutter_style),
            Span::styled(" ", base_bg),
        ];

        let scroll_x = app.scroll.1;
        if scroll_x < chars.len() {
            let mut cur_chunk = String::new();
            let mut cur_style = styles[scroll_x];

            for i in scroll_x..chars.len() {
                let ch = chars[i];
                let st = styles[i];
                if st == cur_style {
                    cur_chunk.push(ch);
                } else {
                    if !cur_chunk.is_empty() {
                        row_spans.push(Span::styled(cur_chunk.clone(), cur_style));
                        cur_chunk.clear();
                    }
                    cur_chunk.push(ch);
                    cur_style = st;
                }
            }
            if !cur_chunk.is_empty() {
                row_spans.push(Span::styled(cur_chunk, cur_style));
            }
        }

        lines_spans.push(Line::from(row_spans));
    }

    // 末尾行以降の ~ チルダ表示（Vim伝統スタイル）
    while lines_spans.len() < area.height as usize {
        lines_spans.push(Line::from(Span::styled(
            "~",
            Style::default().fg(Color::Blue),
        )));
    }

    let paragraph = Paragraph::new(lines_spans);
    frame.render_widget(paragraph, area);
}
