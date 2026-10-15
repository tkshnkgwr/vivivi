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
///
/// 細線（Thin: `─`, `│`, `┌` など）と太線（Thick: `━`, `┃`, `┏` など）を切り替えます。
///
/// # Examples
///
/// ```
/// use vivivi_modes::keisen::KeisenType;
///
/// let default_type = KeisenType::default();
/// assert_eq!(default_type, KeisenType::Thin);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeisenType {
    #[default]
    Thin,
    Thick,
}

/// 罫線を引く、または探索する4方向。
///
/// # Examples
///
/// ```
/// use vivivi_modes::keisen::KeisenDirection;
///
/// assert_eq!(KeisenDirection::Up.opposite(), KeisenDirection::Down);
/// assert_eq!(KeisenDirection::Left.opposite(), KeisenDirection::Right);
/// ```
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
///
/// 与えられた文字が Unicode 罫線ブロック（U+2500〜U+254B）に含まれ、
/// かつ静的テーブルに登録されている場合に、上下左右の接続状態を示す8ビットマスクを返します。
/// 罫線文字でない場合は `None` を返します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::keisen::{char_to_keisen_state, KT_LEFT_THIN, KT_RIGHT_THIN};
///
/// let state = char_to_keisen_state('─').unwrap();
/// assert_eq!(state, KT_LEFT_THIN | KT_RIGHT_THIN);
/// assert_eq!(char_to_keisen_state('A'), None);
/// ```
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

/// 文字が垂直方向（上または下）の罫線接続を持つか判定します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::keisen::is_vertical_keisen;
///
/// assert!(is_vertical_keisen('│'));
/// assert!(is_vertical_keisen('┌'));
/// assert!(!is_vertical_keisen('─'));
/// ```
pub fn is_vertical_keisen(ch: char) -> bool {
    char_to_keisen_state(ch).is_some_and(|s| s & (KT_UP_MASK | KT_DOWN_MASK) != 0)
}

/// 文字が水平方向（左または右）の罫線接続を持つか判定します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::keisen::is_horizontal_keisen;
///
/// assert!(is_horizontal_keisen('─'));
/// assert!(is_horizontal_keisen('┌'));
/// assert!(!is_horizontal_keisen('│'));
/// ```
pub fn is_horizontal_keisen(ch: char) -> bool {
    char_to_keisen_state(ch).is_some_and(|s| s & (KT_LEFT_MASK | KT_RIGHT_MASK) != 0)
}

/// 指定されたビットマスク状態に合致する Unicode 罫線文字を検索して返します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::keisen::{state_to_keisen_char, KT_DOWN_THIN, KT_RIGHT_THIN};
///
/// let ch = state_to_keisen_char(KT_DOWN_THIN | KT_RIGHT_THIN);
/// assert_eq!(ch, Some('┌'));
/// ```
pub fn state_to_keisen_char(state: u8) -> Option<char> {
    for (i, &s) in KEISEN_TABLE.iter().enumerate() {
        if s == state && s != 0 {
            return char::from_u32(KT_CODE_BEG + i as u32);
        }
    }
    None
}

impl KeisenDirection {
    /// 方向に対応するビットマスク（2ビット分）を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::{KeisenDirection, KT_UP_MASK};
    ///
    /// assert_eq!(KeisenDirection::Up.mask(), KT_UP_MASK);
    /// ```
    pub fn mask(&self) -> u8 {
        match self {
            Self::Up => KT_UP_MASK,
            Self::Down => KT_DOWN_MASK,
            Self::Left => KT_LEFT_MASK,
            Self::Right => KT_RIGHT_MASK,
        }
    }

    /// 線種に応じたビットフラグ（細線または太線）を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::{KeisenDirection, KeisenType, KT_UP_THIN, KT_UP_THICK};
    ///
    /// assert_eq!(KeisenDirection::Up.flag(KeisenType::Thin), KT_UP_THIN);
    /// assert_eq!(KeisenDirection::Up.flag(KeisenType::Thick), KT_UP_THICK);
    /// ```
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

