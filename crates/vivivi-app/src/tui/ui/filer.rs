//! 2ペイン ファイラー (Two-Pane Filer) 描画モジュール。
//!
//! ViVi 3 / ViVi 4 の伝統的な2画面ファイラーをモデル化し、
//! 左右ペインの並行ディレクトリ表示、選択状態、カーソル移動、および操作ガイドを描画します。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use vivivi_modes::filer::{ActiveFilerPane, FilerPane};

use super::clear_modal_area;
use crate::tui::app::App;

/// 2ペインファイラーの全画面モーダルを描画します。
pub fn render_filer(frame: &mut Frame, app: &App, area: Rect) {
    let Some(ref filer) = app.filer else {
        return;
    };

    // 画面中央に 90% x 85% のモーダル領域を確保
    let popup_area = centered_rect(90, 85, area);

    // 背景を安全にクリア（全角文字の境界重なり防止）
    clear_modal_area(frame, popup_area);

    // 外枠ブロック
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" 📁 2ペイン ファイラー (Two-Pane Filer) ");
    let inner_area = main_block.inner(popup_area);
    frame.render_widget(main_block, popup_area);

    // 上部ペイン領域と下部操作ヘルプバーに縦分割
    let v_chunks = Layout::vertical([
        Constraint::Min(3),    // 左右2ペイン領域
        Constraint::Length(1), // 最下部ヘルプバー
    ])
    .split(inner_area);

    let panes_area = v_chunks[0];
    let help_area = v_chunks[1];

    // 左右ペインを 50% ずつ水平分割
    let h_chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(panes_area);

    let left_area = h_chunks[0];
    let right_area = h_chunks[1];

    let is_left_active = filer.active == ActiveFilerPane::Left;

    // 左ペイン・右ペインの描画
    render_single_pane(frame, &filer.left, left_area, is_left_active, "左ペイン");
    render_single_pane(frame, &filer.right, right_area, !is_left_active, "右ペイン");

    // 最下部ヘルプバーの描画
    let help_spans = vec![
        Span::styled(
            " Tab",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":左右切替 "),
        Span::styled(
            "j/k",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":移動 "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":開く/入る "),
        Span::styled(
            "BS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":親階層 "),
        Span::styled(
            "Space",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":選択 "),
        Span::styled(
            "c",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":コピー "),
        Span::styled(
            "m",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":移動 "),
        Span::styled(
            "d",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":削除 "),
        Span::styled(
            "q/Esc",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(":閉じる"),
    ];
    let help_widget = Paragraph::new(Line::from(help_spans))
        .style(Style::default().bg(Color::Rgb(25, 30, 40)).fg(Color::White));
    frame.render_widget(help_widget, help_area);
}

/// 単一のファイラーペインを描画します。
fn render_single_pane(
    frame: &mut Frame,
    pane: &FilerPane,
    area: Rect,
    is_active: bool,
    label: &str,
) {
    let border_style = if is_active {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let dir_display = pane.current_dir.to_string_lossy();
    let title = if is_active {
        format!(" ▶ [{label}] {dir_display} ◀ ")
    } else {
        format!(" [{label}] {dir_display} ")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let view_height = inner.height as usize;
    let total_entries = pane.entries.len();

    // カーソル位置に合わせてスクロールオフセットを計算
    let scroll_offset = if pane.cursor >= view_height {
        pane.cursor - view_height + 1
    } else {
        0
    };

    let mut lines = Vec::new();

    if total_entries == 0 {
        lines.push(Line::from(Span::styled(
            "  (空のディレクトリ)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, entry) in pane
            .entries
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(view_height)
        {
            let is_cursor = i == pane.cursor;
            let mut spans = Vec::new();

            // 1. カーソルインジケータ
            if is_cursor && is_active {
                spans.push(Span::styled(
                    "▶ ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if is_cursor {
                spans.push(Span::styled("▷ ", Style::default().fg(Color::DarkGray)));
            } else {
                spans.push(Span::raw("  "));
            }

            // 2. 選択チェックボックス
            if entry.is_selected {
                spans.push(Span::styled(
                    "[✓] ",
                    Style::default()
                        .fg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw("    "));
            }

            // 3. アイコン・プレフィックスと名前
            let (name_style, prefix) = if entry.is_dir {
                (
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    "📁 ",
                )
            } else {
                (Style::default().fg(Color::White), "📄 ")
            };

            spans.push(Span::raw(prefix));

            // エントリ名（行幅に合わせてクランプ）
            let max_name_len = (inner.width as usize).saturating_sub(22);
            let truncated_name = if entry.name.chars().count() > max_name_len && max_name_len > 3 {
                let name_prefix: String = entry.name.chars().take(max_name_len - 3).collect();
                format!("{name_prefix}...")
            } else {
                entry.name.clone()
            };

            spans.push(Span::styled(truncated_name, name_style));

            // 4. サイズ表示（ファイルの場合のみ右側にスペースパディングして表示）
            if !entry.is_dir {
                let size_str = format_size(entry.size);
                spans.push(Span::styled(
                    format!("  ({size_str})"),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let mut line = Line::from(spans);
            if is_cursor && is_active {
                line = line.style(Style::default().bg(Color::Rgb(40, 50, 70)));
            }
            lines.push(line);
        }
    }

    let list_widget = Paragraph::new(lines);
    frame.render_widget(list_widget, inner);
}

/// バイトサイズを人間が読みやすい単位（B, KB, MB, GB）に整形します。
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// 画面全体の中央に指定割合で矩形領域を計算します。
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
