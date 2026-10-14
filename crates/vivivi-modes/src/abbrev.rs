//! 静的略語展開 (Static Abbreviation / Snippet Expansion) モジュール。
//!
//! ViVi 3 / ViVi 4 の `Ctrl-;` 略語展開に準拠し、
//! 事前定義またはファイル定義された略語キーから定型スニペット・コード雛形を一括展開します。

use std::collections::HashMap;

/// 略語マネージャ
#[derive(Debug, Clone)]
pub struct AbbrevManager {
    /// 略語キー -> 展開文字列のマップ
    table: HashMap<String, String>,
}

impl Default for AbbrevManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AbbrevManager {
    /// デフォルトの主要言語スニペットを備えた `AbbrevManager` を生成します。
    pub fn new() -> Self {
        let mut mgr = Self {
            table: HashMap::new(),
        };
        mgr.load_default_snippets();
        mgr
    }

    /// デフォルトの組み込みスニペットを読み込みます。
    fn load_default_snippets(&mut self) {
        // Rust
        self.register("fn", "fn name() {\n    \n}");
        self.register("struct", "pub struct Name {\n    \n}");
        self.register("enum", "pub enum Name {\n    \n}");
        self.register("impl", "impl Name {\n    \n}");
        self.register("test", "#[test]\nfn test_name() {\n    \n}");
        self.register("p", "println!(\"{}\", );");

        // C / C++
        self.register("inc", "#include <stdio.h>");
        self.register("inci", "#include <iostream>");
        self.register(
            "main",
            "int main(int argc, char *argv[]) {\n    return 0;\n}",
        );

        // HTML
        self.register(
            "html5",
            "<!DOCTYPE html>\n<html lang=\"ja\">\n<head>\n  <meta charset=\"UTF-8\">\n  <title>Document</title>\n</head>\n<body>\n  \n</body>\n</html>",
        );

        // Markdown / その他
        self.register("todo", "- [ ] ");
        self.register("date", "%DATE%");
    }

    /// 略語を登録します。
    pub fn register(&mut self, key: &str, expansion: &str) {
        self.table.insert(key.to_string(), expansion.to_string());
    }

    /// 略語キーに対応する展開文字列を取得します。
    pub fn expand(&self, key: &str) -> Option<&str> {
        self.table.get(key).map(|s| s.as_str())
    }

    /// テキスト行から略語辞書定義（"key = value" または "key: value"）をパースして追加読み込みします。
    pub fn load_dict_text(&mut self, text: &str) {
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }

            if let Some((k, v)) = trimmed.split_once('=') {
                self.register(k.trim(), v.trim().replace("\\n", "\n").as_str());
            } else if let Some((k, v)) = trimmed.split_once(':') {
                self.register(k.trim(), v.trim().replace("\\n", "\n").as_str());
            }
        }
    }

    /// 登録済みの略語数を返します。
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// 略語が空かどうかを返します。
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_abbrev_expansion() {
        let mgr = AbbrevManager::new();
        assert_eq!(mgr.expand("fn"), Some("fn name() {\n    \n}"));
        assert_eq!(mgr.expand("p"), Some("println!(\"{}\", );"));
        assert_eq!(mgr.expand("unknown_abbrev"), None);
    }

    #[test]
    fn test_custom_dict_loading() {
        let mut mgr = AbbrevManager::new();
        let dict = r#"
        # コメント行
        hw = Hello World!
        greet: Welcome to vivivi\nEnjoy editing!
        "#;
        mgr.load_dict_text(dict);
        assert_eq!(mgr.expand("hw"), Some("Hello World!"));
        assert_eq!(
            mgr.expand("greet"),
            Some("Welcome to vivivi\nEnjoy editing!")
        );
    }
}
