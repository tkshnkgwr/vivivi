//! エディタ画面骨格レイアウト ＆ 共通 App コア接続・日本語 IME 統合モジュール（ステップ 2, 3, 4）。
//!
//! `eframe::App` のライフサイクル内で、Tokyo Night を基調としたレイアウトを構築し、
//! `egui` 標準の `TextEdit` を一切使用せず、`ui.painter()` による等幅グリッド描画、
//! OS ネイティブの日本語 IME インライン入力・座標通知、および削除文字落下アニメーションを実現します。

use eframe::egui;
use egui::{
    Align2, Color32, CornerRadius, Event, FontId, Frame, ImeEvent, Key, Margin, Modifiers, Pos2,
    Rect, Sense, Stroke, Vec2,
};
use std::path::{Path, PathBuf};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use vivivi_app::{App, AppKeyEvent, KeyCode, KeyModifiers, LogicalRect};
use vivivi_modes::{LineChangeState, TokenType};
use vivivi_vi::ViMode;

use crate::particles::{ParticleManager, extract_deleted_line_segment};
use crate::theme::colors;

/// 左サイドバーの表示タブ種別。
///
/// # Examples
/// ```
/// use vivivi_gui::SidebarTab;
///
/// let tab = SidebarTab::Files;
/// assert_eq!(tab, SidebarTab::Files);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    /// プロジェクト・ファイルツリー
    Files,
    /// シンボル・見出しアウトライン
    Outline,
}

/// `vivivi-gui` デスクトップアプリケーションの全体状態。
///
/// `vivivi-app` の共通ヘッドレスコア（[`App`]）を内包し、サイドバーやミニマップの開閉状態、
/// 日本語 IME のプリエディット文字列、および削除文字パーティクル物理エンジンを管理します。
///
/// # Examples
/// ```
/// use vivivi_app::App;
/// use vivivi_core::{CharEncoding, NewlineType};
/// use vivivi_gui::ViviviGuiApp;
///
/// let core = App::new("fn main() {}", None, CharEncoding::Utf8, NewlineType::Lf);
/// let gui = ViviviGuiApp::from_core(core);
/// assert!(gui.sidebar_open);
/// assert!(gui.minimap_open);
/// ```
pub struct ViviviGuiApp {
    /// エディタ基幹セッション層（App）
    pub core: App,
    /// サイドバー開閉フラグ
    pub sidebar_open: bool,
    /// サイドバーの選択タブ
    pub sidebar_tab: SidebarTab,
    /// サイドバーの幅
    pub sidebar_width: f32,
    /// ミニマップ表示フラグ
    pub minimap_open: bool,
    /// ミニマップの幅
    pub minimap_width: f32,
    /// ファイルツリーが現在参照しているディレクトリパス
    pub current_dir: PathBuf,
    /// ファイルツリー用キャッシュ（`current_dir` 直下の項目）
    pub cached_files: Vec<PathBuf>,
    /// 日本語 IME が現在有効（変換セッション中）かどうか
    pub ime_enabled: bool,
    /// 日本語 IME の未確定文字列（Preedit）
    pub ime_preedit: String,
    /// 直近フレームで計算されたアクティブカーソルのスクリーン矩形（IME 座標通知用）
    pub last_cursor_rect: Option<Rect>,
    /// エディタのフォントサイズ（pt）
    pub font_size: f32,
    /// エディタの行高（px）
    pub line_height: f32,
    /// 削除文字の落下アニメーション用パーティクルマネージャ（ステップ 4）
    pub particles: ParticleManager,
    /// 直近フレームにおけるアクティブウィンドウ本文領域の左上スクリーン座標
    pub last_active_text_origin: Pos2,
    /// 直近フレームにおける等幅半角1文字のピクセル幅
    pub last_char_width: f32,
}

impl ViviviGuiApp {
    /// 指定されたディレクトリ直下のファイル・フォルダ一覧を読み込んでソートします。
    fn read_directory_entries(dir: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(dir)
            .map(|read_dir| {
                let mut entries: Vec<PathBuf> =
                    read_dir.filter_map(|e| e.ok().map(|e| e.path())).collect();
                entries.sort_by(|a, b| {
                    let a_is_dir = a.is_dir();
                    let b_is_dir = b.is_dir();
                    if a_is_dir != b_is_dir {
                        b_is_dir.cmp(&a_is_dir)
                    } else {
                        a.cmp(b)
                    }
                });
                entries
            })
            .unwrap_or_default()
    }

    /// 共通 [`App`] コアインスタンスから [`ViviviGuiApp`] を生成します。
    ///
    /// `eframe::CreationContext` を必要としないため、ヘッドレスの結合テストや初期化処理から直接呼び出せます。
    ///
    /// # Examples
    /// ```
    /// use vivivi_app::App;
    /// use vivivi_core::{CharEncoding, NewlineType};
    /// use vivivi_gui::ViviviGuiApp;
    ///
    /// let core = App::new("hello", None, CharEncoding::Utf8, NewlineType::Lf);
    /// let gui = ViviviGuiApp::from_core(core);
    /// assert_eq!(gui.core.buffer.to_string(), "hello");
    /// ```
    pub fn from_core(core: App) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let cached_files = Self::read_directory_entries(&current_dir);

