//! ViVi 3 / ViVi 4 準拠の式置換 (`:s/pattern/expression/e`) モジュール。
//!
//! 置換後文字列を算術式・スクリプト式として動的に評価し、
//! 正規表現キャプチャグループ（`$0`〜`$9`）、連番カウンタ（`$i`, `$n`）、
//! 行番号（`$line`）、四則演算（`+`, `-`, `*`, `/`, `%`）、および文字列結合をサポートします。

use regex::RegexBuilder;
use std::collections::HashMap;

/// 式評価時の値の型（数値または文字列）。
///
/// # Examples
///
/// ```
/// use vivivi_modes::expr_subst::ExprValue;
///
/// let num = ExprValue::Number(42.0);
/// assert_eq!(num.as_number(), Some(42.0));
/// assert_eq!(num.to_display_string(), "42");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ExprValue {
    /// 浮動小数点数値
    Number(f64),
    /// 文字列
    String(String),
}

impl ExprValue {
    /// 数値としての値を取得します。文字列値の場合は数値へのパースを試みます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::expr_subst::ExprValue;
    ///
    /// assert_eq!(ExprValue::String("123".into()).as_number(), Some(123.0));
    /// assert_eq!(ExprValue::String("abc".into()).as_number(), None);
    /// ```
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ExprValue::Number(n) => Some(*n),
            ExprValue::String(s) => s.trim().parse::<f64>().ok(),
        }
    }

    /// 表示用の文字列表現を取得します。
    ///
    /// 整数値の場合は小数点なし（例: `"42"`）、小数の場合はそのまま出力します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::expr_subst::ExprValue;
    ///
    /// assert_eq!(ExprValue::Number(10.0).to_display_string(), "10");
    /// assert_eq!(ExprValue::Number(3.14).to_display_string(), "3.14");
    /// ```
    pub fn to_display_string(&self) -> String {
        match self {
            ExprValue::Number(n) => {
                if n.is_finite()
                    && n.fract() == 0.0
                    && *n >= (i64::MIN as f64)
                    && *n <= (i64::MAX as f64)
                {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            ExprValue::String(s) => s.clone(),
        }
    }
}

/// 式評価の実行コンテキスト（変数マップ）。
///
/// 正規表現キャプチャグループ（`$0`, `$1`）やループカウンタ（`$i`, `$n`）の値を保持します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::expr_subst::{ExprContext, ExprValue};
///
/// let mut ctx = ExprContext::new();
/// ctx.set_var("i", ExprValue::Number(1.0));
/// assert_eq!(ctx.get_var("i"), Some(&ExprValue::Number(1.0)));
/// ```
#[derive(Debug, Default)]
pub struct ExprContext {
    /// 登録変数マップ
    pub variables: HashMap<String, ExprValue>,
}

impl ExprContext {
    /// 空のコンテキストを生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::expr_subst::ExprContext;
    ///
    /// let ctx = ExprContext::new();
    /// assert!(ctx.variables.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 変数を登録または上書きします。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::expr_subst::{ExprContext, ExprValue};
    ///
    /// let mut ctx = ExprContext::new();
    /// ctx.set_var("x", ExprValue::Number(100.0));
    /// assert_eq!(ctx.get_var("x"), Some(&ExprValue::Number(100.0)));
    /// ```
    pub fn set_var(&mut self, name: &str, value: ExprValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// 変数値を取得します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::expr_subst::ExprContext;
    ///
    /// let ctx = ExprContext::new();
    /// assert_eq!(ctx.get_var("unknown"), None);
    /// ```
    pub fn get_var(&self, name: &str) -> Option<&ExprValue> {
        self.variables.get(name)
    }
}

/// 簡易トークン種別。
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    StringLiteral(String),
    Variable(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
    Assign,
    Eof,
}

/// 式の字句解析器。
struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();
        let c = match self.chars.next() {
            Some(ch) => ch,
            None => return Ok(Token::Eof),
        };

        match c {
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => Ok(Token::Slash),
            '%' => Ok(Token::Percent),
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '=' => Ok(Token::Assign),
            '"' | '\'' => {
                let quote = c;
                let mut s = String::new();
                let mut escaped = false;
                for ch in self.chars.by_ref() {
                    if escaped {
                        match ch {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            '\\' => s.push('\\'),
                            q if q == quote => s.push(quote),
                            _ => {
                                s.push('\\');
                                s.push(ch);
                            }
                        }
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == quote {
                        return Ok(Token::StringLiteral(s));
                    } else {
                        s.push(ch);
                    }
                }
                Err("文字列リテラルが閉じられていません".to_string())
            }
            '$' => {
                let mut var_name = String::new();
                while let Some(&ch) = self.chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        var_name.push(ch);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                if var_name.is_empty() {
                    Err("'$' の後に変数名が必要です".to_string())
                } else {
                    Ok(Token::Variable(var_name))
                }
            }
            '0'..='9' => {
                let mut num_str = String::new();
                num_str.push(c);
                let mut has_dot = false;
                while let Some(&ch) = self.chars.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(ch);
                        self.chars.next();
                    } else if ch == '.' && !has_dot {
                        has_dot = true;
                        num_str.push(ch);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                let val = num_str
                    .parse::<f64>()
                    .map_err(|e| format!("数値のパースに失敗しました: {e}"))?;
                Ok(Token::Number(val))
            }
            _ => {
                // クォートなしの単語識別子（変数または文字列リテラル）
                let mut ident = String::new();
                ident.push(c);
                while let Some(&ch) = self.chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                Ok(Token::StringLiteral(ident))
            }
        }
    }
}

/// 再帰下降構文解析器による式評価器。
///
/// 四則演算、文字列結合、変数参照、代入式を字句解析・構文解析して評価します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::expr_subst::{ExprContext, ExprParser, ExprValue};
///
/// let mut ctx = ExprContext::new();
/// let mut parser = ExprParser::new("1 + 2 * 3", &mut ctx).unwrap();
/// let val = parser.parse_expression().unwrap();
/// assert_eq!(val, ExprValue::Number(7.0));
/// ```
pub struct ExprParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    context: &'a mut ExprContext,
}

