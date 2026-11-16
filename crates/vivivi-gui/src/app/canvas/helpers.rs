//! 中央エディタ描画用の文字幅・折り返し計算・罫線ベクター描画ヘルパーモジュール。

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke};
use unicode_width::UnicodeWidthChar;

#[inline]
pub(super) fn compute_line_syntax_key(text: &str, lang: vivivi_modes::SyntaxLang) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    (lang as u8).hash(&mut hasher);
    hasher.finish()
}

/// 文字の視覚幅を計算します（タブは `tab_width` 境界まで展開）。
#[inline]
pub(super) fn char_display_width(ch: char, visual_col: usize, tab_width: usize) -> usize {
    if ch == '\t' {
        tab_width - (visual_col % tab_width)
    } else {
        UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
    }
}

/// 折り返し幅 `wrap_cols` に基づき、現在位置 `(sub_row, local_col)` を必要に応じて次サブ行へ折り返します。
///
/// 折り返しが発生した場合は `true` を返します。
#[inline]
pub(super) fn advance_wrap_cursor(
    sub_row: &mut usize,
    local_col: &mut usize,
    ch_cols: usize,
    wrap_cols: Option<usize>,
) -> bool {
    if let Some(max_cols) = wrap_cols
        && *local_col > 0
        && *local_col + ch_cols > max_cols
    {
        *sub_row += 1;
        *local_col = 0;
        true
    } else {
        false
    }
}

/// 1 論理行が指定された折り返し幅で占める視覚サブ行数（最低 1 行）を計算します。
pub(super) fn count_line_visual_rows(
    chars: &[char],
    tab_width: usize,
    wrap_cols: Option<usize>,
) -> usize {
    let Some(max_cols) = wrap_cols else {
        return 1;
    };
    let mut sub_row = 0usize;
    let mut local_col = 0usize;
    let mut abs_col = 0usize;
    for &ch in chars {
        let w = char_display_width(ch, abs_col, tab_width);
        advance_wrap_cursor(&mut sub_row, &mut local_col, w, Some(max_cols));
        local_col += w;
        abs_col += w;
    }
    sub_row + 1
}

/// ソフトラップ有効時の視覚行内をクリックされた位置から、対象論理行内の文字インデックスを解決します。
pub(super) fn resolve_wrapped_char_col(
    line_str: &str,
    tab_width: usize,
    wrap_cols: usize,
    target_sub_row: usize,
    target_local_col: usize,
) -> usize {
    let mut sub_row = 0usize;
    let mut local_col = 0usize;
    let mut abs_col = 0usize;
    let mut last_idx_in_target_row = 0usize;
    let mut saw_target_row = target_sub_row == 0;

    for (c_idx, ch) in line_str.chars().enumerate() {
        let w = char_display_width(ch, abs_col, tab_width);
        let wrapped = advance_wrap_cursor(&mut sub_row, &mut local_col, w, Some(wrap_cols));
        if wrapped && sub_row == target_sub_row {
            saw_target_row = true;
            last_idx_in_target_row = c_idx;
        }
        if sub_row > target_sub_row {
            return last_idx_in_target_row;
        }
        if sub_row == target_sub_row {
            if local_col + w > target_local_col {
                return c_idx;
            }
            last_idx_in_target_row = c_idx + 1;
        }
        local_col += w;
        abs_col += w;
    }

    if saw_target_row {
        last_idx_in_target_row
    } else {
        line_str.chars().count()
    }
}