        Self {
            core,
            sidebar_open: true,
            sidebar_tab: SidebarTab::Files,
            sidebar_width: 220.0,
            minimap_open: true,
            minimap_width: 64.0,
            current_dir,
            cached_files,
            ime_enabled: false,
            ime_preedit: String::new(),
            last_cursor_rect: None,
            font_size: 15.0,
            line_height: 21.0,
            particles: ParticleManager::new(),
            last_active_text_origin: Pos2::new(56.0, 56.0),
            last_char_width: 9.0,
        }
    }

    /// `eframe::run_native` の初期化コールバックから [`ViviviGuiApp`] を生成します。
    pub fn new(core: App, _cc: &eframe::CreationContext<'_>) -> Self {
        Self::from_core(core)
    }

    /// サイドバーのファイルツリーが参照するディレクトリを変更し、エントリ一覧を再読み込みします。
    pub fn navigate_sidebar_dir(&mut self, target_dir: PathBuf) {
        if target_dir.is_dir() {
            self.cached_files = Self::read_directory_entries(&target_dir);
            self.current_dir = target_dir;
        }
    }

    /// 現在のモードに応じたステータスバッジの背景色と文字色を取得します。
    fn mode_badge_color(&self, mode: ViMode) -> (Color32, Color32) {
        match mode {
            ViMode::Normal => (colors::ACCENT_BLUE, Color32::from_rgb(16, 20, 36)),
            ViMode::Insert => (colors::ACCENT_GREEN, Color32::from_rgb(16, 32, 16)),
            ViMode::Visual | ViMode::VisualLine | ViMode::VisualBlock => {
                (colors::ACCENT_PURPLE, Color32::from_rgb(26, 16, 36))
            }
            ViMode::CommandLine => (colors::ACCENT_YELLOW, Color32::from_rgb(36, 28, 12)),
            ViMode::Replace => (colors::ACCENT_RED, Color32::WHITE),
        }
    }

    /// シンタックストークン種別に応じた Tokyo Night 文字色を返します。
    fn syntax_token_color(token_type: TokenType) -> Color32 {
        match token_type {
            TokenType::Keyword => colors::ACCENT_PURPLE,
            TokenType::Type => colors::ACCENT_CYAN,
            TokenType::StringLiteral => colors::ACCENT_GREEN,
            TokenType::Number => colors::ACCENT_YELLOW,
            TokenType::Comment => colors::FG_DIMMED,
            TokenType::Punctuation => colors::ACCENT_BLUE,
            TokenType::Text => colors::FG_TEXT,
        }
    }

    /// キーイベントを共通 `App` コアへ転送し、文字削除が発生した場合は
    /// 該当グリフのスクリーン座標から落下アニメーション用パーティクルを生成します（ステップ 4）。
    fn dispatch_key_with_particles(&mut self, key: AppKeyEvent) {
        if !self.particles.enabled {
            self.core.handle_key(key);
            return;
        }

        let prev_len = self.core.buffer.len();
        let prev_line_count = self.core.buffer.line_count();
        let prev_cur_line = self.core.cursor.pos.line;
        let prev_line_str = self.core.buffer.get_line(prev_cur_line).unwrap_or_default();

        self.core.handle_key(key);

        let new_len = self.core.buffer.len();
        if new_len >= prev_len {
            return;
        }

        let vis_row = prev_cur_line.saturating_sub(self.core.scroll.0);
        let row_y = self.last_active_text_origin.y + (vis_row as f32) * self.line_height;
        let scroll_col = self.core.scroll.1;
        let tab_width = self.core.tab_width.max(1);

        if self.core.buffer.line_count() == prev_line_count {
            // 同一行内の文字削除 (x, X, dw, D, Backspace, Delete 等)
            let new_line_str = self.core.buffer.get_line(prev_cur_line).unwrap_or_default();
            if let Some((start_char_idx, deleted_chars)) =
                extract_deleted_line_segment(&prev_line_str, &new_line_str)
            {
                let mut vis_col: usize = 0;
                for (idx, ch) in prev_line_str.chars().enumerate() {
                    let ch_w = if ch == '\t' {
                        tab_width - (vis_col % tab_width)
                    } else {
                        UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
                    };
                    if idx >= start_char_idx && idx < start_char_idx + deleted_chars.len() {
                        let char_x = self.last_active_text_origin.x
                            + (vis_col.saturating_sub(scroll_col) as f32) * self.last_char_width;
                        self.particles
                            .spawn_char(ch, Pos2::new(char_x, row_y), colors::ACCENT_RED);
                    }
                    vis_col += ch_w;
                }
            }
        } else {
            // 行削除 (dd, dj 等) または改行結合時の落下パーティクル生成
            let mut vis_col: usize = 0;
            for ch in prev_line_str.chars().take(64) {
                let ch_w = if ch == '\t' {
                    tab_width - (vis_col % tab_width)
                } else {
                    UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
                };
                let char_x = self.last_active_text_origin.x
                    + (vis_col.saturating_sub(scroll_col) as f32) * self.last_char_width;
                self.particles
                    .spawn_char(ch, Pos2::new(char_x, row_y), colors::ACCENT_RED);
                vis_col += ch_w;
            }
        }
    }

    /// `egui` の入力イベント列（`Event::Key`, `Event::Text`, `Event::Ime`, `Event::Paste`）を解析し、
    /// 共通 [`App`] コア層の入力ハンドラへ流し込みます。
    ///
    /// 日本語 IME の未確定文字列（`ImeEvent::Preedit`）が存在する間は、OS 側の変換操作と競合する
    /// `Backspace` / `Enter` / `Escape` / 矢印キーを自動的にブロックして背後バッファを保護します。
    ///
    /// # Examples
    /// ```
    /// use egui::{Event, Modifiers};
    /// use vivivi_app::App;
    /// use vivivi_core::{CharEncoding, NewlineType};
    /// use vivivi_gui::ViviviGuiApp;
    /// use vivivi_vi::ViMode;
    ///
    /// let core = App::new("abc", None, CharEncoding::Utf8, NewlineType::Lf);
    /// let mut gui = ViviviGuiApp::from_core(core);
    /// gui.process_egui_events(&[Event::Text("i".to_string())], Modifiers::NONE);
    /// assert_eq!(gui.core.mode(), ViMode::Insert);
    /// ```
    pub fn process_egui_events(&mut self, events: &[Event], current_modifiers: Modifiers) {
        for event in events {
            match event {
                // 1. 日本語 IME イベントの処理
                Event::Ime(ime_event) => match ime_event {
                    ImeEvent::Enabled => {
                        self.ime_enabled = true;
                    }
                    ImeEvent::Preedit(text) => {
                        if text != "\n" && text != "\r" {
                            self.ime_enabled = true;
                            self.ime_preedit = text.clone();
                        }
                    }
                    ImeEvent::Commit(text) => {
                        self.ime_preedit.clear();
                        self.ime_enabled = false;
                        for ch in text.chars() {
                            if ch != '\r' && ch != '\n' {
                                self.dispatch_key_with_particles(AppKeyEvent::char(ch));
                            }
                        }
                    }
                    ImeEvent::Disabled => {
                        self.ime_preedit.clear();
                        self.ime_enabled = false;
                    }
                },

                // 2. 確定テキスト入力イベント（通常キー打鍵）
                Event::Text(text) => {
                    // IME 変換中（未確定文字列が存在する間）または Ctrl/Alt 押下時は Text イベントを無視
                    let is_ctrl_or_alt = current_modifiers.ctrl
                        || current_modifiers.alt
                        || current_modifiers.command;
                    if !self.ime_preedit.is_empty() || is_ctrl_or_alt {
                        continue;
                    }

                    let shift_mod = if current_modifiers.shift {
                        KeyModifiers::SHIFT
                    } else {
                        KeyModifiers::NONE
                    };

                    for ch in text.chars() {
                        if ch != '\r' && ch != '\n' && ch != '\t' && !ch.is_control() {
                            let mods = if ch.is_ascii_alphabetic() {
                                shift_mod
                            } else {
                                KeyModifiers::NONE
                            };
                            self.dispatch_key_with_particles(AppKeyEvent::new(
                                KeyCode::Char(ch),
                                mods,
                            ));
                        }
                    }
                }

                // 3. ペースト（Ctrl+V 等）イベント
                Event::Paste(text) => {
                    if self.core.mode() == ViMode::Normal && !self.core.is_migemo_search {
                        // Vi ノーマルモードでの Ctrl+V は矩形ビジュアル選択（VisualBlock）を起動
                        self.dispatch_key_with_particles(AppKeyEvent::new(
                            KeyCode::Char('v'),
                            KeyModifiers::CONTROL,
                        ));
                    } else {
                        for ch in text.chars() {
                            if ch == '\n' {
                                self.dispatch_key_with_particles(AppKeyEvent::new(
                                    KeyCode::Enter,
                                    KeyModifiers::NONE,
                                ));
                            } else if ch != '\r' {
                                self.dispatch_key_with_particles(AppKeyEvent::char(ch));
                            }
                        }
                    }
                }

                // 4. 物理・論理キー押下イベント（特殊キーおよび Ctrl/Alt ショートカット）
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    // IME 未確定文字列の変換中は、OS 側の IME 操作と競合する編集キーを無視
                    if !self.ime_preedit.is_empty()
                        && matches!(
                            key,
                            Key::Backspace
                                | Key::Enter
                                | Key::Escape
                                | Key::Space
                                | Key::ArrowLeft
                                | Key::ArrowRight
                                | Key::ArrowUp
                                | Key::ArrowDown
                        )
                    {
                        continue;
                    }

                    if let Some(app_key) = egui_key_to_app_key(*key, *modifiers) {
                        self.dispatch_key_with_particles(app_key);
                    }
                }

                // 5. マウスホイールによるスクロール
                Event::MouseWheel { delta, .. } => {
                    if delta.y < -0.1 {
                        let lines = ((-delta.y).ceil() as usize).clamp(1, 5);
                        let max_line = self.core.buffer.line_count().saturating_sub(1);
                        self.core.scroll.0 = (self.core.scroll.0 + lines).min(max_line);
                        if self.core.cursor.pos.line < self.core.scroll.0 {
                            self.core.cursor.pos.line = self.core.scroll.0;
                            self.core.clamp_cursor();
                        }
                    } else if delta.y > 0.1 {
                        let lines = (delta.y.ceil() as usize).clamp(1, 5);
                        self.core.scroll.0 = self.core.scroll.0.saturating_sub(lines);
                    }
                }

                _ => {}
            }
        }

        // ノーマルモードへ復帰した際は IME の未確定状態を確実にクリア
        if self.core.mode() == ViMode::Normal && !self.core.is_migemo_search {
            self.ime_preedit.clear();
            self.ime_enabled = false;
        }
    }

    /// 1. メニューバー ＆ マルチタブバーの描画
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        self.core.sync_active_tab_state();

        egui::TopBottomPanel::top("top_panel")
            .frame(Frame::NONE.fill(colors::BG_PANEL))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.hovered.bg_fill = colors::BG_HOVER;

                    ui.menu_button("File", |ui| {
                        if ui.button("New Tab  (:tabnew)").clicked() {
                            self.core.new_tab(None);
                            ui.close_menu();
                        }
                        if ui.button("Save  (:w)").clicked() {
                            self.core.execute_ex("w");
                            ui.close_menu();
                        }
                        if ui.button("Recent Files  (:recent)").clicked() {
                            self.core.execute_ex("recent");
                            ui.close_menu();
                        }
                        if ui.button("2-Pane Filer  (:filer)").clicked() {
                            self.core.execute_ex("filer");
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Close Tab  (:tabclose)").clicked() {
                            let _ = self.core.close_tab();
                            ui.close_menu();
                        }
                        if ui.button("Quit  (:q)").clicked() {
                            self.core.execute_ex("q");
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo  (u)").clicked() {
                            self.core.execute_ex("undo");
                            ui.close_menu();
                        }
                        if ui.button("Redo  (Ctrl-R)").clicked() {
                            self.core.execute_ex("redo");
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Toggle Keisen Mode  (F5)").clicked() {
                            self.core.keisen_mode.toggle();
                            ui.close_menu();
                        }
                        if ui.button("Toggle CSV Mode  (:csv)").clicked() {
                            self.core.execute_ex("csv");
                            ui.close_menu();
                        }
                        if ui.button("Clipboard History  (F6)").clicked() {
                            self.core.show_clipboard_hist = !self.core.show_clipboard_hist;
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("View", |ui| {
                        if ui.checkbox(&mut self.sidebar_open, "Sidebar").clicked() {
                            ui.close_menu();
                        }
                        if ui.checkbox(&mut self.minimap_open, "Minimap").clicked() {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(&mut self.core.show_line_numbers, "Line Numbers")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(&mut self.core.relative_number, "Relative Line Numbers")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(
                                &mut self.core.list_chars_config.enabled,
                                "Invisible Chars (:set list)",
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(&mut self.particles.enabled, "Delete Particles")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Window", |ui| {
                        if ui.button("Split Horizontal  (:sp)").clicked() {
                            self.core.execute_ex("sp");
                            ui.close_menu();
                        }
                        if ui.button("Split Vertical  (:vsp)").clicked() {
                            self.core.execute_ex("vsp");
                            ui.close_menu();
                        }
                        if ui.button("Close Window  (:close)").clicked() {
                            self.core.execute_ex("close");
                            ui.close_menu();
                        }
                        if ui.button("Only Window  (:only)").clicked() {
                            self.core.execute_ex("only");
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Help", |ui| {
                        if ui.button("Vi Cheatsheet  (:help)").clicked() {
                            self.core.execute_ex("help");
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label("vivivi - Vi-like Editor in Rust");
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let minimap_label = if self.minimap_open {
                            "🗺 Minimap"
                        } else {
                            "🗺 (Off)"
                        };
                        if ui
                            .selectable_label(self.minimap_open, minimap_label)
                            .clicked()
                        {
                            self.minimap_open = !self.minimap_open;
                        }

                        let sidebar_label = if self.sidebar_open {
                            "📁 Sidebar"
                        } else {
                            "📁 (Off)"
                        };
                        if ui
                            .selectable_label(self.sidebar_open, sidebar_label)
                            .clicked()
                        {
                            self.sidebar_open = !self.sidebar_open;
                        }
                    });
                });

                ui.separator();

                // 下段: マルチタブバー
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

                    let active_idx = self.core.active_tab;
                    let tab_count = self.core.tabs.len();
                    let mut tab_to_close = None;
                    let mut tab_to_switch = None;

                    for (idx, tab) in self.core.tabs.iter().enumerate() {
                        let is_active = idx == active_idx;

                        let file_name = tab
                            .file_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("[No Name]");

                        let dirty_badge = if tab.buffer.is_modified() { " •" } else { "" };
                        let tab_label = format!("{}{}", file_name, dirty_badge);

                        let bg_color = if is_active {
                            colors::BG_EDITOR
                        } else {
                            colors::BG_PANEL
                        };

                        let text_color = if is_active {
                            colors::FG_TEXT
                        } else {
                            colors::FG_DIMMED
                        };

                        Frame::NONE
                            .fill(bg_color)
                            .corner_radius(CornerRadius {
                                nw: 4,
                                ne: 4,
                                sw: 0,
                                se: 0,
                            })
                            .inner_margin(Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let btn = ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(tab_label)
                                                .color(text_color)
                                                .strong(),
                                        )
                                        .sense(Sense::click()),
                                    );

                                    if btn.clicked() {
                                        tab_to_switch = Some(idx);
                                    }

                                    if tab_count > 1 {
                                        let close_btn = ui.add(
                                            egui::Label::new(
                                                egui::RichText::new("×")
                                                    .color(colors::FG_DIMMED)
                                                    .size(12.0),
                                            )
                                            .sense(Sense::click()),
                                        );
                                        if close_btn.clicked() {
                                            tab_to_close = Some(idx);
                                        }
                                    }
                                });
                            });
                    }

                    let add_tab_btn = ui.add(
                        egui::Button::new(egui::RichText::new(" + ").color(colors::FG_DIMMED))
                            .frame(false),
                    );
                    if add_tab_btn.clicked() {
                        self.core.new_tab(None);
                    }

                    if let Some(idx) = tab_to_switch {
                        self.core.switch_tab_1based(idx + 1);
                    }
                    if let Some(idx) = tab_to_close {
                        if idx == self.core.active_tab {
                            let _ = self.core.close_tab();
                        } else if tab_count > 1 {
                            self.core.tabs.remove(idx);
                            if self.core.active_tab > idx {
                                self.core.active_tab -= 1;
                            }
                        }
                    }
                });
            });
    }

    /// 2. ファイルツリー / アウトライン用折りたたみサイドバーの描画
    fn render_sidebar(&mut self, ctx: &egui::Context) {
        if !self.sidebar_open {
            return;
        }

        egui::SidePanel::left("sidebar_panel")
            .frame(
                Frame::NONE
                    .fill(colors::BG_PANEL)
                    .inner_margin(Margin::same(6)),
            )
            .default_width(self.sidebar_width)
            .width_range(160.0..=450.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Files, " 📁 Files ");
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Outline, " 🔖 Outline ");
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| match self.sidebar_tab {
                    SidebarTab::Files => {
                        ui.label(
                            egui::RichText::new("PROJECT EXPLORER")
                                .color(colors::FG_DIMMED)
                                .size(11.0),
                        );
                        ui.add_space(4.0);

                        let current_dir_name = self
                            .current_dir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("workspace");

                        ui.label(
                            egui::RichText::new(format!("📂 {}", current_dir_name))
                                .strong()
                                .color(colors::FG_TEXT),
                        );

                        let mut dir_to_navigate: Option<PathBuf> = None;
                        let mut file_to_open: Option<String> = None;

                        // 親ディレクトリ (..) への移動リンク
                        if let Some(parent) = self.current_dir.parent() {
                            let parent_path = parent.to_path_buf();
                            let up_label = ui.add(
                                egui::Label::new(
                                    egui::RichText::new("  ⬆ ..").color(colors::ACCENT_BLUE),
                                )
                                .sense(Sense::click()),
                            );
                            if up_label.clicked() {
                                dir_to_navigate = Some(parent_path);
                            }
                        }

                        for path in &self.cached_files {
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            let is_dir = path.is_dir();
                            let icon = if is_dir { "📁" } else { "📄" };
                            let label_text = format!("  {} {}", icon, name);

                            let label = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(label_text).color(colors::FG_TEXT),
                                )
                                .sense(Sense::click()),
                            );

                            if label.double_clicked() || (is_dir && label.clicked()) {
                                if is_dir {
                                    dir_to_navigate = Some(path.clone());
                                } else {
                                    file_to_open = Some(path.to_string_lossy().to_string());
                                }
                            }
                        }

                        if let Some(target_dir) = dir_to_navigate {
                            self.navigate_sidebar_dir(target_dir);
                        }
                        if let Some(path_str) = file_to_open {
                            self.core.execute_ex(&format!("e {}", path_str));
                        }
                    }
                    SidebarTab::Outline => {
                        ui.label(
                            egui::RichText::new("SYMBOLS / OUTLINE")
                                .color(colors::FG_DIMMED)
                                .size(11.0),
                        );
                        ui.add_space(4.0);

                        if self.core.outline_items.is_empty() {
                            ui.label(
                                egui::RichText::new("(No symbols detected)")
                                    .color(colors::FG_DIMMED),
                            );
                        } else {
                            let mut jump_line = None;
                            for item in &self.core.outline_items {
                                let indent = "  ".repeat(item.level.saturating_sub(1));
                                let symbol_label =
                                    format!("{}L{}: {}", indent, item.line_idx + 1, item.title);
                                let label = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(symbol_label).color(colors::FG_TEXT),
                                    )
                                    .sense(Sense::click()),
                                );
                                if label.clicked() {
                                    jump_line = Some(item.line_idx);
                                }
                            }
                            if let Some(line) = jump_line {
                                self.core.cursor.pos.line = line;
                                self.core.cursor.pos.col = 0;
                                self.core.clamp_cursor();
                            }
                        }
                    }
                });
            });
    }

    /// 3. ステータスバー ＆ 最下段コマンドラインの描画（`TextEdit` 不使用）
    fn render_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_status_panel")
            .frame(Frame::NONE.fill(colors::BG_PANEL))
            .show(ctx, |ui| {
                // 上段: ステータスライン
                Frame::NONE
                    .fill(colors::BG_STATUSLINE)
                    .inner_margin(Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mode = self.core.mode();
                            let (badge_bg, badge_fg) = self.mode_badge_color(mode);
                            let mode_name = format!(" {} ", mode.name());

                            Frame::NONE
                                .fill(badge_bg)
                                .corner_radius(CornerRadius::same(3))
                                .inner_margin(Margin::symmetric(6, 2))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(mode_name)
                                            .color(badge_fg)
                                            .strong()
                                            .size(11.0),
                                    );
                                });

                            if self.core.keisen_mode.is_active {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[KEISEN]")
                                        .color(colors::ACCENT_CYAN)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            if self.core.is_csv_active {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[CSV]")
                                        .color(colors::ACCENT_GREEN)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            if self.core.is_recording_macro {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[REC]")
                                        .color(colors::ACCENT_RED)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            ui.add_space(6.0);

                            let file_name = self
                                .core
                                .file_path
                                .as_ref()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("[No Name]");

                            let dirty_badge = if self.core.buffer.is_modified() {
                                " [+]"
                            } else {
                                ""
                            };
                            ui.label(
                                egui::RichText::new(format!("{}{}", file_name, dirty_badge))
                                    .color(colors::FG_TEXT)
                                    .strong(),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let lang_str = format!("{:?}", self.core.syntax_lang);
                                    ui.label(
                                        egui::RichText::new(lang_str)
                                            .color(colors::FG_DIMMED)
                                            .size(11.0),
                                    );
                                    ui.separator();

                                    let nl_name = format!("[{}]", self.core.newline.name());
                                    ui.label(
                                        egui::RichText::new(nl_name)
                                            .color(colors::FG_DIMMED)
                                            .size(11.0),
                                    );

                                    let enc_name = format!("[{}]", self.core.encoding.name());
                                    ui.label(
                                        egui::RichText::new(enc_name)
                                            .color(colors::FG_DIMMED)
                                            .size(11.0),
                                    );
                                    ui.separator();

                                    let cursor = self.core.cursor_pos();
                                    let pos_str =
                                        format!("Ln {}, Col {}", cursor.line + 1, cursor.col + 1);
                                    ui.label(
                                        egui::RichText::new(pos_str)
                                            .color(colors::FG_TEXT)
                                            .strong()
                                            .size(11.0),
                                    );
                                },
                            );
                        });
                    });

                // 下段: 最下段コマンドライン / メッセージ表示行（共通 App コアのパーサーバッファを直接描画）
                Frame::NONE
                    .fill(colors::BG_PANEL)
                    .inner_margin(Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if self.core.is_migemo_search {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "g/{}{}",
                                        self.core.migemo_query, self.ime_preedit
                                    ))
                                    .color(colors::ACCENT_YELLOW)
                                    .family(egui::FontFamily::Monospace),
                                );
                            } else if self.core.mode() == ViMode::CommandLine {
                                let cmd_str = self
                                    .core
                                    .vi
                                    .parser()
                                    .command_line_buffer()
                                    .map(|(prefix, buf)| {
                                        format!("{prefix}{buf}{}", self.ime_preedit)
                                    })
                                    .unwrap_or_else(|| ":".to_string());
                                ui.label(
                                    egui::RichText::new(cmd_str)
                                        .color(colors::FG_TEXT)
                                        .family(egui::FontFamily::Monospace),
                                );
                            } else if let Some(ref msg) = self.core.status_message {
                                let is_err = msg.contains("未保存")
                                    || msg.contains("エラー")
                                    || msg.contains("見つかりません")
                                    || msg.starts_with('E');
                                let color = if is_err {
                                    colors::ACCENT_RED
                                } else {
                                    colors::FG_TEXT
                                };
                                ui.label(egui::RichText::new(msg).color(color));
                            } else {
                                ui.label(
                                    egui::RichText::new("ready")
                                        .color(colors::FG_DIMMED)
                                        .size(11.0),
                                );
                            }
                        });
                    });
            });
    }

    /// 4. 中央エディタ分割ビュー ＆ ミニマップ領域の描画（`ui.painter()` によるカスタム等幅レンダリング）
    fn render_central_panel(&mut self, ctx: &egui::Context) {
        // 4.1 入力イベント (`Event::Key`, `Event::Text`, `Event::Ime`) の取得とディスパッチ
        let (events, current_modifiers) = ctx.input(|i| (i.events.clone(), i.modifiers));
        self.process_egui_events(&events, current_modifiers);

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

                // 変更追跡マーカーと対応括弧ペアの計算
                let total_lines = self.core.buffer.line_count();
                let all_lines: Vec<String> = (0..total_lines)
                    .filter_map(|i| self.core.buffer.get_line(i))
                    .collect();
                let change_markers = self.core.change_tracker.compute_markers(&all_lines);

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
                        self.core.cursor.pos.line = target_line;
                        self.core.cursor.pos.col = clicked_vis_col;
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
                        Stroke::new(1.0, colors::BG_STATUSLINE),
                    );

                    let visible_lines = ((win_rect.height() - padding * 2.0) / line_height)
                        .floor()
                        .max(1.0) as usize;
                    let tab_width = self.core.tab_width.max(1);

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

                        // 変更追跡ガターマーカー (Added / Modified / SavedModified) の描画
                        if let Some(marker) = change_markers.get(line_idx) {
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
                        }

                        // 行テキストの取得と折り畳み・CSV・不可視文字フォーマット
                        let raw_line = self.core.buffer.get_line(line_idx).unwrap_or_default();
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

                        let chars: Vec<char> = processed_line.chars().collect();
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
                            vivivi_modes::tokenize_line(&display_line, self.core.syntax_lang)
                        };

                        let mut char_colors: Vec<Color32> = Vec::with_capacity(chars.len());
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
                        let mut search_matched = vec![false; chars.len()];
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
                        let mut visual_selected = vec![false; chars.len().max(1)];
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

                        for idx in 0..=chars.len() {
                            // アクティブカーソル位置に到達した場合、IME Preedit（未確定文字列）をインライン描画
                            if is_active_window
                                && is_current_line
                                && idx == cur.col.min(chars.len())
                                && !self.ime_preedit.is_empty()
                            {
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
                                        Stroke::new(2.0, colors::ACCENT_CYAN),
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

                                if ch != '\t' && ch != ' ' {
                                    let fg =
                                        char_colors.get(idx).copied().unwrap_or(colors::FG_TEXT);
                                    painter.text(
                                        Pos2::new(char_x, row_y),
                                        Align2::LEFT_TOP,
                                        ch.to_string(),
                                        font_id.clone(),
                                        fg,
                                    );
                                }
                            }

                            visual_col += ch_cols;
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
                                                ch.to_string(),
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
                        Stroke::new(1.0, stroke_color),
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
                        Pos2::new(editor_rect.min.x + 40.0, editor_rect.min.y + 20.0),
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

                    // ミニマップクリックでスクロールジャンプ
                    if let Some(click_pos) = pointer_click_pos
                        && minimap_rect.contains(click_pos)
                    {
                        let ratio = ((click_pos.y - minimap_rect.top()) / minimap_rect.height())
                            .clamp(0.0, 1.0);
                        let target_line = (ratio
                            * self.core.buffer.line_count().saturating_sub(1) as f32)
                            as usize;
                        self.core.scroll.0 = target_line;
                        self.core.cursor.pos.line = target_line;
                        self.core.clamp_cursor();
                    }

                    let mm_painter = ui.painter().with_clip_rect(minimap_rect);
                    mm_painter.rect_filled(minimap_rect, 0.0, colors::BG_PANEL);
                    mm_painter.line_segment(
                        [minimap_rect.left_top(), minimap_rect.left_bottom()],
                        Stroke::new(1.0, colors::BG_STATUSLINE),
                    );

                    let total_lines_mm = self.core.buffer.line_count().max(1);
                    let scroll_row = self.core.scroll.0;
                    let visible_rows = (editor_rect.height() / line_height).max(1.0) as usize;

                    // 可視範囲周辺のコード構造ミニライン描画（Viewport Culling）
                    let max_mm_rows = (minimap_rect.height() / 3.0) as usize;
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
                                let y = minimap_rect.top() + (i as f32) * 3.0 + 2.0;
                                let x1 = minimap_rect.left() + 6.0 + indent * 1.2;
                                let x2 = (x1 + len).min(minimap_rect.right() - 4.0);
                                if x2 > x1 {
                                    mm_painter.line_segment(
                                        [Pos2::new(x1, y), Pos2::new(x2, y)],
                                        Stroke::new(1.5, colors::FG_LINENUMBER),
                                    );
                                }
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
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(122, 162, 247, 100)),
                        egui::StrokeKind::Inside,
                    );
                }
            });
    }

    /// 共通 `App` コアのモーダル状態（ヘルプ・印刷プレビュー・履歴・クリップボード・ファイラー）を
    /// フローティングウィンドウとして描画します。
    fn render_modals(&mut self, ctx: &egui::Context) {
        // 5.1 ヘルプビューア (:help / F1)
        if self.core.show_help {
            let mut open = self.core.show_help;
            let title = if self.core.help_title.is_empty() {
                "vivivi Help".to_string()
            } else {
                self.core.help_title.clone()
            };
            egui::Window::new(title)
                .open(&mut open)
                .default_size([620.0, 460.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for line in self.core.help_lines.iter().skip(self.core.help_scroll) {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(colors::FG_TEXT)
                                    .family(egui::FontFamily::Monospace),
                            );
                        }
                    });
                });
            self.core.show_help = open;
        }

        // 5.2 印刷・出力プレビュー (:preview / :todo / :diff 等)
        if self.core.show_preview {
            let mut open = self.core.show_preview;
            egui::Window::new("Preview / Output")
                .open(&mut open)
                .default_size([640.0, 480.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Export PDF (p)").clicked()
                            && let Err(e) = self.core.export_pdf(None)
                        {
                            self.core.status_message = Some(format!("PDF出力エラー: {e}"));
                        }
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for line in self
                            .core
                            .preview_lines
                            .iter()
                            .skip(self.core.preview_scroll)
                        {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(colors::FG_TEXT)
                                    .family(egui::FontFamily::Monospace),
                            );
                        }
                    });
                });
            self.core.show_preview = open;
        }

        // 5.3 クリップボード履歴 (F6)
        if self.core.show_clipboard_hist {
            let mut open = self.core.show_clipboard_hist;
            egui::Window::new("Clipboard History (F6)")
                .open(&mut open)
                .default_size([480.0, 320.0])
                .show(ctx, |ui| {
                    let items: Vec<String> = self
                        .core
                        .vi
                        .registers()
                        .history()
                        .items()
                        .iter()
                        .map(|rc| rc.content.preview_text(60))
                        .collect();
                    if items.is_empty() {
                        ui.label(
                            egui::RichText::new("(履歴はありません)").color(colors::FG_DIMMED),
                        );
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (idx, preview) in items.iter().enumerate() {
                                let is_sel = idx == self.core.selected_cb_idx;
                                if ui
                                    .selectable_label(is_sel, format!("{}: {}", idx + 1, preview))
                                    .clicked()
                                {
                                    self.core.selected_cb_idx = idx;
                                }
                            }
                        });
                    }
                });
            self.core.show_clipboard_hist = open;
        }

        // 5.4 最近使ったファイル ＆ お気に入り (:recent / F7)
        if self.core.show_recent_modal {
            let mut open = self.core.show_recent_modal;
            let mut file_to_open: Option<PathBuf> = None;
            egui::Window::new("Recent & Favorite Files (F7)")
                .open(&mut open)
                .default_size([520.0, 360.0])
                .show(ctx, |ui| {
                    let mut all_entries = Vec::new();
                    for fav in &self.core.recent_manager.favorite_files {
                        all_entries.push((true, fav.clone()));
                    }
                    for rec in &self.core.recent_manager.recent_files {
                        if !self.core.recent_manager.favorite_files.contains(rec) {
                            all_entries.push((false, rec.clone()));
                        }
                    }

                    if all_entries.is_empty() {
                        ui.label(
                            egui::RichText::new("(履歴はありません)").color(colors::FG_DIMMED),
                        );
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (idx, (is_fav, path)) in all_entries.iter().enumerate() {
                                let icon = if *is_fav { "★" } else { "📄" };
                                let label = format!("{} {}", icon, path.display());
                                if ui
                                    .selectable_label(idx == self.core.selected_recent_idx, label)
                                    .clicked()
                                {
                                    file_to_open = Some(path.clone());
                                }
                            }
                        });
                    }
                });
            if let Some(target) = file_to_open {
                open = false;
                self.core
                    .execute_ex(&format!("e {}", target.to_string_lossy()));
            }
            self.core.show_recent_modal = open;
        }

        // 5.5 2ペイン・ファイラー (:filer)
        if self.core.show_filer {
            let mut open = self.core.show_filer;
            egui::Window::new("2-Pane Filer (:filer)")
                .open(&mut open)
                .default_size([680.0, 440.0])
                .show(ctx, |ui| {
                    if let Some(ref filer) = self.core.filer {
                        ui.columns(2, |cols| {
                            cols[0].label(
                                egui::RichText::new(format!(
                                    "Left: {}",
                                    filer.left.current_dir.display()
                                ))
                                .strong(),
                            );
                            cols[0].separator();
                            egui::ScrollArea::vertical().id_salt("filer_left").show(
                                &mut cols[0],
                                |ui| {
                                    for (i, entry) in filer.left.entries.iter().enumerate() {
                                        let prefix = if entry.is_dir { "📁" } else { "📄" };
                                        let is_left_active = filer.active
                                            == vivivi_modes::filer::ActiveFilerPane::Left;
                                        let _ = ui.selectable_label(
                                            is_left_active && i == filer.left.cursor,
                                            format!("{} {}", prefix, entry.name),
                                        );
                                    }
                                },
                            );

                            cols[1].label(
                                egui::RichText::new(format!(
                                    "Right: {}",
                                    filer.right.current_dir.display()
                                ))
                                .strong(),
                            );
                            cols[1].separator();
                            egui::ScrollArea::vertical().id_salt("filer_right").show(
                                &mut cols[1],
                                |ui| {
                                    for (i, entry) in filer.right.entries.iter().enumerate() {
                                        let prefix = if entry.is_dir { "📁" } else { "📄" };
                                        let is_right_active = filer.active
                                            == vivivi_modes::filer::ActiveFilerPane::Right;
                                        let _ = ui.selectable_label(
                                            is_right_active && i == filer.right.cursor,
                                            format!("{} {}", prefix, entry.name),
                                        );
                                    }
                                },
                            );
                        });
                    }
                });
            if !open {
                self.core.close_filer();
            }
        }
    }
}

