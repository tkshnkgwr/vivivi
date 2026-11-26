//! Tree-sitter 構文木解析・AST ハイライト・スマート選択エンジン
//!
//! 多言語の抽象構文木（AST）解析エンジン `tree-sitter` を統合し、
//! ネストされたスコープや括弧・識別子の高精度シンタックスハイライト、
//! AST ノードに基づくスマート選択（拡大/縮小）、
//! およびタイピング遅延ゼロのインクリメンタルパース（差分構文解析）を提供します。

use tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree};

use crate::syntax::{SyntaxLang, SyntaxToken, TokenType};

/// 構文木解析に基づくハイライトトークン（行内カラム範囲付き）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstToken {
    /// トークン文字列
    pub text: String,
    /// 構文種別
    pub token_type: TokenType,
    /// 行内の開始列 (0-based)
    pub start_col: usize,
    /// 行内の終了列 (0-based)
    pub end_col: usize,
}

impl AstToken {
    /// 既存の [`SyntaxToken`] へ変換します。
    pub fn to_syntax_token(&self) -> SyntaxToken {
        SyntaxToken {
            text: self.text.clone(),
            token_type: self.token_type,
        }
    }
}

/// 構文木ノードに基づくテキスト選択範囲（バイト単位および行・列単位）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstSelectionRange {
    /// 開始バイトオフセット
    pub start_byte: usize,
    /// 終了バイトオフセット
    pub end_byte: usize,
    /// 開始行 (0-based)
    pub start_line: usize,
    /// 開始列 (0-based)
    pub start_col: usize,
    /// 終了行 (0-based)
    pub end_line: usize,
    /// 終了列 (0-based)
    pub end_col: usize,
}

impl AstSelectionRange {
    /// Tree-sitter の Node から選択範囲を生成します。
    pub fn from_node(node: &Node) -> Self {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        Self {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_line: start_pos.row,
            start_col: start_pos.column,
            end_line: end_pos.row,
            end_col: end_pos.column,
        }
    }
}

/// Tree-sitter 構文解析エンジン
///
/// # Examples
/// ```
/// use vivivi_modes::{SyntaxLang, TreeSitterEngine};
///
/// let mut engine = TreeSitterEngine::new();
/// assert!(engine.set_language(SyntaxLang::Rust));
/// assert!(engine.parse("fn main() { let x = 42; }"));
/// assert!(engine.has_tree());
///
/// let tokens = engine.highlight_line(0, "fn main() { let x = 42; }");
/// assert!(!tokens.is_empty());
/// ```
pub struct TreeSitterEngine {
    /// Tree-sitter パーサーインスタンス
    parser: Parser,
    /// 現在ロードされている言語
    current_lang: Option<SyntaxLang>,
    /// キャッシュされた最新の構文木 (AST)
    tree: Option<Tree>,
    /// 差分計算用に保持する直近パース済みソーステキスト
    last_source: String,
    /// スマート選択の拡大履歴スタック（縮小用）
    selection_history: Vec<AstSelectionRange>,
}

