//! `vivivi-gui`: `egui` / `eframe` を用いた次世代デスクトップ GUI フロントエンドライブラリ。
//!
//! 本クレートは、`vivivi-app` のヘッドレス共通セッション層（[`vivivi_app::App`]）と直結し、
//! Tokyo Night テーマ、PlemolJP 等幅フォント優先設定、4ペイン画面骨格レイアウト、
//! `ui.painter()` によるカスタム等幅グリッド描画、OS ネイティブの日本語 IME インライン統合、
//! および削除文字の落下物理アニメーション（省電力ハイブリッド制御）を提供します。
//!
//! # Examples
//! ```
//! use vivivi_gui::particles::ParticleManager;
//!
//! let mgr = ParticleManager::new();
//! assert!(!mgr.is_active());
//! ```

pub mod app;
pub mod fonts;
pub mod particles;
pub mod theme;
pub mod visualize;
pub mod window_state;

pub use app::{
    AsyncFileTree, FileEntryNode, ScanStatus, SettingsCategory, SettingsDialogState, SidebarTab,
    VisibleTreeRow, ViviviGuiApp, egui_key_to_app_key,
};
pub use fonts::setup_fonts;
pub use particles::{Particle, ParticleManager, extract_deleted_line_segment};
pub use theme::{GuiPalette, apply_theme, colors, setup_visuals, to_color32};
pub use visualize::ForceGraph;
pub use window_state::WindowState;
