//! 外部コマンド実行ダイアログ (`cmdExecDlg`, ViVi 4 準拠) の TUI 描画モジュール。

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::tui::app::App;
use crate::tui::ui::clear_modal_area;

/// 外部コマンド実行ダイアログを中央モーダルとして描画します。
pub fn render_cmd_exec_modal(frame: &mut Frame, app: &App, area: Rect) {
    let width = (area.width * 70 / 100).max(40).min(area.width);
    let height = (area.height * 50 / 100).max(12).min(area.height);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(x, y, width, height);

    clear_modal_area(frame, modal_area);

    let border_color = Color::Rgb(122, 162, 247); // Tokyo Night Blue
    let title_color = Color::Rgb(224, 175, 104); // Warm Gold

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " 外部コマンド実行ダイアログ (cmdExecDlg) ",
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(Clear, modal_area);
    frame.render_widget(block, modal_area);

    let inner = Rect::new(
        modal_area.x + 2,
        modal_area.y + 1,
        modal_area.width.saturating_sub(4),
        modal_area.height.saturating_sub(2),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // モード選択
            Constraint::Length(3), // 入力欄
            Constraint::Min(3),    // プレビュー / 出力結果
            Constraint::Length(1), // ヘルプ
        ])
        .split(inner);

    // 1. モード選択
    let mode_names = [
        "1. 実行のみ (結果確認)",
        "2. バッファ挿入 (:r!)",
        "3. 選択行置換 (:[range]!)",
    ];
    let mut mode_spans = vec![Span::raw("実行モード [Tab]: ")];
    for (i, name) in mode_names.iter().enumerate() {
        if i == app.cmd_exec_mode {
            mode_spans.push(Span::styled(
                format!(" [{name}] "),
                Style::default()
                    .fg(Color::Rgb(158, 206, 106))
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            mode_spans.push(Span::styled(
                format!("  {name}  "),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }
    frame.render_widget(Paragraph::new(Line::from(mode_spans)), chunks[0]);

    // 2. コマンド入力欄
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(187, 154, 247)))
        .title(Span::styled(
            " コマンド入力 ",
            Style::default().fg(Color::Cyan),
        ));
    let input_text = format!("> {}█", app.cmd_exec_input);
    let input_p = Paragraph::new(input_text)
        .block(input_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(input_p, chunks[1]);

    // 3. プレビュー / 実行結果表示
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " 実行結果 / プレビュー ",
            Style::default().fg(Color::Gray),
        ));
    let preview_text = if let Some(ref out) = app.cmd_exec_output_preview {
        out.clone()
    } else {
        "(まだ実行されていません - Enter で実行します)".to_string()
    };
    let preview_p = Paragraph::new(preview_text)
        .block(preview_block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Rgb(192, 202, 245)));
    frame.render_widget(preview_p, chunks[2]);

    // 4. フッターヘルプ
    let help_line = Line::from(vec![
        Span::styled(
            "[Enter] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("実行  "),
        Span::styled(
            "[Tab] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("モード切替  "),
        Span::styled(
            "[Esc] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("閉じる"),
    ]);
    frame.render_widget(Paragraph::new(help_line), chunks[3]);
}
