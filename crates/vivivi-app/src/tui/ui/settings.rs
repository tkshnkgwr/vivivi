//! TUI 設定ポップアップダイアログ (Preferences Dialog) 描画モジュール。
//!
//! Ratatui による中央フローティングウィンドウを用い、
//! 6大カテゴリの設定項目をキーボード（Viキー）で操作・プレビュー・永続化可能にします。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::clear_modal_area;
use crate::tui::app::App;

/// 画面中央に指定パーセンテージの矩形領域を計算します。
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

/// TUI 設定ダイアログを描画します。
pub fn render_settings_modal(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(76, 76, area);

    // 背景を安全にクリア（全角文字の境界重なり防止）
    clear_modal_area(frame, popup_area);

    let theme_name = &app.config.appearance.theme;
    let title = format!(" 設定 (Preferences) - [{theme_name}] ");

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = main_block.inner(popup_area);
    frame.render_widget(main_block, popup_area);

    // 内部をメイン領域（上部）と操作ヘルプバー（最下部）に分割
    let chunks = Layout::vertical([
        Constraint::Min(1),    // 設定ペイン領域
        Constraint::Length(1), // 操作ガイド
    ])
    .split(inner);

    // 上部を左右 2 カラム（カテゴリ一覧: 24, 詳細項目: 残り幅）に分割
    let col_chunks = Layout::horizontal([
        Constraint::Length(24), // 左: カテゴリツリー
        Constraint::Min(1),     // 右: 詳細設定項目
    ])
    .split(chunks[0]);

    // 1. 左ペイン: カテゴリ一覧の描画
    render_categories_pane(frame, app, col_chunks[0]);

    // 2. 右ペイン: 選択中カテゴリの詳細設定項目描画
    render_items_pane(frame, app, col_chunks[1]);

    // 3. 最下部: 操作ヘルプバー
    render_footer_help(frame, chunks[1]);
}

