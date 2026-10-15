//! テキスト検索オプションおよびマッチ結果モデル。

/// テキスト検索のオプション設定構造体。
///
/// 大小文字無視、正規表現検索、単語単位検索の各フラグを保持します。
///
/// # Examples
///
/// ```
/// use vivivi_core::search::SearchOptions;
///
/// let opts = SearchOptions {
///     ignore_case: true,
///     use_regex: false,
///     whole_word: true,
/// };
/// assert!(opts.ignore_case);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// 大文字・小文字を区別しないか（`true` のとき無視）
    pub ignore_case: bool,
    /// 正規表現パターンとして解釈するか
    pub use_regex: bool,
    /// 単語境界に一致するもののみを対象とするか
    pub whole_word: bool,
}

/// テキスト検索の一致箇所を表す構造体。
///
/// 一致した行番号、文字開始列、および一致文字列の文字数を保持します。
///
/// # Examples
///
/// ```
/// use vivivi_core::search::SearchMatch;
///
/// let m = SearchMatch { line: 0, col: 5, length: 3 };
/// assert_eq!(m.line, 0);
/// assert_eq!(m.length, 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    /// 0始まりの一致行番号
    pub line: usize,
    /// 0始まりの一致開始列番号
    pub col: usize,
    /// 一致した文字列の文字長
    pub length: usize,
}
