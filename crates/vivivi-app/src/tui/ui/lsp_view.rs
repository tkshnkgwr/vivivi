//! LSP (Language Server Protocol) TUI 表示コンポーネント
//!
//! ホバー情報 (Hover) のフローティングポップアップ描画および
//! 診断情報 (Diagnostics: エラー・警告) 一覧パネルを描画します。

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use vivivi_modes::lsp::DiagnosticSeverity;

use crate::tui::app::App;

/// ホバー情報フローティングポップアップを描画します。
pub fn render_lsp_hover(frame: &mut Frame, app: &App, size: Rect) {
    let hover_text = match &app.lsp_hover_text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    let popup_width = (size.width * 2 / 3).clamp(30, 80);
    let popup_height = (size.height / 2).clamp(6, 20);

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width.min(size.width),
        height: popup_height.min(size.height),
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" LSP Hover (型・ドキュメント情報) [Esc で閉じる] ")
        .border_style(Style::default().fg(Color::Cyan));

    let lines: Vec<Line> = hover_text
        .lines()
        .map(|l| Line::from(Span::raw(l.to_string())))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

/// 診断情報（Diagnostics: エラー・警告）パネルを描画します。
pub fn render_lsp_diagnostics(frame: &mut Frame, app: &App, size: Rect) {
    if !app.lsp_show_diagnostics_panel {
        return;
    }

    let diags = if let (Some(mgr), Some(path)) = (&app.lsp_manager, &app.file_path) {
        let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
        mgr.get_diagnostics(&uri).unwrap_or(&[])
    } else {
        &[]
    };

    let popup_width = (size.width * 4 / 5).clamp(40, 100);
    let popup_height = (size.height * 2 / 3).clamp(10, 25);

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width.min(size.width),
        height: popup_height.min(size.height),
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" LSP 診断情報 (Diagnostics) [:lsp diag でトグル] ")
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if diags.is_empty() {
        let empty_msg = Paragraph::new("診断情報（エラー・警告）はありません。")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let mut lines = Vec::new();
    for d in diags {
        let (sev_str, sev_color) = match d.severity {
            Some(DiagnosticSeverity::Error) => ("ERROR", Color::Red),
            Some(DiagnosticSeverity::Warning) => ("WARN ", Color::Yellow),
            Some(DiagnosticSeverity::Information) => ("INFO ", Color::Blue),
            Some(DiagnosticSeverity::Hint) => ("HINT ", Color::DarkGray),
            None => ("DIAG ", Color::White),
        };

        let src = d.source.as_deref().unwrap_or("lsp");
        let line_num = d.range.start.line + 1;
        let col_num = d.range.start.character + 1;

        let span1 = Span::styled(
            format!("[{sev_str}] "),
            Style::default().fg(sev_color).add_modifier(Modifier::BOLD),
        );
        let span2 = Span::styled(
            format!("行 {line_num}:{col_num} ({src}): "),
            Style::default().fg(Color::Cyan),
        );
        let span3 = Span::raw(d.message.clone());

        lines.push(Line::from(vec![span1, span2, span3]));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
