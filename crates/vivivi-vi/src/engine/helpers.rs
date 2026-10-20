//! ViEngine の特定機能ヘルパーメソッド群（インクリメント、真偽値トグル、上行コピー、タグジャンプ）。

use super::{ViEngine, ViExecutionResult};
use vivivi_core::{TextBuffer, TextCursor};

impl ViEngine {
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
                    num_start = Some(i);
                    let mut e = if chars[i] == '-' { i + 1 } else { i };
                    while e < chars.len() && chars[e].is_ascii_digit() {
                        e += 1;
                    }
                    num_end = Some(e);
                    break;
                }
            }
        }

        if let (Some(start), Some(end)) = (num_start, num_end) {
            let num_str: String = chars[start..end].iter().collect();
            if let Ok(val) = num_str.parse::<i64>() {
                let new_val = val.saturating_add(delta);
                let new_str = new_val.to_string();

                let line_start_pos = buffer.line_start_position(line_idx);
                let replace_pos = line_start_pos + start;

                buffer.open_undo_block();
                buffer.delete_text(replace_pos, end - start, false);
                buffer.insert_text(replace_pos, &new_str);
                buffer.close_undo_block();

                cursor.pos.col = start + new_str.chars().count().saturating_sub(1);
                self.last_edit_pos = Some(cursor.pos);
                result.need_redraw = true;
                return;
            }
        }

        // 2. 数値が見つからない場合、カーソル文字が英字ならアルファベットのインクリメント/デクリメント
        let ch = chars[col];
        if ch.is_ascii_alphabetic() {
            let is_upper = ch.is_ascii_uppercase();
            let base = if is_upper { b'A' } else { b'a' };
            let current_offset = (ch as u8) - base;
            let new_offset = ((current_offset as i64 + delta).rem_euclid(26)) as u8;
            let new_ch = (base + new_offset) as char;

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
