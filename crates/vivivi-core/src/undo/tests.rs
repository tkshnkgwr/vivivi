//! Undo/Redo マネージャのユニットテスト。

use super::*;

#[test]
fn test_continuous_insert_merging() {
    let mut undo_mgr = UndoMgr::new();
    undo_mgr.push_insert(0, "h");
    undo_mgr.push_insert(1, "e");
    undo_mgr.push_insert(2, "l");
    undo_mgr.push_insert(3, "l");
    undo_mgr.push_insert(4, "o");

    assert_eq!(undo_mgr.undo_stack.len(), 1);
    if let UndoAction::Insert { text, .. } = &undo_mgr.undo_stack[0].actions[0] {
        assert_eq!(text, "hello");
    } else {
        panic!("Expected Insert action");
    }

    let undo_actions = undo_mgr.pop_undo().unwrap();
    assert_eq!(undo_actions.len(), 1);
    assert_eq!(
        undo_actions[0],
        UndoAction::Delete {
            pos: 0,
            text: "hello".to_string(),
            is_backspace: false,
        }
    );
}

#[test]
fn test_undo_block_transaction() {
    let mut undo_mgr = UndoMgr::new();
    undo_mgr.open_block();
    undo_mgr.push_delete(0, "old", false);
    undo_mgr.push_insert(0, "new");
    undo_mgr.close_block();

    assert_eq!(undo_mgr.undo_stack.len(), 1);
    assert_eq!(undo_mgr.undo_stack[0].actions.len(), 2);

    let reverse = undo_mgr.pop_undo().unwrap();
    assert_eq!(reverse.len(), 2);
    // 逆順に取り消し: "new" を削除、次に "old" を挿入
    assert_eq!(
        reverse[0],
        UndoAction::Delete {
            pos: 0,
            text: "new".to_string(),
            is_backspace: false,
        }
    );
    assert_eq!(
        reverse[1],
        UndoAction::Insert {
            pos: 0,
            text: "old".to_string(),
            prohibit_merge: true,
        }
    );
}

#[test]
fn test_undo_serialize_deserialize() {
    let mut undo_mgr = UndoMgr::new();
    undo_mgr.open_block();
    undo_mgr.push_insert(0, "line 1\nline 2");
    undo_mgr.push_delete(5, "\"quoted text\\escaped\"", true);
    undo_mgr.push_replace(10, "old\nfoo", "new\nbar");
    undo_mgr.close_block();
    undo_mgr.on_saved();

    let serialized = undo_mgr.serialize();
    let restored = UndoMgr::deserialize(&serialized).expect("deserialize failed");

    assert_eq!(undo_mgr.undo_stack.len(), restored.undo_stack.len());
    assert_eq!(undo_mgr.save_point, restored.save_point);
    assert_eq!(undo_mgr.undo_stack[0], restored.undo_stack[0]);
}

#[test]
fn test_post_save_typing_modified_state() {
    let mut undo_mgr = UndoMgr::new();
    undo_mgr.push_insert(0, "hello");
    assert!(undo_mgr.is_modified());

    undo_mgr.on_saved();
    assert!(!undo_mgr.is_modified());

    // 保存直後に連続して文字を入力
    undo_mgr.push_insert(5, "!");
    // マージが禁止され、新しいブロックとして追加されるため Modified が true になる
    assert!(undo_mgr.is_modified());
    assert_eq!(undo_mgr.undo_stack.len(), 2);
}

#[test]
fn test_undo_serialize_complex_escaped_quotes() {
    let mut undo_mgr = UndoMgr::new();
    undo_mgr.open_block();
    // バックスラッシュ直後にダブルクォートを含む文字列
    undo_mgr.push_insert(0, r#"let s = "hello \"world\"\\";"#);
    undo_mgr.push_replace(10, r#"\"old\""#, r#"\"new\""#);
    undo_mgr.close_block();

    let serialized = undo_mgr.serialize();
    let restored = UndoMgr::deserialize(&serialized).expect("deserialize complex escape failed");

    assert_eq!(undo_mgr.undo_stack, restored.undo_stack);
}
