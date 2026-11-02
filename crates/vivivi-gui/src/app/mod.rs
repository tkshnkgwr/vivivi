//! エディタ画面骨格レイアウト ＆ 共通 App コア接続・日本語 IME 統合モジュール（ステップ 2, 3, 4）。
//!
//! `eframe::App` のライフサイクル内で、Tokyo Night を基調としたレイアウトを構築し、
//! `egui` 標準の `TextEdit` を一切使用せず、`ui.painter()` による等幅グリッド描画、
//! OS ネイティブの日本語 IME インライン入力・座標通知、および削除文字落下アニメーションを実現します。

mod canvas;
mod input;
mod modals;
mod panels;
mod settings_dialog;

pub use settings_dialog::{SettingsCategory, SettingsDialogState};

pub use input::egui_key_to_app_key;

use eframe::egui;
use egui::{Color32, Pos2, Rect};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vivivi_app::App;
use vivivi_modes::TokenType;
use vivivi_vi::ViMode;

use crate::particles::ParticleManager;
#[cfg(test)]
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
    /// ミニマップ構造キャッシュ（インデント幅, 長さ）のリスト
    pub minimap_cache: Vec<(f32, f32)>,
    /// ミニマップキャッシュ作成時のバッファ状態キー (文字数, 行数)
    pub minimap_cache_key: (usize, usize),
    /// 構文ハイライト用行トークンキャッシュ（行テキストハッシュ + 言語 -> トークン列）
    pub syntax_cache: HashMap<u64, Vec<vivivi_modes::SyntaxToken>>,
    /// ウィンドウ表示状態（座標、サイズ、最大化フラグ）
    pub window_state: crate::window_state::WindowState,
    /// 現在適用中のカラーテーマ定義
    pub active_theme: vivivi_app::theme::Theme,
    /// 左右2ペイン設定ダイアログ状態
    pub settings_dialog: SettingsDialogState,
}

impl ViviviGuiApp {
    /// 指定されたディレクトリ直下のファイル・フォルダ一覧を読み込んでソートします。
    pub(crate) fn read_directory_entries(dir: &Path) -> Vec<PathBuf> {
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
        let active_theme = vivivi_app::theme::Theme::load_by_name(&core.config.appearance.theme);
        let initial_config = core.config.clone();

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
            minimap_cache: Vec::new(),
            minimap_cache_key: (usize::MAX, usize::MAX),
            syntax_cache: HashMap::new(),
            window_state: crate::window_state::WindowState::default(),
            active_theme,
            settings_dialog: SettingsDialogState::new(&initial_config),
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
    pub(crate) fn mode_badge_color(&self, mode: ViMode) -> (Color32, Color32) {
        let bg_theme = match mode {
            ViMode::Normal => self.active_theme.modes.normal,
            ViMode::Insert => self.active_theme.modes.insert,
            ViMode::Visual | ViMode::VisualLine | ViMode::VisualBlock => {
                self.active_theme.modes.visual
            }
            ViMode::CommandLine => self.active_theme.modes.command,
            ViMode::Replace => self.active_theme.modes.replace,
        };
        let bg = crate::theme::to_color32(bg_theme);
        let luminance = 0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
        let fg = if luminance > 128.0 {
            Color32::from_rgb(16, 16, 20)
        } else {
            Color32::WHITE
        };
        (bg, fg)
    }

    /// 現在適用中のテーマに応じた構文トークン色を返します。
    pub(crate) fn syntax_token_color(&self, token_type: TokenType) -> Color32 {
        let tc = match token_type {
            TokenType::Keyword => self.active_theme.syntax.keyword,
            TokenType::Type => self.active_theme.syntax.type_name,
            TokenType::StringLiteral => self.active_theme.syntax.string,
            TokenType::Number => self.active_theme.syntax.number,
            TokenType::Comment => self.active_theme.syntax.comment,
            TokenType::Punctuation => self.active_theme.syntax.operator,
            TokenType::Text => self.active_theme.ui.fg_text,
        };
        crate::theme::to_color32(tc)
    }
}

impl eframe::App for ViviviGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // テーマや外観設定（Exコマンドや外部設定ファイル読込等）の動的同期
        if self.core.config.appearance.theme != self.active_theme.meta.name {
            self.active_theme =
                vivivi_app::theme::Theme::load_by_name(&self.core.config.appearance.theme);
            crate::theme::apply_theme(ctx, &self.active_theme);
            self.syntax_cache.clear();
        }
        if self.minimap_open != self.core.config.appearance.gui.show_minimap {
            self.minimap_open = self.core.config.appearance.gui.show_minimap;
        }
        if self.particles.enabled != self.core.config.appearance.gui.enable_gravity_drop {
            self.particles.enabled = self.core.config.appearance.gui.enable_gravity_drop;
        }

        // 0. ウィンドウ位置・サイズ・最大化状態の追跡
        let (outer_rect, is_maximized) = ctx.input(|i| {
            let info = i.viewport();
            (
                info.outer_rect.or(info.inner_rect),
                info.maximized.unwrap_or(false),
            )
        });

        if is_maximized {
            self.window_state.is_maximized = true;
        } else {
            self.window_state.is_maximized = false;
            if let Some(rect) = outer_rect
                && rect.width() >= crate::window_state::MIN_WINDOW_WIDTH
                && rect.height() >= crate::window_state::MIN_WINDOW_HEIGHT
                && rect.min.x.is_finite()
                && rect.min.y.is_finite()
            {
                self.window_state.x = rect.min.x;
                self.window_state.y = rect.min.y;
                self.window_state.width = rect.width();
                self.window_state.height = rect.height();
            }
        }
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

        // 5.5 設定モーダルダイアログ (:preferences / Ctrl+,)
        if self.core.show_preferences_dialog {
            self.settings_dialog.open(&self.core.config);
            self.core.show_preferences_dialog = false;
        }
        self.render_settings_dialog(ctx);

        // 6. OS ウィンドウの「×」閉じる要求の検知と未保存データ保護
        if ctx.input(|i| i.viewport().close_requested()) {
            let is_any_modified = self.core.buffer.is_modified()
                || self.core.tabs.iter().any(|t| t.buffer.is_modified());
            if is_any_modified && !self.core.should_quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.core.status_message = Some(
                    "E37: 未保存の変更があります (:w で保存、または :q! で強制終了)".to_string(),
                );
            } else {
                let _ = self.window_state.save();
                self.core.remove_swap_file();
                self.core.should_quit = true;
            }
        }

        // 7. :q 等で終了フラグが立った場合は OS ウィンドウを閉じる
        if self.core.should_quit {
            let _ = self.window_state.save();
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
            gui.syntax_token_color(TokenType::Keyword),
            colors::ACCENT_PURPLE
        );
        assert_eq!(
            gui.syntax_token_color(TokenType::StringLiteral),
            colors::ACCENT_GREEN
        );
    }
}
