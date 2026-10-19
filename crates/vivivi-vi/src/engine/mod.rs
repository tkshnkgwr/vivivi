//! ViEngine 状態機械およびコマンド実行エンジンモジュール。
//!
//! OpenViVi の `ViEngine.cpp` に基づき、キーボード入力の解釈と、
//! `TextBuffer` および `TextCursor` に対する Vi コマンドの完全な実行・適用を行います。

mod ex;
mod exec;
mod motion;
mod operate;
mod visual;

#[cfg(test)]
mod tests;

pub use visual::BlockInsertSession;

use crate::command::{Operator, ScrollTarget, ViCommand};
use crate::mode::ViMode;
use crate::parser::CommandParser;
use crate::register::RegisterMgr;
use vivivi_core::CursorPosition;

/// ドットリピート (`.`) で反復実行可能な変更操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepeatableChange {
    /// 挿入系コマンド（開始コマンド、挿入テキスト）
    Insert { cmd: ViCommand, text: String },
    /// コマンド単体（削除・置換・インデント・貼り付け等）
    Command(ViCommand),
}

/// exコマンドやエディタ制御の実行結果を表す構造体。
///
/// 画面再描画の要否、ファイル保存/終了の要求フラグ、ステータスメッセージなどを伝達します。
///
/// # Examples
///
/// ```
/// use vivivi_vi::engine::ViExecutionResult;
///
/// let res = ViExecutionResult::default();
/// assert!(!res.quit_requested);
/// assert!(!res.save_requested);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViExecutionResult {
    /// 画面再描画が必要かどうか
    pub need_redraw: bool,
    /// ファイル保存が要求されたか (`:w`)
    pub save_requested: bool,
    /// エディタ終了が要求されたか (`:q`)
    pub quit_requested: bool,
    /// 強制終了フラグ (`:q!`)
    pub force_quit: bool,
    /// ステータスライン等に表示するメッセージ
    pub message: Option<String>,
    /// ファイルオープン要求（ファイラー等から）
    pub open_file_requested: Option<String>,
    /// プレビュー表示要求（印刷プレビュー等）
    pub preview_requested: Option<String>,
    /// 画面位置のスクロール調整要求 (`zz`, `zt`, `zb`)
    pub scroll_adjust: Option<ScrollTarget>,
}

/// Vi コマンドのパース、モード管理、およびバッファ操作を統合実行するエンジン本体。
///
/// # Examples
///
/// ```
/// use vivivi_core::{TextBuffer, TextCursor};
/// use vivivi_vi::{Motion, ViCommand, ViEngine, ViMode};
///
/// let mut engine = ViEngine::new();
/// let mut buffer = TextBuffer::from_text("hello");
/// let mut cursor = TextCursor::new(0, 0);
///
/// engine.execute(&ViCommand::Move(Motion::Right(2)), &mut buffer, &mut cursor);
/// assert_eq!(cursor.pos.col, 2);
/// ```
#[derive(Debug, Default)]
pub struct ViEngine {
    mode: ViMode,
    registers: RegisterMgr,
    parser: CommandParser,
    /// 直前の検索文字列
    pub(crate) last_search_pattern: Option<String>,
    /// 直前の検索方向 (true: 前方 `/`, false: 後方 `?`)
    pub(crate) last_search_forward: bool,
    /// 矩形一括挿入セッション
    pub block_insert_session: Option<BlockInsertSession>,
    /// 現在マクロ記録中のレジスタ名
    pub recording_register: Option<char>,
    /// 記録中のキー操作シーケンス
    pub recorded_macro_keys: Vec<char>,
    /// 直前に実行したマクロのレジスタ名 (`@@` 用)
    pub last_macro_reg: Option<char>,
    /// マクロレジスタ記憶領域
    pub macro_storage: std::collections::HashMap<char, Vec<char>>,
    /// 直前の文字検索情報 (対象文字, forward, till)
    pub last_find_char: Option<(char, bool, bool)>,
    /// 直前編集位置 (`@`, `'@`, `` `@ `` 用)
    pub last_edit_pos: Option<CursorPosition>,
    /// マーク記憶領域 (`mA`〜`mZ`, `ma`〜`mz`)
    pub marks: std::collections::HashMap<char, (String, CursorPosition)>,
    /// タグジャンプ履歴スタック (ファイルパス, ジャンプ元位置, タグ名)
    pub tag_stack: Vec<(String, CursorPosition, String)>,
    /// 現在編集中のファイルパス
    pub current_file_path: String,
    /// Grep 検索セッション
    pub grep_session: Option<vivivi_modes::GrepSession>,
    /// コード折り畳みマネージャ
    pub fold_manager: vivivi_modes::FoldManager,
    /// 抽出された ToDo アイテム一覧
    pub todo_items: Vec<vivivi_modes::TodoItem>,
    /// 2ペインファイラー
    pub filer: Option<vivivi_modes::TwoPaneFiler>,
    /// 世代バックアップログ
    pub difflog: Option<vivivi_modes::DiffLog>,
    /// 現在の挿入セッションを開始したコマンド (ドットリピート用)
    pub current_insert_cmd: Option<ViCommand>,
    /// 現在の挿入セッションで入力された文字列 (ドットリピート用)
    pub current_insert_chars: Vec<char>,
    /// 直前の変更操作 (ドットリピート用)
    pub last_change: Option<RepeatableChange>,
}

