//! Keisen (Box Drawing) mode faithfully ported from OpenViVi's `keisen.cpp`.

pub const KT_CODE_BEG: u32 = 0x2500;
pub const KT_CODE_END: u32 = 0x254b;

pub const KT_UP_MASK: u8 = 0xc0;
pub const KT_LEFT_MASK: u8 = 0x30;
pub const KT_DOWN_MASK: u8 = 0x0c;
pub const KT_RIGHT_MASK: u8 = 0x03;

pub const KT_UP_THICK: u8 = 0x80;
pub const KT_UP_THIN: u8 = 0x40;
pub const KT_LEFT_THICK: u8 = 0x20;
pub const KT_LEFT_THIN: u8 = 0x10;
pub const KT_DOWN_THICK: u8 = 0x08;
pub const KT_DOWN_THIN: u8 = 0x04;
pub const KT_RIGHT_THICK: u8 = 0x02;
pub const KT_RIGHT_THIN: u8 = 0x01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeisenType {
    #[default]
    Thin,
    Thick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeisenDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Lookup table mapping (codepoint - 0x2500) to state bitmask, ported from OpenViVi.
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

/// Finds the box drawing char that matches the bitmask state.
pub fn state_to_keisen_char(state: u8) -> Option<char> {
    for (i, &s) in KEISEN_TABLE.iter().enumerate() {
        if s == state && s != 0 {
            return char::from_u32(KT_CODE_BEG + i as u32);
        }
    }
    None
}

/// KeisenMode state and operations.
#[derive(Debug, Clone, Default)]
pub struct KeisenMode {
    pub is_active: bool,
    pub keisen_type: KeisenType,
}

impl KeisenMode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keisen_table_lookups() {
        assert_eq!(state_to_keisen_char(KT_LEFT_THIN | KT_RIGHT_THIN), Some('─'));
        assert_eq!(state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN), Some('│'));
        assert_eq!(state_to_keisen_char(KT_DOWN_THIN | KT_RIGHT_THIN), Some('┌'));
        assert_eq!(state_to_keisen_char(KT_UP_THIN | KT_DOWN_THIN | KT_LEFT_THIN | KT_RIGHT_THIN), Some('┼'));
    }
}
