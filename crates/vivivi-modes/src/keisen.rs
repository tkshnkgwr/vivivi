//! 罫線（Box Drawing）描画・自動結合モードモジュール。
//!
//! OpenViVi の `keisen.cpp` から忠実に移植されたUnicode罫線接続ロジック。
//! 各罫線文字の上下左右の細線/太線状態をビットマスクで管理し、
//! カーソル周辺の接続状態に応じた適切なUnicode罫線文字（U+2500〜U+254B）を決定します。

/// Unicode 罫線ブロック開始コードポイント (─)
pub const KT_CODE_BEG: u32 = 0x2500;
/// Unicode 罫線ブロック終了コードポイント (╋)
pub const KT_CODE_END: u32 = 0x254b;

// --- 方向ビットマスク (2ビットずつ) ---
pub const KT_UP_MASK: u8 = 0xc0;
pub const KT_LEFT_MASK: u8 = 0x30;
pub const KT_DOWN_MASK: u8 = 0x0c;
pub const KT_RIGHT_MASK: u8 = 0x03;

// --- 線種フラグ (太線 / 細線) ---
pub const KT_UP_THICK: u8 = 0x80;
pub const KT_UP_THIN: u8 = 0x40;
pub const KT_LEFT_THICK: u8 = 0x20;
pub const KT_LEFT_THIN: u8 = 0x10;
pub const KT_DOWN_THICK: u8 = 0x08;
pub const KT_DOWN_THIN: u8 = 0x04;
pub const KT_RIGHT_THICK: u8 = 0x02;
pub const KT_RIGHT_THIN: u8 = 0x01;

/// 罫線の線種（細線 / 太線）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeisenType {
    #[default]
    Thin,
    Thick,
}

/// 罫線を引く方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeisenDirection {
    Up,
    Down,
    Left,
    Right,
}