impl ViEngine {
    /// ノーマルモードで新しい [`ViEngine`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{ViEngine, ViMode};
    ///
    /// let engine = ViEngine::new();
    /// assert_eq!(engine.mode(), ViMode::Normal);
    /// ```
    pub fn new() -> Self {
        Self {
            mode: ViMode::Normal,
            registers: RegisterMgr::new(),
            parser: CommandParser::new(),
            last_search_pattern: None,
            last_search_forward: true,
            block_insert_session: None,
            recording_register: None,
            recorded_macro_keys: Vec::new(),
            last_macro_reg: None,
            macro_storage: std::collections::HashMap::new(),
            last_find_char: None,
            last_edit_pos: None,
            marks: std::collections::HashMap::new(),
            tag_stack: Vec::new(),
            current_file_path: String::new(),
            grep_session: None,
            fold_manager: vivivi_modes::FoldManager::new(),
            todo_items: Vec::new(),
            filer: None,
            difflog: None,
            current_insert_cmd: None,
            current_insert_chars: Vec::new(),
            last_change: None,
        }
    }

    /// 現在の Vi 動作モード（[`ViMode`]）を取得します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{ViEngine, ViMode};
    ///
    /// let engine = ViEngine::new();
    /// assert_eq!(engine.mode(), ViMode::Normal);
    /// ```
    pub fn mode(&self) -> ViMode {
        self.mode
    }

    /// Vi 動作モードを直接設定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{ViEngine, ViMode};
    ///
    /// let mut engine = ViEngine::new();
    /// engine.set_mode(ViMode::Insert);
    /// assert_eq!(engine.mode(), ViMode::Insert);
    /// ```
    pub fn set_mode(&mut self, mode: ViMode) {
        self.mode = mode;
    }

    /// レジスタマネージャ（[`RegisterMgr`]）への不変参照を取得します。
    pub fn registers(&self) -> &RegisterMgr {
        &self.registers
    }

    /// レジスタマネージャ（[`RegisterMgr`]）への可変参照を取得します。
    pub fn registers_mut(&mut self) -> &mut RegisterMgr {
        &mut self.registers
    }

    /// 外部クリップボードプロバイダを設定します。
    pub fn set_clipboard_provider(
        &mut self,
        provider: std::sync::Arc<std::sync::Mutex<dyn crate::register::ClipboardProvider>>,
    ) {
        self.registers.set_clipboard_provider(provider);
    }

    /// クリップボード同期オプション（`:set clipboard=...`）を設定します。
    pub fn set_clipboard_option(&mut self, opt: crate::register::ClipboardOption) {
        self.registers.set_clipboard_option(opt);
    }

