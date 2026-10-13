use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::app::App;
use vivivi_vi::ViMode;

/// TUI の画面描画エントリーポイント。
pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // 画面をエディタ領域（上部）とステータスバー／コマンドライン（最下部）に分割
    let chunks = Layout::vertical([
        Constraint::Min(1),    // エディタ領域
        Constraint::Length(1), // ステータスライン / コマンドライン
    ])
    .split(size);

    let editor_area = chunks[0];
    let status_area = chunks[1];

    // エディタ領域の高さ・幅に合わせてスクロール調整
    let line_number_width = 5; // " 123 │" の幅
    let text_view_width = (editor_area.width as usize).saturating_sub(line_number_width);
    let text_view_height = editor_area.height as usize;
    app.adjust_scroll(text_view_height, text_view_width);

    // 1. エディタバッファの描画
    render_editor(frame, app, editor_area);

    // 2. 最下行（ステータスバーまたはコマンドライン）の描画
    render_status_bar(frame, app, status_area);

    // 3. カーソルの画面配置
    set_terminal_cursor(frame, app, editor_area, status_area, line_number_width);
}

/// エディタバッファ（行番号＋テキスト）の描画
fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
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

    for line_idx in start_line..end_line {
        let is_current_line = line_idx == cur.line;

        // 行番号のフォーマット
        let line_num_str = format!("{:>4} │", line_idx + 1);
        let num_style = if is_current_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // 行テキストの取得（CSVモード有効時は整列フォーマット適用）
        let raw_line = app.buffer.get_line(line_idx).unwrap_or_default();
        let display_line = if app.is_csv_active {
            app.csv_mode.format_line(&raw_line, is_current_line)
        } else {
            raw_line
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
        let chars: Vec<char> = display_line.chars().collect();
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

        // カレント行の背景色（暗いブルーグレー）
        if is_current_line {
            for s in &mut styles {
                *s = s.bg(Color::Rgb(30, 35, 45));
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

        // 行スパンの構築（水平スクロール適用）
        let base_bg = if is_current_line {
            Style::default().bg(Color::Rgb(30, 35, 45))
        } else {
            Style::default()
        };

        let mut row_spans = vec![
            Span::styled(line_num_str, num_style),
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

/// 最下行ステータスバー・コマンドラインの描画
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mode = app.mode();

    if mode == ViMode::CommandLine {
        // コマンドライン入力中の表示
        let (prefix, cmd_text) = app
            .vi
            .parser()
            .command_line_buffer()
            .map(|(&p, s)| (p, s))
            .unwrap_or((':', ""));
        let cmd_line = format!("{prefix}{cmd_text}");

        let paragraph = Paragraph::new(Line::from(Span::styled(
            cmd_line,
            Style::default().fg(Color::White),
        )));
        frame.render_widget(paragraph, area);
        return;
    }

    // 通常のステータスライン表示
    let mode_str = match mode {
        ViMode::Normal => "-- NORMAL --",
        ViMode::Insert => "-- INSERT --",
        ViMode::Replace => "-- REPLACE --",
        ViMode::Visual => "-- VISUAL --",
        ViMode::VisualLine => "-- VISUAL LINE --",
        ViMode::VisualBlock => "-- VISUAL BLOCK --",
        ViMode::CommandLine => "",
    };

    let keisen_badge = if app.keisen_mode.is_active {
        " [KEISEN]"
    } else {
        ""
    };

    let csv_badge = if app.is_csv_active { " [CSV]" } else { "" };

    let modified_badge = if app.buffer.is_modified() { " [+]" } else { "" };

    let file_name = app
        .file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[No Name]");

    let cur = app.cursor_pos();
    let total_lines = app.buffer.line_count();
    let percent = ((cur.line + 1) * 100)
        .checked_div(total_lines)
        .unwrap_or(100);

    let lang_str = match app.syntax_lang {
        vivivi_modes::SyntaxLang::PlainText => "Text",
        vivivi_modes::SyntaxLang::Rust => "Rust",
        vivivi_modes::SyntaxLang::Cpp => "C/C++",
        vivivi_modes::SyntaxLang::Python => "Python",
        vivivi_modes::SyntaxLang::JavaScript => "JS/TS",
        vivivi_modes::SyntaxLang::Json => "JSON",
        vivivi_modes::SyntaxLang::Markdown => "Markdown",
    };

    let enc_info = format!(
        "[{}/{}/{}]",
        app.encoding.name(),
        app.newline.name(),
        lang_str
    );

    let left_info = format!("{mode_str}{keisen_badge}{csv_badge} {file_name}{modified_badge}");
    let right_info = format!("{enc_info} {}:{} | {percent}% ", cur.line + 1, cur.col + 1);

    // 一時通知メッセージがある場合は上書き表示
    let display_left = if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        left_info
    };

    let width = area.width as usize;
    let spaces_needed = width
        .saturating_sub(display_left.chars().count())
        .saturating_sub(right_info.chars().count());
    let status_line_content = format!(
        "{}{}{}",
        display_left,
        " ".repeat(spaces_needed),
        right_info
    );

    let status_style = Style::default()
        .bg(Color::Rgb(40, 44, 52))
        .fg(Color::Rgb(220, 223, 228))
        .add_modifier(Modifier::BOLD);

    let paragraph = Paragraph::new(Line::from(Span::styled(status_line_content, status_style)));
    frame.render_widget(paragraph, area);
}

/// ターミナルカーソルの座標設定
fn set_terminal_cursor(
    frame: &mut Frame,
    app: &App,
    editor_area: Rect,
    status_area: Rect,
    line_number_width: usize,
) {
    if app.mode() == ViMode::CommandLine {
        // コマンドラインのカーソル位置
        let cmd_len = app
            .vi
            .parser()
            .command_line_buffer()
            .map(|(_, s)| s.chars().count())
            .unwrap_or(0)
            + 1; // +1 for ':'
        let cursor_x = (status_area.x as usize + cmd_len).min(status_area.width as usize - 1);
        frame.set_cursor_position(Position::new(cursor_x as u16, status_area.y));
        return;
    }

    let cur = app.cursor_pos();
    let screen_row = cur.line.saturating_sub(app.scroll.0);
    let screen_col = cur.col.saturating_sub(app.scroll.1);

    if screen_row < editor_area.height as usize {
        let cursor_x = editor_area.x as usize + line_number_width + 1 + screen_col;
        let cursor_y = editor_area.y as usize + screen_row;

        if cursor_x < (editor_area.x + editor_area.width) as usize {
            frame.set_cursor_position(Position::new(cursor_x as u16, cursor_y as u16));
        }
    }
}