    /// 180度逆方向を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::KeisenDirection;
    ///
    /// assert_eq!(KeisenDirection::Left.opposite(), KeisenDirection::Right);
    /// ```
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// 同一軸（水平: 左右、垂直: 上下）の両方向マスクを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::{KeisenDirection, KT_LEFT_MASK, KT_RIGHT_MASK};
    ///
    /// assert_eq!(KeisenDirection::Left.axis_mask(), KT_LEFT_MASK | KT_RIGHT_MASK);
    /// ```
    pub fn axis_mask(&self) -> u8 {
        match self {
            Self::Left | Self::Right => KT_LEFT_MASK | KT_RIGHT_MASK,
            Self::Up | Self::Down => KT_UP_MASK | KT_DOWN_MASK,
        }
    }

    /// 直角交差方向のマスクを返します。
    ///
    /// 水平方向なら上下、垂直方向なら左右のマスクを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::{KeisenDirection, KT_UP_MASK, KT_DOWN_MASK};
    ///
    /// assert_eq!(KeisenDirection::Left.perp_mask(), KT_UP_MASK | KT_DOWN_MASK);
    /// ```
    pub fn perp_mask(&self) -> u8 {
        match self {
            Self::Left | Self::Right => KT_UP_MASK | KT_DOWN_MASK,
            Self::Up | Self::Down => KT_LEFT_MASK | KT_RIGHT_MASK,
        }
    }

    /// 線種に応じた同一軸両方向フラグを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::{KeisenDirection, KeisenType, KT_LEFT_THIN, KT_RIGHT_THIN};
    ///
    /// assert_eq!(
    ///     KeisenDirection::Left.axis_flags(KeisenType::Thin),
    ///     KT_LEFT_THIN | KT_RIGHT_THIN
    /// );
    /// ```
    pub fn axis_flags(&self, keisen_type: KeisenType) -> u8 {
        self.flag(keisen_type) | self.opposite().flag(keisen_type)
    }
}

/// 罫線枠内の文字列揃え方式。
///
/// 罫線で囲まれたセル内の文字列を左寄せ、中央揃え、右寄せに再配置する際に指定します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::keisen::BoxAlignment;
///
/// let align = BoxAlignment::Center;
/// assert_eq!(align, BoxAlignment::Center);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxAlignment {
    /// 左揃え
    Left,
    /// 中央揃え
    Center,
    /// 右揃え
    Right,
}

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use vivivi_core::{CursorPosition, TextBuffer};

/// 罫線モード状態管理構造体。
///
/// カーソル移動に伴う罫線の動的接続（太線/細線対応）、枠線保護編集、
/// 枠内文字列の自動アライメント機能を提供します。
///
/// # Examples
///
/// ```
/// use vivivi_core::{CursorPosition, TextBuffer};
/// use vivivi_modes::keisen::{KeisenDirection, KeisenMode, KeisenType};
///
/// let mut buffer = TextBuffer::new();
/// let mut cursor = CursorPosition::new(0, 0);
/// let mode = KeisenMode::new();
///
/// // 右方向へ罫線を1ステップ描画
/// mode.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
/// assert_eq!(cursor.col, 1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct KeisenMode {
    /// 罫線描画モードが有効かどうか (Shift-F5 等でトグル)
    pub is_active: bool,
    /// 描画する線種 (細線 / 太線)
    pub keisen_type: KeisenType,
}

