//! vivivi-app: エディタ基幹状態管理 ＆ フロントエンド共通基盤ライブラリ。
//!
//! TUI フロントエンド（`src/main.rs`）およびデスクトップ GUI フロントエンド（`vivivi-gui`）から
//! 共有される共通アプリケーション状態（[`App`]）、タブ管理、画面分割木、ファイルI/O、
//! 設定管理、および ex コマンドエンジンを提供します。

pub mod config;
pub mod tui;
pub mod types;

// 主要な共通型およびイベントディスパッチャをトップレベルに再エクスポート
pub use config::EditorConfig;
pub use tui::app::App;
pub use tui::event::{execute_ex_command, handle_key_event, play_macro};
pub use tui::tab::TabPage;
pub use tui::window::{EditorWindow, SplitAxis, WindowDirection, WindowTreeNode};
pub use types::{AppKeyEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, LogicalRect};

/// [`TabPage`] の型エイリアス（GUI設計書 `Tab` との互換用）。
pub type Tab = TabPage;