impl TreeSitterEngine {
    /// 新規 TreeSitterEngine を作成します。
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            current_lang: None,
            tree: None,
            last_source: String::new(),
            selection_history: Vec::new(),
        }
    }

    /// 言語種別に応じた Tree-sitter 言語文法を設定します。
    ///
    /// 対応言語: Rust, C/C++, Python, JavaScript/TypeScript, JSON, Markdown, SQL, PL/SQL,
    /// Java, HTML, Bash, Go, Ruby, TOML, PowerShell, YAML, XML, Batch（計 18 言語）
    ///
    /// # 戻り値
    /// 設定に成功した場合は `true`、未対応の言語の場合は `false` を返します。
    pub fn set_language(&mut self, lang: SyntaxLang) -> bool {
        let ts_lang: Option<Language> = match lang {
            SyntaxLang::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            SyntaxLang::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
            SyntaxLang::Python => Some(tree_sitter_python::LANGUAGE.into()),
            SyntaxLang::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            SyntaxLang::Json => Some(tree_sitter_json::LANGUAGE.into()),
            SyntaxLang::Markdown => Some(tree_sitter_md::LANGUAGE.into()),
            SyntaxLang::Sql | SyntaxLang::PlSql => Some(tree_sitter_sequel::LANGUAGE.into()),
            SyntaxLang::Java => Some(tree_sitter_java::LANGUAGE.into()),
            SyntaxLang::Html => Some(tree_sitter_html::LANGUAGE.into()),
            SyntaxLang::Bash => Some(tree_sitter_bash::LANGUAGE.into()),
            SyntaxLang::Go => Some(tree_sitter_go::LANGUAGE.into()),
            SyntaxLang::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
            SyntaxLang::Toml => Some(tree_sitter_toml_ng::LANGUAGE.into()),
            SyntaxLang::PowerShell => Some(tree_sitter_powershell::LANGUAGE.into()),
            SyntaxLang::Yaml => Some(tree_sitter_yaml::LANGUAGE.into()),
            SyntaxLang::Xml => Some(tree_sitter_xml::LANGUAGE_XML.into()),
            SyntaxLang::Batch => Some(tree_sitter_batch::LANGUAGE.into()),
            _ => None,
        };

        if let Some(ts_lang) = ts_lang
            && self.parser.set_language(&ts_lang).is_ok()
        {
            self.current_lang = Some(lang);
            self.tree = None;
            self.last_source.clear();
            self.selection_history.clear();
            return true;
        }

        self.current_lang = None;
        self.tree = None;
        self.last_source.clear();
        false
    }

    /// 現在設定されている言語を取得します。
    pub fn current_language(&self) -> Option<SyntaxLang> {
        self.current_lang
    }

    /// ソースコード全体をパースし、構文木（AST）を更新します。
    ///
    /// 既存の構文木が存在する場合は、既存のツリーを再利用して再構築します。
    pub fn parse(&mut self, source_code: &str) -> bool {
        if self.current_lang.is_none() {
            return false;
        }

        let new_tree = self.parser.parse(source_code, self.tree.as_ref());
        if let Some(tree) = new_tree {
            self.tree = Some(tree);
            self.last_source.clear();
            self.last_source.push_str(source_code);
            true
        } else {
            false
        }
    }

    /// 前回パースしたソースとの差分を自動検出し、[`Self::edit`] を適用したうえでインクリメンタルパースを行います。
    ///
    /// 前回の構文木が存在しない場合は通常の [`Self::parse`] にフォールバックし、
    /// テキスト内容が同一の場合は再パースをスキップして `true` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::{SyntaxLang, TreeSitterEngine};
    ///
    /// let mut engine = TreeSitterEngine::new();
    /// assert!(engine.set_language(SyntaxLang::Rust));
    /// assert!(engine.parse("fn main() { let x = 1; }"));
    /// assert!(engine.parse_incremental("fn main() { let x = 42; }"));
    /// ```
    pub fn parse_incremental(&mut self, source_code: &str) -> bool {
        if self.current_lang.is_none() {
            return false;
        }
        if self.tree.is_none() {
            return self.parse(source_code);
        }
        if self.last_source == source_code {
            return true;
        }

        let old = self.last_source.as_str();
        let old_bytes = old.as_bytes();
        let new_bytes = source_code.as_bytes();

        // 先頭共通プレフィックスを探索し、双方の文字境界へ揃える
        let min_len = old_bytes.len().min(new_bytes.len());
        let mut start_byte = 0usize;
        while start_byte < min_len && old_bytes[start_byte] == new_bytes[start_byte] {
            start_byte += 1;
        }
        while start_byte > 0
            && (!old.is_char_boundary(start_byte) || !source_code.is_char_boundary(start_byte))
        {
            start_byte -= 1;
        }

        // 末尾共通サフィックスを探索し、双方の文字境界へ揃える
        let max_suffix = (old_bytes.len() - start_byte).min(new_bytes.len() - start_byte);
        let mut suffix = 0usize;
        while suffix < max_suffix
            && old_bytes[old_bytes.len() - 1 - suffix] == new_bytes[new_bytes.len() - 1 - suffix]
        {
            suffix += 1;
        }
        while suffix > 0
            && (!old.is_char_boundary(old_bytes.len() - suffix)
                || !source_code.is_char_boundary(new_bytes.len() - suffix))
        {
            suffix -= 1;
        }

        let old_end_byte = old_bytes.len() - suffix;
        let new_end_byte = new_bytes.len() - suffix;

        let start_point = Self::byte_offset_to_point(old, start_byte);
        let old_end_point = Self::byte_offset_to_point(old, old_end_byte);
        let new_end_point = Self::byte_offset_to_point(source_code, new_end_byte);

        self.edit(
            start_byte,
            old_end_byte,
            new_end_byte,
            start_point,
            old_end_point,
            new_end_point,
        );

        let new_tree = self.parser.parse(source_code, self.tree.as_ref());
        if let Some(tree) = new_tree {
            self.tree = Some(tree);
            self.last_source.clear();
            self.last_source.push_str(source_code);
            true
        } else {
            false
        }
    }

    /// テキスト内のバイトオフセットから `(row, col)` を計算します。
    fn byte_offset_to_point(text: &str, byte_offset: usize) -> (usize, usize) {
        let clamped = byte_offset.min(text.len());
        let prefix = &text[..clamped];
        let row = prefix.bytes().filter(|&b| b == b'\n').count();
        let col = prefix
            .rfind('\n')
            .map(|pos| clamped - (pos + 1))
            .unwrap_or(clamped);
        (row, col)
    }

    /// テキスト変更時のインクリメンタル編集（InputEdit）を構文木に適用します。
    ///
    /// # 引数
    /// * `start_byte` - 変更開始バイト位置
    /// * `old_end_byte` - 変更前の終了バイト位置
    /// * `new_end_byte` - 変更後の終了バイト位置
    /// * `start_point` - 変更開始行・列 (row, col)
    /// * `old_end_point` - 変更前の終了行・列 (row, col)
    /// * `new_end_point` - 変更後の終了行・列 (row, col)
    pub fn edit(
        &mut self,
        start_byte: usize,
        old_end_byte: usize,
        new_end_byte: usize,
        start_point: (usize, usize),
        old_end_point: (usize, usize),
        new_end_point: (usize, usize),
    ) {
        if let Some(ref mut tree) = self.tree {
            let edit = InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position: Point {
                    row: start_point.0,
                    column: start_point.1,
                },
                old_end_position: Point {
                    row: old_end_point.0,
                    column: old_end_point.1,
                },
                new_end_position: Point {
                    row: new_end_point.0,
                    column: new_end_point.1,
                },
            };
            tree.edit(&edit);
        }
    }

    /// 現在の構文木が存在するかどうかを取得します。
    pub fn has_tree(&self) -> bool {
        self.tree.is_some()
    }

    /// ルート構文木ノードを取得します。
    pub fn root_node(&self) -> Option<Node<'_>> {
        self.tree.as_ref().map(|t| t.root_node())
    }

    /// 構文木全体の S 式（S-expression）文字列表現を取得します。
    pub fn to_sexp(&self) -> Option<String> {
        self.root_node().map(|r| r.to_sexp())
    }

    /// 指定した行 (0-based) の構文木ハイライトトークン一覧を取得します。
    ///
    /// 指定行と交差する AST 葉ノードを探索し、行内カラム位置付きトークンへ変換します。
    pub fn highlight_line(&self, line_index: usize, line_text: &str) -> Vec<AstToken> {
        let tree = match &self.tree {
            Some(t) => t,
            None => return Vec::new(),
        };

        let root = tree.root_node();
        let start_point = Point {
            row: line_index,
            column: 0,
        };
        let end_point = Point {
            row: line_index,
            column: line_text.len(),
        };

        let mut tokens = Vec::new();
        let mut cursor = root.walk();
        let lang = self.current_lang.unwrap_or(SyntaxLang::PlainText);

        Self::collect_tokens_for_line(
            &mut cursor,
            lang,
            line_index,
            start_point,
            end_point,
            line_text,
            &mut tokens,
        );

        tokens
    }

    /// 指定行のトークンを [`SyntaxToken`] 一覧として取得します。
    ///
    /// AST トークン間の未分類文字・空白（ギャップ）を [`TokenType::Text`] で補完するため、
    /// 返却される全トークンを連結すると常に元の `line_text`（解析成功時）と一致し、
    /// 描画側の文字位置ズレを防ぎます。
    pub fn highlight_syntax_tokens(&self, line_index: usize, line_text: &str) -> Vec<SyntaxToken> {
        let mut ast_tokens = self.highlight_line(line_index, line_text);
        if ast_tokens.is_empty() {
            return Vec::new();
        }

        ast_tokens.sort_by_key(|t| (t.start_col, t.end_col));

        let len = line_text.len();
        let mut result = Vec::with_capacity(ast_tokens.len() * 2 + 1);
        let mut cursor_byte = 0usize;

        for tok in ast_tokens {
            let raw_start = tok.start_col.min(len);
            let raw_end = tok.end_col.min(len);
            if raw_end <= cursor_byte || raw_end <= raw_start {
                continue;
            }
            let actual_start = raw_start.max(cursor_byte);
            if !line_text.is_char_boundary(cursor_byte)
                || !line_text.is_char_boundary(actual_start)
                || !line_text.is_char_boundary(raw_end)
            {
                continue;
            }

            if actual_start > cursor_byte {
                result.push(SyntaxToken {
                    text: line_text[cursor_byte..actual_start].to_string(),
                    token_type: TokenType::Text,
                });
            }

            result.push(SyntaxToken {
                text: line_text[actual_start..raw_end].to_string(),
                token_type: tok.token_type,
            });
            cursor_byte = raw_end;
        }

        if cursor_byte < len && line_text.is_char_boundary(cursor_byte) {
            result.push(SyntaxToken {
                text: line_text[cursor_byte..].to_string(),
                token_type: TokenType::Text,
            });
        }

        result
    }

    fn collect_tokens_for_line(
        cursor: &mut tree_sitter::TreeCursor,
        lang: SyntaxLang,
        target_row: usize,
        _line_start: Point,
        _line_end: Point,
        line_text: &str,
        tokens: &mut Vec<AstToken>,
    ) {
        let node = cursor.node();
        let start = node.start_position();
        let end = node.end_position();

        // 該当行と全く交差しないノードは探索枝刈り
        if start.row > target_row || end.row < target_row {
            return;
        }

        // コメントや文字列/文字リテラル等は内部に子ノード（"//", "comment_content", "\"", "string_content" 等）を
        // 持つため、子へ降りずノード全体で単一トークンとして確定させる
        let is_atomic_token_node = Self::is_atomic_node(lang, node.kind());

        // 葉ノードまたは単一色複合ノードであればトークン化
        if node.child_count() == 0 || is_atomic_token_node {
            let col_start = if start.row == target_row {
                start.column
            } else {
                0
            };
            let col_end = if end.row == target_row {
                end.column
            } else {
                line_text.len()
            };

            let len = line_text.len();
            if col_start < len {
                let clamped_end = col_end.min(len);
                if clamped_end > col_start
                    && line_text.is_char_boundary(col_start)
                    && line_text.is_char_boundary(clamped_end)
                {
                    let text = &line_text[col_start..clamped_end];
                    if let Some(token_type) = Self::node_to_token_type(lang, &node, text) {
                        tokens.push(AstToken {
                            text: text.to_string(),
                            token_type,
                            start_col: col_start,
                            end_col: clamped_end,
                        });
                    }
                }
            }
            return;
        }

        // 子ノードを再帰走査
        if cursor.goto_first_child() {
            loop {
                Self::collect_tokens_for_line(
                    cursor,
                    lang,
                    target_row,
                    _line_start,
                    _line_end,
                    line_text,
                    tokens,
                );
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    /// 子ノードへ降りず単一トークンとして扱う複合ノードかどうかを判定します。
    fn is_atomic_node(lang: SyntaxLang, kind: &str) -> bool {
        if matches!(
            kind,
            "line_comment"
                | "block_comment"
                | "comment"
                | "string_literal"
                | "raw_string_literal"
                | "char_literal"
        ) {
            return true;
        }

        match lang {
            SyntaxLang::Python => matches!(kind, "string"),
            SyntaxLang::Bash => matches!(kind, "string" | "raw_string" | "heredoc_body"),
            SyntaxLang::Sql => matches!(kind, "marginalia" | "literal"),
            SyntaxLang::Markdown => matches!(
                kind,
                "atx_heading"
                    | "setext_heading"
                    | "indented_code_block"
                    | "code_fence_content"
                    | "info_string"
                    | "link_destination"
                    | "link_title"
                    | "link_label"
                    | "html_block"
            ),
            SyntaxLang::PowerShell => matches!(
                kind,
                "expandable_string_literal"
                    | "verbatim_string_literal"
                    | "expandable_here_string_literal"
                    | "verbatim_here_string_literal"
                    | "integer_literal"
                    | "real_literal"
                    | "type_spec"
                    | "variable"
            ),
            SyntaxLang::Batch => matches!(
                kind,
                "string" | "label" | "variable_reference" | "for_variable" | "echo_off"
            ),
            SyntaxLang::Toml => matches!(kind, "string" | "quoted_key"),
            SyntaxLang::Json => matches!(kind, "string"),
            SyntaxLang::Yaml => matches!(
                kind,
                "double_quote_scalar" | "single_quote_scalar" | "block_scalar" | "string_scalar"
            ),
            SyntaxLang::JavaScript => matches!(kind, "string" | "template_string" | "regex"),
            SyntaxLang::Cpp => matches!(
                kind,
                "string_literal" | "raw_string_literal" | "char_literal" | "system_lib_string"
            ),
            SyntaxLang::Java => matches!(kind, "string_literal" | "character_literal"),
            SyntaxLang::Go => matches!(
                kind,
                "interpreted_string_literal" | "raw_string_literal" | "rune_literal"
            ),
            SyntaxLang::Ruby => matches!(
                kind,
                "string"
                    | "bare_string"
                    | "subshell"
                    | "heredoc_body"
                    | "regex"
                    | "simple_symbol"
                    | "delimited_symbol"
                    | "hash_key_symbol"
            ),
            SyntaxLang::Html => matches!(kind, "attribute_value" | "quoted_attribute_value"),
            SyntaxLang::Xml => matches!(
                kind,
                "AttValue" | "PseudoAttValue" | "EntityValue" | "Comment" | "CData"
            ),
            _ => false,
        }
    }

    /// Tree-sitter ノードの種類（kind）を言語に応じて [`TokenType`] にマッピングします。
    fn node_to_token_type(lang: SyntaxLang, node: &Node, text: &str) -> Option<TokenType> {
        let kind = node.kind();

        // 全言語共通のコメント
        if matches!(
            kind,
            "line_comment" | "block_comment" | "comment" | "Comment"
        ) {
            return Some(TokenType::Comment);
        }

        match lang {
            SyntaxLang::Rust => Self::rust_node_to_token_type(kind),
            SyntaxLang::Cpp => Self::cpp_node_to_token_type(kind),
            SyntaxLang::Python => Self::python_node_to_token_type(kind),
            SyntaxLang::JavaScript => Self::javascript_node_to_token_type(kind),
            SyntaxLang::Json => Self::json_node_to_token_type(node, kind),
            SyntaxLang::Markdown => Self::markdown_node_to_token_type(kind),
            SyntaxLang::Sql | SyntaxLang::PlSql => Self::sql_node_to_token_type(kind, text),
            SyntaxLang::Java => Self::java_node_to_token_type(node, kind),
            SyntaxLang::Html => Self::html_node_to_token_type(kind),
            SyntaxLang::Bash => Self::bash_node_to_token_type(kind),
            SyntaxLang::Go => Self::go_node_to_token_type(kind),
            SyntaxLang::Ruby => Self::ruby_node_to_token_type(kind),
            SyntaxLang::Toml => Self::toml_node_to_token_type(kind),
            SyntaxLang::PowerShell => Self::powershell_node_to_token_type(kind),
            SyntaxLang::Yaml => Self::yaml_node_to_token_type(node, kind),
            SyntaxLang::Xml => Self::xml_node_to_token_type(kind),
            SyntaxLang::Batch => Self::batch_node_to_token_type(kind, text),
            _ => Self::rust_node_to_token_type(kind),
        }
    }

    fn rust_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            // 文字列・リテラル
            "string_literal" | "raw_string_literal" | "char_literal" | "string_content" => {
                Some(TokenType::StringLiteral)
            }
            "integer_literal" | "float_literal" | "boolean_literal" => Some(TokenType::Number),

            // 予約語・キーワード
            "fn" | "let" | "mut" | "pub" | "struct" | "enum" | "impl" | "trait" | "match"
            | "if" | "else" | "loop" | "while" | "for" | "in" | "return" | "break" | "continue"
            | "use" | "mod" | "crate" | "self" | "super" | "as" | "where" | "async" | "await"
            | "unsafe" | "type" | "const" | "static" | "ref" | "move" | "dyn" | "macro_rules!" => {
                Some(TokenType::Keyword)
            }

            // 識別子・関数・型
            "type_identifier" | "primitive_type" | "self_type" => Some(TokenType::Type),
            "identifier" => Some(TokenType::Text),

            // 演算子・区切り記号
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | "=>" | "->" | ".." | "..=" | "..."
            | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | "?" | "#"
            | "@" => Some(TokenType::Punctuation),

            // 括弧・デリミタ
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "::" | "," | "." => {
                Some(TokenType::Punctuation)
            }

            _ => None,
        }
    }

    fn python_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "string" | "string_content" | "escape_sequence" => Some(TokenType::StringLiteral),
            "integer" | "float" | "true" | "false" | "none" => Some(TokenType::Number),
            "as" | "assert" | "async" | "await" | "break" | "class" | "continue" | "def"
            | "del" | "elif" | "else" | "except" | "exec" | "finally" | "for" | "from"
            | "global" | "if" | "import" | "in" | "is" | "is not" | "lambda" | "match" | "case"
            | "nonlocal" | "not" | "not in" | "and" | "or" | "pass" | "print" | "raise"
            | "return" | "try" | "while" | "with" | "yield" => Some(TokenType::Keyword),
            "identifier" => Some(TokenType::Text),
            "+" | "-" | "*" | "**" | "/" | "//" | "%" | "@" | "<<" | ">>" | "&" | "|" | "^"
            | "~" | ":=" | "<" | ">" | "<=" | ">=" | "==" | "!=" | "=" | "+=" | "-=" | "*="
            | "/=" | "//=" | "%=" | "**=" | "&=" | "|=" | "^=" | ">>=" | "<<=" | "@=" | "->"
            | "(" | ")" | "[" | "]" | "{" | "}" | "," | ":" | "." | ";" => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        }
    }

    fn bash_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "string" | "raw_string" | "string_content" | "heredoc_body" | "heredoc_start"
            | "heredoc_end" => Some(TokenType::StringLiteral),
            "number" | "file_descriptor" => Some(TokenType::Number),
            "case" | "do" | "done" | "elif" | "else" | "esac" | "export" | "fi" | "for"
            | "function" | "if" | "in" | "local" | "readonly" | "select" | "then" | "unset"
            | "until" | "while" => Some(TokenType::Keyword),
            "variable_name" | "special_variable_name" => Some(TokenType::Type),
            "$" | "&&" | "||" | "|" | "|&" | ">" | ">>" | "<" | "<<" | "<<-" | "<<<" | "&>"
            | "&>>" | "=" | "+=" | "==" | "!=" | ";" | ";;" | "(" | ")" | "{" | "}" | "[" | "]"
            | "[[" | "]]" | "!" => Some(TokenType::Punctuation),
            "word" => Some(TokenType::Text),
            _ => None,
        }
    }

    fn sql_node_to_token_type(kind: &str, text: &str) -> Option<TokenType> {
        if kind == "marginalia" {
            return Some(TokenType::Comment);
        }
        if kind == "literal" {
            let trimmed = text.trim();
            let is_num = !trimmed.is_empty()
                && trimmed
                    .trim_start_matches(['+', '-'])
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.');
            return Some(if is_num {
                TokenType::Number
            } else {
                TokenType::StringLiteral
            });
        }
        if let Some(kw) = kind.strip_prefix("keyword_") {
            return Some(match kw {
                "true" | "false" => TokenType::Number,
                "int" | "integer" | "smallint" | "mediumint" | "bigint" | "tinyint" | "decimal"
                | "float" | "double" | "numeric" | "real" | "char" | "nchar" | "varchar"
                | "nvarchar" | "text" | "string" | "boolean" | "binary" | "varbinary" | "bit"
                | "date" | "datetime" | "datetime2" | "time" | "timestamp" | "timestamptz"
                | "interval" | "uuid" | "json" | "jsonb" | "xml" | "bytea" | "serial"
                | "smallserial" | "bigserial" | "money" | "smallmoney" | "enum" | "null" => {
                    TokenType::Type
                }
                _ => TokenType::Keyword,
            });
        }
        match kind {
            "double" => Some(TokenType::Type),
            "parameter" => Some(TokenType::Type),
            "identifier" => Some(TokenType::Text),
            "+" | "-" | "*" | "/" | "%" | "^" | ":=" | "=>" | "=" | "<" | "<=" | "!=" | ">="
            | ">" | "<>" | "(" | ")" | ";" | "," | "." | "op_other" | "op_unary_other" => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        }
    }

    fn markdown_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "atx_heading" | "setext_heading" => Some(TokenType::Keyword),
            "indented_code_block" | "code_fence_content" | "link_title" | "link_destination" => {
                Some(TokenType::StringLiteral)
            }
            "info_string" | "link_label" => Some(TokenType::Type),
            "html_block" => Some(TokenType::Comment),
            "atx_h1_marker"
            | "atx_h2_marker"
            | "atx_h3_marker"
            | "atx_h4_marker"
            | "atx_h5_marker"
            | "atx_h6_marker"
            | "setext_h1_underline"
            | "setext_h2_underline"
            | "fenced_code_block_delimiter"
            | "list_marker_plus"
            | "list_marker_minus"
            | "list_marker_star"
            | "list_marker_dot"
            | "list_marker_parenthesis"
            | "thematic_break"
            | "block_quote_marker"
            | "block_continuation"
            | "backslash_escape" => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn powershell_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "string_literal"
            | "expandable_string_literal"
            | "verbatim_string_literal"
            | "expandable_here_string_literal"
            | "verbatim_here_string_literal" => Some(TokenType::StringLiteral),
            "integer_literal"
            | "decimal_integer_literal"
            | "hexadecimal_integer_literal"
            | "real_literal" => Some(TokenType::Number),
            "param" | "dynamicparam" | "begin" | "process" | "end" | "if" | "elseif" | "else"
            | "switch" | "foreach" | "for" | "while" | "do" | "until" | "function" | "filter"
            | "workflow" | "break" | "continue" | "throw" | "return" | "exit" | "trap" | "try"
            | "catch" | "finally" | "data" | "inlinescript" | "parallel" | "sequence" | "in" => {
                Some(TokenType::Keyword)
            }
            "type_spec" | "variable" | "command_name" | "function_name" => Some(TokenType::Type),
            "+"
            | "-"
            | "*"
            | "/"
            | "\\"
            | "%"
            | "="
            | "+="
            | "-="
            | "*="
            | "/="
            | "%="
            | ".."
            | "|"
            | ";"
            | ","
            | "."
            | "("
            | ")"
            | "{"
            | "}"
            | "["
            | "]"
            | "@"
            | "command_invokation_operator" => Some(TokenType::Punctuation),
            k if k.starts_with('-') => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn batch_node_to_token_type(kind: &str, text: &str) -> Option<TokenType> {
        match kind {
            "string" | "for_set_literal" | "redirect_target" => Some(TokenType::StringLiteral),
            "integer" => Some(TokenType::Number),
            "echo_off" | "set_keyword" => Some(TokenType::Keyword),
            "label" | "variable_reference" | "variable_name" | "for_variable" => {
                Some(TokenType::Type)
            }
            "set_option" | "for_options" | "command_option" => Some(TokenType::Punctuation),
            "comparison_op" | "redirect_op" | "fd_redirect" | "(" | ")" | "=" | "@" | "&"
            | "&&" | "||" | "|" => Some(TokenType::Punctuation),
            "command_name" => Some(TokenType::Keyword),
            _ => {
                let lower = text.trim().to_ascii_lowercase();
                if matches!(
                    lower.as_str(),
                    "if" | "else"
                        | "not"
                        | "exist"
                        | "defined"
                        | "for"
                        | "in"
                        | "do"
                        | "goto"
                        | "call"
                        | "setlocal"
                        | "endlocal"
                        | "exit"
                        | "echo"
                        | "set"
                        | "on"
                        | "off"
                ) {
                    Some(TokenType::Keyword)
                } else {
                    None
                }
            }
        }
    }

    fn toml_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "bare_key" => Some(TokenType::Type),
            "quoted_key" | "string" | "offset_date_time" | "local_date_time" | "local_date"
            | "local_time" => Some(TokenType::StringLiteral),
            "integer" | "float" => Some(TokenType::Number),
            "boolean" => Some(TokenType::Keyword),
            "." | "," | "=" | "[" | "]" | "[[" | "]]" | "{" | "}" => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn json_node_to_token_type(node: &Node, kind: &str) -> Option<TokenType> {
        match kind {
            "string" => {
                let is_key = node.parent().is_some_and(|p| {
                    p.kind() == "pair"
                        && p.child_by_field_name("key")
                            .is_some_and(|k| k.id() == node.id())
                });
                Some(if is_key {
                    TokenType::Type
                } else {
                    TokenType::StringLiteral
                })
            }
            "number" => Some(TokenType::Number),
            "true" | "false" | "null" => Some(TokenType::Keyword),
            "{" | "}" | "[" | "]" | ":" | "," => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn yaml_node_to_token_type(node: &Node, kind: &str) -> Option<TokenType> {
        match kind {
            "double_quote_scalar" | "single_quote_scalar" | "string_scalar" => {
                // block_mapping_pair / flow_pair の key フィールド配下にある場合はキー色 (Type)
                let mut cur = *node;
                let mut is_key = false;
                while let Some(parent) = cur.parent() {
                    if matches!(parent.kind(), "block_mapping_pair" | "flow_pair") {
                        is_key = parent
                            .child_by_field_name("key")
                            .is_some_and(|k| k.id() == cur.id());
                        break;
                    }
                    cur = parent;
                }
                Some(if is_key {
                    TokenType::Type
                } else {
                    TokenType::StringLiteral
                })
            }
            "block_scalar" => Some(TokenType::StringLiteral),
            "integer_scalar" | "float_scalar" => Some(TokenType::Number),
            "boolean_scalar" | "null_scalar" => Some(TokenType::Keyword),
            "anchor_name" | "alias_name" | "tag" | "yaml_directive" | "tag_directive"
            | "reserved_directive" => Some(TokenType::Type),
            "," | "-" | ":" | ">" | "?" | "|" | "[" | "]" | "{" | "}" | "*" | "&" | "---"
            | "..." => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn javascript_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "string" | "template_string" | "regex" => Some(TokenType::StringLiteral),
            "number" => Some(TokenType::Number),
            "true" | "false" | "null" | "undefined" | "this" | "super" | "as" | "async"
            | "await" | "break" | "case" | "catch" | "class" | "const" | "continue"
            | "debugger" | "default" | "delete" | "do" | "else" | "export" | "extends"
            | "finally" | "for" | "from" | "function" | "get" | "if" | "import" | "in"
            | "instanceof" | "let" | "new" | "of" | "return" | "set" | "static" | "switch"
            | "target" | "throw" | "try" | "typeof" | "var" | "void" | "while" | "with"
            | "yield" => Some(TokenType::Keyword),
            "property_identifier" => Some(TokenType::Type),
            "identifier" => Some(TokenType::Text),
            ";" | "." | "," | "optional_chain" | "-" | "--" | "-=" | "+" | "++" | "+=" | "*"
            | "*=" | "**" | "**=" | "/" | "/=" | "%" | "%=" | "<" | "<=" | "<<" | "<<=" | "="
            | "==" | "===" | "!" | "!=" | "!==" | "=>" | ">" | ">=" | ">>" | ">>=" | ">>>"
            | ">>>=" | "~" | "^" | "&" | "|" | "^=" | "&=" | "|=" | "&&" | "||" | "??" | "&&="
            | "||=" | "??=" | "(" | ")" | "[" | "]" | "{" | "}" | ":" | "?" => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        }
    }

    fn cpp_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "string_literal" | "raw_string_literal" | "char_literal" | "system_lib_string"
            | "string_content" => Some(TokenType::StringLiteral),
            "number_literal" | "true" | "false" => Some(TokenType::Number),
            "auto" | "primitive_type" | "type_identifier" | "sized_type_specifier" => {
                Some(TokenType::Type)
            }
            "break" | "case" | "catch" | "class" | "co_await" | "co_return" | "co_yield"
            | "concept" | "const" | "constexpr" | "constinit" | "consteval" | "continue"
            | "default" | "delete" | "do" | "else" | "enum" | "explicit" | "extern" | "final"
            | "for" | "friend" | "goto" | "if" | "inline" | "mutable" | "namespace" | "new"
            | "noexcept" | "nullptr" | "override" | "private" | "protected" | "public"
            | "register" | "requires" | "return" | "sizeof" | "static" | "struct" | "switch"
            | "template" | "this" | "throw" | "try" | "typedef" | "typename" | "union"
            | "using" | "virtual" | "volatile" | "while" | "#include" | "#define" | "#ifdef"
            | "#ifndef" | "#if" | "#else" | "#elif" | "#endif" | "#pragma" => {
                Some(TokenType::Keyword)
            }
            "identifier" | "field_identifier" | "namespace_identifier" => Some(TokenType::Text),
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | "->" | "::" | "." | "," | ";"
            | ":" | "(" | ")" | "[" | "]" | "{" | "}" | "?" | "+=" | "-=" | "*=" | "/=" | "++"
            | "--" => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn java_node_to_token_type(node: &Node, kind: &str) -> Option<TokenType> {
        match kind {
            "string_literal" | "character_literal" | "string_fragment" => {
                Some(TokenType::StringLiteral)
            }
            "decimal_integer_literal"
            | "hex_integer_literal"
            | "octal_integer_literal"
            | "binary_integer_literal"
            | "decimal_floating_point_literal"
            | "hex_floating_point_literal"
            | "true"
            | "false"
            | "null_literal" => Some(TokenType::Number),
            "type_identifier"
            | "boolean_type"
            | "integral_type"
            | "floating_point_type"
            | "void_type" => Some(TokenType::Type),
            "abstract" | "assert" | "break" | "case" | "catch" | "class" | "continue"
            | "default" | "do" | "else" | "enum" | "exports" | "extends" | "final" | "finally"
            | "for" | "if" | "implements" | "import" | "instanceof" | "interface" | "module"
            | "native" | "new" | "non-sealed" | "open" | "opens" | "package" | "permits"
            | "private" | "protected" | "provides" | "public" | "record" | "requires"
            | "return" | "sealed" | "static" | "strictfp" | "super" | "switch" | "synchronized"
            | "this" | "throw" | "throws" | "to" | "transient" | "transitive" | "try" | "uses"
            | "var" | "volatile" | "when" | "while" | "with" | "yield" => Some(TokenType::Keyword),
            "identifier" => {
                let is_type_decl = node.parent().is_some_and(|p| {
                    matches!(
                        p.kind(),
                        "class_declaration" | "interface_declaration" | "enum_declaration"
                    ) && p
                        .child_by_field_name("name")
                        .is_some_and(|n| n.id() == node.id())
                });
                Some(if is_type_decl {
                    TokenType::Type
                } else {
                    TokenType::Text
                })
            }
            "@" | "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&&" | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | ">>>" | "->" | "::" | "."
            | "," | ";" | ":" | "(" | ")" | "[" | "]" | "{" | "}" | "?" | "++" | "--" | "+="
            | "-=" => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn go_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "interpreted_string_literal"
            | "raw_string_literal"
            | "rune_literal"
            | "escape_sequence" => Some(TokenType::StringLiteral),
            "int_literal" | "float_literal" | "imaginary_literal" | "true" | "false" | "nil"
            | "iota" => Some(TokenType::Number),
            "type_identifier" => Some(TokenType::Type),
            "break" | "case" | "chan" | "const" | "continue" | "default" | "defer" | "else"
            | "fallthrough" | "for" | "func" | "go" | "goto" | "if" | "import" | "interface"
            | "map" | "package" | "range" | "return" | "select" | "struct" | "switch" | "type"
            | "var" => Some(TokenType::Keyword),
            "identifier" | "field_identifier" | "package_identifier" => Some(TokenType::Text),
            "--" | "-" | "-=" | ":=" | "!" | "!=" | "..." | "*" | "*=" | "/" | "/=" | "&"
            | "&&" | "&=" | "%" | "%=" | "^" | "^=" | "+" | "++" | "+=" | "<-" | "<" | "<<"
            | "<<=" | "<=" | "=" | "==" | ">" | ">=" | ">>" | ">>=" | "|" | "|=" | "||" | "~"
            | "(" | ")" | "[" | "]" | "{" | "}" | "," | "." | ";" | ":" => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        }
    }

    fn ruby_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "string" | "bare_string" | "subshell" | "heredoc_body" | "heredoc_beginning"
            | "regex" | "string_content" | "escape_sequence" => Some(TokenType::StringLiteral),
            "simple_symbol" | "delimited_symbol" | "hash_key_symbol" | "bare_symbol"
            | "constant" | "class_variable" | "instance_variable" => Some(TokenType::Type),
            "integer" | "float" | "nil" | "true" | "false" => Some(TokenType::Number),
            "alias" | "and" | "begin" | "break" | "case" | "class" | "def" | "do" | "else"
            | "elsif" | "end" | "ensure" | "for" | "if" | "in" | "module" | "next" | "or"
            | "rescue" | "retry" | "return" | "self" | "super" | "then" | "unless" | "until"
            | "when" | "while" | "yield" | "defined?" => Some(TokenType::Keyword),
            "identifier" => Some(TokenType::Text),
            "=" | "=>" | "->" | "," | ";" | "." | "(" | ")" | "[" | "]" | "{" | "}" | "%w("
            | "%i(" | "+" | "-" | "*" | "/" | "==" | "!=" => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn html_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "tag_name" | "doctype" => Some(TokenType::Keyword),
            "attribute_name" => Some(TokenType::Type),
            "attribute_value" | "quoted_attribute_value" => Some(TokenType::StringLiteral),
            "<" | ">" | "</" | "/>" | "=" | "<!" => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    fn xml_node_to_token_type(kind: &str) -> Option<TokenType> {
        match kind {
            "Name" | "PITarget" => Some(TokenType::Keyword),
            "EncName" | "AttValue" | "PseudoAttValue" | "EntityValue" | "CData" | "URI"
            | "PubidLiteral" => Some(TokenType::StringLiteral),
            "VersionNum" => Some(TokenType::Number),
            "xml" | "DOCTYPE" | "ELEMENT" | "ATTLIST" | "ENTITY" | "NOTATION" | "PUBLIC"
            | "SYSTEM" | "yes" | "no" => Some(TokenType::Keyword),
            "EntityRef" | "CharRef" | "PEReference" | "StringType" | "TokenizedType" => {
                Some(TokenType::Type)
            }
            "<?" | "?>" | "<!" | "]]>" | "<" | ">" | "</" | "/>" | "(" | ")" | "[" | "]" | "\""
            | "'" | "," | "|" | "=" | "*" | "?" | "+" => Some(TokenType::Punctuation),
            _ => None,
        }
    }

    /// カーソル位置（行・列）にある最小の AST ノードを取得します。
    pub fn node_at_point(&self, line: usize, col: usize) -> Option<Node<'_>> {
        let tree = self.tree.as_ref()?;
        let point = Point {
            row: line,
            column: col,
        };
        tree.root_node().descendant_for_point_range(point, point)
    }

    /// 構文木ノードに基づくスマート選択拡大（Expand Selection）を実行します。
    ///
    /// カーソル位置の最小ノード（識別子など）から開始し、
    /// 呼び出すごとにより上位の親ノード（式、文、ブロック、関数全体）へ
    /// 選択範囲を段階的に拡大します。
    pub fn smart_expand_selection(
        &mut self,
        line: usize,
        col: usize,
        current_range: Option<AstSelectionRange>,
    ) -> Option<AstSelectionRange> {
        let tree = self.tree.as_ref()?;
        let root = tree.root_node();

        let new_range = match current_range {
            None => {
                // 初回: カーソル直下の最小ノードを選択
                let point = Point {
                    row: line,
                    column: col,
                };
                let leaf = root.descendant_for_point_range(point, point)?;
                let range = AstSelectionRange::from_node(&leaf);
                self.selection_history.clear();
                self.selection_history.push(range);
                range
            }
            Some(curr) => {
                // 拡大: 現在の選択範囲を包含する親ノードを探索
                let start_point = Point {
                    row: curr.start_line,
                    column: curr.start_col,
                };
                let end_point = Point {
                    row: curr.end_line,
                    column: curr.end_col,
                };

                let mut node = root.descendant_for_point_range(start_point, end_point)?;

                // 現在の範囲より確実に大きい親ノードが見つかるまで親を辿る
                while let Some(parent) = node.parent() {
                    let p_range = AstSelectionRange::from_node(&parent);
                    if p_range.start_byte < curr.start_byte || p_range.end_byte > curr.end_byte {
                        self.selection_history.push(curr);
                        return Some(p_range);
                    }
                    node = parent;
                }
                curr
            }
        };

        Some(new_range)
    }

    /// スマート選択縮小（Shrink Selection）を実行します。
    ///
    /// 拡大履歴スタックから直前の選択範囲を復元します。
    pub fn smart_shrink_selection(&mut self) -> Option<AstSelectionRange> {
        self.selection_history.pop()
    }
}

impl Default for TreeSitterEngine {
    fn default() -> Self {
        Self::new()
    }
}