/// [`egui::Key`] と [`egui::Modifiers`] を `vivivi-app` 共通の [`AppKeyEvent`] へ変換します。
///
/// 通常の印字可能文字（Ctrl/Alt なし）は `egui::Event::Text` 側で正確に処理されるため、
/// ここでは特殊制御キー（`Esc`, `Enter`, `Backspace`, `Tab`, 矢印キー, ファンクションキー等）および
/// `Ctrl` / `Alt` 修飾付きのショートカットキーのみを変換します。
///
/// # Examples
/// ```
/// use egui::{Key, Modifiers};
/// use vivivi_app::{KeyCode, KeyModifiers};
/// use vivivi_gui::egui_key_to_app_key;
///
/// // Escape などの特殊キーは常に変換される
/// let esc = egui_key_to_app_key(Key::Escape, Modifiers::NONE).unwrap();
/// assert_eq!(esc.code, KeyCode::Esc);
///
/// // 修飾なしの文字キーは Event::Text 側で処理するため None を返す
/// assert!(egui_key_to_app_key(Key::A, Modifiers::NONE).is_none());
///
/// // Ctrl+W などのショートカットは変換される
/// let ctrl_w = egui_key_to_app_key(Key::W, Modifiers::CTRL).unwrap();
/// assert_eq!(ctrl_w.code, KeyCode::Char('w'));
/// assert!(ctrl_w.modifiers.contains(KeyModifiers::CONTROL));
/// ```
pub fn egui_key_to_app_key(key: Key, modifiers: Modifiers) -> Option<AppKeyEvent> {
    let mut mods = KeyModifiers::NONE;
    if modifiers.shift {
        mods |= KeyModifiers::SHIFT;
    }
    if modifiers.ctrl || modifiers.command || modifiers.mac_cmd {
        mods |= KeyModifiers::CONTROL;
    }
    if modifiers.alt {
        mods |= KeyModifiers::ALT;
    }

    // 1. 特殊キー・ナビゲーションキー・ファンクションキーは常に Event::Key から変換
    let special_code = match key {
        Key::Escape => Some(KeyCode::Esc),
        Key::Enter => Some(KeyCode::Enter),
        Key::Backspace => Some(KeyCode::Backspace),
        Key::Delete => Some(KeyCode::Delete),
        Key::Tab => Some(KeyCode::Tab),
        Key::Insert => Some(KeyCode::Insert),
        Key::Home => Some(KeyCode::Home),
        Key::End => Some(KeyCode::End),
        Key::PageUp => Some(KeyCode::PageUp),
        Key::PageDown => Some(KeyCode::PageDown),
        Key::ArrowLeft => Some(KeyCode::Left),
        Key::ArrowRight => Some(KeyCode::Right),
        Key::ArrowUp => Some(KeyCode::Up),
        Key::ArrowDown => Some(KeyCode::Down),
        Key::F1 => Some(KeyCode::F(1)),
        Key::F2 => Some(KeyCode::F(2)),
        Key::F3 => Some(KeyCode::F(3)),
        Key::F4 => Some(KeyCode::F(4)),
        Key::F5 => Some(KeyCode::F(5)),
        Key::F6 => Some(KeyCode::F(6)),
        Key::F7 => Some(KeyCode::F(7)),
        Key::F8 => Some(KeyCode::F(8)),
        Key::F9 => Some(KeyCode::F(9)),
        Key::F10 => Some(KeyCode::F(10)),
        Key::F11 => Some(KeyCode::F(11)),
        Key::F12 => Some(KeyCode::F(12)),
        _ => None,
    };

    if let Some(code) = special_code {
        return Some(AppKeyEvent::new(code, mods));
    }

    // 2. Ctrl または Alt が押されている場合のみ、文字キーをショートカットとして変換
    if !mods.contains(KeyModifiers::CONTROL) && !mods.contains(KeyModifiers::ALT) {
        return None;
    }

    let ch = match key {
        Key::A => 'a',
        Key::B => 'b',
        Key::C => 'c',
        Key::D => 'd',
        Key::E => 'e',
        Key::F => 'f',
        Key::G => 'g',
        Key::H => 'h',
        Key::I => 'i',
        Key::J => 'j',
        Key::K => 'k',
        Key::L => 'l',
        Key::M => 'm',
        Key::N => 'n',
        Key::O => 'o',
        Key::P => 'p',
        Key::Q => 'q',
        Key::R => 'r',
        Key::S => 's',
        Key::T => 't',
        Key::U => 'u',
        Key::V => 'v',
        Key::W => 'w',
        Key::X => 'x',
        Key::Y => 'y',
        Key::Z => 'z',
        Key::Semicolon => ';',
        Key::Comma => ',',
        Key::Period => '.',
        Key::Slash => '/',
        Key::OpenBracket => '[',
        Key::CloseBracket => ']',
        Key::Minus => '-',
        Key::Equals => '=',
        _ => return None,
    };

    let final_ch = if mods.contains(KeyModifiers::SHIFT) {
        ch.to_ascii_uppercase()
    } else {
        ch
    };

    Some(AppKeyEvent::new(KeyCode::Char(final_ch), mods))
}

