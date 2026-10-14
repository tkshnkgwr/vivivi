//! ソースコード内 ToDo / 未完了タグ抽出モジュール。
//!
//! ViVi 4 の ToDo 管理機能に基づき、バッファまたはプロジェクト内のソースコードから
//! `TODO`, `FIXME`, `HACK`, `NOTE`, `UNDONE` などのコメントタグを抽出し、
//! 一覧リストおよびジャンプ用座標を生成します。

use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// ToDo タグの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TodoTag {
    Fixme,
    Bug,
    Todo,
    Undone,
    Hack,
    Xxx,
    Note,
    Other,
}

impl TodoTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fixme => "FIXME",
            Self::Bug => "BUG",
            Self::Todo => "TODO",
            Self::Undone => "UNDONE",
            Self::Hack => "HACK",
            Self::Xxx => "XXX",
            Self::Note => "NOTE",
            Self::Other => "OTHER",
        }
    }
}

/// 抽出された ToDo アイテム。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    /// ファイルパス（単一バッファ解析時は None またはバッファ名）
    pub file_path: Option<String>,
    /// 行番号（1-indexed）
    pub lineno: usize,
    /// 列番号（1-indexed）
    pub col: usize,
    /// タグ種別
    pub tag: TodoTag,
    /// 本文メッセージ
    pub message: String,
}

static TODO_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(TODO|FIXME|BUG|UNDONE|HACK|XXX|NOTE)\s*[:\s\-]\s*(.*)$")
        .expect("regex compile")
});

/// 単一テキスト（バッファ内容）から ToDo アイテムを抽出します。
pub fn scan_buffer_todos(text: &str, file_name: Option<&str>) -> Vec<TodoItem> {
    let mut items = Vec::new();

    for (i, line) in text.lines().enumerate() {
        if let Some(caps) = TODO_REGEX.captures(line) {
            let matched_tag = caps
                .get(1)
                .map(|m| m.as_str().to_uppercase())
                .unwrap_or_default();
            let message = caps
                .get(2)
                .map(|m| m.as_str().trim())
                .unwrap_or("")
                .to_string();
            let col = caps.get(1).map(|m| m.start() + 1).unwrap_or(1);

            let tag = match matched_tag.as_str() {
                "FIXME" => TodoTag::Fixme,
                "BUG" => TodoTag::Bug,
                "TODO" => TodoTag::Todo,
                "UNDONE" => TodoTag::Undone,
                "HACK" => TodoTag::Hack,
                "XXX" => TodoTag::Xxx,
                "NOTE" => TodoTag::Note,
                _ => TodoTag::Other,
            };

            items.push(TodoItem {
                file_path: file_name.map(|s| s.to_string()),
                lineno: i + 1,
                col,
                tag,
                message,
            });
        }
    }

    items
}

/// 指定ディレクトリ配下のソースファイルから ToDo アイテムを再帰走査して抽出します。
pub fn scan_dir_todos<P: AsRef<Path>>(dir: P, target_exts: &[&str]) -> Vec<TodoItem> {
    let mut results = Vec::new();
    let dir = dir.as_ref();
    if !dir.is_dir() {
        return results;
    }

    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            // 隠しフォルダやビルドフォルダのスキップ
            if file_name.starts_with('.')
                || file_name == "target"
                || file_name == "node_modules"
                || file_name == "build"
            {
                continue;
            }

            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default()
                    .to_lowercase();
                if (target_exts.is_empty()
                    || target_exts.iter().any(|&e| e.eq_ignore_ascii_case(&ext)))
                    && let Ok(content) = fs::read_to_string(&path)
                {
                    let path_str = path.to_string_lossy();
                    let items = scan_buffer_todos(&content, Some(&path_str));
                    results.extend(items);
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_buffer_todos() {
        let code = r#"
// normal line
// TODO: implement feature X
fn add(a: i32, b: i32) -> i32 {
    /* FIXME: handle overflow properly */
    // UNDONE - add validation
    a + b
}
"#;
        let items = scan_buffer_todos(code, Some("calc.rs"));
        assert_eq!(items.len(), 3);

        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].message, "implement feature X");
        assert_eq!(items[0].lineno, 3);

        assert_eq!(items[1].tag, TodoTag::Fixme);
        assert!(items[1].message.starts_with("handle overflow"));

        assert_eq!(items[2].tag, TodoTag::Undone);
        assert_eq!(items[2].message, "add validation");
    }
}