impl<'a> ExprParser<'a> {
    /// 式文字列とコンテキストからパーサーを初期化します。
    ///
    /// # Errors
    /// トークン分割（字句解析）に失敗した場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::expr_subst::{ExprContext, ExprParser};
    ///
    /// let mut ctx = ExprContext::new();
    /// let parser = ExprParser::new("10 + 20", &mut ctx);
    /// assert!(parser.is_ok());
    /// ```
    pub fn new(input: &'a str, context: &'a mut ExprContext) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token()?;
            if tok == Token::Eof {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        Ok(Self {
            tokens,
            pos: 0,
            context,
        })
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    /// 式を評価して結果値（[`ExprValue`]）を返します。
    ///
    /// # Errors
    /// 構文エラーやゼロ除算、未定義変数の参照が発生した場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::expr_subst::{ExprContext, ExprParser, ExprValue};
    ///
    /// let mut ctx = ExprContext::new();
    /// let mut parser = ExprParser::new("(5 - 2) * 4", &mut ctx).unwrap();
    /// assert_eq!(parser.parse_expression().unwrap(), ExprValue::Number(12.0));
    /// ```
    pub fn parse_expression(&mut self) -> Result<ExprValue, String> {
        self.parse_assignment()
    }

    /// 代入式: `$var = expr`
    fn parse_assignment(&mut self) -> Result<ExprValue, String> {
        if let (Token::Variable(var_name), Some(Token::Assign)) =
            (self.current().clone(), self.tokens.get(self.pos + 1))
        {
            self.advance(); // consume variable
            self.advance(); // consume '='
            let val = self.parse_additive()?;
            self.context.set_var(&var_name, val.clone());
            return Ok(val);
        }
        self.parse_additive()
    }

    /// 加減算 / 文字列結合: `+`, `-`
    fn parse_additive(&mut self) -> Result<ExprValue, String> {
        let mut left = self.parse_multiplicative()?;

        while matches!(self.current(), Token::Plus | Token::Minus) {
            let op = self.advance().clone();
            let right = self.parse_multiplicative()?;

            match op {
                Token::Plus => {
                    // どちらかが文字列なら文字列結合、両方数値（または数値変換可能）なら加算
                    match (&left, &right) {
                        (ExprValue::Number(a), ExprValue::Number(b)) => {
                            left = ExprValue::Number(a + b);
                        }
                        (ExprValue::String(s), ExprValue::Number(_)) => {
                            if let (Ok(a), Some(b)) = (s.parse::<f64>(), right.as_number()) {
                                left = ExprValue::Number(a + b);
                                continue;
                            }
                            left = ExprValue::String(format!("{}{}", s, right.to_display_string()));
                        }
                        (ExprValue::Number(_), ExprValue::String(s)) => {
                            if let (Some(a), Ok(b)) = (left.as_number(), s.parse::<f64>()) {
                                left = ExprValue::Number(a + b);
                                continue;
                            }
                            left = ExprValue::String(format!("{}{}", left.to_display_string(), s));
                        }
                        (ExprValue::String(a), ExprValue::String(b)) => {
                            if let (Ok(num_a), Ok(num_b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                                left = ExprValue::Number(num_a + num_b);
                            } else {
                                left = ExprValue::String(format!("{a}{b}"));
                            }
                        }
                    }
                }
                Token::Minus => {
                    let a = left
                        .as_number()
                        .ok_or_else(|| format!("減算の左辺が数値ではありません: {:?}", left))?;
                    let b = right
                        .as_number()
                        .ok_or_else(|| format!("減算の右辺が数値ではありません: {:?}", right))?;
                    left = ExprValue::Number(a - b);
                }
                _ => unreachable!(),
            }
        }

        Ok(left)
    }

    /// 乗除余: `*`, `/`, `%`
    fn parse_multiplicative(&mut self) -> Result<ExprValue, String> {
        let mut left = self.parse_unary()?;

        while matches!(self.current(), Token::Star | Token::Slash | Token::Percent) {
            let op = self.advance().clone();
            let right = self.parse_unary()?;

            let a = left
                .as_number()
                .ok_or_else(|| format!("算術演算の左辺が数値ではありません: {:?}", left))?;
            let b = right
                .as_number()
                .ok_or_else(|| format!("算術演算の右辺が数値ではありません: {:?}", right))?;

            left = match op {
                Token::Star => ExprValue::Number(a * b),
                Token::Slash => {
                    if b == 0.0 {
                        return Err("ゼロ除算が発生しました".to_string());
                    }
                    ExprValue::Number(a / b)
                }
                Token::Percent => {
                    if b == 0.0 {
                        return Err("ゼロ除算余りが発生しました".to_string());
                    }
                    ExprValue::Number(a % b)
                }
                _ => unreachable!(),
            };
        }

        Ok(left)
    }

    /// 単項演算: `+`, `-`
    fn parse_unary(&mut self) -> Result<ExprValue, String> {
        match self.current() {
            Token::Plus => {
                self.advance();
                self.parse_unary()
            }
            Token::Minus => {
                self.advance();
                let val = self.parse_unary()?;
                let num = val
                    .as_number()
                    .ok_or_else(|| format!("単項マイナスの対象が数値ではありません: {:?}", val))?;
                Ok(ExprValue::Number(-num))
            }
            _ => self.parse_primary(),
        }
    }

    /// プライマリ要素: 数値、文字列、変数、括弧
    fn parse_primary(&mut self) -> Result<ExprValue, String> {
        match self.current().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(ExprValue::Number(n))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(ExprValue::String(s))
            }
            Token::Variable(name) => {
                self.advance();
                self.context
                    .get_var(&name)
                    .cloned()
                    .ok_or_else(|| format!("未定義の変数です: ${name}"))
            }
            Token::LParen => {
                self.advance(); // consume '('
                let val = self.parse_expression()?;
                if self.current() != &Token::RParen {
                    return Err("閉じ括弧 ')' がありません".to_string());
                }
                self.advance(); // consume ')'
                Ok(val)
            }
            tok => Err(format!("予期しないトークンです: {:?}", tok)),
        }
    }
}

