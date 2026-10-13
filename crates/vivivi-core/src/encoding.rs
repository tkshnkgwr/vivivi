//! Character encoding support (UTF-8, Shift_JIS, EUC-JP, UTF-16LE, UTF-16BE).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharEncoding {
    #[default]
    Utf8,
    Utf8Bom,
    ShiftJis,
    EucJp,
    Utf16Le,
    Utf16Be,
}

impl CharEncoding {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 (BOM)",
            Self::ShiftJis => "Shift_JIS",
            Self::EucJp => "EUC-JP",
            Self::Utf16Le => "UTF-16LE",
            Self::Utf16Be => "UTF-16BE",
        }
    }
}
