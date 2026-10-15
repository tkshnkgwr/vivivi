//! vivivi-modes: 罫線描画、インラインCSV/TSV、Diff、アウトライン等の高度編集モード群。
//!
//! 日本語全角文字を考慮した罫線モード（[`keisen`]）、高速アライメント（[`align`]）、
//! 矩形計算（[`calc`]）、文字種変換（[`conv`]）、Migemo 検索（[`migemo`]）など、
//! ViVi の高度な拡張編集機能を純粋な Rust ロジック（UI非依存）で提供します。
//!
//! # Examples
//!
//! ```
//! use vivivi_modes::conv::{ConversionType, convert_text};
//!
//! let converted = convert_text("あいうえお", ConversionType::ToKatakana);
//! assert_eq!(converted, "アイウエオ");
//! ```

pub mod abbrev;
pub mod align;
pub mod calc;
pub mod conv;
pub mod csv;
pub mod dabbrev;
pub mod dynamic_macro;
pub mod keisen;
pub mod lex_edit;
pub mod line_edit;
pub mod migemo;
pub mod outline;
pub mod syntax;
pub mod zen_coding;

pub mod expr_subst;
pub mod fftt;
pub mod renumber;

pub mod diff;
pub mod difflog;
pub mod filer;
pub mod folding;
pub mod grep;
pub mod gutter;
pub mod invisible;
pub mod print;
pub mod project;
pub mod recent;
pub mod template;
pub mod todo;

pub use abbrev::AbbrevManager;
pub use align::align_lines;
pub use calc::{BoxCalculationResult, calculate_box_lines, calculate_box_selection};
pub use conv::{ConversionType, convert_text};
pub use csv::{CsvCell, CsvMode, CsvOptions, Delimiter};
pub use dabbrev::DabbrevSession;
pub use diff::{
    DiffLine, DiffType, SideBySideLine, apply_unified_patch, create_side_by_side,
    create_unified_diff, diff_lines,
};
pub use difflog::{BlameInfo, DiffLog, DiffLogEntry};
pub use dynamic_macro::DynamicMacroEngine;
pub use expr_subst::{ExprContext, ExprParser, ExprValue, execute_expr_substitute};
pub use fftt::{char_matches_fftt, find_char_backward, find_char_forward};
pub use filer::{ActiveFilerPane, FileEntry, FilerPane, TwoPaneFiler};
pub use folding::{FoldManager, FoldRange};
pub use grep::{GrepMatch, GrepSession, execute_grep};
pub use gutter::{ChangeTracker, LineChangeState};
pub use invisible::{InvisibleChar, ListCharsConfig, format_line_invisibles};
pub use keisen::{BoxAlignment, KeisenDirection, KeisenMode, KeisenType};
pub use lex_edit::{LexEditEngine, WordEditRecord};
pub use line_edit::insert_beg_end_lines;
pub use migemo::MigemoEngine;
pub use outline::{OutlineItem, OutlineKind, detect_language_from_content, parse_outline};
pub use print::{PrintPage, PrintSettings, export_to_pdf, format_preview_text, paginate_text};
pub use project::ProjectManager;
pub use recent::RecentManager;
pub use renumber::renumber_lines;
pub use syntax::{SyntaxLang, SyntaxToken, TokenType, tokenize_line};
pub use template::{TemplateManager, TemplateResult};
pub use todo::{TodoItem, TodoTag, scan_buffer_todos, scan_dir_todos};
pub use zen_coding::{ZenElement, expand_zen_coding};
