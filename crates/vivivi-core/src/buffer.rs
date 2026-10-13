//! Text buffer abstraction, modeled after OpenViVi's Buffer design.

use crate::cursor::CursorPosition;

/// Buffer trait and implementation providing line-based and offset-based text editing.
#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    lines: Vec<String>,
    is_modified: bool,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            is_modified: false,
        }
    }

    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        Self {
            lines: if lines.is_empty() { vec![String::new()] } else { lines },
            is_modified: false,
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.is_modified = modified;
    }

    pub fn line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    pub fn char_at(&self, pos: CursorPosition) -> Option<char> {
        let line = self.lines.get(pos.line)?;
        line.chars().nth(pos.col)
    }

    pub fn insert_char(&mut self, pos: CursorPosition, ch: char) {
        if pos.line >= self.lines.len() {
            self.lines.resize(pos.line + 1, String::new());
        }
        let line = &mut self.lines[pos.line];
        if ch == '\n' {
            let remainder = if pos.col < line.chars().count() {
                let char_idx = line.char_indices().nth(pos.col).map(|(i, _)| i).unwrap_or(line.len());
                let rem = line[char_idx..].to_string();
                line.truncate(char_idx);
                rem
            } else {
                String::new()
            };
            self.lines.insert(pos.line + 1, remainder);
        } else {
            let char_idx = line.char_indices().nth(pos.col).map(|(i, _)| i).unwrap_or(line.len());
            line.insert(char_idx, ch);
        }
        self.is_modified = true;
    }

    pub fn delete_char(&mut self, pos: CursorPosition) -> Option<char> {
        if pos.line >= self.lines.len() {
            return None;
        }
        let line = &mut self.lines[pos.line];
        let char_count = line.chars().count();
        if pos.col < char_count {
            let (idx, ch) = line.char_indices().nth(pos.col)?;
            line.remove(idx);
            self.is_modified = true;
            Some(ch)
        } else if pos.line + 1 < self.lines.len() {
            // Join with next line
            let next_line = self.lines.remove(pos.line + 1);
            self.lines[pos.line].push_str(&next_line);
            self.is_modified = true;
            Some('\n')
        } else {
            None
        }
    }

}

impl std::fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_basic() {
        let mut buf = TextBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.line_count(), 1);

        buf.insert_char(CursorPosition::new(0, 0), 'a');
        assert_eq!(buf.to_string(), "a");
        assert!(buf.is_modified());

        buf.insert_char(CursorPosition::new(0, 1), '\n');
        buf.insert_char(CursorPosition::new(1, 0), 'b');
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.to_string(), "a\nb");

        let deleted = buf.delete_char(CursorPosition::new(0, 1));
        assert_eq!(deleted, Some('\n'));
        assert_eq!(buf.to_string(), "ab");
    }
}
