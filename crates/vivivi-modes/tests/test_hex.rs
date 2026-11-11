//! バイナリモード (Hex Editor) 公開API統合テスト (tests/test_hex.rs)

use vivivi_modes::hex::{
    HexCursor, HexDocument, HexPane, Nibble, calculate_row_count, format_hex_row,
};

#[test]
fn test_hex_document_creation_and_access() {
    let doc = HexDocument::from_text("Hello, World!");
    assert_eq!(doc.len(), 13);
    assert!(!doc.is_empty());
    assert!(!doc.is_dirty());
    assert_eq!(doc.get_byte(0), Some(b'H'));
    assert_eq!(doc.get_byte(12), Some(b'!'));
    assert_eq!(doc.get_byte(13), None);
    assert_eq!(doc.to_utf8_lossy(), "Hello, World!");
}

#[test]
fn test_hex_document_set_byte_and_undo_redo() {
    let mut doc = HexDocument::from_text("ABCDEF");
    assert!(!doc.is_dirty());

    // 1バイト置換 ('B' -> 'X')
    assert!(doc.set_byte(1, b'X'));
    assert_eq!(doc.data(), b"AXCDEF");
    assert!(doc.is_dirty());

    // Undo
    assert!(doc.undo());
    assert_eq!(doc.data(), b"ABCDEF");

    // Redo
    assert!(doc.redo());
    assert_eq!(doc.data(), b"AXCDEF");
}

#[test]
fn test_hex_document_set_nibble() {
    let mut doc = HexDocument::from_bytes(vec![0x00]);

    // 上位ニブルを 0xA に更新 -> 0xA0
    assert!(doc.set_nibble(0, Nibble::High, 0x0A));
    assert_eq!(doc.get_byte(0), Some(0xA0));

    // 下位ニブルを 0x5 に更新 -> 0xA5
    assert!(doc.set_nibble(0, Nibble::Low, 0x05));
    assert_eq!(doc.get_byte(0), Some(0xA5));

    // Undo で元に戻る
    assert!(doc.undo());
    assert_eq!(doc.get_byte(0), Some(0xA0));
    assert!(doc.undo());
    assert_eq!(doc.get_byte(0), Some(0x00));
}

#[test]
fn test_hex_document_insert_and_delete() {
    let mut doc = HexDocument::from_bytes(vec![0x10, 0x20, 0x30]);

    // 挿入
    doc.insert_bytes(1, &[0xAA, 0xBB]);
    assert_eq!(doc.data(), &[0x10, 0xAA, 0xBB, 0x20, 0x30]);
    assert_eq!(doc.len(), 5);

    // Undo
    assert!(doc.undo());
    assert_eq!(doc.data(), &[0x10, 0x20, 0x30]);

    // 削除
    let deleted = doc.delete_bytes(1, 2);
    assert_eq!(deleted, vec![0x20, 0x30]);
    assert_eq!(doc.data(), &[0x10]);

    // Undo で復活
    assert!(doc.undo());
    assert_eq!(doc.data(), &[0x10, 0x20, 0x30]);
}

#[test]
fn test_hex_document_search_bytes() {
    let doc = HexDocument::from_text("Hello, Hello World!");
    let matches = doc.search_bytes(b"Hello");
    assert_eq!(matches, vec![0, 7]);

    let no_match = doc.search_bytes(b"Rust");
    assert!(no_match.is_empty());
}

#[test]
fn test_format_hex_row() {
    let data = b"Hello World! 1234"; // 17バイト (16バイト + 1バイト)
    assert_eq!(calculate_row_count(data.len()), 2);

    // 1行目 (16バイト)
    let row0 = format_hex_row(data, 0).expect("row 0 should exist");
    assert_eq!(row0.offset, 0);
    assert_eq!(row0.byte_count, 16);
    assert_eq!(row0.ascii_dump, "Hello World! 123");
    assert!(row0.hex_dump.starts_with("48 65 6c 6c 6f"));

    // 2行目 (1バイト)
    let row1 = format_hex_row(data, 1).expect("row 1 should exist");
    assert_eq!(row1.offset, 16);
    assert_eq!(row1.byte_count, 1);
    assert_eq!(row1.ascii_dump, "4");

    // 3行目 (範囲外)
    assert!(format_hex_row(data, 2).is_none());
}

#[test]
fn test_hex_cursor_toggle_pane() {
    let mut cursor = HexCursor::new();
    assert_eq!(cursor.pane, HexPane::Hex);
    assert_eq!(cursor.offset, 0);

    cursor.toggle_pane();
    assert_eq!(cursor.pane, HexPane::Ascii);

    cursor.toggle_pane();
    assert_eq!(cursor.pane, HexPane::Hex);
}