/// Unicode 罫線文字（`U+2500..=U+254B`, `╭╮╯╰`）を 1 セル矩形内のベクター線分として描画します。
///
/// 日本語フォールバックフォントでは罫線グリフが全角幅として描画され列ズレや行間隙間が生じるため、
/// 文字グリフではなくセル境界いっぱいのベクター線分で描画します。
pub(super) fn draw_keisen_glyph(
    painter: &egui::Painter,
    cell_rect: Rect,
    ch: char,
    color: Color32,
) -> bool {
    let state = match vivivi_modes::keisen::char_to_keisen_state(ch) {
        Some(s) => s,
        None => match ch {
            '╭' => vivivi_modes::keisen::KT_RIGHT_THIN | vivivi_modes::keisen::KT_DOWN_THIN,
            '╮' => vivivi_modes::keisen::KT_LEFT_THIN | vivivi_modes::keisen::KT_DOWN_THIN,
            '╯' => vivivi_modes::keisen::KT_LEFT_THIN | vivivi_modes::keisen::KT_UP_THIN,
            '╰' => vivivi_modes::keisen::KT_RIGHT_THIN | vivivi_modes::keisen::KT_UP_THIN,
            _ => return false,
        },
    };
    let cx = cell_rect.center().x;
    let cy = cell_rect.center().y;
    let thin_w = (cell_rect.width() * 0.14).clamp(1.2, 2.0);
    let thick_w = (thin_w * 2.1).clamp(2.4, 4.0);

    let arm_width = |mask: u8, thin: u8, thick: u8| -> f32 {
        let bits = state & mask;
        if bits == thick {
            thick_w
        } else if bits == thin || bits != 0 {
            thin_w
        } else {
            0.0
        }
    };

    let up_w = arm_width(
        vivivi_modes::keisen::KT_UP_MASK,
        vivivi_modes::keisen::KT_UP_THIN,
        vivivi_modes::keisen::KT_UP_THICK,
    );
    let left_w = arm_width(
        vivivi_modes::keisen::KT_LEFT_MASK,
        vivivi_modes::keisen::KT_LEFT_THIN,
        vivivi_modes::keisen::KT_LEFT_THICK,
    );
    let down_w = arm_width(
        vivivi_modes::keisen::KT_DOWN_MASK,
        vivivi_modes::keisen::KT_DOWN_THIN,
        vivivi_modes::keisen::KT_DOWN_THICK,
    );
    let right_w = arm_width(
        vivivi_modes::keisen::KT_RIGHT_MASK,
        vivivi_modes::keisen::KT_RIGHT_THIN,
        vivivi_modes::keisen::KT_RIGHT_THICK,
    );

    let h_half = left_w.max(right_w) * 0.5;
    let v_half = up_w.max(down_w) * 0.5;

    if up_w > 0.0 {
        painter.line_segment(
            [Pos2::new(cx, cell_rect.top()), Pos2::new(cx, cy + h_half)],
            Stroke::new(up_w, color),
        );
    }
    if down_w > 0.0 {
        painter.line_segment(
            [
                Pos2::new(cx, cy - h_half),
                Pos2::new(cx, cell_rect.bottom()),
            ],
            Stroke::new(down_w, color),
        );
    }
    if left_w > 0.0 {
        painter.line_segment(
            [Pos2::new(cell_rect.left(), cy), Pos2::new(cx + v_half, cy)],
            Stroke::new(left_w, color),
        );
    }
    if right_w > 0.0 {
        painter.line_segment(
            [Pos2::new(cx - v_half, cy), Pos2::new(cell_rect.right(), cy)],
            Stroke::new(right_w, color),
        );
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_wrap_row_counting_and_click_resolution() {
        // wrap なし（None）では常に 1 視覚行
        let chars: Vec<char> = "1234567890".chars().collect();
        assert_eq!(count_line_visual_rows(&chars, 4, None), 1);

        // wrap 幅 5 桁では 10 文字は 2 行になる
        assert_eq!(count_line_visual_rows(&chars, 4, Some(5)), 2);
        assert_eq!(resolve_wrapped_char_col("1234567890", 4, 5, 0, 2), 2);
        assert_eq!(resolve_wrapped_char_col("1234567890", 4, 5, 1, 0), 5);
        assert_eq!(resolve_wrapped_char_col("1234567890", 4, 5, 1, 3), 8);

        // 全角文字（幅 2）が残り 1 桁に収まらない場合は手前で次行へ折り返す
        let jp_chars: Vec<char> = "aあb".chars().collect();
        assert_eq!(count_line_visual_rows(&jp_chars, 4, Some(2)), 3);
        assert_eq!(resolve_wrapped_char_col("aあb", 4, 2, 1, 0), 1);
        assert_eq!(resolve_wrapped_char_col("aあb", 4, 2, 2, 0), 2);
    }
}
