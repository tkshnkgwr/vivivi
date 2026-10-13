use vivivi_core::TextBuffer;
use vivivi_modes::{CsvMode, KeisenMode};
use vivivi_vi::ViEngine;

fn main() {
    println!("=== vivivi: OpenViVi reconstructed in Rust ===");
    let buf = TextBuffer::from_text("Hello, vivivi!\nWelcome to OpenViVi reimplementation.");
    let mut vi = ViEngine::new();
    let csv = CsvMode::default();
    let keisen = KeisenMode::default();

    println!("Lines: {}", buf.line_count());
    println!("Vi Mode: {}", vi.mode().name());
    println!("CSV Mode options: {:?}", csv.options);
    println!("Keisen Mode active: {}", keisen.is_active);

    vi.feed_char('i');
    println!("After 'i' key, Mode: {}", vi.mode().name());
    vi.feed_char('\x1b');
    println!("After Escape key, Mode: {}", vi.mode().name());
}