impl KeisenMode {
    /// デフォルト設定（無効状態・細線）で罫線モードを生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::{KeisenMode, KeisenType};
    ///
    /// let mode = KeisenMode::new();
    /// assert!(!mode.is_active);
    /// assert_eq!(mode.keisen_type, KeisenType::Thin);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 罫線描画モードの有効/無効をトグル反転します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::keisen::KeisenMode;
    ///
    /// let mut mode = KeisenMode::new();
    /// mode.toggle();
    /// assert!(mode.is_active);
    /// mode.toggle();
    /// assert!(!mode.is_active);
    /// ```
    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }

    /// 罫線の描画または消去を1ステップ実行し、カーソルを移動します。
    ///
    /// 現在のカーソルセルと移動先セルの接続ビットマスクを更新し、
    /// 適切なUnicode罫線文字へ置き換えます。必要に応じて末尾空白や空行を自動拡張します。
    ///
    /// # Parameters
    /// - `buffer`: 編集対象のテキストバッファ。
    /// - `cursor`: 現在のカーソル座標（操作後に進行方向へ1ステップ更新されます）。
    /// - `dir`: 描画・消去の進行方向。
    /// - `is_erase`: `true` の場合は罫線消去、`false` の場合は描画。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::{KeisenDirection, KeisenMode};
    ///
    /// let mut buffer = TextBuffer::new();
    /// let mut cursor = CursorPosition::new(0, 0);
    /// let mode = KeisenMode::new();
    ///
    /// mode.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
    /// assert_eq!(cursor.line, 1);
    /// ```
    pub fn step(
        &self,
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        dir: KeisenDirection,
        is_erase: bool,
    ) {
        buffer.open_undo_block();

        // 0. 現在位置の行・列を確実に確保
        while buffer.line_count() <= cursor.line {
            let end_pos = buffer.len();
            buffer.insert_char(end_pos, '\n');
        }
        Self::pad_line_to_col(buffer, cursor.line, cursor.col);

        // 1. 移動先の位置を確保・計算
        let next_pos = Self::ensure_target_position(buffer, *cursor, dir);

        if is_erase {
            // 消去の場合: 現在セルから進行方向、移動先セルから逆方向の接続を除去
            Self::update_keisen_cell(buffer, *cursor, None, Some(dir), self.keisen_type);
            Self::update_keisen_cell(
                buffer,
                next_pos,
                None,
                Some(dir.opposite()),
                self.keisen_type,
            );
        } else {
            // 描画の場合: 現在セルに進行方向、移動先セルに逆方向の接続を追加
            Self::update_keisen_cell(buffer, *cursor, Some(dir), None, self.keisen_type);
            Self::update_keisen_cell(
                buffer,
                next_pos,
                Some(dir.opposite()),
                None,
                self.keisen_type,
            );
        }

        // 2. カーソルを移動先に更新
        *cursor = next_pos;

        buffer.close_undo_block();
    }

    /// セル `pos` の罫線状態を、隣接セルとの接続関係に基づいて精密に更新します。
    fn update_keisen_cell(
        buffer: &mut TextBuffer,
        pos: CursorPosition,
        add_dir: Option<KeisenDirection>,
        remove_dir: Option<KeisenDirection>,
        keisen_type: KeisenType,
    ) {
        let abs_pos = buffer.cursor_to_position(pos);
        let curr_ch = buffer.char_at(abs_pos).unwrap_or(' ');

        // 隣接セルからの本物の接続を取得
        let neighbors = Self::neighbor_connections(buffer, pos, keisen_type);

        // 新しい状態の計算
        let new_state = if let Some(dir) = add_dir {
            // 現在の隣接接続 ＋ 新たに追加する接続
            neighbors | dir.flag(keisen_type)
        } else if let Some(dir) = remove_dir {
            // 指定方向を消去
            neighbors & !dir.mask()
        } else {
            neighbors
        };

        let replace_ch = if new_state == 0 {
            if char_to_keisen_state(curr_ch).is_some() {
                ' '
            } else {
                curr_ch
            }
        } else if let Some(ch) = state_to_keisen_char(new_state) {
            ch
        } else {
            // 接続が1方向のみ（端点）の場合、直線の罫線文字を表示
            let is_horizontal = (new_state & (KT_LEFT_MASK | KT_RIGHT_MASK)) != 0;
            let is_vertical = (new_state & (KT_UP_MASK | KT_DOWN_MASK)) != 0;
            if is_horizontal && !is_vertical {
                state_to_keisen_char(KeisenDirection::Right.axis_flags(keisen_type)).unwrap_or('─')
            } else if is_vertical && !is_horizontal {
                state_to_keisen_char(KeisenDirection::Down.axis_flags(keisen_type)).unwrap_or('│')
            } else {
                curr_ch
            }
        };

        if replace_ch != curr_ch {
            buffer.delete_range(abs_pos, abs_pos + 1);
            buffer.insert_char(abs_pos, replace_ch);
        }
    }

    /// 指定位置の隣接4セルから、こちら側に向かって接続している方向ビットマスクを取得します。
    ///
    /// 周囲のセル（上・下・左・右）が現在位置に対して接続を持つ罫線文字であるかを検査し、
    /// 現在セルが接続すべき方向のビットフラグを合成して返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::{KeisenMode, KeisenType};
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│\n \n");
    /// // 2行目の位置 (1, 0) から見て上の行 (0, 0) に垂直罫線がある
    /// let mask = KeisenMode::neighbor_connections(&buffer, CursorPosition::new(1, 0), KeisenType::Thin);
    /// assert_ne!(mask, 0);
    /// ```
    pub fn neighbor_connections(
        buffer: &TextBuffer,
        pos: CursorPosition,
        keisen_type: KeisenType,
    ) -> u8 {
        let mut mask = 0u8;

        // 上のセルが下への接続を持っているか
        if pos.line > 0 {
            let up_state = Self::get_cell_keisen_state(buffer, pos.line - 1, pos.col);
            if (up_state & (KT_DOWN_THIN | KT_DOWN_THICK)) != 0 {
                mask |= KeisenDirection::Up.flag(keisen_type);
            }
        }

        // 下のセルが上への接続を持っているか
        let down_state = Self::get_cell_keisen_state(buffer, pos.line + 1, pos.col);
        if (down_state & (KT_UP_THIN | KT_UP_THICK)) != 0 {
            mask |= KeisenDirection::Down.flag(keisen_type);
        }

        // 左のセルが右への接続を持っているか
        if pos.col > 0 {
            let left_state = Self::get_cell_keisen_state(buffer, pos.line, pos.col - 1);
            if (left_state & (KT_RIGHT_THIN | KT_RIGHT_THICK)) != 0 {
                mask |= KeisenDirection::Left.flag(keisen_type);
            }
        }

        // 右のセルが左への接続を持っているか
        let right_state = Self::get_cell_keisen_state(buffer, pos.line, pos.col + 1);
        if (right_state & (KT_LEFT_THIN | KT_LEFT_THICK)) != 0 {
            mask |= KeisenDirection::Right.flag(keisen_type);
        }

        mask
    }

    /// 指定セルの罫線状態ビットマスクを取得します（範囲外や空白なら 0）。
    ///
    /// 行番号や列番号が存在しない範囲を指している場合、またはセルが罫線文字ではない場合は `0` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    /// use vivivi_modes::keisen::{KeisenMode, KT_LEFT_THIN, KT_RIGHT_THIN};
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "─");
    /// let state = KeisenMode::get_cell_keisen_state(&buffer, 0, 0);
    /// assert_eq!(state, KT_LEFT_THIN | KT_RIGHT_THIN);
    /// ```
    pub fn get_cell_keisen_state(buffer: &TextBuffer, line: usize, col: usize) -> u8 {
        if line >= buffer.line_count() {
            return 0;
        }
        let line_text = match buffer.get_line(line) {
            Some(t) => t,
            None => return 0,
        };
        let chars: Vec<char> = line_text.chars().collect();
        if col >= chars.len() {
            return 0;
        }
        char_to_keisen_state(chars[col]).unwrap_or(0)
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
    ///
    /// # Returns
    /// 整列処理が実行された場合は `true`、縦罫線が2本未満など整列対象が存在しない場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::TextBuffer;
    /// use vivivi_modes::keisen::{BoxAlignment, KeisenMode};
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ hello   │");
    /// KeisenMode::align_box_text(&mut buffer, 0, BoxAlignment::Right);
    /// assert_eq!(buffer.get_line(0).unwrap(), "│    hello│");
    /// ```
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
    ///
    /// 左右に縦罫線が存在し、カーソルがその内部にある場合に `Some((左列, 右列))` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::KeisenMode;
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ test │");
    /// let cols = KeisenMode::find_enclosing_box_cols(&buffer, CursorPosition::new(0, 3));
    /// assert_eq!(cols, Some((0, 7)));
    /// ```
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
    /// 外側の枠線位置を保ったまま文字を挿入します。空白の余白がない場合は枠の崩れを防ぐため挿入を拒否（`false`）します。
    /// 枠外の場合は通常の文字挿入を行います。
    ///
    /// # Returns
    /// 挿入に成功した場合は `true`、枠内に余白がなく拒否された場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::KeisenMode;
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ a   │");
    /// let mut cursor = CursorPosition::new(0, 3);
    /// assert!(KeisenMode::insert_char_protected(&mut buffer, &mut cursor, 'b'));
    /// assert_eq!(buffer.get_line(0).unwrap(), "│ ab  │");
    ///
    /// // 全角文字（幅2）の入力時は末尾の半角スペース2文字分を自動消費して枠幅を完全維持
    /// let mut cursor_zenkaku = CursorPosition::new(0, 1);
    /// assert!(KeisenMode::insert_char_protected(&mut buffer, &mut cursor_zenkaku, 'あ'));
    /// assert_eq!(buffer.get_line(0).unwrap(), "│あ ab│");
    /// ```
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
            let char_w = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);

            // カーソル位置から右罫線直前までの範囲で、末尾側から必要な幅分の空白を探索
            let mut collected_spaces: Vec<(usize, usize)> = Vec::new(); // (col, width)
            let mut accumulated_width = 0;

            for i in (cursor.col..right_col).rev() {
                let c = chars[i];
                if c == ' ' {
                    collected_spaces.push((i, 1));
                    accumulated_width += 1;
                } else if c == '　' {
                    collected_spaces.push((i, 2));
                    accumulated_width += 2;
                }
                if accumulated_width >= char_w {
                    break;
                }
            }

            if accumulated_width >= char_w {
                buffer.open_undo_block();
                let line_start = buffer.line_start_position(cursor.line);

                // 1. 空白を末尾側から削除 (インデックスを保持するため降順に削除)
                for &(col, _) in &collected_spaces {
                    buffer.delete_range(line_start + col, line_start + col + 1);
                }

                // 2. 削りすぎた幅（全角スペースを半角文字で消費した場合等）があれば半角スペースを補填
                let extra_width = accumulated_width - char_w;
                let adjusted_right_col = right_col - collected_spaces.len();
                for _ in 0..extra_width {
                    buffer.insert_char(line_start + adjusted_right_col, ' ');
                }

                // 3. カーソル位置に文字を挿入
                let insert_pos = line_start + cursor.col;
                buffer.insert_char(insert_pos, ch);

                cursor.col += 1;
                buffer.close_undo_block();
                return true;
            } else {
                // 枠内に十分な余白がないため、枠線を壊さないよう挿入拒否
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
    /// 枠線内にカーソルがある場合、文字を削除した分（全角文字なら2文字分、半角文字なら1文字分）だけ
    /// 右枠線の直前に空白を自動補填し、外側の枠線位置を保ちます。枠線の文字自体は誤消去されないよう保護されます。
    ///
    /// # Returns
    /// 削除に成功した場合は `true`、枠線自体の削除試行など無効な操作の場合は `false` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::{CursorPosition, TextBuffer};
    /// use vivivi_modes::keisen::KeisenMode;
    ///
    /// let mut buffer = TextBuffer::new();
    /// buffer.insert_text(0, "│ a b │");
    /// let mut cursor = CursorPosition::new(0, 5); // 'b' の直後
    /// assert!(KeisenMode::delete_char_protected(&mut buffer, &mut cursor, true));
    /// assert_eq!(buffer.get_line(0).unwrap(), "│ a   │");
    /// ```
    pub fn delete_char_protected(
        buffer: &mut TextBuffer,
        cursor: &mut CursorPosition,
        is_backspace: bool,
    ) -> bool {
        if let Some((left_col, right_col)) = Self::find_enclosing_box_cols(buffer, *cursor) {
            let line_text = match buffer.get_line(cursor.line) {
                Some(t) => t,
                None => return false,
            };
            let chars: Vec<char> = line_text.chars().collect();

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

            if delete_col > left_col && delete_col < right_col && delete_col < chars.len() {
                let del_ch = chars[delete_col];
                let del_width = UnicodeWidthChar::width(del_ch).unwrap_or(1).max(1);

                buffer.open_undo_block();
                let line_start = buffer.line_start_position(cursor.line);

                // 1. 指定位置の文字を削除
                buffer.delete_range(line_start + delete_col, line_start + delete_col + 1);

                // 2. 削除した文字の表示幅分（半角なら1、全角なら2）右枠線の手前に空白を補填
                let pad_pos = line_start + right_col - 1;
                for _ in 0..del_width {
                    buffer.insert_char(pad_pos, ' ');
                }

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

        // 下へ1回描画: 水平線の端点 (0, 2) から下へ曲がったため右上角「┐」に結合
        km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
        assert_eq!(cursor, CursorPosition::new(1, 2));
        assert_eq!(buffer.char_at(2), Some('┐'));

        // 消去テスト: 上へ戻りながら消去
        km.step(&mut buffer, &mut cursor, KeisenDirection::Up, true);
        assert_eq!(cursor, CursorPosition::new(0, 2));
        // 下成分が消えて水平線 ─ に綺麗に戻る
        assert_eq!(buffer.char_at(2), Some('─'));
    }

    #[test]
    fn test_keisen_draw_box_clean_corners() {
        let mut buffer = TextBuffer::new();
        let mut cursor = CursorPosition::new(0, 0);
        let km = KeisenMode::new();

        // 1. (0, 0) から右へ 2回描画: ─ ─
        km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
        km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
        assert_eq!(cursor, CursorPosition::new(0, 2));

        // 2. (0, 2) から下へ 2回描画:
        // (0, 2) は右上角「┐」となり、右へ突き出た「┬」にはならない！
        km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
        assert_eq!(buffer.char_at(2), Some('┐'));

        km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
        assert_eq!(cursor, CursorPosition::new(2, 2));

        // 3. (2, 2) から左へ 2回描画:
        // (2, 2) は右下角「┘」となり、下へ突き出た「┤」にはならない！
        km.step(&mut buffer, &mut cursor, KeisenDirection::Left, false);
        assert_eq!(cursor, CursorPosition::new(2, 1));
        let pos_2_2 = buffer.cursor_to_position(CursorPosition::new(2, 2));
        assert_eq!(buffer.char_at(pos_2_2), Some('┘'));

        km.step(&mut buffer, &mut cursor, KeisenDirection::Left, false);
        assert_eq!(cursor, CursorPosition::new(2, 0));

        // 4. (2, 0) から上へ 2回描画して四角を閉じる:
        // (2, 0) は左下角「└」となり、左へ突き出た「┴」にはならない！
        km.step(&mut buffer, &mut cursor, KeisenDirection::Up, false);
        let pos_2_0 = buffer.cursor_to_position(CursorPosition::new(2, 0));
        assert_eq!(buffer.char_at(pos_2_0), Some('└'));

        km.step(&mut buffer, &mut cursor, KeisenDirection::Up, false);
        assert_eq!(cursor, CursorPosition::new(0, 0));
        // (0, 0) は左上角「┌」となり、上へ突き出た「├」にはならない！
        assert_eq!(buffer.char_at(0), Some('┌'));

        // 四角形全体の完成形状を確認（糸のほつれが一切ないこと）
        assert_eq!(buffer.get_line(0), Some("┌─┐".to_string()));
        assert_eq!(buffer.get_line(1), Some("│ │".to_string()));
        assert_eq!(buffer.get_line(2), Some("└─┘".to_string()));
    }

    #[test]
    fn test_keisen_draw_down_extends_lines() {
        // 最終行（初期は1行）から下へ罫線を引いた場合に行が自動増加することを確認
        let mut buffer = TextBuffer::new();
        let mut cursor = CursorPosition::new(0, 0);
        let km = KeisenMode::new();

        assert_eq!(buffer.line_count(), 1);
        km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
        assert_eq!(cursor, CursorPosition::new(1, 0));
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.get_line(0), Some("│".to_string()));
        assert_eq!(buffer.get_line(1), Some("│".to_string()));

        km.step(&mut buffer, &mut cursor, KeisenDirection::Down, false);
        assert_eq!(cursor, CursorPosition::new(2, 0));
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(2), Some("│".to_string()));

        // さらに右へ引いた場合は下へ突き抜けず綺麗な左下角「└─」として結合
        km.step(&mut buffer, &mut cursor, KeisenDirection::Right, false);
        assert_eq!(cursor, CursorPosition::new(2, 1));
        assert_eq!(buffer.get_line(2), Some("└─".to_string()));
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

        // 全角文字の挿入テスト: 全角「あ」（幅2）を挿入 -> 末尾の空白が2つ削られ、右枠線の位置と全体の表示幅は維持される
        assert!(KeisenMode::insert_char_protected(
            &mut buffer,
            &mut cursor,
            'あ'
        ));
        assert_eq!(buffer.get_line(0), Some("│ helloあ   │".to_string()));
        assert_eq!(cursor, CursorPosition::new(0, 8));

        // BackSpace で全角「あ」を削除 -> 末尾に空白が2つ補填され、右枠線の位置と表示幅が維持される
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

        // 余白が1文字分（半角スペース1個）しかない場合、全角文字（幅2）は挿入拒否される
        let mut tight_box_buffer = TextBuffer::from_text("│1234 │");
        let mut tight_cursor = CursorPosition::new(0, 3);
        assert!(!KeisenMode::insert_char_protected(
            &mut tight_box_buffer,
            &mut tight_cursor,
            'あ'
        ));
        assert_eq!(tight_box_buffer.get_line(0), Some("│1234 │".to_string()));
    }
}