    /// 現在のクリップボード同期オプションを取得します。
    pub fn clipboard_option(&self) -> crate::register::ClipboardOption {
        self.registers.clipboard_option()
    }

    /// 内部のコマンドパーサー（[`CommandParser`]）への参照を取得します。
    pub fn parser(&self) -> &CommandParser {
        &self.parser
    }

    /// 直前に実行された検索パターン文字列を取得します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ViEngine;
    ///
    /// let engine = ViEngine::new();
    /// assert!(engine.last_search_pattern().is_none());
    /// ```
    pub fn last_search_pattern(&self) -> Option<&str> {
        self.last_search_pattern.as_deref()
    }

    /// 単一文字のキー入力を状態機械に供給し、確定した [`ViCommand`] があれば返却します。
    ///
    /// モード変更コマンドが確定した場合はエンジンの内部 `mode` も自動的に更新されます。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::{Motion, ViCommand, ViEngine};
    ///
    /// let mut engine = ViEngine::new();
    /// let cmd = engine.feed_char('k');
    /// assert_eq!(cmd, Some(ViCommand::Move(Motion::Up(1))));
    /// ```
    pub fn feed_char(&mut self, ch: char) -> Option<ViCommand> {
        let cmd = self.parser.feed_char(ch, self.mode);

        if self.recording_register.is_some() && !matches!(cmd, Some(ViCommand::StopRecordMacro)) {
            self.recorded_macro_keys.push(ch);
        }

        if let Some(ref c) = cmd {
            match c {
                ViCommand::EnterMode(new_mode) => {
                    self.mode = *new_mode;
                    if *new_mode == ViMode::Normal
                        && let Some(insert_cmd) = self.current_insert_cmd.take()
                    {
                        let text: String = self.current_insert_chars.drain(..).collect();
                        self.last_change = Some(RepeatableChange::Insert {
                            cmd: insert_cmd,
                            text,
                        });
                    }
                }
                ViCommand::InsertBefore
                | ViCommand::InsertAfter
                | ViCommand::InsertLineStart
                | ViCommand::InsertLineEnd
                | ViCommand::OpenLineBelow
                | ViCommand::OpenLineAbove
                | ViCommand::SubstituteChar(_)
                | ViCommand::VisualBlockInsert { .. } => {
                    self.mode = ViMode::Insert;
                }
                ViCommand::Operate {
                    op: Operator::Change,
                    ..
                }
                | ViCommand::OperateLine {
                    op: Operator::Change,
                    ..
                }
                | ViCommand::ChangeToEol(_) => {
                    self.mode = ViMode::Insert;
                }
                ViCommand::ExCommand(_)
                | ViCommand::SearchForward(_)
                | ViCommand::SearchBackward(_) => {
                    self.mode = ViMode::Normal;
                }
                _ => {}
            }
        }
        cmd
    }

    /// 挿入モード中にタイプされた文字を記録します。
    ///
    /// 矩形ビジュアル一括挿入の適用、キーボードマクロ、およびドットリピート（`.`）に使用されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_vi::ViEngine;
    ///
    /// let mut engine = ViEngine::new();
    /// engine.record_inserted_char('a');
    /// ```
    pub fn record_inserted_char(&mut self, ch: char) {
        if self.recording_register.is_some() {
            self.recorded_macro_keys.push(ch);
        }
        if let Some(session) = &mut self.block_insert_session {
            session.typed_chars.push(ch);
        }
        if self.current_insert_cmd.is_some() {
            self.current_insert_chars.push(ch);
        }
    }

    /// 挿入モード中に Backspace が押下された際、記録文字バッファから1文字取り消します。
    pub fn record_inserted_backspace(&mut self) {
        if let Some(session) = &mut self.block_insert_session {
            session.typed_chars.pop();
        }
        if self.current_insert_cmd.is_some() {
            self.current_insert_chars.pop();
        }
    }
}
