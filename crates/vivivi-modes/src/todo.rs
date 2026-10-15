//! ソースコード内 ToDo / 未完了タグ抽出モジュール。
//!
//! ViVi 4 の ToDo 管理機能に基づき、バッファまたはプロジェクト内のソースコードから
//! `TODO`, `FIXME`, `HACK`, `NOTE`, `UNDONE` などのコメントタグを抽出し、
//! 一覧リストおよびジャンプ用座標を生成します。

use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// 検出された ToDo タグのカテゴリ種別。
///
/// # Examples
/// ```
/// use vivivi_modes::todo::TodoTag;
///
/// let tag = TodoTag::Todo;
/// assert_eq!(tag.as_str(), "TODO");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TodoTag {
    /// 修正が必要な既知の不具合・クリティカルな問題
    Fixme,
    /// バグ箇所のマーク
    Bug,
    /// 今後実装・対応予定のタスク
    Todo,
    /// 未完了・仕掛かり中の処理
    Undone,
    /// 暫定的なハック・改善が望まれる実装
    Hack,
    /// 要注意・警告・危険なコード
    Xxx,
    /// 補足事項・留意事項のメモ
    Note,
    /// その他のカスタムまたは未分類タグ
    Other,
}

impl TodoTag {
    /// タグの標準表記文字列表現（大文字）を取得します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_modes::todo::TodoTag;
    ///
    /// assert_eq!(TodoTag::Fixme.as_str(), "FIXME");
    /// assert_eq!(TodoTag::Todo.as_str(), "TODO");
    /// ```
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

/// 抽出された単一の ToDo 項目。
///
/// ファイル名、出現行・列番号、タグの種類、およびコメントメッセージ本文を保持します。
///
/// # Examples
/// ```
/// use vivivi_modes::todo::{TodoItem, TodoTag};
///
/// let item = TodoItem {
///     file_path: Some("main.rs".to_string()),
///     lineno: 10,
///     col: 5,
///     tag: TodoTag::Todo,
///     message: "リファクタリング".to_string(),
/// };
/// assert_eq!(item.lineno, 10);
/// assert_eq!(item.tag, TodoTag::Todo);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    /// ファイルパス（単一バッファ解析時は None またはバッファ名）
    pub file_path: Option<String>,
    /// 行番号（1-based）
    pub lineno: usize,
    /// 列番号（1-based、タグの開始カラム）
    pub col: usize,
    /// タグの種別
    pub tag: TodoTag,
    /// コメント本文メッセージ（コロンやハイフン直後のテキスト）
    pub message: String,
}

static TODO_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(TODO|FIXME|BUG|UNDONE|HACK|XXX|NOTE)\s*[:\s\-]\s*(.*)$")
        .expect("regex compile")
});

/// 単一テキスト（エディタバッファ内容等）から ToDo アイテムを抽出します。
///
/// コメント行等に含まれる `TODO: ...`, `FIXME - ...` などのパターンを正規表現で走査し、
/// 行番号・列番号・種別・メッセージ本文を収集して返します。
///
/// # 引数
/// - `text`: 検索対象のテキスト文字列
/// - `file_name`: 登録用ファイル名（オプション）
///
/// # 戻り値
/// 抽出された [`TodoItem`] のベクタ
///
/// # Examples
/// ```
/// use vivivi_modes::todo::{scan_buffer_todos, TodoTag};
///
/// let code = "// TODO: エラーハンドリングの追加\nlet x = 1;";
/// let todos = scan_buffer_todos(code, Some("lib.rs"));
/// assert_eq!(todos.len(), 1);
/// assert_eq!(todos[0].tag, TodoTag::Todo);
/// assert_eq!(todos[0].message, "エラーハンドリングの追加");
/// assert_eq!(todos[0].lineno, 1);
/// ```
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

/// 指定ディレクトリ配下のソースファイルを再帰走査し、全 ToDo アイテムを抽出します。
///
/// 隠しディレクトリ（`.` 始まり）、`target`、`node_modules`、`build` ディレクトリは自動的にスキップされます。
/// `target_exts` に対象拡張子一覧（例: `&["rs", "toml"]`）を指定でき、空スライスの場合は全ファイルを対象とします。
///
/// # 引数
/// - `dir`: 走査対象のルートディレクトリパス
/// - `target_exts`: 対象とするファイル拡張子のスライス（空スライスの場合は全拡張子）
///
/// # 戻り値
/// 抽出された全 [`TodoItem`] のベクタ
///
/// # Examples
/// ```rust,no_run
/// use vivivi_modes::todo::scan_dir_todos;
///
/// let todos = scan_dir_todos("src", &["rs"]);
/// println!("見つかったTODO件数: {}", todos.len());
/// ```
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