/// コードポイント差分 (codepoint - 0x2500) から接続状態ビットマスクを引く静的テーブル。
///
/// OpenViVi の `keisenTable[]` を Rust の配列定数として忠実に移植。
pub const KEISEN_TABLE: [u8; (KT_CODE_END - KT_CODE_BEG + 1) as usize] = {
    let mut table = [0u8; (KT_CODE_END - KT_CODE_BEG + 1) as usize];
    // 0x2500: ─
    table[0x00] = KT_LEFT_THIN | KT_RIGHT_THIN;
    // 0x2501: ━
    table[0x01] = KT_LEFT_THICK | KT_RIGHT_THICK;
    // 0x2502: │
    table[0x02] = KT_UP_THIN | KT_DOWN_THIN;
    // 0x2503: ┃
    table[0x03] = KT_UP_THICK | KT_DOWN_THICK;
    // 0x250c: ┌
    table[0x0c] = KT_DOWN_THIN | KT_RIGHT_THIN;
    // 0x250f: ┏
    table[0x0f] = KT_DOWN_THICK | KT_RIGHT_THICK;
    // 0x2510: ┐
    table[0x10] = KT_LEFT_THIN | KT_DOWN_THIN;
    // 0x2513: ┓
    table[0x13] = KT_LEFT_THICK | KT_DOWN_THICK;
    // 0x2514: └
    table[0x14] = KT_UP_THIN | KT_RIGHT_THIN;
    // 0x2517: ┗
    table[0x17] = KT_UP_THICK | KT_RIGHT_THICK;
    // 0x2518: ┘
    table[0x18] = KT_LEFT_THIN | KT_UP_THIN;
    // 0x251b: ┛
    table[0x1b] = KT_LEFT_THICK | KT_UP_THICK;
    // 0x251c: ├
    table[0x1c] = KT_UP_THIN | KT_DOWN_THIN | KT_RIGHT_THIN;
    // 0x251d: ┝
    table[0x1d] = KT_UP_THIN | KT_DOWN_THIN | KT_RIGHT_THICK;
    // 0x2520: ┠
    table[0x20] = KT_UP_THICK | KT_DOWN_THICK | KT_RIGHT_THIN;
    // 0x2523: ┣
    table[0x23] = KT_UP_THICK | KT_DOWN_THICK | KT_RIGHT_THICK;
    // 0x2524: ┤
    table[0x24] = KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THIN;
    // 0x2525: ┥
    table[0x25] = KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THICK;
    // 0x2528: ┨
    table[0x28] = KT_UP_THICK | KT_DOWN_THICK | KT_LEFT_THIN;
    // 0x252b: ┫
    table[0x2b] = KT_UP_THICK | KT_DOWN_THICK | KT_LEFT_THICK;
    // 0x252c: ┬
    table[0x2c] = KT_DOWN_THIN | KT_LEFT_THIN | KT_RIGHT_THIN;
    // 0x252f: ┯
    table[0x2f] = KT_LEFT_THICK | KT_RIGHT_THICK | KT_DOWN_THIN;
    // 0x2530: ┰
    table[0x30] = KT_LEFT_THIN | KT_RIGHT_THIN | KT_DOWN_THICK;
    // 0x2533: ┳
    table[0x33] = KT_DOWN_THICK | KT_LEFT_THICK | KT_RIGHT_THICK;
    // 0x2534: ┴
    table[0x34] = KT_UP_THIN | KT_LEFT_THIN | KT_RIGHT_THIN;
    // 0x2537: ┷
    table[0x37] = KT_LEFT_THICK | KT_RIGHT_THICK | KT_UP_THIN;
    // 0x2538: ┸
    table[0x38] = KT_LEFT_THIN | KT_RIGHT_THIN | KT_UP_THICK;
    // 0x253b: ┻
    table[0x3b] = KT_UP_THICK | KT_LEFT_THICK | KT_RIGHT_THICK;
    // 0x253c: ┼
    table[0x3c] = KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THIN | KT_RIGHT_THIN;
    // 0x253f: ┿
    table[0x3f] = KT_LEFT_THICK | KT_RIGHT_THICK | KT_UP_THIN | KT_DOWN_THIN;
    // 0x2542: ╂
    table[0x42] = KT_UP_THICK | KT_DOWN_THICK | KT_LEFT_THIN | KT_RIGHT_THIN;
    // 0x254b: ╋
    table[0x4b] = KT_UP_THICK | KT_DOWN_THICK | KT_LEFT_THICK | KT_RIGHT_THICK;
    table
};

/// 文字から対応する罫線接続ビットマスクを取得します。
pub fn char_to_keisen_state(ch: char) -> Option<u8> {
    let code = ch as u32;
    if (KT_CODE_BEG..=KT_CODE_END).contains(&code) {
        let idx = (code - KT_CODE_BEG) as usize;
        let state = KEISEN_TABLE[idx];
        if state != 0 {
            return Some(state);
        }
    }
    None
}

/// 文字が垂直方向の罫線接続を持つか判定します。
pub fn is_vertical_keisen(ch: char) -> bool {
    char_to_keisen_state(ch).is_some_and(|s| s & (KT_UP_MASK | KT_DOWN_MASK) != 0)
}

/// 文字が水平方向の罫線接続を持つか判定します。
pub fn is_horizontal_keisen(ch: char) -> bool {
    char_to_keisen_state(ch).is_some_and(|s| s & (KT_LEFT_MASK | KT_RIGHT_MASK) != 0)
}

/// 指定されたビットマスク状態に合致する Unicode 罫線文字を検索して返します。
pub fn state_to_keisen_char(state: u8) -> Option<char> {
    for (i, &s) in KEISEN_TABLE.iter().enumerate() {
        if s == state && s != 0 {
            return char::from_u32(KT_CODE_BEG + i as u32);
        }
    }
    None
}

