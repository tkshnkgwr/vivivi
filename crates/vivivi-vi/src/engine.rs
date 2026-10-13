//! ViEngine 状態機械およびコマンド実行エンジンモジュール。
//!
//! OpenViVi の `ViEngine.cpp` に基づき、キーボード入力の解釈と、
//! `TextBuffer` および `TextCursor` に対する Vi コマンドの完全な実行・適用を行います。

use crate::command::{Motion, Operator, ViCommand};
use crate::mode::ViMode;
use crate::parser::CommandParser;
use crate::register::{RegisterContent, RegisterMgr};
use vivivi_core::{TextBuffer, TextCursor};

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
        if let Some(ref c) = cmd {
            match c {
                ViCommand::EnterMode(new_mode) => self.mode = *new_mode,
                ViCommand::InsertBefore
                | ViCommand::InsertAfter
                | ViCommand::InsertLineStart
                | ViCommand::InsertLineEnd
                | ViCommand::OpenLineBelow
                | ViCommand::OpenLineAbove => {
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
                _ => {}
            }
        }
        cmd
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
                self.mode = *mode;
                buffer.prohibit_merge_undo();
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

            ViCommand::SearchWordUnderCursor { forward } => {
                if let Some(word) = self.word_under_cursor(buffer, cursor) {
                    self.last_search_pattern = Some(word.clone());
                    self.last_search_forward = *forward;
                    self.search_pattern(&word, *forward, buffer, cursor, &mut result);
                }
            }

            ViCommand::ExCommand(cmd_str) => {
                self.execute_ex(cmd_str, buffer, cursor, &mut result);
            }
        }

        result
    }

    /// カーソルを現在の行の有効範囲内にクランプします。
    fn clamp_cursor(&self, buffer: &TextBuffer, cursor: &mut TextCursor) {
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
    fn find_first_non_blank(&self, buffer: &TextBuffer, line: usize) -> usize {
        buffer
            .line_text(line)
            .and_then(|text| text.chars().position(|c| !c.is_whitespace()))
            .unwrap_or(0)
    }

    /// 単語構成文字（英数字またはアンダースコア）かどうかを判定します。
    fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    /// カーソル位置の単語を取得します。
    fn word_under_cursor(&self, buffer: &TextBuffer, cursor: &TextCursor) -> Option<String> {
        let line = buffer.line_text(cursor.pos.line)?;
        let chars: Vec<char> = line.chars().collect();
        if cursor.pos.col >= chars.len() {
            return None;
        }

        let is_target_word = Self::is_word_char(chars[cursor.pos.col]);
        let mut start = cursor.pos.col;
        while start > 0
            && Self::is_word_char(chars[start - 1]) == is_target_word
            && !chars[start - 1].is_whitespace()
        {
            start -= 1;
        }

        let mut end = cursor.pos.col;
        while end < chars.len()
            && Self::is_word_char(chars[end]) == is_target_word
            && !chars[end].is_whitespace()
        {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// モーションを計算してカーソルを移動させます。
    fn execute_motion(&mut self, motion: &Motion, buffer: &TextBuffer, cursor: &mut TextCursor) {
        match motion {
            Motion::Left(count) => {
                cursor.pos.col = cursor.pos.col.saturating_sub(*count);
            }
            Motion::Right(count) => {
                cursor.pos.col += *count;
                self.clamp_cursor(buffer, cursor);
            }
            Motion::Up(count) => {
                cursor.pos.line = cursor.pos.line.saturating_sub(*count);
                self.clamp_cursor(buffer, cursor);
            }
            Motion::Down(count) => {
                cursor.pos.line =
                    (cursor.pos.line + *count).min(buffer.line_count().saturating_sub(1));
                self.clamp_cursor(buffer, cursor);
            }
            Motion::StartOfLine => {
                cursor.pos.col = 0;
            }
            Motion::EndOfLine => {
                let line_len = buffer
                    .line_text(cursor.pos.line)
                    .map(|s| s.chars().count())
                    .unwrap_or(0);
                cursor.pos.col = line_len.saturating_sub(1);
            }
            Motion::FirstNonBlank => {
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            }
            Motion::DocumentStart => {
                cursor.pos.line = 0;
                cursor.pos.col = 0;
            }
            Motion::DocumentEnd(target) => {
                if let Some(target_line) = target {
                    cursor.pos.line = (*target_line)
                        .saturating_sub(1)
                        .min(buffer.line_count().saturating_sub(1));
                } else {
                    cursor.pos.line = buffer.line_count().saturating_sub(1);
                }
                cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            }
            Motion::NextWord(count) => {
                for _ in 0..*count {
                    self.move_next_word(buffer, cursor, false);
                }
            }
            Motion::PrevWord(count) => {
                for _ in 0..*count {
                    self.move_prev_word(buffer, cursor, false);
                }
            }
            Motion::EndWord(count) => {
                for _ in 0..*count {
                    self.move_end_word(buffer, cursor, false);
                }
            }
            Motion::NextBigWord(count) => {
                for _ in 0..*count {
                    self.move_next_word(buffer, cursor, true);
                }
            }
            Motion::PrevBigWord(count) => {
                for _ in 0..*count {
                    self.move_prev_word(buffer, cursor, true);
                }
            }
            Motion::EndBigWord(count) => {
                for _ in 0..*count {
                    self.move_end_word(buffer, cursor, true);
                }
            }
        }
    }

    /// 次の単語先頭への移動 (`w`, `W`)
    fn move_next_word(&self, buffer: &TextBuffer, cursor: &mut TextCursor, is_big: bool) {
        let mut pos = buffer.cursor_to_position(cursor.pos);
        let total_len = buffer.len();
        if pos >= total_len {
            return;
        }

        let start_ch = buffer.char_at(pos).unwrap_or(' ');
        let start_type = Self::char_class(start_ch, is_big);

        // 1. 現在の単語・記号グループを抜ける
        while pos < total_len {
            if buffer
                .char_at(pos)
                .is_some_and(|ch| ch == '\n' || Self::char_class(ch, is_big) != start_type)
            {
                break;
            }
            pos += 1;
        }

        // 2. 空白文字をスキップする
        while pos < total_len {
            if buffer.char_at(pos).is_some_and(|ch| !ch.is_whitespace()) {
                break;
            }
            pos += 1;
        }

        cursor.pos = buffer.position_to_cursor(pos.min(total_len.saturating_sub(1)));
    }

    /// 前の単語先頭への移動 (`b`, `B`)
    fn move_prev_word(&self, buffer: &TextBuffer, cursor: &mut TextCursor, is_big: bool) {
        let mut pos = buffer.cursor_to_position(cursor.pos);
        if pos == 0 {
            return;
        }
        pos -= 1;

        // 1. 直前の空白をスキップ
        while pos > 0 {
            if buffer.char_at(pos).is_some_and(|ch| !ch.is_whitespace()) {
                break;
            }
            pos -= 1;
        }

        let target_ch = buffer.char_at(pos).unwrap_or(' ');
        let target_type = Self::char_class(target_ch, is_big);

        // 2. 同一単語クラスの先頭まで巻き戻す
        while pos > 0 {
            if buffer
                .char_at(pos - 1)
                .is_some_and(|ch| ch.is_whitespace() || Self::char_class(ch, is_big) != target_type)
            {
                break;
            }
            pos -= 1;
        }

        cursor.pos = buffer.position_to_cursor(pos);
    }

    /// 単語末尾への移動 (`e`, `E`)
    fn move_end_word(&self, buffer: &TextBuffer, cursor: &mut TextCursor, is_big: bool) {
        let mut pos = buffer.cursor_to_position(cursor.pos);
        let total_len = buffer.len();
        if pos + 1 >= total_len {
            return;
        }
        pos += 1;

        // 空白スキップ
        while pos < total_len {
            if buffer.char_at(pos).is_some_and(|ch| !ch.is_whitespace()) {
                break;
            }
            pos += 1;
        }

        let target_ch = buffer.char_at(pos).unwrap_or(' ');
        let target_type = Self::char_class(target_ch, is_big);

        // 同一単語クラスの末尾へ進める
        while pos + 1 < total_len {
            if buffer
                .char_at(pos + 1)
                .is_some_and(|ch| ch.is_whitespace() || Self::char_class(ch, is_big) != target_type)
            {
                break;
            }
            pos += 1;
        }

        cursor.pos = buffer.position_to_cursor(pos);
    }

    /// 文字の種類（単語、記号、空白）を分類
    fn char_class(ch: char, is_big: bool) -> u8 {
        if ch.is_whitespace() {
            0
        } else if is_big || Self::is_word_char(ch) {
            1
        } else {
            2
        }
    }

    /// オペレータ＋モーションの実行 (`dw`, `cw`, `y$`)
    fn execute_operate(
        &mut self,
        op: Operator,
        motion: &Motion,
        reg: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let start_pos = buffer.cursor_to_position(cursor.pos);
        let mut target_cursor = cursor.clone();
        self.execute_motion(motion, buffer, &mut target_cursor);
        let end_pos = buffer.cursor_to_position(target_cursor.pos);

        let (from, to) = if start_pos <= end_pos {
            (start_pos, end_pos)
        } else {
            (end_pos, start_pos)
        };

        let len = to - from;
        if len == 0 {
            return;
        }

        let text = buffer.get_text(from, len);
        self.registers
            .set(reg, RegisterContent::CharacterWise(text));

        match op {
            Operator::Yank => {
                // ヤンクはバッファを変更しない
            }
            Operator::Delete => {
                buffer.open_undo_block();
                buffer.delete_text(from, len, false);
                buffer.close_undo_block();
                cursor.pos = buffer.position_to_cursor(from);
                self.clamp_cursor(buffer, cursor);
            }
            Operator::Change => {
                buffer.open_undo_block();
                buffer.delete_text(from, len, false);
                buffer.close_undo_block();
                cursor.pos = buffer.position_to_cursor(from);
                self.mode = ViMode::Insert;
            }
        }
    }

    /// 行単位オペレータの実行 (`dd`, `yy`, `cc`)
    fn execute_operate_line(
        &mut self,
        op: Operator,
        count: usize,
        reg: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let start_line = cursor.pos.line;
        let end_line = (start_line + count).min(buffer.line_count());
        let start_pos = buffer.line_start_position(start_line);
        let end_pos = buffer.line_start_position(end_line);

        let len = end_pos.saturating_sub(start_pos);
        if len == 0 {
            return;
        }

        let text = buffer.get_text(start_pos, len);
        self.registers.set(reg, RegisterContent::LineWise(text));

        match op {
            Operator::Yank => {
                // ヤンクは変更なし
            }
            Operator::Delete => {
                buffer.open_undo_block();
                buffer.delete_text(start_pos, len, false);
                buffer.close_undo_block();
                self.clamp_cursor(buffer, cursor);
            }
            Operator::Change => {
                buffer.open_undo_block();
                buffer.delete_text(start_pos, len, false);
                // 新しい行を1行挿入してインデント位置へ
                buffer.insert_text(start_pos, "\n");
                buffer.close_undo_block();
                cursor.pos.col = 0;
                self.mode = ViMode::Insert;
            }
        }
    }

    /// 貼り付けの実行 (`p`, `P`)
    fn execute_paste(
        &mut self,
        after: bool,
        count: usize,
        reg: Option<char>,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
    ) {
        let content = match self.registers.get(reg) {
            Some(c) => c.clone(),
            None => return,
        };

        buffer.open_undo_block();
        for _ in 0..count {
            match &content {
                RegisterContent::LineWise(text) => {
                    if after {
                        let insert_pos = buffer.line_start_position(cursor.pos.line + 1);
                        buffer.insert_text(insert_pos, text);
                        cursor.pos.line += 1;
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    } else {
                        let insert_pos = buffer.line_start_position(cursor.pos.line);
                        buffer.insert_text(insert_pos, text);
                        cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
                    }
                }
                RegisterContent::CharacterWise(text) => {
                    let cur_pos = buffer.cursor_to_position(cursor.pos);
                    let insert_pos = if after {
                        (cur_pos + 1).min(buffer.len())
                    } else {
                        cur_pos
                    };
                    buffer.insert_text(insert_pos, text);
                    let new_pos = insert_pos + text.chars().count().saturating_sub(1);
                    cursor.pos = buffer.position_to_cursor(new_pos);
                }
                RegisterContent::BlockWise(_) => {}
            }
        }
        buffer.close_undo_block();
    }

    /// 文字列検索 (`/`, `?`, `n`, `N`)
    fn search_pattern(
        &mut self,
        pat: &str,
        forward: bool,
        buffer: &TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        if pat.is_empty() {
            return;
        }

        let full_text = buffer.to_string();
        let cur_pos = buffer.cursor_to_position(cursor.pos);

        if forward {
            // 現在位置の後方を検索
            let target = if cur_pos + 1 < full_text.chars().count() {
                cur_pos + 1
            } else {
                0
            };

            // 全テキストの char インデックスで検索
            let found = full_text
                .char_indices()
                .filter(|(idx, _)| *idx >= target)
                .find(|(idx, _)| full_text[*idx..].starts_with(pat))
                .or_else(|| {
                    // ラップアラウンドして先頭から検索
                    full_text
                        .char_indices()
                        .find(|(idx, _)| full_text[*idx..].starts_with(pat))
                });

            if let Some((byte_idx, _)) = found {
                let char_idx = full_text[..byte_idx].chars().count();
                cursor.pos = buffer.position_to_cursor(char_idx);
                result.need_redraw = true;
            } else {
                result.message = Some(format!("パターンが見つかりません: {pat}"));
            }
        } else {
            // 現在位置より前方を後方検索
            let found = full_text
                .char_indices()
                .filter(|(idx, _)| *idx < cur_pos)
                .rfind(|(idx, _)| full_text[*idx..].starts_with(pat))
                .or_else(|| {
                    // ラップアラウンドして末尾から検索
                    full_text
                        .char_indices()
                        .rfind(|(idx, _)| full_text[*idx..].starts_with(pat))
                });

            if let Some((byte_idx, _)) = found {
                let char_idx = full_text[..byte_idx].chars().count();
                cursor.pos = buffer.position_to_cursor(char_idx);
                result.need_redraw = true;
            } else {
                result.message = Some(format!("パターンが見つかりません: {pat}"));
            }
        }
    }

    /// exコマンドのパース・実行 (`:w`, `:q`, `:%s/before/after/g`, `:行番号`)
    fn execute_ex(
        &mut self,
        cmd_str: &str,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        let cmd = cmd_str.trim();

        if cmd == "w" {
            result.save_requested = true;
            result.message = Some("ファイルを保存しました".to_string());
        } else if cmd == "q" {
            result.quit_requested = true;
        } else if cmd == "q!" {
            result.quit_requested = true;
            result.force_quit = true;
        } else if cmd == "wq" || cmd == "x" {
            result.save_requested = true;
            result.quit_requested = true;
        } else if let Ok(line_num) = cmd.parse::<usize>() {
            // 指定行へのジャンプ
            cursor.pos.line = line_num
                .saturating_sub(1)
                .min(buffer.line_count().saturating_sub(1));
            cursor.pos.col = self.find_first_non_blank(buffer, cursor.pos.line);
            result.need_redraw = true;
        } else if cmd.starts_with("%s/") || cmd.starts_with("s/") {
            // 置換コマンド
            self.execute_substitute(cmd, buffer, cursor, result);
        } else {
            result.message = Some(format!("未定義のコマンドです: :{cmd}"));
        }
    }

    /// 置換コマンド (`:s/old/new/`, `:%s/old/new/g`)
    fn execute_substitute(
        &mut self,
        cmd: &str,
        buffer: &mut TextBuffer,
        cursor: &mut TextCursor,
        result: &mut ViExecutionResult,
    ) {
        let is_global = cmd.starts_with("%s/");
        let spec = if is_global { &cmd[3..] } else { &cmd[2..] };
        let parts: Vec<&str> = spec.split('/').collect();

        if parts.len() >= 2 {
            let before = parts[0];
            let after = parts[1];
            let flags = parts.get(2).copied().unwrap_or("");
            let replace_all = flags.contains('g');

            if before.is_empty() {
                return;
            }

            buffer.open_undo_block();
            let mut replace_count = 0;

            if is_global {
                let full_text = buffer.to_string();
                let new_text = if replace_all {
                    full_text.replace(before, after)
                } else {
                    full_text.replacen(before, after, 1)
                };

                if new_text != full_text {
                    let len = buffer.len();
                    buffer.delete_text(0, len, false);
                    buffer.insert_text(0, &new_text);
                    replace_count += 1;
                }
            } else if let Some(line_str) = buffer.line_text(cursor.pos.line) {
                let new_line = if replace_all {
                    line_str.replace(before, after)
                } else {
                    line_str.replacen(before, after, 1)
                };

                if new_line != line_str {
                    let start_pos = buffer.line_start_position(cursor.pos.line);
                    buffer.delete_text(start_pos, line_str.chars().count(), false);
                    buffer.insert_text(start_pos, &new_line);
                    replace_count += 1;
                }
            }

            buffer.close_undo_block();
            self.clamp_cursor(buffer, cursor);
            result.need_redraw = true;
            result.message = Some(format!("{replace_count} 箇所の置換を行いました"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motions_and_clamp() {
        let mut vi = ViEngine::new();
        let mut buf = TextBuffer::from_text("first line\nsecond line\nthird line");
        let mut cur = TextCursor::new(0, 0);

        // j (下へ)
        vi.execute(&ViCommand::Move(Motion::Down(1)), &mut buf, &mut cur);
        assert_eq!(cur.pos.line, 1);
        assert_eq!(cur.pos.col, 0);

        // $ (行末へ)
        vi.execute(&ViCommand::Move(Motion::EndOfLine), &mut buf, &mut cur);
        assert_eq!(cur.pos.col, 10); // "second line" length is 11, col is 10

        // 0 (行頭へ)
        vi.execute(&ViCommand::Move(Motion::StartOfLine), &mut buf, &mut cur);
        assert_eq!(cur.pos.col, 0);

        // G (文書末尾へ)
        vi.execute(
            &ViCommand::Move(Motion::DocumentEnd(None)),
            &mut buf,
            &mut cur,
        );
        assert_eq!(cur.pos.line, 2);

        // gg (文書先頭へ)
        vi.execute(&ViCommand::Move(Motion::DocumentStart), &mut buf, &mut cur);
        assert_eq!(cur.pos.line, 0);
    }

    #[test]
    fn test_delete_and_yank_and_paste() {
        let mut vi = ViEngine::new();
        let mut buf = TextBuffer::from_text("apple\nbanana\ncherry");
        let mut cur = TextCursor::new(0, 0);

        // dd (1行削除) -> "banana\ncherry"
        vi.execute(
            &ViCommand::OperateLine {
                op: Operator::Delete,
                count: 1,
                register: None,
            },
            &mut buf,
            &mut cur,
        );
        assert_eq!(buf.to_string(), "banana\ncherry");
        assert_eq!(vi.registers().get(None).unwrap().as_str(), "apple\n");

        // p (下行に貼り付け) -> "banana\napple\ncherry"
        vi.execute(
            &ViCommand::PasteAfter {
                count: 1,
                register: None,
            },
            &mut buf,
            &mut cur,
        );
        assert_eq!(buf.to_string(), "banana\napple\ncherry");

        // u (Undo) -> "banana\ncherry"
        vi.execute(&ViCommand::Undo, &mut buf, &mut cur);
        assert_eq!(buf.to_string(), "banana\ncherry");
    }

    #[test]
    fn test_word_motion() {
        let mut vi = ViEngine::new();
        let mut buf = TextBuffer::from_text("hello world test");
        let mut cur = TextCursor::new(0, 0);

        // w (次の単語へ: world)
        vi.execute(&ViCommand::Move(Motion::NextWord(1)), &mut buf, &mut cur);
        assert_eq!(cur.pos.col, 6);

        // w (次の単語へ: test)
        vi.execute(&ViCommand::Move(Motion::NextWord(1)), &mut buf, &mut cur);
        assert_eq!(cur.pos.col, 12);

        // b (前の単語へ: world)
        vi.execute(&ViCommand::Move(Motion::PrevWord(1)), &mut buf, &mut cur);
        assert_eq!(cur.pos.col, 6);
    }

    #[test]
    fn test_search_and_substitute() {
        let mut vi = ViEngine::new();
        let mut buf = TextBuffer::from_text("alpha beta gamma\nalpha delta");
        let mut cur = TextCursor::new(0, 0);

        // /delta で検索
        let res = vi.execute(
            &ViCommand::SearchForward("delta".to_string()),
            &mut buf,
            &mut cur,
        );
        assert!(res.need_redraw);
        assert_eq!(cur.pos.line, 1);
        assert_eq!(cur.pos.col, 6); // "alpha " is 6

        // :%s/alpha/ALPHA/g で一括置換
        vi.execute(
            &ViCommand::ExCommand("%s/alpha/ALPHA/g".to_string()),
            &mut buf,
            &mut cur,
        );
        assert_eq!(buf.to_string(), "ALPHA beta gamma\nALPHA delta");

        // :w で保存要求
        let res = vi.execute(&ViCommand::ExCommand("w".to_string()), &mut buf, &mut cur);
        assert!(res.save_requested);
    }
}
