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
            selection_history: Vec::new(),
        }
    }

    /// 言語種別に応じた Tree-sitter 言語文法を設定します。
    ///
    /// # 戻り値
    /// 設定に成功した場合は `true`、未対応の言語の場合は `false` を返します。
    pub fn set_language(&mut self, lang: SyntaxLang) -> bool {
        let ts_lang: Option<Language> = match lang {
            SyntaxLang::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            _ => None,
        };

        if let Some(ts_lang) = ts_lang
            && self.parser.set_language(&ts_lang).is_ok()
        {
            self.current_lang = Some(lang);
            self.tree = None;
            self.selection_history.clear();
            return true;
        }

        self.current_lang = None;
        self.tree = None;
        false
    }

    /// 現在設定されている言語を取得します。
    pub fn current_language(&self) -> Option<SyntaxLang> {
        self.current_lang
    }

    /// ソースコード全体をパースし、構文木（AST）を更新します。
    ///
    /// 既存の構文木が存在する場合は、差分インクリメンタルパースを活用して
    /// タイピング遅延ゼロで再構築します。
    pub fn parse(&mut self, source_code: &str) -> bool {
        if self.current_lang.is_none() {
            return false;
        }

        let new_tree = self.parser.parse(source_code, self.tree.as_ref());
        if let Some(tree) = new_tree {
            self.tree = Some(tree);
            true
        } else {
            false
        }
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

        Self::collect_tokens_for_line(
            &mut cursor,
            line_index,
            start_point,
            end_point,
            line_text,
            &mut tokens,
        );

        tokens
    }

    /// 指定行のトークンを [`SyntaxToken`] 一覧として取得します。
    pub fn highlight_syntax_tokens(&self, line_index: usize, line_text: &str) -> Vec<SyntaxToken> {
        self.highlight_line(line_index, line_text)
            .into_iter()
            .map(|t| t.to_syntax_token())
            .collect()
    }

    fn collect_tokens_for_line(
        cursor: &mut tree_sitter::TreeCursor,
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

        // 葉ノード（子ノードを持たない末端トークン）であればトークン化
        if node.child_count() == 0 {
            if let Some(token_type) = Self::node_to_token_type(&node) {
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

                if col_end > col_start && col_start < line_text.len() {
                    let clamped_end = col_end.min(line_text.len());
                    let text = &line_text[col_start..clamped_end];
                    tokens.push(AstToken {
                        text: text.to_string(),
                        token_type,
                        start_col: col_start,
                        end_col: clamped_end,
                    });
                }
            }
            return;
        }

        // 子ノードを再帰走査
        if cursor.goto_first_child() {
            loop {
                Self::collect_tokens_for_line(
                    cursor,
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

    /// Tree-sitter ノードの種類（kind）を [`TokenType`] にマッピングします。
    fn node_to_token_type(node: &Node) -> Option<TokenType> {
        let kind = node.kind();
        match kind {
            // コメント
            "line_comment" | "block_comment" | "comment" => Some(TokenType::Comment),

            // 文字列・リテラル
            "string_literal" | "raw_string_literal" | "char_literal" | "string_content" => {
                Some(TokenType::StringLiteral)
            }
            "integer_literal" | "float_literal" | "boolean_literal" => Some(TokenType::Number),

            // 予約語・キーワード
            "fn" | "let" | "mut" | "pub" | "struct" | "enum" | "impl" | "trait" | "match"
            | "if" | "else" | "loop" | "while" | "for" | "in" | "return" | "break" | "continue"
            | "use" | "mod" | "crate" | "self" | "super" | "as" | "where" | "async" | "await"
            | "unsafe" | "type" | "const" | "static" | "ref" | "move" => Some(TokenType::Keyword),

            // 識別子・関数・型
            "type_identifier" | "primitive_type" => Some(TokenType::Type),
            "identifier" => Some(TokenType::Text),

            // 演算子・区切り記号
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | "=>" | "->" | ".." | "..=" => {
                Some(TokenType::Punctuation)
            }

            // 括弧・デリミタ
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "::" | "," | "." => {
                Some(TokenType::Punctuation)
            }

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
