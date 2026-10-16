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

#[derive(Debug, Default)]
struct MockClipboard {
    text: String,
}

impl vivivi_vi::ClipboardProvider for MockClipboard {
    fn get_text(&mut self) -> Result<String, String> {
        Ok(self.text.clone())
    }
    fn set_text(&mut self, text: String) -> Result<(), String> {
        self.text = text;
        Ok(())
    }
}

#[test]
fn test_system_clipboard_sync() {
    use std::sync::{Arc, Mutex};
    use vivivi_vi::ClipboardOption;

    let mock = Arc::new(Mutex::new(MockClipboard::default()));
    let mut mgr = RegisterMgr::new();
    mgr.set_clipboard_provider(mock.clone());

    // 1. "+ レジスタへの設定 -> MockClipboard に同期される
    mgr.set(
        Some('+'),
        RegisterContent::CharacterWise("from_plus".to_string()),
    );
    assert_eq!(mock.lock().unwrap().text, "from_plus");
    assert_eq!(mgr.get(Some('+')).unwrap().as_str(), "from_plus");

    // 2. 外部クリップボードの更新 -> fetch(Some('+')) で取得できる
    mock.lock().unwrap().text = "external_text\n".to_string();
    let fetched = mgr.fetch(Some('+')).unwrap();
    assert_eq!(fetched.as_str(), "external_text\n");
    assert!(fetched.is_line_wise());

    // 3. clipboard=unnamed の検証
    mgr.set_clipboard_option(ClipboardOption::Unnamed);
    mgr.set(
        None,
        RegisterContent::CharacterWise("unnamed_val".to_string()),
    );
    assert_eq!(mock.lock().unwrap().text, "unnamed_val");

    mock.lock().unwrap().text = "new_external".to_string();
    let fetched_unnamed = mgr.fetch(None).unwrap();
    assert_eq!(fetched_unnamed.as_str(), "new_external");
}
