//! 不可視文字・空白文字の可視化 (list / listchars) モジュール。
//!
//! 全角スペース (U+3000)、半角スペース、タブ (\t)、行末記号 (EOL) を
//! 視覚的に識別可能な記号に置換・フォーマットします。

/// 不可視文字表示の設定。
///
/// タブ、半角スペース、全角スペース（`\u{3000}`）、行末改行記号（EOL）の可視化記号を設定します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::invisible::ListCharsConfig;
///
/// let config = ListCharsConfig {
///     enabled: true,
///     space: '·',
///     ..Default::default()
/// };
/// assert!(config.enabled);
/// assert_eq!(config.space, '·');
/// ```
#[derive(Debug, Clone)]
pub struct ListCharsConfig {
    /// 不可視文字表示が有効かどうか (:set list / :set nolist)
    pub enabled: bool,
    /// タブ記号 (先頭文字, 埋め文字)。デフォルト: ('▸', '-')
    pub tab: (char, char),
    /// 半角スペース記号。デフォルト: '·'
    pub space: char,
    /// 全角スペース記号。デフォルト: "□" (U+25A1)
    pub zenkaku_space: &'static str,
    /// 行末改行記号。デフォルト: '$'
    pub eol: char,
    /// 通常スペースもすべて可視化するか、末尾スペースのみ可視化するか
    pub show_all_spaces: bool,
}

impl Default for ListCharsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tab: ('▸', '-'),
            space: '·',
            zenkaku_space: "□",
            eol: '$',
            show_all_spaces: false,
        }
    }
}

/// 不可視文字可視化後の1文字あたりの描画情報。
///
/// # Examples
///
/// ```
/// use vivivi_modes::invisible::InvisibleChar;
///
/// let ch = InvisibleChar {
///     ch: '·',
///     is_invisible: true,
///     is_zenkaku_space: false,
/// };
/// assert!(ch.is_invisible);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvisibleChar {
    /// 表示する文字
    pub ch: char,
    /// 不可視文字表示用スタイル（薄い文字色や全角警告色）を適用すべきか
    pub is_invisible: bool,
    /// 全角スペースなどの警告文字であるか
    pub is_zenkaku_space: bool,
}

/// 1行のテキストを不可視文字設定に従って変換し、描画用文字リストを生成します。
///
/// タブ文字の空白桁パディング展開や、全角スペース・末尾空白のハイライト用フラグ付与を行います。
///
/// # Examples
///
/// ```
/// use vivivi_modes::invisible::{format_line_invisibles, ListCharsConfig};
///
/// let config = ListCharsConfig {
///     enabled: true,
///     show_all_spaces: true,
///     ..Default::default()
/// };
/// let formatted = format_line_invisibles(" a ", 4, &config);
/// assert_eq!(formatted[0].ch, '·');
/// assert!(formatted[0].is_invisible);
/// assert_eq!(formatted[1].ch, 'a');
/// assert!(!formatted[1].is_invisible);
/// ```
pub fn format_line_invisibles(
    line: &str,
    tab_width: usize,
    config: &ListCharsConfig,
) -> Vec<InvisibleChar> {
    if !config.enabled {
        return line
            .chars()
            .map(|ch| InvisibleChar {
                ch,
                is_invisible: false,
                is_zenkaku_space: false,
            })
            .collect();
    }

    let mut result = Vec::new();
    let chars: Vec<char> = line.chars().collect();

    // 末尾空白の開始インデックスを検出
    let trailing_space_start = chars
        .iter()
        .rposition(|&c| c != ' ' && c != '\u{3000}' && c != '\t')
        .map(|idx| idx + 1)
        .unwrap_or(0);

    let mut col = 0;
    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '\t' => {
                let spaces_count = tab_width - (col % tab_width);
                result.push(InvisibleChar {
                    ch: config.tab.0,
                    is_invisible: true,
                    is_zenkaku_space: false,
                });
                for _ in 1..spaces_count {
                    result.push(InvisibleChar {
                        ch: config.tab.1,
                        is_invisible: true,
                        is_zenkaku_space: false,
                    });
                }
                col += spaces_count;
            }
            '\u{3000}' => {
                // 全角スペース (幅2): '□'
                for z_ch in config.zenkaku_space.chars() {
                    result.push(InvisibleChar {
                        ch: z_ch,
                        is_invisible: true,
                        is_zenkaku_space: true,
                    });
                }
                col += 2;
            }
            ' ' => {
                let is_trailing = i >= trailing_space_start;
                if config.show_all_spaces || is_trailing {
                    result.push(InvisibleChar {
                        ch: config.space,
                        is_invisible: true,
                        is_zenkaku_space: false,
                    });
                } else {
                    result.push(InvisibleChar {
                        ch: ' ',
                        is_invisible: false,
                        is_zenkaku_space: false,
                    });
                }
                col += 1;
            }
            _ => {
                result.push(InvisibleChar {
                    ch,
                    is_invisible: false,
                    is_zenkaku_space: false,
                });
                col += 1;
            }
        }
    }

    // 行末改行記号の追加
    result.push(InvisibleChar {
        ch: config.eol,
        is_invisible: true,
        is_zenkaku_space: false,
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_disabled() {
        let config = ListCharsConfig::default();
        let chars = format_line_invisibles("hello world", 4, &config);
        let s: String = chars.into_iter().map(|c| c.ch).collect();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_format_zenkaku_space_and_eol() {
        let config = ListCharsConfig {
            enabled: true,
            ..Default::default()
        };

        let chars = format_line_invisibles("テスト　コード", 4, &config);
        let s: String = chars.iter().map(|c| c.ch).collect();
        assert_eq!(s, "テスト□コード$");

        let zenkaku_items: Vec<_> = chars.iter().filter(|c| c.is_zenkaku_space).collect();
        assert_eq!(zenkaku_items.len(), 1);
    }

    #[test]
    fn test_format_tab_and_trailing_space() {
        let config = ListCharsConfig {
            enabled: true,
            ..Default::default()
        };

        let chars = format_line_invisibles("\tx  ", 4, &config);
        let s: String = chars.iter().map(|c| c.ch).collect();
        // タブ(4幅: ▸---) + x + 末尾スペース(··) + $
        assert_eq!(s, "▸---x··$");
    }
}
