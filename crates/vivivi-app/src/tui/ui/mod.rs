//! TUI 画面描画モジュール。
//!
//! 各領域（エディタ、アウトライン、ステータスバー）のレイアウト計算と描画ディスパッチを担当します。

pub mod clipboard_hist;
pub mod editor;
pub mod help;
pub mod outline;
pub mod status;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
};

use super::app::App;
use super::window::SplitAxis;
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

    let main_area = chunks[0];
    let status_area = chunks[1];

    // アウトライン表示が有効な場合は画面左側にサイドペインを配置
    let (outline_area, editor_container) = if app.show_outline {
        let h_chunks = Layout::horizontal([
            Constraint::Length(30), // アウトラインサイドペイン
            Constraint::Min(1),     // エディタメイン領域
        ])
        .split(main_area);
        (Some(h_chunks[0]), h_chunks[1])
    } else {
        (None, main_area)
    };

    // 画面分割 (Split / Vsplit) のレイアウト計算
    let window_count = app.windows.len();
    let window_areas = if window_count <= 1 {
        vec![editor_container]
    } else {
        let constraints = vec![Constraint::Ratio(1, window_count as u32); window_count];
        match app.split_axis {
            SplitAxis::Horizontal => Layout::vertical(constraints)
                .split(editor_container)
                .to_vec(),
            SplitAxis::Vertical => Layout::horizontal(constraints)
                .split(editor_container)
                .to_vec(),
        }
    };

    let active_idx = app.active_window.min(window_areas.len().saturating_sub(1));
    let active_editor_area = window_areas[active_idx];

    // アクティブウィンドウに合わせてスクロール調整
    let line_number_width = 5; // " 123 │" の幅
    let text_view_width = (active_editor_area.width as usize).saturating_sub(line_number_width);
    let text_view_height = active_editor_area.height as usize;
    app.adjust_scroll(text_view_height, text_view_width);

    // 1. 各エディタウィンドウの描画
    for (i, &area) in window_areas.iter().enumerate() {
        let is_active = i == active_idx;
        editor::render_window(frame, app, area, i, is_active);
    }

    // 2. アウトラインペインの描画
    if let Some(o_area) = outline_area {
        outline::render_outline(frame, app, o_area);
    }

    // 3. 最下行（ステータスバーまたはコマンドライン）の描画
    status::render_status_bar(frame, app, status_area);

    // 4. クリップボード履歴ポップアップの描画（表示中のみ最前面に重ねて描画）
    if app.show_clipboard_hist {
        clipboard_hist::render_clipboard_history(frame, app, size);
    }

    // 5. ヘルプビューアの描画（表示中のみ最前面に重ねて描画）
    if app.show_help {
        help::render_help_view(frame, app, size);
    }

    // 6. カーソルの画面配置
    set_terminal_cursor(
        frame,
        app,
        active_editor_area,
        status_area,
        line_number_width,
    );
}

/// ターミナルカーソルの座標設定
fn set_terminal_cursor(
    frame: &mut Frame,
    app: &App,
    editor_area: Rect,
    status_area: Rect,
    line_number_width: usize,
) {
    if app.is_migemo_search {
        // Migemo 検索入力プロンプト (g/<query>)
        let prompt_len = 2 + app.migemo_query.chars().count(); // "g/" + query
        let cursor_x = (status_area.x as usize + prompt_len).min(status_area.width as usize - 1);
        frame.set_cursor_position(Position::new(cursor_x as u16, status_area.y));
        return;
    }

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
