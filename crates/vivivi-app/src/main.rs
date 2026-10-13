use vivivi_core::{CursorPosition, TextBuffer};
use vivivi_modes::{CsvMode, KeisenMode};
use vivivi_vi::ViEngine;

fn main() {
    println!("=== vivivi: OpenViVi reconstructed in Rust ===");
    let mut buf = TextBuffer::from_text("Hello, vivivi!\nWelcome to OpenViVi reimplementation.");
    let mut vi = ViEngine::new();
    let csv = CsvMode::default();
    let keisen = KeisenMode::default();

    println!("Lines: {}", buf.line_count());
    println!("Initial text:\n{}", buf);

    // Test Undo/Redo & GapBuffer integration
    buf.open_undo_block();
    let pos = buf.cursor_to_position(CursorPosition::new(0, 14));
    buf.insert_text(pos, " [Rust Edition]");
    buf.close_undo_block();

    println!("\nAfter insertion:\n{}", buf);
    println!("Is modified: {}", buf.is_modified());

    buf.undo();
    println!("\nAfter undo:\n{}", buf);
    println!("Is modified: {}", buf.is_modified());

    buf.redo();
    println!("\nAfter redo:\n{}", buf);

    println!("\nVi Mode: {}", vi.mode().name());
    println!("CSV Mode options: {:?}", csv.options);
    println!("Keisen Mode active: {}", keisen.is_active);

    vi.feed_char('i');
    println!("After 'i' key, Mode: {}", vi.mode().name());
    vi.feed_char('\x1b');
    println!("After Escape key, Mode: {}", vi.mode().name());
}
