//! マルチタブ管理モジュール (Tab Page Management)。
//!
//! 各タブページは独立したテキストバッファ、カーソル、画面分割レイアウト（[`EditorWindow`] 列およびレイアウトツリー）を保持します。

use crate::tui::window::{EditorWindow, SplitAxis, WindowTreeNode};
use std::path::PathBuf;
use vivivi_core::{CharEncoding, NewlineType, TextBuffer, TextCursor};
use vivivi_modes::{ChangeTracker, SyntaxLang};

/// 1つのタブページが保持するエディタ状態。
#[derive(Debug, Clone)]
pub struct TabPage {
    /// 一意なタブ識別ID
    #[allow(dead_code)]
    pub id: usize,
    /// 編集中のテキストバッファ
    pub buffer: TextBuffer,
    /// アクティブウィンドウのカーソル
    pub cursor: TextCursor,
    /// ウィンドウ一覧
    pub windows: Vec<EditorWindow>,
    /// アクティブウィンドウのインデックス
    pub active_window: usize,
    /// ウィンドウ分割ツリー
    pub window_tree: WindowTreeNode,
    /// 直近の分割軸
    pub split_axis: SplitAxis,
    /// 開いているファイルのパス
    pub file_path: Option<PathBuf>,
    /// 文字エンコーディング
    pub encoding: CharEncoding,
    /// 改行コード種別
    pub newline: NewlineType,
    /// シンタックスハイライト言語
    pub syntax_lang: SyntaxLang,
    /// スクロール座標
    pub scroll: (usize, usize),
    /// 変更行ガター追跡
    pub change_tracker: ChangeTracker,
}

impl TabPage {
    /// 新規バッファを持つタブページを初期化します。
    pub fn new(
        id: usize,
        initial_text: &str,
        file_path: Option<PathBuf>,
        encoding: CharEncoding,
        newline: NewlineType,
    ) -> Self {
        let buffer = TextBuffer::from_text(initial_text);
        let syntax_lang = file_path
            .as_ref()
            .map(SyntaxLang::from_path)
            .unwrap_or(SyntaxLang::PlainText);

        let lines: Vec<String> = buffer.all_lines();
        let change_tracker = ChangeTracker::new(&lines);

        Self {
            id,
            buffer,
            cursor: TextCursor::new(0, 0),
            windows: vec![EditorWindow::new(0, 0)],
            active_window: 0,
            window_tree: WindowTreeNode::new(0),
            split_axis: SplitAxis::Horizontal,
            file_path,
            encoding,
            newline,
            syntax_lang,
            scroll: (0, 0),
            change_tracker,
        }
    }

    /// タブバー表示用のタイトル文字列を取得します。
    /// 未保存の変更がある場合は末尾に `[+]` を付加します。
    pub fn display_title(&self) -> String {
        let base_name = if let Some(ref path) = self.file_path {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "[無名]".to_string())
        } else {
            "[無名]".to_string()
        };

        if self.buffer.is_modified() {
            format!("{base_name} [+]")
        } else {
            base_name
        }
    }
}