impl KeisenDirection {
    /// 方向に対応するビットマスクを返します。
    pub fn mask(&self) -> u8 {
        match self {
            Self::Up => KT_UP_MASK,
            Self::Down => KT_DOWN_MASK,
            Self::Left => KT_LEFT_MASK,
            Self::Right => KT_RIGHT_MASK,
        }
    }

    /// 線種に応じたビットフラグを返します。
    pub fn flag(&self, keisen_type: KeisenType) -> u8 {
        match (self, keisen_type) {
            (Self::Up, KeisenType::Thin) => KT_UP_THIN,
            (Self::Up, KeisenType::Thick) => KT_UP_THICK,
            (Self::Down, KeisenType::Thin) => KT_DOWN_THIN,
            (Self::Down, KeisenType::Thick) => KT_DOWN_THICK,
            (Self::Left, KeisenType::Thin) => KT_LEFT_THIN,
            (Self::Left, KeisenType::Thick) => KT_LEFT_THICK,
            (Self::Right, KeisenType::Thin) => KT_RIGHT_THIN,
            (Self::Right, KeisenType::Thick) => KT_RIGHT_THICK,
        }
    }

    /// 逆方向を返します。
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// 同一軸（水平: 左右、垂直: 上下）の両方向マスクを返します。
    pub fn axis_mask(&self) -> u8 {
        match self {
            Self::Left | Self::Right => KT_LEFT_MASK | KT_RIGHT_MASK,
            Self::Up | Self::Down => KT_UP_MASK | KT_DOWN_MASK,
        }
    }

    /// 直角方向のマスクを返します。
    pub fn perp_mask(&self) -> u8 {
        match self {
            Self::Left | Self::Right => KT_UP_MASK | KT_DOWN_MASK,
            Self::Up | Self::Down => KT_LEFT_MASK | KT_RIGHT_MASK,
        }
    }

    /// 線種に応じた同一軸両方向フラグを返します。
    pub fn axis_flags(&self, keisen_type: KeisenType) -> u8 {
        self.flag(keisen_type) | self.opposite().flag(keisen_type)
    }
}

/// 罫線枠内の文字列揃え方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxAlignment {
    Left,
    Center,
    Right,
}

use unicode_width::UnicodeWidthStr;
use vivivi_core::{CursorPosition, TextBuffer};

/// 罫線モード状態管理構造体。
#[derive(Debug, Clone, Default)]
pub struct KeisenMode {
    /// 罫線描画モードが有効かどうか (Shift-F5 等でトグル)
    pub is_active: bool,
    /// 描画する線種 (細線 / 太線)
    pub keisen_type: KeisenType,
}

