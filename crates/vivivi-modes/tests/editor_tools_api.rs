//! エディタ拡張ツール・セッション管理機能の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、`abbrev`（略語展開）、`dabbrev`（動的補完）、
//! `dynamic_macro`（動的マクロ）、`filer`（2画面ファイラー）、`grep`（検索セッション）、
//! `lex_edit`（編集語彙学習）、`migemo`（ローマ字検索）、`project`（プロジェクト管理）、
//! `recent`（最近開いたファイル）、`template`（テンプレート）、`todo`（TODO一覧）を検証します。

use std::path::PathBuf;
use vivivi_core::TextBuffer;
use vivivi_modes::abbrev::AbbrevManager;
use vivivi_modes::dabbrev::DabbrevSession;
use vivivi_modes::dynamic_macro::DynamicMacroEngine;
use vivivi_modes::filer::{ActiveFilerPane, TwoPaneFiler};
use vivivi_modes::grep::{GrepMatch, GrepSession};
use vivivi_modes::lex_edit::LexEditEngine;
use vivivi_modes::migemo::MigemoEngine;
use vivivi_modes::project::ProjectManager;
use vivivi_modes::recent::RecentManager;
use vivivi_modes::template::TemplateManager;
use vivivi_modes::todo::{TodoTag, scan_buffer_todos};

// ==========================================
// 1. Abbrev (略語展開)
// ==========================================

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

// ==========================================
// 2. Dabbrev (動的補完)
// ==========================================

#[test]
fn test_dabbrev_session_cycling() {
    let text = "apple_pie\napple_juice\napple_sauce\napp";
    let buffer = TextBuffer::from_text(text);
    let mut session = DabbrevSession::new();

    let first = session.start(&buffer, 3, 3).expect("should start session");
    assert_eq!(first, "apple_sauce");

    let second = session.next_candidate().expect("next candidate");
    assert_eq!(second, "apple_juice");

    let third = session.next_candidate().expect("third candidate");
    assert_eq!(third, "apple_pie");

    let wrapped = session.next_candidate().expect("wrap to first");
    assert_eq!(wrapped, "apple_sauce");
}

// ==========================================
// 3. Dynamic Macro (動的マクロ)
// ==========================================

#[test]
fn test_dynamic_macro_simple_repeat() {
    let mut dmacro = DynamicMacroEngine::new();
    for ch in "abcabc".chars() {
        dmacro.record_char(ch);
    }
    let repeat = dmacro.detect_repeat();
    assert_eq!(repeat, Some(vec!['a', 'b', 'c']));
}

#[test]
fn test_dynamic_macro_longest_match() {
    let mut dmacro = DynamicMacroEngine::new();
    for ch in "foobarfoobar".chars() {
        dmacro.record_char(ch);
    }
    let repeat = dmacro.detect_repeat();
    assert_eq!(repeat, Some("foobar".chars().collect()));
}

// ==========================================
// 4. Filer (ファイラーセッション)
// ==========================================

#[test]
fn test_filer_switch_and_select() {
    let mut filer = TwoPaneFiler::new(".");
    assert_eq!(filer.active, ActiveFilerPane::Left);
    filer.switch_pane();
    assert_eq!(filer.active, ActiveFilerPane::Right);
    filer.switch_pane();
    assert_eq!(filer.active, ActiveFilerPane::Left);
}

// ==========================================
// 5. Grep (Grep検索セッション)
// ==========================================

#[test]
fn test_grep_session_nav() {
    let mut session = GrepSession {
        pattern: "test".into(),
        matches: vec![
            GrepMatch {
                file_path: PathBuf::from("a.rs"),
                lineno: 10,
                col: 5,
                line_text: "test 1".into(),
                match_len: 4,
            },
            GrepMatch {
                file_path: PathBuf::from("b.rs"),
                lineno: 20,
                col: 1,
                line_text: "test 2".into(),
                match_len: 4,
            },
        ],
        current_idx: 0,
    };

    assert_eq!(session.current_match().unwrap().lineno, 10);
    session.next();
    assert_eq!(session.current_match().unwrap().lineno, 20);
}

// ==========================================
// 6. LexEdit (語彙学習・置換予測)
// ==========================================

#[test]
fn test_exact_match_prediction() {
    let mut engine = LexEditEngine::new();
    engine.record_edit("old_fn", "new_fn");

    let candidates = engine.predict("old_fn");
    assert_eq!(candidates, vec!["new_fn"]);
}

#[test]
fn test_prefix_analogy_prediction() {
    let mut engine = LexEditEngine::new();
    engine.record_edit("user", "member");

    let candidates = engine.predict("user_name");
    assert_eq!(candidates, vec!["member_name"]);
}

// ==========================================
// 7. Migemo (ローマ字検索)
// ==========================================

#[test]
fn test_migemo_romaji_syllable_matching() {
    let migemo = MigemoEngine::new();
    let regex = migemo.compile_query("nihon").expect("should compile");

    assert!(regex.is_match("これはにほんです"));
    assert!(regex.is_match("これはニホンです"));
    assert!(regex.is_match("this is nihon"));
    assert!(!regex.is_match("これはとうきょうです"));
}

// ==========================================
// 8. Project (プロジェクト管理)
// ==========================================

#[test]
fn test_project_manager_add_remove() {
    let mut pm = ProjectManager::new(".");
    pm.add_file("src/main.rs");
    assert_eq!(pm.open_files.len(), 1);
    pm.remove_file("src/main.rs");
    assert_eq!(pm.open_files.len(), 0);
}

// ==========================================
// 9. Recent (最近開いたファイル)
// ==========================================

#[test]
fn test_recent_manager_add_and_truncate() {
    let mut mgr = RecentManager::new();
    mgr.max_recent = 3;

    mgr.add_recent(PathBuf::from("a.txt"));
    mgr.add_recent(PathBuf::from("b.txt"));
    mgr.add_recent(PathBuf::from("c.txt"));
    mgr.add_recent(PathBuf::from("d.txt"));

    assert_eq!(mgr.recent_files.len(), 3);
    assert_eq!(mgr.recent_files[0], PathBuf::from("d.txt"));
    assert_eq!(mgr.recent_files[1], PathBuf::from("c.txt"));
    assert_eq!(mgr.recent_files[2], PathBuf::from("b.txt"));

    mgr.add_recent(PathBuf::from("b.txt"));
    assert_eq!(mgr.recent_files.len(), 3);
    assert_eq!(mgr.recent_files[0], PathBuf::from("b.txt"));
}

// ==========================================
// 10. Template (テンプレート展開)
// ==========================================

#[test]
fn test_template_macro_expansion() {
    let mut mgr = TemplateManager::new();
    let raw = "// File: %FileName%\n// Author: %UserName%\n// Date: %Date%\nfn main() {\n    %Cursor%\n}\n";
    mgr.register_template("rs", raw);

    let result = mgr
        .load_and_expand("src/app.rs", "2026-10-14", "12:00:00")
        .expect("template loaded");

    assert!(result.text.contains("// File: app.rs"));
    assert!(result.text.contains("// Date: 2026-10-14"));
    assert!(!result.text.contains("%Cursor%"));
    assert!(result.cursor_pos > 0);
}

// ==========================================
// 11. TODO (TODO一覧抽出)
// ==========================================

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
