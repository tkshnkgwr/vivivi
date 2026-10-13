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

    // --- Phase 2: Keisen (Box-drawing) & Alignment Demo ---
    println!("\n=== Phase 2 Demo: Keisen Mode ===");
    let mut keisen_buf = TextBuffer::new();
    let mut keisen_cur = CursorPosition::new(0, 0);
    // Draw 3 horizontal steps
    keisen.step(
        &mut keisen_buf,
        &mut keisen_cur,
        vivivi_modes::KeisenDirection::Right,
        false,
    );
    keisen.step(
        &mut keisen_buf,
        &mut keisen_cur,
        vivivi_modes::KeisenDirection::Right,
        false,
    );
    keisen.step(
        &mut keisen_buf,
        &mut keisen_cur,
        vivivi_modes::KeisenDirection::Down,
        false,
    );
    println!("Drawn Keisen buffer:\n{}", keisen_buf);

    let mut box_buf = TextBuffer::from_text("│ hello     │");
    KeisenMode::align_box_text(&mut box_buf, 0, vivivi_modes::BoxAlignment::Center);
    println!("Centered box text: {}", box_buf);

    // --- Phase 2: CSV Mode Demo ---
    println!("\n=== Phase 2 Demo: Inline CSV Mode ===");
    let mut csv_mgr = CsvMode::default();
    let sample_csv = ["id,name,score", "1,\"Alice Smith\",95", "2,\"Bob\",80"];
    csv_mgr.calculate_widths(sample_csv.iter().copied());
    println!("Column widths: {:?}", csv_mgr.column_widths);
    for (i, line) in sample_csv.iter().enumerate() {
        println!("Row {}: {}", i, csv_mgr.format_line(line, false));
    }
}