impl KeisenMode {
    /// デフォルト設定で罫線モードを生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// 罫線描画モードの ON/OFF をトグルします。
    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }

    /// 罫線の描画または消去を1ステップ実行し、カーソルを移動します。
    ///
    /// OpenViVi の `drawKeisenLeft/Right/Up/Down` 準拠のロジックです。
    pub fn step(
        &self,
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        dir: KeisenDirection,
        is_erase: bool,
    ) {
        buffer.open_undo_block();

        // 1. 現在位置の文字の罫線状態を更新
        Self::update_keisen_at(buffer, *cursor, dir, is_erase, self.keisen_type);

        // 2. 移動先の位置を確保・計算
        let next_pos = Self::ensure_target_position(buffer, *cursor, dir);

        // 3. 移動先の罫線状態を逆方向で結合/更新
        Self::update_keisen_at(buffer, next_pos, dir.opposite(), is_erase, self.keisen_type);

        // 4. カーソルを移動先に更新
        *cursor = next_pos;

        buffer.close_undo_block();
    }

    /// 指定位置の文字の罫線接続状態を計算し、適切な文字（または空白）に置換します。
    fn update_keisen_at(
        buffer: &mut TextBuffer,
        pos: CursorPosition,
        dir: KeisenDirection,
        is_erase: bool,
        keisen_type: KeisenType,
    ) {
        let abs_pos = buffer.cursor_to_position(pos);
        let curr_ch = buffer.char_at(abs_pos).unwrap_or(' ');
        let curr_state = char_to_keisen_state(curr_ch).unwrap_or(0);
        let has_perp = (curr_state & dir.perp_mask()) != 0;

        let new_state = if is_erase {
            if !has_perp {
                curr_state & !dir.axis_mask()
            } else {
                curr_state & !dir.mask()
            }
        } else if !has_perp {
            // 直角方向の接続がない場合、直線（水平または垂直）として両方向を設定
            (curr_state & !dir.axis_mask()) | dir.axis_flags(keisen_type)
        } else {
            // 直角方向がある場合は該当方向を結合
            (curr_state & !dir.mask()) | dir.flag(keisen_type)
        };

        let replace_ch = if new_state == 0 {
            if is_erase && char_to_keisen_state(curr_ch).is_some() {
                ' '
            } else {
                curr_ch
            }
        } else {
            state_to_keisen_char(new_state).unwrap_or(curr_ch)
        };

        if replace_ch != curr_ch {
            buffer.delete_range(abs_pos, abs_pos + 1);
            buffer.insert_char(abs_pos, replace_ch);
        }
    }

    /// 移動先の座標を確保します（行末外や存在しない行への移動時に空白・改行を補完）。
    fn ensure_target_position(
        buffer: &mut TextBuffer,
        pos: CursorPosition,
        dir: KeisenDirection,
    ) -> CursorPosition {
        match dir {
            KeisenDirection::Right => {
                let target = CursorPosition::new(pos.line, pos.col + 1);
                Self::pad_line_to_col(buffer, target.line, target.col);
                target
            }
            KeisenDirection::Left => CursorPosition::new(pos.line, pos.col.saturating_sub(1)),
            KeisenDirection::Down => {
                let next_line = pos.line + 1;
                while buffer.line_count() <= next_line {
                    let end_pos = buffer.len();
                    buffer.insert_char(end_pos, '\n');
                }
                Self::pad_line_to_col(buffer, next_line, pos.col);
                CursorPosition::new(next_line, pos.col)
            }
            KeisenDirection::Up => {
                let prev_line = pos.line.saturating_sub(1);
                Self::pad_line_to_col(buffer, prev_line, pos.col);
                CursorPosition::new(prev_line, pos.col)
            }
        }
    }

    /// 指定行の文字数が `target_col` に満たない場合、スペースでパディングします。
    fn pad_line_to_col(buffer: &mut TextBuffer, line: usize, target_col: usize) {
        if let Some(line_text) = buffer.get_line(line) {
            let char_count = line_text.chars().count();
            if char_count < target_col + 1 {
                let end_of_line_pos = buffer.line_start_position(line) + char_count;
                let spaces_needed = target_col + 1 - char_count;
                let padding = " ".repeat(spaces_needed);
                buffer.insert_text(end_of_line_pos, &padding);
            }
        }
    }

    /// 罫線で囲まれた枠内の文字列を指定した揃え（左・中央・右）に整列します。
    ///
    /// 行内の左右の縦罫線（または行頭・行末）に挟まれたテキスト区間を検出し、
    /// 幅に合わせて前後の空白を再配置します。
    pub fn align_box_text(
        buffer: &mut TextBuffer,
        line_idx: usize,
        alignment: BoxAlignment,
    ) -> bool {
        let line_text = match buffer.get_line(line_idx) {
            Some(t) => t,
            None => return false,
        };

        // 縦罫線の位置（charインデックス）を探索
        let vertical_indices: Vec<usize> = line_text
            .chars()
            .enumerate()
            .filter(|(_, ch)| is_vertical_keisen(*ch))
            .map(|(i, _)| i)
            .collect();

        if vertical_indices.len() < 2 {
            return false;
        }

        buffer.open_undo_block();
        let chars: Vec<char> = line_text.chars().collect();
        let mut new_chars = chars.clone();

        for window in vertical_indices.windows(2) {
            let left_v = window[0];
            let right_v = window[1];
            if right_v <= left_v + 1 {
                continue;
            }

            let segment: String = chars[left_v + 1..right_v].iter().collect();
            let trimmed = segment.trim();
            let total_width = right_v - left_v - 1;
            let text_width = UnicodeWidthStr::width(trimmed);

            if text_width > total_width {
                continue;
            }

            let space_diff = total_width.saturating_sub(text_width);
            let (left_pad, right_pad) = match alignment {
                BoxAlignment::Left => (0, space_diff),
                BoxAlignment::Right => (space_diff, 0),
                BoxAlignment::Center => (space_diff / 2, space_diff - (space_diff / 2)),
            };

            let aligned_content = format!(
                "{}{}{}",
                " ".repeat(left_pad),
                trimmed,
                " ".repeat(right_pad)
            );
            let aligned_chars: Vec<char> = aligned_content.chars().collect();

            // 長さが一致する場合に置換
            if aligned_chars.len() == total_width {
                new_chars.splice(left_v + 1..right_v, aligned_chars);
            }
        }

        let new_line_str: String = new_chars.into_iter().collect();
        if new_line_str != line_text {
            let start = buffer.line_start_position(line_idx);
            let end = start + line_text.chars().count();
            buffer.delete_range(start, end);
            buffer.insert_text(start, &new_line_str);
        }

        buffer.close_undo_block();
        true
    }

    /// カーソル位置を囲む左右の縦罫線のカラム位置（左罫線, 右罫線）を探索します。
    pub fn find_enclosing_box_cols(
        buffer: &TextBuffer,
        pos: CursorPosition,
    ) -> Option<(usize, usize)> {
        let line_text = buffer.get_line(pos.line)?;
        let chars: Vec<char> = line_text.chars().collect();
        if chars.is_empty() {
            return None;
        }

        let mut left_col = None;
        let mut right_col = None;

        // カーソル以前の直近の縦罫線
        for i in (0..pos.col.min(chars.len())).rev() {
            if is_vertical_keisen(chars[i]) {
                left_col = Some(i);
                break;
            }
        }

        // カーソル以降の直近の縦罫線
        for (i, &ch) in chars.iter().enumerate().skip(pos.col) {
            if is_vertical_keisen(ch) {
                right_col = Some(i);
                break;
            }
        }

        match (left_col, right_col) {
            (Some(l), Some(r)) if l < r => Some((l, r)),
            _ => None,
        }
    }

    /// 罫線枠保護モードでの文字挿入。
    ///
    /// 枠線内にカーソルがある場合、カーソルから右枠線までの間にある末尾の空白文字を削ることで、
    /// 外側の枠線位置を保ったまま文字を挿入します。空白の余白がない場合は挿入を拒否（false）します。
    pub fn insert_char_protected(
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        ch: char,
    ) -> bool {
        if let Some((left_col, right_col)) = Self::find_enclosing_box_cols(buffer, *cursor)
            && cursor.col > left_col
            && cursor.col < right_col
        {
            let line_text = match buffer.get_line(cursor.line) {
                Some(t) => t,
                None => return false,
            };
            let chars: Vec<char> = line_text.chars().collect();

            // カーソル位置から右罫線直前までの範囲で、末尾側の空白を探索
            let mut space_idx = None;
            for i in (cursor.col..right_col).rev() {
                if chars[i] == ' ' || chars[i] == '　' {
                    space_idx = Some(i);
                    break;
                }
            }

            if let Some(space_col) = space_idx {
                buffer.open_undo_block();
                let line_start = buffer.line_start_position(cursor.line);

                // 1. 空白を削除
                buffer.delete_range(line_start + space_col, line_start + space_col + 1);

                // 2. カーソル位置に文字を挿入
                let insert_pos = line_start + cursor.col;
                buffer.insert_char(insert_pos, ch);

                cursor.col += 1;
                buffer.close_undo_block();
                return true;
            } else {
                // 枠内に空白の余白がないため、枠線を壊さないよう挿入拒否
                return false;
            }
        }

        // 枠外の場合は通常の文字挿入
        let abs_pos = buffer.cursor_to_position(*cursor);
        buffer.insert_char(abs_pos, ch);
        cursor.col += 1;
        true
    }

    /// 罫線枠保護モードでの文字削除（BackSpaceまたはDelete）。
    ///
    /// 枠線内にカーソルがある場合、文字を削除した分だけ右枠線の直前に半角スペースを補填し、
    /// 外側の枠線位置をキープします。
    pub fn delete_char_protected(
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        is_backspace: bool,
    ) -> bool {
        if let Some((left_col, right_col)) = Self::find_enclosing_box_cols(buffer, *cursor) {
            let delete_col = if is_backspace {
                if cursor.col <= left_col + 1 {
                    return false; // 左枠線は削除させない
                }
                cursor.col - 1
            } else {
                if cursor.col >= right_col {
                    return false; // 右枠線は削除させない
                }
                cursor.col
            };

            if delete_col > left_col && delete_col < right_col {
                buffer.open_undo_block();
                let line_start = buffer.line_start_position(cursor.line);

                // 1. 指定位置の文字を削除
                buffer.delete_range(line_start + delete_col, line_start + delete_col + 1);

                // 2. 右枠線の手前（right_col - 1 の位置）に空白を補填
                let pad_pos = line_start + right_col - 1;
                buffer.insert_char(pad_pos, ' ');

                if is_backspace {
                    cursor.col -= 1;
                }
                buffer.close_undo_block();
                return true;
            }
        }

        // 枠外の場合は通常の削除
        let abs_pos = buffer.cursor_to_position(*cursor);
        if is_backspace {
            if abs_pos > 0 {
                buffer.delete_range(abs_pos - 1, abs_pos);
                cursor.col = cursor.col.saturating_sub(1);
                return true;
            }
        } else if abs_pos < buffer.len() {
            buffer.delete_range(abs_pos, abs_pos + 1);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keisen_table_lookups() {
        // 水平線・垂直線
        assert_eq!(
            state_to_keisen_char(KT_LEFT_THIN | KT_RIGHT_THIN),
            Some('─')
        );
        assert_eq!(
            state_to_keisen_char(KT_LEFT_THICK | KT_RIGHT_THICK),
            Some('━')
        );
        assert_eq!(state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN), Some('│'));
        assert_eq!(state_to_keisen_char(KT_UP_THICK | KT_DOWN_THICK), Some('┃'));

        // コーナー
        assert_eq!(
            state_to_keisen_char(KT_DOWN_THIN | KT_RIGHT_THIN),
            Some('┌')
        );
        assert_eq!(
            state_to_keisen_char(KT_DOWN_THICK | KT_RIGHT_THICK),
            Some('┏')
        );
        assert_eq!(state_to_keisen_char(KT_LEFT_THIN | KT_DOWN_THIN), Some('┐'));
        assert_eq!(state_to_keisen_char(KT_UP_THIN | KT_RIGHT_THIN), Some('└'));
        assert_eq!(state_to_keisen_char(KT_LEFT_THIN | KT_UP_THIN), Some('┘'));

        // T字路・十字路
        assert_eq!(
            state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN | KT_RIGHT_THIN),
            Some('├')
        );
        assert_eq!(
            state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THIN),
            Some('┤')
        );
        assert_eq!(
            state_to_keisen_char(KT_DOWN_THIN | KT_LEFT_THIN | KT_RIGHT_THIN),
            Some('┬')
        );
        assert_eq!(
            state_to_keisen_char(KT_UP_THIN | KT_LEFT_THIN | KT_RIGHT_THIN),
            Some('┴')
        );
        assert_eq!(
            state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THIN | KT_RIGHT_THIN),
            Some('┼')
        );
        assert_eq!(
            state_to_keisen_char(KT_UP_THICK | KT_DOWN_THICK | KT_LEFT_THICK | KT_RIGHT_THICK),
            Some('╋')
        );
    }

    #[test]
    fn test_keisen_mode_toggle() {
        let mut km = KeisenMode::new();
        assert!(!km.is_active);
        km.toggle();
        assert!(km.is_active);
        km.toggle();
        assert!(!km.is_active);
    }

    #[test]
    fn test_keisen_draw_and_erase() {
        let mut buffer = TextBuffer::new();
        let mut cursor = CursorPosition::new(0, 0);
        let km = KeisenMode::new();

        // 右へ2回描画: ─ ─
        km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
        km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
        assert_eq!(cursor, CursorPosition::new(0, 2));
        assert_eq!(buffer.char_at(0), Some('─'));
        assert_eq!(buffer.char_at(1), Some('─'));

        // 下へ1回描画: 水平線 ─ の途中で下に引いたため T字路 ┬ に自動結合
        km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
        assert_eq!(cursor, CursorPosition::new(1, 2));
        assert_eq!(buffer.char_at(2), Some('┬'));

        // 消去テスト: 上へ戻りながら消去
        km.step(&mut buffer, &mut cursor, KeisenDirection::Up, true);
        assert_eq!(cursor, CursorPosition::new(0, 2));
        // 下成分が消えて水平線 ─ に綺麗に戻る
        assert_eq!(buffer.char_at(2), Some('─'));
    }

    #[test]
    fn test_align_box_text() {
        let mut buffer = TextBuffer::from_text("│ hello     │\n│    world  │");

        // 1行目を右揃えに整列
        assert!(KeisenMode::align_box_text(
            &mut buffer,
            0,
            BoxAlignment::Right
        ));
        assert_eq!(buffer.get_line(0), Some("│      hello│".to_string()));

        // 1行目を中央揃えに整列
        assert!(KeisenMode::align_box_text(
            &mut buffer,
            0,
            BoxAlignment::Center
        ));
        assert_eq!(buffer.get_line(0), Some("│   hello   │".to_string()));

        // 1行目を左揃えに整列
        assert!(KeisenMode::align_box_text(
            &mut buffer,
            0,
            BoxAlignment::Left
        ));
        assert_eq!(buffer.get_line(0), Some("│hello      │".to_string()));
    }

    #[test]
    fn test_keisen_protected_edit() {
        // "│ hello     │" (全長13文字: │, 空白, h, e, l, l, o, 空白x5, │)
        let mut buffer = TextBuffer::from_text("│ hello     │");
        let mut cursor = CursorPosition::new(0, 7); // 'o'の直後

        // '!' を挿入 -> 末尾の空白が1つ削られ、右枠線 '│' の位置は維持される
        assert!(KeisenMode::insert_char_protected(
            &mut buffer,
            &mut cursor,
            '!'
        ));
        assert_eq!(buffer.get_line(0), Some("│ hello!    │".to_string()));
        assert_eq!(cursor, CursorPosition::new(0, 8));

        // BackSpace で '!' を削除 -> 末尾に空白が補填され、右枠線の位置は維持される
        assert!(KeisenMode::delete_char_protected(
            &mut buffer,
            &mut cursor,
            true
        ));
        assert_eq!(buffer.get_line(0), Some("│ hello     │".to_string()));
        assert_eq!(cursor, CursorPosition::new(0, 7));

        // 空白に余裕がないボックスでの保護テスト
        let mut full_box_buffer = TextBuffer::from_text("│12345│");
        let mut full_cursor = CursorPosition::new(0, 3);
        // 空白がないため、枠線を壊さないよう挿入拒否される
        assert!(!KeisenMode::insert_char_protected(
            &mut full_box_buffer,
            &mut full_cursor,
            'x'
        ));
        assert_eq!(full_box_buffer.get_line(0), Some("│12345│".to_string()));
    }
}