impl eframe::App for ViviviGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. トップパネル（メニューバー ＆ マルチタブバー）
        self.render_top_panel(ctx);

        // 2. ボトムパネル（ステータスバー ＆ コマンドライン行）
        self.render_bottom_panel(ctx);

        // 3. サイドパネル（ファイルツリー / アウトライン）
        self.render_sidebar(ctx);

        // 4. セントラルパネル（入力ディスパッチ、Painter描画、カーソル＆IME座標通知、ミニマップ）
        self.render_central_panel(ctx);

        // 5. モーダルウィンドウ（ヘルプ、印刷プレビュー、履歴、ファイラー等）
        self.render_modals(ctx);

        // 6. :q 等で終了フラグが立った場合は OS ウィンドウを閉じる
        if self.core.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vivivi_core::{CharEncoding, NewlineType};

    #[test]
    fn test_internal_mode_badge_and_syntax_colors() {
        let core = App::new("", None, CharEncoding::Utf8, NewlineType::Lf);
        let gui = ViviviGuiApp::from_core(core);

        assert_eq!(gui.mode_badge_color(ViMode::Normal).0, colors::ACCENT_BLUE);
        assert_eq!(gui.mode_badge_color(ViMode::Insert).0, colors::ACCENT_GREEN);
        assert_eq!(
            gui.mode_badge_color(ViMode::Visual).0,
            colors::ACCENT_PURPLE
        );
        assert_eq!(
            gui.mode_badge_color(ViMode::CommandLine).0,
            colors::ACCENT_YELLOW
        );
        assert_eq!(gui.mode_badge_color(ViMode::Replace).0, colors::ACCENT_RED);

        assert_eq!(
            ViviviGuiApp::syntax_token_color(TokenType::Keyword),
            colors::ACCENT_PURPLE
        );
        assert_eq!(
            ViviviGuiApp::syntax_token_color(TokenType::StringLiteral),
            colors::ACCENT_GREEN
        );
    }
}