/// 左ペイン: 6大カテゴリ一覧
fn render_categories_pane(frame: &mut Frame, app: &App, area: Rect) {
    let categories = [
        ("⚙", "1. 一般 (General)"),
        ("🎨", "2. 外観 (Appearance)"),
        ("👁", "3. 不可視 (Invisibles)"),
        ("✏", "4. 編集 (Editing)"),
        ("🚀", "5. ツール (Tools)"),
        ("⌨", "6. キーマップ (Keymap)"),
    ];

    let is_focused = !app.tui_settings_in_items;

    let items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(idx, (icon, name))| {
            let is_selected = idx == app.tui_settings_category;
            let prefix = if is_selected { "▶ " } else { "  " };
            let text = format!("{prefix}{icon} {name}");

            let style = if is_selected && is_focused {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            ListItem::new(Span::styled(text, style))
        })
        .collect();

    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// 右ペイン: 各カテゴリの詳細項目
fn render_items_pane(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.tui_settings_in_items;
    let selected_item = app.tui_settings_item;

    let items_data = get_category_items(app, app.tui_settings_category);

    let list_items: Vec<ListItem> = items_data
        .iter()
        .enumerate()
        .map(|(idx, (label, val, desc))| {
            let is_selected = idx == selected_item;
            let prefix = if is_selected { "● " } else { "  " };

            let row_style = if is_selected && is_focused {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled(prefix, row_style),
                Span::styled(format!("{label:<32}"), row_style),
                Span::styled(
                    format!("{val:<20}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {desc}"), Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(list_items);
    frame.render_widget(list, area);
}

/// カテゴリインデックスに応じた項目リストを取得します: `(ラベル, 現在値, 説明)`
pub fn get_category_items(app: &App, category_idx: usize) -> Vec<(String, String, String)> {
    let cfg = &app.config;
    match category_idx {
        0 => vec![
            (
                "文字エンコーディング".to_string(),
                format!("[ {} ]", cfg.general.default_encoding),
                "utf-8 / sjis / euc-jp".to_string(),
            ),
            (
                "デフォルト改行形式".to_string(),
                format!("[ {} ]", cfg.general.default_fileformat),
                "unix (LF) / dos (CRLF)".to_string(),
            ),
            (
                "クリップボード同期".to_string(),
                format!("[ {} ]", cfg.general.clipboard),
                "none / unnamed / unnamedplus".to_string(),
            ),
            (
                "クラッシュリカバリ".to_string(),
                format!("[{}]", if cfg.general.crash_recovery { "x" } else { " " }),
                "異常終了時自動スワップ保存".to_string(),
            ),
            (
                "自動更新確認".to_string(),
                format!("[{}]", if cfg.general.check_update { "x" } else { " " }),
                "起動時の新版チェック".to_string(),
            ),
        ],
        1 => vec![
            (
                "カラーテーマ".to_string(),
                format!("[ {} ]", cfg.appearance.theme),
                "Spaceキーで切替 (即時反映)".to_string(),
            ),
            (
                "行番号表示 (number)".to_string(),
                format!("[{}]", if app.show_line_numbers { "x" } else { " " }),
                ":set nu / :set nonu".to_string(),
            ),
            (
                "相対行番号 (relativenumber)".to_string(),
                format!("[{}]", if app.relative_number { "x" } else { " " }),
                ":set rnu / :set nornu".to_string(),
            ),
            (
                "行折り返し (wrap)".to_string(),
                format!("[{}]", if app.wrap { "x" } else { " " }),
                ":set wrap / :set nowrap".to_string(),
            ),
            (
                "変更行ガター強調 (gutter)".to_string(),
                format!("[{}]", if app.change_tracker.enabled { "x" } else { " " }),
                "黄:未保存, 緑:保存済み".to_string(),
            ),
            (
                "ミニマップ表示 (GUI)".to_string(),
                format!(
                    "[{}]",
                    if cfg.appearance.gui.show_minimap {
                        "x"
                    } else {
                        " "
                    }
                ),
                ":set minimap".to_string(),
            ),
        ],
        2 => vec![
            (
                "不可視文字表示 (list)".to_string(),
                format!(
                    "[{}]",
                    if app.list_chars_config.enabled {
                        "x"
                    } else {
                        " "
                    }
                ),
                ":set list / :set nolist".to_string(),
            ),
            (
                "全角スペース強調".to_string(),
                format!("[ {} ]", cfg.invisibles.zenkaku_space),
                "全角2マス幅ワインレッド背景".to_string(),
            ),
            (
                "半角スペース記号".to_string(),
                format!("[ {} ]", cfg.invisibles.space),
                ":set listchars=space:·".to_string(),
            ),
            (
                "タブ文字表示記号".to_string(),
                format!("[ {} ]", cfg.invisibles.tab),
                ":set listchars=tab:▸---".to_string(),
            ),
            (
                "改行記号 (EOL)".to_string(),
                format!("[ {} ]", cfg.invisibles.eol),
                ":set listchars=eol:$".to_string(),
            ),
        ],
        3 => vec![
            (
                "タブ幅 (tabstop)".to_string(),
                format!("[ {} マス ]", app.tab_width),
                ":set ts=N (Spaceで+2)".to_string(),
            ),
            (
                "スペース展開 (expandtab)".to_string(),
                format!("[{}]", if cfg.editing.expand_tab { "x" } else { " " }),
                "Tab入力を空白へ展開".to_string(),
            ),
            (
                "罫線保護編集 (keisen)".to_string(),
                format!("[{}]", if cfg.editing.keisen_protect { "x" } else { " " }),
                "枠線内文字伸縮保護".to_string(),
            ),
            (
                "対応括弧の強調表示 (%)".to_string(),
                format!("[{}]", if cfg.editing.match_paren { "x" } else { " " }),
                "対応ペアハイライト".to_string(),
            ),
            (
                "動的補完大文字同一視".to_string(),
                format!(
                    "[{}]",
                    if cfg.editing.dabbrev_ignorecase {
                        "x"
                    } else {
                        " "
                    }
                ),
                "dabbrev 大文字小文字無視".to_string(),
            ),
            (
                "検索大文字同一視 (ic)".to_string(),
                format!("[{}]", if cfg.editing.ignorecase { "x" } else { " " }),
                ":set ignorecase".to_string(),
            ),
            (
                "スマートケース (scs)".to_string(),
                format!("[{}]", if cfg.editing.smartcase { "x" } else { " " }),
                ":set smartcase".to_string(),
            ),
        ],
        4 => vec![
            (
                "文字削除落下演出 (Particles)".to_string(),
                format!(
                    "[{}]",
                    if cfg.appearance.gui.enable_gravity_drop {
                        "x"
                    } else {
                        " "
                    }
                ),
                "OpenViVi 物理落下ギミック".to_string(),
            ),
            (
                "省電力制御 (Reactive Sleep)".to_string(),
                format!(
                    "[{}]",
                    if cfg.appearance.gui.reactive_idle_sleep {
                        "x"
                    } else {
                        " "
                    }
                ),
                "アイドル時 CPU 0% 制御".to_string(),
            ),
            (
                "Migemo ローマ字日本語検索".to_string(),
                "[ 有効 (g/) ]".to_string(),
                "内蔵辞書による高速検索".to_string(),
            ),
            (
                "ダイナミックマクロ".to_string(),
                "[ 有効 (Alt-M) ]".to_string(),
                "直前の反復打鍵を自動再現".to_string(),
            ),
            (
                "字句編集予測 (LexEdit)".to_string(),
                "[ 有効 (Ctrl-J) ]".to_string(),
                "バッファ文脈からの自動予測".to_string(),
            ),
        ],
        _ => vec![
            (
                "Normal: 移動".to_string(),
                "h, j, k, l".to_string(),
                "左, 下, 上, 右 移動".to_string(),
            ),
            (
                "Normal: モード切替".to_string(),
                "i, a, o, v, V, Ctrl-v".to_string(),
                "挿入, 追記, 行追加, 選択".to_string(),
            ),
            (
                "Normal: アンドゥ/リドゥ".to_string(),
                "u / Ctrl-r".to_string(),
                "元に戻す / やり直す".to_string(),
            ),
            (
                "Normal: 設定ダイアログ".to_string(),
                ":preferences / :set".to_string(),
                "本設定パネルを開く".to_string(),
            ),
            (
                "Normal: ファイラー".to_string(),
                ":filer".to_string(),
                "2ペイン・ファイラー起動".to_string(),
            ),
            (
                "Insert: 予測・補完".to_string(),
                "Ctrl-j, Ctrl-;, Ctrl-l".to_string(),
                "LexEdit, Abbrev, Dabbrev".to_string(),
            ),
        ],
    }
}

/// 最下部: 操作ヘルプバー
fn render_footer_help(frame: &mut Frame, area: Rect) {
    let help_line = Line::from(vec![Span::styled(
        " [j/k]:移動  [h/l/Tab]:ペイン切替  [Space]:トグル/変更  [s]:保存して閉じる  [Esc/q]:閉じる ",
        Style::default().fg(Color::Black).bg(Color::Cyan),
    )]);
    let paragraph = Paragraph::new(help_line);
    frame.render_widget(paragraph, area);
}
