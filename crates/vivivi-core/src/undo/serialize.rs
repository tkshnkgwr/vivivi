//! undofile 形式への Undo/Redo 履歴の永続化および復元モジュール。

use super::{UndoAction, UndoBlock, UndoMgr};

impl UndoMgr {
    /// Undo/Redo スタックを永続化文字列（undofile 形式）にシリアライズします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut mgr = UndoMgr::new();
    /// mgr.push_insert(0, "hello");
    /// let text = mgr.serialize();
    /// assert!(text.starts_with("VIVIVI_UNDO_V1"));
    /// ```
    pub fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str("VIVIVI_UNDO_V1\n");
        match self.save_point {
            Some(pt) => out.push_str(&format!("save_point: {}\n", pt)),
            None => out.push_str("save_point: none\n"),
        }
        out.push_str(&format!("UNDO_STACK: {}\n", self.undo_stack.len()));
        for block in &self.undo_stack {
            Self::serialize_block(&mut out, block);
        }
        out.push_str(&format!("REDO_STACK: {}\n", self.redo_stack.len()));
        for block in &self.redo_stack {
            Self::serialize_block(&mut out, block);
        }
        out
    }

    fn serialize_block(out: &mut String, block: &UndoBlock) {
        out.push_str("BLOCK_START\n");
        for action in &block.actions {
            match action {
                UndoAction::Insert {
                    pos,
                    text,
                    prohibit_merge,
                } => {
                    out.push_str(&format!(
                        "INSERT {} {} \"{}\"\n",
                        pos,
                        if *prohibit_merge { 1 } else { 0 },
                        escape_str(text)
                    ));
                }
                UndoAction::Delete {
                    pos,
                    text,
                    is_backspace,
                } => {
                    out.push_str(&format!(
                        "DELETE {} {} \"{}\"\n",
                        pos,
                        if *is_backspace { 1 } else { 0 },
                        escape_str(text)
                    ));
                }
                UndoAction::Replace {
                    pos,
                    old_text,
                    new_text,
                } => {
                    out.push_str(&format!(
                        "REPLACE {} \"{}\" \"{}\"\n",
                        pos,
                        escape_str(old_text),
                        escape_str(new_text)
                    ));
                }
            }
        }
        out.push_str("BLOCK_END\n");
    }

    /// 永続化文字列から [`UndoMgr`] を復元します。
    ///
    /// # Errors
    ///
    /// ヘッダーが `VIVIVI_UNDO_V1` でない場合や、ブロック構文の解析に失敗した場合にエラー文字列を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::UndoMgr;
    ///
    /// let mut orig = UndoMgr::new();
    /// orig.push_insert(0, "data");
    /// let serialized = orig.serialize();
    ///
    /// let restored = UndoMgr::deserialize(&serialized).unwrap();
    /// assert!(restored.can_undo());
    /// ```
    pub fn deserialize(data: &str) -> Result<Self, String> {
        let lines: Vec<&str> = data.lines().collect();
        if lines.is_empty() || lines[0].trim() != "VIVIVI_UNDO_V1" {
            return Err("不正な undofile ヘッダーです".into());
        }

        let mut mgr = Self::new();
        let mut idx = 1;

        if idx < lines.len() && lines[idx].starts_with("save_point:") {
            let val = lines[idx]["save_point:".len()..].trim();
            if val != "none" {
                mgr.save_point = Some(val.parse::<usize>().map_err(|e| e.to_string())?);
            } else {
                mgr.save_point = None;
            }
            idx += 1;
        }

        let mut reading_target = 0; // 0: undo, 1: redo
        while idx < lines.len() {
            let line = lines[idx].trim();
            idx += 1;
            if line.is_empty() {
                continue;
            }
            if line.starts_with("UNDO_STACK:") {
                reading_target = 0;
            } else if line.starts_with("REDO_STACK:") {
                reading_target = 1;
            } else if line == "BLOCK_START" {
                let mut block = UndoBlock::default();
                while idx < lines.len() {
                    let bline = lines[idx].trim();
                    idx += 1;
                    if bline == "BLOCK_END" {
                        break;
                    }
                    if bline.is_empty() {
                        continue;
                    }
                    let action = parse_action_line(bline)?;
                    block.actions.push(action);
                }
                if reading_target == 0 {
                    mgr.undo_stack.push(block);
                } else {
                    mgr.redo_stack.push(block);
                }
            }
        }
        Ok(mgr)
    }

    /// undofile を指定パスに保存します。
    ///
    /// 保存先ディレクトリが存在しない場合は自動作成されます。
    ///
    /// # Errors
    ///
    /// ファイルの作成や履歴データの書き込みに失敗した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_core::UndoMgr;
    ///
    /// let mgr = UndoMgr::new();
    /// mgr.save_to_file(".test.un~").unwrap();
    /// ```
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let serialized = self.serialize();
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serialized)
    }

    /// undofile を指定パスから読み込み復元します。
    ///
    /// # Errors
    ///
    /// ファイルが存在しない場合、読み込み権限がない場合、またはデータのパースに失敗した場合に [`std::io::Error`] を返します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use vivivi_core::UndoMgr;
    ///
    /// let mgr = UndoMgr::load_from_file(".test.un~").unwrap();
    /// ```
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::deserialize(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape_str(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => return Err("末尾の不正なエスケープシーケンスです".into()),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// クォートで囲まれた文字列を抽出します。
fn extract_quotes(line: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_quote = false;
    let mut escape = false;
    let mut current = String::new();

    for c in line.chars() {
        if escape {
            current.push('\\');
            current.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            if in_quote {
                in_quote = false;
                results.push(current.clone());
                current.clear();
            } else {
                in_quote = true;
                current.clear();
            }
            continue;
        }
        if in_quote {
            current.push(c);
        }
    }
    results
}

fn parse_action_line(line: &str) -> Result<UndoAction, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("空の行です".into());
    }

    let quotes = extract_quotes(line);

    match parts[0] {
        "INSERT" => {
            if parts.len() < 3 || quotes.is_empty() {
                return Err(format!("不正な INSERT 行: {}", line));
            }
            let pos = parts[1].parse::<usize>().map_err(|e| e.to_string())?;
            let prohibit_merge = parts[2] == "1";
            let text = unescape_str(&quotes[0])?;
            Ok(UndoAction::Insert {
                pos,
                text,
                prohibit_merge,
            })
        }
        "DELETE" => {
            if parts.len() < 3 || quotes.is_empty() {
                return Err(format!("不正な DELETE 行: {}", line));
            }
            let pos = parts[1].parse::<usize>().map_err(|e| e.to_string())?;
            let is_backspace = parts[2] == "1";
            let text = unescape_str(&quotes[0])?;
            Ok(UndoAction::Delete {
                pos,
                text,
                is_backspace,
            })
        }
        "REPLACE" => {
            if quotes.len() < 2 {
                return Err(format!("不正な REPLACE 行: {}", line));
            }
            let pos = parts[1].parse::<usize>().map_err(|e| e.to_string())?;
            let old_text = unescape_str(&quotes[0])?;
            let new_text = unescape_str(&quotes[1])?;
            Ok(UndoAction::Replace {
                pos,
                old_text,
                new_text,
            })
        }
        other => Err(format!("未知の Undo アクション: {}", other)),
    }
}