/// 正規表現マッチ箇所に対して数式・文字列評価式を適用して一括置換します。
///
/// マッチしたキャプチャグループ（`$0`, `$1`, ...）や組み込み変数（`$i`, `$n`, `$line`）を
/// 計算式内で参照し、動的に算出した値に置換します。
///
/// # 引数
/// - `text`: 対象文字列（単一行または複数行バッファ全体）
/// - `pattern`: 検索正規表現パターン文字列
/// - `expr_template`: 各マッチに対して評価する式（例: `"$0 * 10"` や `"$1 + \": \" + ($line * 2)"`）
/// - `is_all`: `true` の場合、マッチした全箇所を置換（`g` フラグ相当）。`false` の場合は最初の一致のみ置換
/// - `ignore_case`: `true` の場合、英大文字・小文字を同一視（`i` フラグ相当）
/// - `start_line`: 行番号変数の基準となる開始行番号（1-based）
///
/// # Errors
/// - 正規表現 `pattern` の構文が不正な場合にエラー文字列を返します。
/// - `expr_template` の字句解析または構文解析・型評価に失敗した場合にエラー文字列を返します。
/// - 未定義の変数を参照した場合にエラー文字列を返します。
///
/// # Examples
/// ```
/// use vivivi_modes::expr_subst::execute_expr_substitute;
///
/// let input = "item_1, item_2, item_3";
/// // 各数値を 10 倍に置換
/// let (result, count) = execute_expr_substitute(input, r"\d+", "$0 * 10", true, false, 1).unwrap();
/// assert_eq!(result, "item_10, item_20, item_30");
/// assert_eq!(count, 3);
/// ```
pub fn execute_expr_substitute(
    text: &str,
    pattern: &str,
    expr_template: &str,
    is_all: bool,
    ignore_case: bool,
    start_line: usize,
) -> Result<(String, usize), String> {
    let re = RegexBuilder::new(pattern)
        .case_insensitive(ignore_case)
        .multi_line(true)
        .build()
        .map_err(|e| format!("正規表現エラー: {e}"))?;

    let mut context = ExprContext::new();
    let mut match_count = 0;
    let mut result_text = String::new();
    let mut last_end = 0;

    // 行番号計算用
    let mut current_line = start_line;
    let mut line_byte_offset = 0;

    for caps in re.captures_iter(text) {
        let mat = match caps.get(0) {
            Some(m) => m,
            None => continue,
        };
        match_count += 1;
        if !is_all && match_count > 1 {
            break;
        }

        // マッチ位置までの行番号をカウント
        let match_start = mat.start();
        while let Some(nl_pos) = text[line_byte_offset..match_start].find('\n') {
            current_line += 1;
            line_byte_offset += nl_pos + 1;
        }

        result_text.push_str(&text[last_end..match_start]);

        // キャプチャグループの収集
        for (idx, cap_match) in caps.iter().enumerate() {
            if let Some(m) = cap_match {
                context.set_var(&format!("{idx}"), ExprValue::String(m.as_str().to_string()));
            }
        }

        // 組み込み変数設定
        context.set_var("i", ExprValue::Number(match_count as f64));
        context.set_var("n", ExprValue::Number(match_count as f64));
        context.set_var("line", ExprValue::Number(current_line as f64));

        // 式のパースと評価
        let mut parser = ExprParser::new(expr_template, &mut context)?;
        let evaluated = parser.parse_expression()?;
        result_text.push_str(&evaluated.to_display_string());

        last_end = mat.end();
    }

    result_text.push_str(&text[last_end..]);
    Ok((result_text, match_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_expression() {
        let mut ctx = ExprContext::new();
        ctx.set_var("1", ExprValue::String("25".to_string()));
        ctx.set_var("i", ExprValue::Number(3.0));

        let mut parser = ExprParser::new("$1 * 2 + $i", &mut ctx).unwrap();
        let val = parser.parse_expression().unwrap();
        assert_eq!(val, ExprValue::Number(53.0));
        assert_eq!(val.to_display_string(), "53");
    }

    #[test]
    fn test_renumbering_substitute() {
        let input = "item\nitem\nitem";
        let (output, count) =
            execute_expr_substitute(input, "item", "\"item_\" + $i", true, false, 1).unwrap();
        assert_eq!(count, 3);
        assert_eq!(output, "item_1\nitem_2\nitem_3");
    }

    #[test]
    fn test_numeric_multiplication_substitute() {
        let input = "width: 10px;\nheight: 20px;";
        let (output, count) =
            execute_expr_substitute(input, r"(\d+)px", r#""px_" + ($1 * 2)"#, true, false, 1)
                .unwrap();
        assert_eq!(count, 2);
        assert_eq!(output, "width: px_20;\nheight: px_40;");
    }

    #[test]
    fn test_line_number_variable() {
        let input = "alpha\nbeta\ngamma";
        let (output, count) =
            execute_expr_substitute(input, "^[a-z]+", "$line + \": \" + $0", true, false, 1)
                .unwrap();
        assert_eq!(count, 3);
        assert_eq!(output, "1: alpha\n2: beta\n3: gamma");
    }
}
