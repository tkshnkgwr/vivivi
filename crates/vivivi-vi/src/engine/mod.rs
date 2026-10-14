//! ViEngine 状態機械およびコマンド実行エンジンモジュール。
//!
//! OpenViVi の `ViEngine.cpp` に基づき、キーボード入力の解釈と、
//! `TextBuffer` および `TextCursor` に対する Vi コマンドの完全な実行・適用を行います。

mod ex;
mod motion;
mod operate;
mod visual;

#[cfg(test)]
mod tests;

pub use visual::BlockInsertSession;

use crate::command::{Operator, ViCommand, WordSearchKind};
use crate::mode::ViMode;
use crate::parser::CommandParser;
use crate::register::{RegisterContent, RegisterMgr};
use vivivi_core::{CursorPosition, TextBuffer, TextCursor};

/// exコマンドやエディタ制御の実行結果。
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
}

/// Vi コマンドエンジン本体。
#[derive(Debug, Default)]
pub struct ViEngine {
    mode: ViMode,
    registers: RegisterMgr,
    parser: CommandParser,
    /// 直前の検索文字列
    last_search_pattern: Option<String>,
    /// 直前の検索方向 (true: 前方 `/`, false: 後方 `?`)
    last_search_forward: bool,
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
}

impl ViEngine {
    /// 新しい ViEngine を生成します。
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
        }
    }

    /// 現在のViモードを取得します。
    pub fn mode(&self) -> ViMode {
        self.mode
    }

    /// Viモードを直接変更します。
    pub fn set_mode(&mut self, mode: ViMode) {
        self.mode = mode;
    }

    /// レジスタマネージャへの参照を取得します。
    pub fn registers(&self) -> &RegisterMgr {
        &self.registers
    }

    /// レジスタマネージャへの可変参照を取得します。
    pub fn registers_mut(&mut self) -> &mut RegisterMgr {
        &mut self.registers
    }

    /// パーサーの状態を取得します。
    pub fn parser(&self) -> &CommandParser {
        &self.parser
    }

    /// 単一文字のキー入力を状態機械に供給し、確定した ViCommand があれば返します。
    pub fn feed_char(&mut self, ch: char) -> Option<ViCommand> {
        let cmd = self.parser.feed_char(ch, self.mode);

        if self.recording_register.is_some() && !matches!(cmd, Some(ViCommand::StopRecordMacro)) {
            self.recorded_macro_keys.push(ch);
        }

        if let Some(ref c) = cmd {
            match c {
                ViCommand::EnterMode(new_mode) => self.mode = *new_mode,
                ViCommand::InsertBefore
                | ViCommand::InsertAfter
                | ViCommand::InsertLineStart
                | ViCommand::InsertLineEnd
                | ViCommand::OpenLineBelow
                | ViCommand::OpenLineAbove
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
                | ViCommand::ChangeToEol(_)
                | ViCommand::VisualOperate {
                    op: Operator::Change,
                    ..
                } => {
                    self.mode = ViMode::Insert;
                }
                ViCommand::VisualOperate { .. } | ViCommand::VisualSwapCase => {
                    self.mode = ViMode::Normal;
                }
                _ => {}
            }
        }
        cmd
    }

    /// 挿入モード中にタイプされた文字を記録します（矩形一括挿入およびマクロの追跡用）。
    pub fn record_inserted_char(&mut self, ch: char) {
        if self.recording_register.is_some() {
            self.recorded_macro_keys.push(ch);
        }
        if let Some(session) = &mut self.block_insert_session {
            session.typed_chars.push(ch);
        }
    }

    /// パースされた ViCommand をバッファとカーソルに適用して実行します。
    pub fn execute(
        &mut self,
        cmd: &ViCommand,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) -> ViExecutionResult {
        let mut result = ViExecutionResult::default();

        match cmd {
            ViCommand::EnterMode(mode) => {
                let prev_mode = self.mode;
                self.mode = *mode;
                buffer.prohibit_merge_undo();

                if matches!(
                    mode,
                    ViMode::Visual | ViMode::VisualLine | ViMode::VisualBlock
                ) {
                    cursor.anchor = Some(cursor.pos);
                } else if *mode == ViMode::Normal {
                    cursor.clear_selection();

                    // 矩形一括挿入セッションが完了した場合
                    if prev_mode == ViMode::Insert && self.block_insert_session.is_some() {
                        self.finish_block_insert(buffer);
                    }
                }
                result.need_redraw = true;
            }

            ViCommand::Move(motion) => {
                self.execute_motion(motion, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::InsertBefore => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                result.need_redraw = true;
            }

            ViCommand::InsertAfter => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                if cursor.pos.col < line_len {
                    cursor.pos.col += 1;
                }
                result.need_redraw = true;
            }

            ViCommand::InsertLineStart => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                result.need_redraw = true;
            }

            ViCommand::InsertLineEnd => {
                self.mode = ViMode::Insert;
                buffer.prohibit_merge_undo();
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                cursor.pos.col = line_len;
                result.need_redraw = true;
            }

            ViCommand::OpenLineBelow => {
                self.mode = ViMode::Insert;
                buffer.open_undo_block();
                let line_end_pos = buffer.line_start_position(cursor.pos.line + 1);
                // 最終行の場合は末尾に改行を追加
                if cursor.pos.line + 1 >= buffer.line_count() {
                    let end_pos = buffer.len();
                    buffer.insert_text(end_pos, "\n");
                } else {
                    buffer.insert_text(line_end_pos.saturating_sub(1), "\n");
                }
                buffer.close_undo_block();
                cursor.pos.line += 1;
                cursor.pos.col = 0;
                result.need_redraw = true;
            }

            ViCommand::OpenLineAbove => {
                self.mode = ViMode::Insert;
                buffer.open_undo_block();
                let line_start_pos = buffer.line_start_position(cursor.pos.line);
                buffer.insert_text(line_start_pos, "\n");
                buffer.close_undo_block();
                cursor.pos.col = 0;
                result.need_redraw = true;
            }

            ViCommand::DeleteChar(count) => {
                buffer.open_undo_block();
                let pos = buffer.cursor_to_position(cursor.pos);
                let deleted = buffer.delete_text(pos, *count, false);
                buffer.close_undo_block();
                if !deleted.is_empty() {
                    self.registers
                        .set(None, RegisterContent::CharacterWise(deleted));
                }
                self.last_edit_pos = Some(cursor.pos);
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::DeletePrevChar(count) => {
                let pos = buffer.cursor_to_position(cursor.pos);
                let delete_len = (*count).min(cursor.pos.col);
                if delete_len > 0 {
                    buffer.open_undo_block();
                    let start_pos = pos.saturating_sub(delete_len);
                    let deleted = buffer.delete_text(start_pos, delete_len, true);
                    buffer.close_undo_block();
                    cursor.pos.col = cursor.pos.col.saturating_sub(delete_len);
                    if !deleted.is_empty() {
                        self.registers
                            .set(None, RegisterContent::CharacterWise(deleted));
                    }
                }
                result.need_redraw = true;
            }

            ViCommand::DeleteToEol(reg) => {
                let pos = buffer.cursor_to_position(cursor.pos);
                let next_line_start = buffer.line_start_position(cursor.pos.line + 1);
                let end_pos = if cursor.pos.line + 1 < buffer.line_count() {
                    next_line_start.saturating_sub(1)
                } else {
                    buffer.len()
                };
                if end_pos > pos {
                    buffer.open_undo_block();
                    let deleted = buffer.delete_text(pos, end_pos - pos, false);
                    buffer.close_undo_block();
                    self.registers
                        .set(*reg, RegisterContent::CharacterWise(deleted));
                }
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::ChangeToEol(reg) => {
                self.execute(&ViCommand::DeleteToEol(*reg), buffer, cursor);
                self.mode = ViMode::Insert;
                result.need_redraw = true;
            }

            ViCommand::OperateLine {
                op,
                count,
                register,
            } => {
                self.execute_operate_line(*op, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::Operate {
                op,
                motion,
                register,
            } => {
                self.execute_operate(*op, motion, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::PasteAfter { count, register } => {
                self.execute_paste(true, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::PasteBefore { count, register } => {
                self.execute_paste(false, *count, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::Undo => {
                buffer.undo();
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::Redo => {
                buffer.redo();
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::SearchForward(pat) => {
                self.last_search_pattern = Some(pat.clone());
                self.last_search_forward = true;
                self.search_pattern(pat, true, buffer, cursor, &mut result);
            }

            ViCommand::SearchBackward(pat) => {
                self.last_search_pattern = Some(pat.clone());
                self.last_search_forward = false;
                self.search_pattern(pat, false, buffer, cursor, &mut result);
            }

            ViCommand::SearchNext => {
                if let Some(ref pat) = self.last_search_pattern.clone() {
                    self.search_pattern(pat, self.last_search_forward, buffer, cursor, &mut result);
                }
            }

            ViCommand::SearchPrev => {
                if let Some(ref pat) = self.last_search_pattern.clone() {
                    self.search_pattern(
                        pat,
                        !self.last_search_forward,
                        buffer,
                        cursor,
                        &mut result,
                    );
                }
            }

            ViCommand::SearchWordUnderCursor { forward, kind } => {
                if let Some(word) = self.word_under_cursor(buffer, cursor) {
                    let escaped = regex::escape(&word);
                    let pat = match kind {
                        WordSearchKind::ExactWord => format!(r"\b{escaped}\b"),
                        WordSearchKind::Partial => escaped,
                        WordSearchKind::Prefix => format!(r"\b{escaped}"),
                        WordSearchKind::Suffix => format!(r"{escaped}\b"),
                    };
                    self.last_search_pattern = Some(pat.clone());
                    self.last_search_forward = *forward;
                    self.search_pattern(&pat, *forward, buffer, cursor, &mut result);
                }
            }

            ViCommand::Increment { delta } => {
                self.execute_increment(*delta, buffer, cursor, &mut result);
            }

            ViCommand::ToggleBoolean => {
                self.execute_toggle_boolean(buffer, cursor, &mut result);
            }

            ViCommand::JumpToLastEdit { line_only } => {
                if let Some(pos) = self.last_edit_pos {
                    cursor.pos.line = pos.line.min(buffer.line_count().saturating_sub(1));
                    if *line_only {
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    } else {
                        cursor.pos.col = pos.col;
                        self.clamp_cursor(buffer, cursor);
                    }
                    result.need_redraw = true;
                } else {
                    result.message = Some("直前の編集位置が記録されていません".to_string());
                }
            }

            ViCommand::CopyFromAboveLine { target } => {
                self.execute_copy_from_above(*target, buffer, cursor, &mut result);
            }

            ViCommand::SetMark(mark) => {
                self.marks
                    .insert(*mark, (self.current_file_path.clone(), cursor.pos));
                result.message = Some(format!("マーク '{mark}' を設定しました"));
            }

            ViCommand::JumpMark { mark, line_only } => {
                if let Some((_file, pos)) = self.marks.get(mark).cloned() {
                    cursor.pos.line = pos.line.min(buffer.line_count().saturating_sub(1));
                    if *line_only {
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    } else {
                        cursor.pos.col = pos.col;
                        self.clamp_cursor(buffer, cursor);
                    }
                    result.need_redraw = true;
                } else {
                    result.message = Some(format!("マーク '{mark}' は設定されていません"));
                }
            }

            ViCommand::ListMarks => {
                if self.marks.is_empty() {
                    result.message = Some("登録されているマークはありません".to_string());
                } else {
                    let mut sorted: Vec<_> = self.marks.iter().collect();
                    sorted.sort_by_key(|&(k, _)| *k);
                    let list: Vec<String> = sorted
                        .iter()
                        .map(|(k, (file, pos))| {
                            let f = if file.is_empty() { "<current>" } else { file };
                            format!("'{k}': {}行{}列 ({f})", pos.line + 1, pos.col + 1)
                        })
                        .collect();
                    result.message = Some(list.join("; "));
                }
            }

            ViCommand::TagJump(target_opt) => {
                let tag = target_opt
                    .clone()
                    .or_else(|| self.word_under_cursor(buffer, cursor));
                if let Some(tag_name) = tag {
                    self.execute_tag_jump(&tag_name, buffer, cursor, &mut result);
                } else {
                    result.message = Some("カーソル位置にタグがありません".to_string());
                }
            }

            ViCommand::TagPop => {
                if let Some((_file, pos, tag)) = self.tag_stack.pop() {
                    cursor.pos = pos;
                    self.clamp_cursor(buffer, cursor);
                    result.need_redraw = true;
                    result.message = Some(format!("タグ '{tag}' のジャンプ元へ復帰しました"));
                } else {
                    result.message = Some("タグスタックが空です".to_string());
                }
            }

            ViCommand::ListTags => {
                if self.tag_stack.is_empty() {
                    result.message = Some("タグスタックは空です".to_string());
                } else {
                    let list: Vec<String> = self
                        .tag_stack
                        .iter()
                        .enumerate()
                        .map(|(i, (_f, pos, tag))| format!("{}: {tag} ({}行)", i + 1, pos.line + 1))
                        .collect();
                    result.message = Some(list.join("; "));
                }
            }

            ViCommand::ExCommand(cmd_str) => {
                self.execute_ex(cmd_str, buffer, cursor, &mut result);
            }

            ViCommand::VisualOperate { op, register } => {
                self.execute_visual_operate(*op, *register, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::VisualSwapCase => {
                self.execute_visual_swap_case(buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::VisualBlockInsert { at_end } => {
                self.execute_visual_block_insert(*at_end, buffer, cursor);
                result.need_redraw = true;
            }

            ViCommand::RecordMacro(reg) => {
                self.recording_register = Some(*reg);
                self.recorded_macro_keys.clear();
                result.message = Some(format!("recording @{reg}"));
                result.need_redraw = true;
            }

            ViCommand::StopRecordMacro => {
                if let Some(reg) = self.recording_register.take() {
                    self.macro_storage
                        .insert(reg, self.recorded_macro_keys.clone());
                    result.message = Some(format!("recorded @{reg}"));
                    result.need_redraw = true;
                }
            }

            ViCommand::ExecuteMacro(target_reg) => {
                let reg = if *target_reg == '@' {
                    match self.last_macro_reg {
                        Some(last) => last,
                        None => return result,
                    }
                } else {
                    *target_reg
                };
                self.last_macro_reg = Some(reg);

                if let Some(keys) = self.macro_storage.get(&reg).cloned() {
                    for ch in keys {
                        if let Some(cmd) = self.feed_char(ch) {
                            self.execute(&cmd, buffer, cursor);
                        } else if self.mode == ViMode::Insert {
                            let pos = buffer.cursor_to_position(cursor.pos);
                            buffer.insert_char(pos, ch);
                            cursor.pos.col += 1;
                        }
                    }
                    result.need_redraw = true;
                }
            }
        }

        result
    }

    /// カーソルを現在の行の有効範囲内にクランプします。
    pub(crate) fn clamp_cursor(&self, buffer: &TextBuffer, cursor: &mut TextCursor) {
        if cursor.pos.line >= buffer.line_count() {
            cursor.pos.line = buffer.line_count().saturating_sub(1);
        }
        let line_len = buffer
            .line_text(cursor.pos.line)
            .map(|s| s.chars().count())
            .unwrap_or(0);
        let max_col = if self.mode == ViMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        cursor.pos.col = cursor.pos.col.min(max_col);
    }

    /// 行内の最初の非空白文字の列番号を返します。
    pub(crate) fn find_first_non_blank(&self, buffer: &TextBuffer, line: usize) -> usize {
        buffer
            .line_text(line)
            .and_then(|text| text.chars().position(|c| !c.is_whitespace()))
            .unwrap_or(0)
    }

    /// 数値または英字のインクリメント/デクリメント (`#+`, `##`, `#-`)
    pub(crate) fn execute_increment(
        &mut self,
        delta: i64,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        let line_idx = cursor.pos.line;
        let line_text = match buffer.line_text(line_idx) {
            Some(text) => text,
            None => return,
        };

        let chars: Vec<char> = line_text.chars().collect();
        if chars.is_empty() {
            return;
        }

        let col = cursor.pos.col.min(chars.len().saturating_sub(1));

        // 1. カーソル位置またはカーソル以降にある最初の数値を探索
        let mut num_start = None;
        let mut num_end = None;

        let mut s = col;
        while s > 0 && chars[s - 1].is_ascii_digit() {
            s -= 1;
        }
        if s > 0
            && chars[s - 1] == '-'
            && (s == 1 || !chars[s - 2].is_alphanumeric())
            && chars[s].is_ascii_digit()
        {
            s -= 1;
        }

        if chars[s].is_ascii_digit()
            || (chars[s] == '-' && s + 1 < chars.len() && chars[s + 1].is_ascii_digit())
        {
            num_start = Some(s);
            let mut e = if chars[s] == '-' { s + 1 } else { s };
            while e < chars.len() && chars[e].is_ascii_digit() {
                e += 1;
            }
            num_end = Some(e);
        } else {
            for i in col..chars.len() {
                if chars[i].is_ascii_digit()
                    || (chars[i] == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
                {
                    let start = i;
                    let mut e = if chars[i] == '-' { i + 1 } else { i };
                    while e < chars.len() && chars[e].is_ascii_digit() {
                        e += 1;
                    }
                    num_start = Some(start);
                    num_end = Some(e);
                    break;
                }
            }
        }

        if let (Some(start), Some(end)) = (num_start, num_end) {
            let num_str: String = chars[start..end].iter().collect();
            if let Ok(val) = num_str.parse::<i64>() {
                let new_val = val.saturating_add(delta);
                let new_str = format!("{new_val}");

                let line_start_pos = buffer.line_start_position(line_idx);
                let replace_pos = line_start_pos + start;

                buffer.open_undo_block();
                buffer.delete_text(replace_pos, end - start, false);
                buffer.insert_text(replace_pos, &new_str);
                buffer.close_undo_block();

                self.last_edit_pos = Some(cursor.pos);
                cursor.pos.col = start + new_str.chars().count().saturating_sub(1);
                self.clamp_cursor(buffer, cursor);
                result.need_redraw = true;
                return;
            }
        }

        // 2. 数値がない場合、カーソル位置の英字をインクリメント/デクリメント
        let ch = chars[col];
        if ch.is_ascii_alphabetic() {
            let is_upper = ch.is_ascii_uppercase();
            let base = if is_upper { b'A' } else { b'a' };
            let idx = (ch as u8 - base) as i64;
            let new_idx = (idx + delta).rem_euclid(26) as u8;
            let new_ch = (base + new_idx) as char;

            let line_start_pos = buffer.line_start_position(line_idx);
            let replace_pos = line_start_pos + col;

            buffer.open_undo_block();
            buffer.delete_text(replace_pos, 1, false);
            buffer.insert_char(replace_pos, new_ch);
            buffer.close_undo_block();

            self.last_edit_pos = Some(cursor.pos);
            result.need_redraw = true;
        }
    }

    /// カーソル下の真偽値トグル (`#!`)
    pub(crate) fn execute_toggle_boolean(
        &mut self,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        let line_idx = cursor.pos.line;
        let line_text = match buffer.line_text(line_idx) {
            Some(text) => text,
            None => return,
        };
        let chars: Vec<char> = line_text.chars().collect();
        if chars.is_empty() {
            return;
        }
        let col = cursor.pos.col.min(chars.len().saturating_sub(1));

        let mut start = col;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        let mut end = col;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        let word: String = chars[start..end].iter().collect();
        let toggled = match word.as_str() {
            "true" => Some("false"),
            "false" => Some("true"),
            "True" => Some("False"),
            "False" => Some("True"),
            "TRUE" => Some("FALSE"),
            "FALSE" => Some("TRUE"),
            "yes" => Some("no"),
            "no" => Some("yes"),
            "Yes" => Some("No"),
            "No" => Some("Yes"),
            "YES" => Some("NO"),
            "NO" => Some("YES"),
            "on" => Some("off"),
            "off" => Some("on"),
            "On" => Some("Off"),
            "Off" => Some("On"),
            "ON" => Some("OFF"),
            "OFF" => Some("ON"),
            "1" => Some("0"),
            "0" => Some("1"),
            _ => None,
        };

        if let Some(replacement) = toggled {
            let line_start_pos = buffer.line_start_position(line_idx);
            let replace_pos = line_start_pos + start;

            buffer.open_undo_block();
            buffer.delete_text(replace_pos, end - start, false);
            buffer.insert_text(replace_pos, replacement);
            buffer.close_undo_block();

            self.last_edit_pos = Some(cursor.pos);
            result.need_redraw = true;
        }
    }

    /// 直前行（上行）の文字または指定文字までを複製挿入 (`=<char>`)
    pub(crate) fn execute_copy_from_above(
        &mut self,
        target: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        if cursor.pos.line == 0 {
            return;
        }
        let above_line_idx = cursor.pos.line - 1;
        let above_line = match buffer.line_text(above_line_idx) {
            Some(text) => text,
            None => return,
        };
        let above_chars: Vec<char> = above_line.chars().collect();
        let cur_col = cursor.pos.col;

        let insert_str = if let Some(target_char) = target {
            if cur_col >= above_chars.len() {
                return;
            }
            let mut found_idx = None;
            for (i, &c) in above_chars.iter().enumerate().skip(cur_col) {
                if vivivi_modes::char_matches_fftt(target_char, c) {
                    found_idx = Some(i);
                    break;
                }
            }
            if let Some(idx) = found_idx {
                above_chars[cur_col..=idx].iter().collect::<String>()
            } else {
                return;
            }
        } else if cur_col < above_chars.len() {
            above_chars[cur_col].to_string()
        } else {
            return;
        };

        let pos = buffer.cursor_to_position(cursor.pos);
        buffer.open_undo_block();
        buffer.insert_text(pos, &insert_str);
        buffer.close_undo_block();

        let inserted_len = insert_str.chars().count();
        cursor.pos.col += inserted_len;
        self.last_edit_pos = Some(cursor.pos);
        result.need_redraw = true;
    }

    /// タグジャンプの実行 (`execute_tag_jump`)
    pub(crate) fn execute_tag_jump(
        &mut self,
        tag_name: &str,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        let mut target_line = None;

        // 1. tags ファイル探索
        if let Ok(tags_content) = std::fs::read_to_string("tags") {
            for line in tags_content.lines() {
                if line.starts_with('!') {
                    continue;
                }
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 && parts[0] == tag_name {
                    let pattern_or_line = parts[2].trim_end_matches(";\"");
                    if let Ok(l) = pattern_or_line.parse::<usize>() {
                        target_line = Some(l.saturating_sub(1));
                        break;
                    } else {
                        let clean_pat = pattern_or_line.trim_matches('/');
                        for l in 0..buffer.line_count() {
                            if buffer
                                .line_text(l)
                                .is_some_and(|line_str| line_str.contains(clean_pat))
                            {
                                target_line = Some(l);
                                break;
                            }
                        }
                        if target_line.is_some() {
                            break;
                        }
                    }
                }
            }
        }

        // 2. tagsファイルで見つからない場合、バッファ内の構文解析でフォールバック検索
        if target_line.is_none() {
            let lines: Vec<String> = (0..buffer.line_count())
                .filter_map(|i| buffer.line_text(i))
                .collect();
            let lang = if self.current_file_path.ends_with(".rs") {
                vivivi_modes::syntax::SyntaxLang::Rust
            } else if self.current_file_path.ends_with(".py") {
                vivivi_modes::syntax::SyntaxLang::Python
            } else if self.current_file_path.ends_with(".md") {
                vivivi_modes::syntax::SyntaxLang::Markdown
            } else {
                vivivi_modes::syntax::SyntaxLang::Rust
            };
            let outlines = vivivi_modes::outline::parse_outline(&lines, lang);
            for item in outlines {
                if item.title == tag_name || item.title.contains(tag_name) {
                    target_line = Some(item.line_idx);
                    break;
                }
            }
        }

        if let Some(target_l) = target_line {
            self.tag_stack.push((
                self.current_file_path.clone(),
                cursor.pos,
                tag_name.to_string(),
            ));
            cursor.pos.line = target_l.min(buffer.line_count().saturating_sub(1));
            cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            result.need_redraw = true;
            result.message = Some(format!(
                "タグ '{tag_name}' へジャンプしました (line {})",
                cursor.pos.line + 1
            ));
        } else {
            result.message = Some(format!("タグ '{tag_name}' が見つかりませんでした"));
        }
    }
}
