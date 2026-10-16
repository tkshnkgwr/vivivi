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
use vivivi_core::CursorPosition;

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
}

mod draw;
mod protect;

#[cfg(test)]
mod tests;
