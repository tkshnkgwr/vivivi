//! `RegisterMgr` の公開API統合テスト
//!
//! クレート外部の利用者視点（ブラックボックス）で、無名レジスタの基本操作、
//! 名前付きレジスタとの自動同期、大文字レジスタによる追記、クリップボード履歴の振る舞いを検証します。

use vivivi_vi::{RegisterContent, RegisterMgr};

#[test]
fn test_register_basic_unnamed() {
    let mut mgr = RegisterMgr::new();
    assert!(mgr.get(None).is_none());

    mgr.set(None, RegisterContent::CharacterWise("hello".to_string()));
    let content = mgr.get(None).unwrap();
    assert_eq!(content.as_str(), "hello");
    assert!(!content.is_line_wise());
}

#[test]
fn test_named_register_and_unnamed_sync() {
    let mut mgr = RegisterMgr::new();

    // "a に設定すると、無名レジスタにも同期される
    mgr.set(Some('a'), RegisterContent::LineWise("line1\n".to_string()));
    assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "line1\n");
    assert_eq!(mgr.get(None).unwrap().as_str(), "line1\n");

    // "b に別内容を設定
    mgr.set(
        Some('b'),
        RegisterContent::CharacterWise("word".to_string()),
    );
    assert_eq!(mgr.get(Some('b')).unwrap().as_str(), "word");
    // "a は維持される
    assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "line1\n");
}

#[test]
fn test_uppercase_register_append() {
    let mut mgr = RegisterMgr::new();
    mgr.set(Some('a'), RegisterContent::CharacterWise("foo".to_string()));

    // 大文字 "A でアペンド
    mgr.set(Some('A'), RegisterContent::CharacterWise("bar".to_string()));
    assert_eq!(mgr.get(Some('a')).unwrap().as_str(), "foobar");
}

#[test]
fn test_clipboard_history() {
    let mut mgr = RegisterMgr::new();
    mgr.set(None, RegisterContent::CharacterWise("first".to_string()));
    mgr.set(
        Some('a'),
        RegisterContent::CharacterWise("second".to_string()),
    );
    // 同一内容の連続は重複排除される
    mgr.set(None, RegisterContent::CharacterWise("second".to_string()));
    mgr.set(None, RegisterContent::CharacterWise("third".to_string()));

    let history = mgr.history();
    assert_eq!(history.len(), 3);
    assert_eq!(history.get(0).unwrap().content.as_str(), "third");
    assert_eq!(history.get(1).unwrap().content.as_str(), "second");
    assert_eq!(history.get(2).unwrap().content.as_str(), "first");
}
