# ViViVi: Next-Generation Desktop GUI Architecture & Technical Specifications (GUI_DESIGN)

> [!NOTE]
> This specification systematically outlines the technical selection, rendering architecture, design principles, and implementation roadmap for the future desktop GUI frontend (Phase 5).
> (Synthesized and elevated from mobile/web Gemini architectural discussions).

---

## 1. Development Workflow & AI Model Selection Strategy (Gemini Flash vs Gemini Pro / Argon)

Optimal model selection strategy based on task characteristics in agentic AI environments such as Antigravity.

### 1.1 Model Characteristics & Task Allocation Criteria

| Development Phase / Problem Domain        | Recommended Model        | Rationale & Role                                                                                               |
| :---------------------------------------- | :----------------------- | :------------------------------------------------------------------------------------------------------------- |
| **GUI Boundaries & Async Architecture**   | **Gemini Pro / 4 Argon** | UI-worker channel design and lifecycle/concurrency invariants.                                                 |
| **Complex Rust Borrow & Lifetime Errors** | **Gemini Pro / 4 Argon** | Mutable aliasing inside immediate-mode (`eframe::App::update`) and thread boundary bounds (`Send + 'Sync`).    |
| **Multi-Crate Workspace Boundaries**      | **Gemini Pro / 4 Argon** | Re-architecting boundaries between `vivivi-core`, `vivivi-vi`, `vivivi-modes`, `vivivi-app`, and `vivivi-gui`. |
| **Widget Placement & Layout Tweaks**      | **Gemini 3.8 Flash**     | Rapid trial-and-error leveraging ultra-low latency code generation.                                            |
| **Local Syntax Fixes & Refactoring**      | **Gemini 3.8 Flash**     | High-velocity incremental tasks maintaining flow state.                                                        |

### 1.2 Practical Switching Rules
1. **Drive normal implementation with Flash**: Basic coding, widget additions, visual tweaks.
2. **Switch to Pro / Argon on "2-Round Trip Rule"**: If 2 round-trips with Flash fail to resolve a compiler borrow check error, switch immediately to Pro / Argon, feeding raw `cargo check` logs to review core architectural assumptions.
3. **Return to Flash once skeletal integrity passes**: Once architectural invariants are established, return to Flash to maximize velocity.

---

## 2. Framework Division of Labor: eframe vs egui

### 2.1 Clear Separation of Responsibilities

* **egui (GUI Library)**:
  * Pure UI logic, layout calculation, input event evaluation, and drawing mesh generation.
  * OS-agnostic; cannot display windows by itself (pure rendering command generator).
* **eframe (egui framework)**:
  * OS window management (via winit), GPU backend initialization (wgpu / OpenGL), OS input event abstraction.
  * Provides `eframe::App` trait and runs the main event loop calling `update` every frame.

### 2.2 Structural Architecture

```text
+-------------------------------------------------------------------+
| OS (Windows / macOS / Linux / WebAssembly)                        |
+-------------------------------------------------------------------+
                                ▲
                                │ Windowing / Input / GPU (DirectX 12 / Metal / Vulkan)
+-------------------------------▼-----------------------------------+
| eframe (NativeOptions, run_native, event loop, wgpu)              |
+-------------------------------------------------------------------+
                                │ Per-frame context (ctx)
+-------------------------------▼-----------------------------------+
| egui (CentralPanel, SidePanel, Layout, Painter, Visuals, State)   |
+-------------------------------------------------------------------+
                                ▲
                                │ Calls
+-------------------------------▼-----------------------------------+
| vivivi Shared Layer (vivivi-core / modes / vi / app headless core)|
+-------------------------------------------------------------------+
```

---

## 3. Comparative Evaluation of Native Rust GUI Frameworks

Excluding web-based wrappers (React, HTML/CSS, Tauri / WebView), evaluation of native Rust GUI candidates:

| Framework                       | Architecture                                 | Rendering Performance & Latency                       | Design Flexibility & Look-and-Feel                                        | Ecosystem & Maturity               | Fitness for vivivi                  |
| :------------------------------ | :------------------------------------------- | :---------------------------------------------------- | :------------------------------------------------------------------------ | :--------------------------------- | :---------------------------------- |
| **egui / eframe**<br>*(Chosen)* | **Immediate Mode**<br>(Per-frame UI re-eval) | **Extremely Fast**<br>(Direct wgpu GPU)               | Single Rust codebase. Fully customizable via theme overrides (`Visuals`). | **Very High**<br>(Abundant crates) | **Optimal**<br>(TUI logic coupling) |
| **GPUI**<br>*(Zed base)*        | **Retained + Direct GPU**<br>(Tailwind-like) | **Ultra-fast, lowest latency**<br>(Direct Metal/DX12) | Tailwind CSS-like utility syntax. Few reusable components.                | **Low**<br>(Rapid API shifts)      | **Premature**<br>(High design cost) |
| **Slint**                       | **Retained**<br>(Declarative DSL + Rust)     | **Fastest class**<br>(Runs on MCUs)                   | Uses dedicated `.slint` DSL. Clean defaults similar to Qt QML.            | **High**<br>(Clean grammar)        | **On Hold**<br>(DSL sync overhead)  |
| **Iced**                        | **Retained Reactive**<br>(Elm Architecture)  | **Fast**<br>(wgpu backend)                            | Robust Model-Update-View state machine. Verbose boilerplate.              | **Moderate - High**                | **On Hold**<br>(Vi latency match)   |

---

## 4. Why Zed Developed GPUI (Importance of Async Architecture)

Key motivations behind Atom creators (Nathan Sobo et al.) building GPUI:

1. **Overcoming Atom (Electron/Web) Latency Limits**:
   Input latency cannot drop to absolute zero when bridging DOM, CSS engines, and JavaScript runtimes.
2. **Seamless 120Hz ProMotion Sync**:
   Direct OS low-level GPU APIs (DirectX 12 / Metal / Vulkan) to ensure complete rendering within 8.33ms per frame.
3. **Power Efficiency in Immediate Mode (Hybrid Repaint)**:
   Maintains 0% CPU via reactive event-driven polling during typing, switching to continuous 120fps GPU loops during scrolling/physics.
4. **Multithreaded Asynchronous Architecture**:
   Tree-sitter parsing, LSP communication, and async I/O never block UI rendering by even a single frame.

---

## 5. Modernizing egui: Eliminating "Toolbox" Aesthetics

Default egui styling feels like a debug utility. Overriding `Visuals` at launch yields a sleek modern code editor.

### 5.1 Modern Theme Comparison

| Dimension           | **Tokyo Night**                       | **Catppuccin (Mocha)**                      |
| :------------------ | :------------------------------------ | :------------------------------------------ |
| **Base Tone**       | Cool, oceanic deep blue-purple/navy   | Warm, mocha/espresso dark charcoal          |
| **Accent Colors**   | Neon cyan, light blue, magenta        | Lavender, mint, peach, rosewater            |
| **Impression**      | Cyberpunk, crisp technical feel       | Soft on eyes, cozy, comfortable             |
| **Best Suited For** | Log viewers, terminal tools, monitors | Code editor, document reading, daily driver |

### 5.2 Four Rules for Clean UI
1. **Eliminate Strokes**: Remove white borders around buttons; represent hover states via subtle brightness contrast.
2. **Apply Rounding**: 4px-6px border radiuses on panels and buttons.
3. **Three-Tier Background Contrast**:
   - Base background (darkest code editor canvas)
   - Panel/sidebar (intermediate dark tone)
   - Selected items/active lines (subtle highlight)
4. **Monospace & Line Icons (Phosphor Icons)**: Avoid text-only buttons; enforce clean spacing (`item_spacing`).

---

## 6. GUI Font Strategy: Standardizing on PlemolJP with Strict 1:2 Width Ratio

While TUI relies on the host terminal font, **GUI (`vivivi-gui`) adopts "PlemolJP" as its official bundled/recommended font**.

### 6.1 Why PlemolJP is the Definitive Choice

1. **Same Elegance as Zed (IBM Plex Mono base)**:
   Uses **IBM Plex Mono** for Latin glyphs (same foundation as Zed Mono), providing a sophisticated, technical coding feel.
2. **Strict "1 Half-width : 2 Full-width" East Asian Ratio**:
   Synthesized with IBM Plex Sans JP so Japanese full-width characters are exactly twice the width of half-width characters.
   - **Box Drawing Mode**: Intersections (`┌ ─ ┐ │ ┼`) align with sub-pixel precision.
   - **CSV Mode & Markdown Tables**: Column borders remain perfectly vertical without jitter.
3. **Safe Licensing (SIL Open Font License)**:
   Fully permissible for commercial/personal use and portable application bundling.

### 6.2 Font Loading Pipeline & Fallback Hierarchy

To preserve lightweight binary size and portable flexibility:

```text
[Priority 1: Portable Local Font]
`fonts/PlemolJP-*.ttf` located in the same directory as vivivi.exe
       │ (If not found)
       ▼
[Priority 2: System Installed Font]
Registered OS fonts ("PlemolJP", "PlemolJP Console")
       │ (If not installed)
       ▼
[Priority 3: Windows High-Quality Default]
"BIZ UD Gothic" (MORISAWA) / "MS Gothic"
       │ (If non-Windows / not found)
       ▼
[Priority 4: System Monospace]
OS default monospace font (Monospace / Courier)
```

---

## 7. Project Structure: Headless Core Layer & Dual Frontends

### 7.1 Re-architecting Crate Boundaries (Preventing Massive Refactoring)

Currently, `vivivi-app` houses not only TUI rendering code, but also core editor session states: multi-tab manager (`Tab`), split-window tree (`WindowTreeNode`), file I/O & encoding auto-detection, swap protection, ex command engine, and configuration management.
To avoid re-implementing these core mechanisms for `vivivi-gui`, we cleanly isolate the **Headless Application Core** from the **Rendering Frontends**:

```text
vivivi/
├── Cargo.toml               (Workspace definition)
├── crates/
│   ├── vivivi-core          (Pure headless buffer, undo manager, encoding layer)
│   ├── vivivi-vi            (Vi mode state machine, command parser, keybinding engine)
│   ├── vivivi-modes         (Box drawing, CSV, Migemo, outline, diff, project management)
│   ├── vivivi-app           (Core editor session state library & TUI frontend)
│   │   ├── src/lib.rs       (Headless App, Tab, WindowTreeNode, Ex command dispatch)
│   │   ├── src/main.rs      (TUI binary entry: vivivi.exe)
│   │   └── src/tui/         (Ratatui / Crossterm rendering & event handlers)
│   └── vivivi-gui           (Phase 5: egui / eframe desktop GUI frontend)
│       ├── src/main.rs      (GUI binary entry: vivivi-gui.exe)
│       └── src/gui/         (egui widgets & Painter rendering layer)
```

### 7.2 Core Architectural Principles & Decoupling Prerequisites
1. **Eliminating TUI-Specific Types (Prerequisite Step 0)**:
   Currently, `vivivi-app::tui::App` directly references `ratatui::layout::Rect` (`window_rects`) and `crossterm::event::KeyEvent` (`recorded_macro_events`), and `Cargo.toml` lacks a `[lib]` target.
   As **Step 0** of Phase 5, these types must be abstracted into frontend-agnostic types (`LogicalRect`, `AppKeyEvent`) and exposed via `src/lib.rs`.
2. **Shared Session State & Multi-Cursor Extensibility**:
   Tab management (`gt`), nested window splits (`:split`/`:vsplit`), and ex commands (`:w`, `:%s`) live in the headless core. Furthermore, to support Phase 5's "Multi-cursor editing", the cursor rendering layer is designed to iterate over both the primary cursor (`cursor: TextCursor`) and secondary cursors (`extra_cursors: Vec<TextCursor>`).
3. **Coordinate Space Abstraction (Character Cells vs. Pixels)**:
   While TUI layouts operate on `u16` character cells, GUI (egui) performs smooth scrolling in `f32` pixels. The shared core tracks logical line/column offsets, which the GUI multiplies by glyph width and `line_height` to compute exact pixel coordinates.

---

## 8. Phase 5 Killer Features: GUI Integration Architecture

Layout and container mapping of Phase 5 features within egui:

```text
+-----------------------------------------------------------------------------------------+
| TopBottomPanel::top("menu_bar") : Menu Bar / Tab Bar (Multi-tab switching)              |
+-----------------------------------------------------------------------------------------+
| SidePanel::left("sidebar") | CentralPanel ("editor_workspace")                       |                                         |             |     |
| :------------------------- | :------------------------------------------------------ | :-------------------------------------- | :---------- | :-- |
|                            |                                                         | View Mode: Text / Hex Editor / 2D Graph | Minimap     |     |
| ▼ project/                 |                                                         | Line No │ Code Buffer (Painter)         | Overview    |     |
| ▶ src/                     |                                                         | 1       │ fn main() {                   | [Bird's-eye |     |
| main.rs                    |                                                         | 2       │     println!("Hello");        | ==          |     |
| lib.rs                     |                                                         | 3       │ }                             |             |     |
| Cargo.toml                 | +----------------------------------------+------------+ |                                         |             |     |
|                            | [Topmost Floating Area] LSP completions / Hover docs    |                                         |             |     |
|                            | [Painter Overlay] Physical falling character animation  |                                         |             |     |
+-----------------------------------------------------------------------------------------+
| TopBottomPanel::bottom("status_bar") : Mode, Line/Col, Update Alert, Ex Command Line    |
+-----------------------------------------------------------------------------------------+
```

### 8.1 Sidebar File Explorer (Box/NAS Resilient Design)
* Managed inside `egui::SidePanel::left("file_explorer")` (`:tree` / `:Explore` / `Ctrl-e` toggle).
* **Async On-Demand Lazy Loading**: Fetches only immediate children in background threads, completely shielding the UI thread from network SMB or Box Drive cloud placeholder synchronization stalls.

### 8.2 Minimap Overview & Gigabyte File (1GB+) Viewport Culling
* Docked on the right side of `CentralPanel`, highlighting the active viewport rectangle with smooth drag-to-scroll synchronization.
* **Viewport Culling & Safeguards**:
  To coexist with Phase 5's "Gigabyte file (1GB+ / `memmap2`) handling", both the main editor canvas and minimap strictly enforce $O(\text{visible lines})$ rendering—tokenizing and meshing only the currently visible lines. For files exceeding a size threshold (e.g., 10MB), full-buffer minimap scanning and full Tree-sitter/LSP parsing automatically throttle down to sampled/partial modes to prevent OOM and frame drops.

### 8.3 Gravity Drop Deletion Animation (2D Physics & Hybrid Repainting)
* On deletion commands (`x`, `dd`, `dw`), spawns particles with initial pixel coordinates, glyph characters, and randomized launch velocities.
* Renders overlay via `ui.painter().text()` simulating downward gravitational acceleration $g$ and fading out.
* **Hybrid Energy-Saving Control**: Triggers `ctx.request_repaint()` (60-120fps) only while active particles exist, immediately returning to reactive 0% CPU sleep once particles vanish.

### 8.4 Tree-sitter & LSP GUI Rendering
* **Multi-Layered Highlighting**: Frame 0 (0ms) renders immediately using the built-in tokenizer (`vivivi-modes::syntax`), seamlessly upgrading to Tree-sitter AST highlights and LSP Semantic Tokens as background workers complete.
* **LSP Diagnostics**: Squiggly error/warning underlines rendered below spans with gutter status icons (`E`/`W`).
* **Hover Docs & Completion Popups**: Displayed in dynamic floating overlays using `egui::Area` directly below cursor position.

### 8.5 Japanese IME Inline Input & Vi Mode Coordination (Critical Custom Widget Requirement)
When bypassing egui's built-in `TextEdit` in favor of custom `Painter` rendering, OS IME will not function without explicit integration. The following protocol is mandatory:
1. **IME Cursor Rect Notification**: While in `ViMode::Insert` or ex command input, emit `ctx.output_mut(|o| o.ime = Some(egui::output::IMEOutput { rect, cursor_rect }))` every frame so the OS candidate window tracks the cursor position.
2. **Inline Preedit Rendering**: Capture not only `egui::Event::Key`, but also committed text `egui::Event::Text(String)` and active composition `egui::Event::Ime(egui::ImeEvent::Preedit(String))`, rendering uncommitted preedit text inline with an underline at the cursor.
3. **Auto-Reset IME on Normal Mode Return**: Upon pressing `Esc` to return to `ViMode::Normal`, immediately set `o.ime = None` to prevent accidental Japanese composition in Normal mode.

### 8.6 Binary / Hex Editor Mode & Project Visualizer Graph
* **Binary / Hex Editor Mode**: Switches `CentralPanel` into a 3-column synchronized grid ("Address Offset", "16-Byte Hex Dump", "ASCII Representation") rendered via `Painter` for direct byte-level navigation and editing.
* **Project Visualizer Graph**: Renders a 2D Force-Directed Graph on `CentralPanel` illustrating include dependencies, file sizes, and edit freshness. Applies Coulomb repulsion and Hooke spring attraction, automatically halting `ctx.request_repaint()` once total kinetic energy settles below threshold.

---

## 9. History of ViVi / openViVi & Purpose of ViViVi

### 9.1 Legacy of Nobuhide Tsuda
* **Pioneer of Windows Vi Clones**:
  Authored ViVi in the Windows 95/NT era using direct Win32 API calls, earning wide acclaim for blistering speed and rock-solid reliability.
* **Harmony Between Japanese IME and Vi Modes**:
  Solved the seamless coordination between inline Japanese IME and Vi modal states long before overseas editors caught on.
* **Qt Port and Open Source (openViVi)**:
  Ported extensive C++ codebases to Qt to achieve cross-platform viability, published as openViVi.

### 9.2 Why ViViVi is Needed in the Era of Zed
While Zed is an expansive code IDE (heavy industrial machinery), ViViVi serves as an ultra-lightweight, razor-sharp craft tool.

1. **Box Drawing Mode**:
   Draws unicode box borders (`┌ ─ ┐ │ └ ┘ ┼`) following cursor movement with automatic cross-junction recognition—ideal for instant architectural diagrams.
2. **CSV / TSV Column Mode**:
   Auto-aligns tabular columns without opening spreadsheets, enabling instant column hopping, calculation, and filtering.
3. **TUI & GUI Dual Capability**:
   Deployable as a zero-dependency terminal TUI over SSH, while delivering a sleek 120Hz GPU GUI on local desktops.

---

## 10. Concrete Prompt Templates for GUI (egui) Implementation

Standard prompts ready to use when initiating Phase 5 implementation:

### 10.1 Step-by-Step Recommended Model & Implementation Status Quick Reference

| Implementation Step                       | Recommended Model    | Primary Role & Selection Rationale                          |    Status    |
| :---------------------------------------- | :------------------- | :---------------------------------------------------------- | :----------: |
| **Step 0**: Decouple `vivivi-app` Library | **Gemini 4 Argon**   | Crate boundary redesign, type abstraction, 100% test pass   | ✅ Completed |
| **Step 1**: Styling & Font Setup          | **Gemini 3.8 Flash** | Rapid generation of boilerplate PlemolJP/theme config       | ✅ Completed |
| **Step 2**: Window Skeletal Layout        | **Gemini 3.8 Flash** | Fast trial-and-error for panels and responsive layout       | ✅ Completed |
| **Step 3**: Core Coupling & Japanese IME  | **Gemini 4 Argon**   | **[Crucial]** `eframe` borrow puzzles, ownership, IME rects | ✅ Completed |
| **Step 4**: Physics & Energy Saving       | **Gemini 3.8 Flash** | Tuning particle physics parameters with rapid iteration     | ✅ Completed |

> [!TIP]
> **Simple Rule of Thumb**:
> Use **Gemini 4 Argon** for foundational architectural steps (Steps 0 & 3) to solve Rust borrow checker invariants in one shot. Use ultra-fast **Gemini 3.8 Flash** for visual layout, styling, and physics animation tuning (Steps 1, 2, & 4).

### Step 0: Decoupling `vivivi-app` into a Shared Headless Library (Prerequisite) [Recommended: Gemini 4 Argon]
```text
Refactor the vivivi-app crate so that the Phase 5 GUI crate (vivivi-gui) can share all editor session states (App, Tab, WindowTreeNode, Ex commands):
1. Add a [lib] section to crates/vivivi-app/Cargo.toml and create src/lib.rs exposing shared modules.
2. Abstract direct dependencies on ratatui::layout::Rect and crossterm::event::KeyEvent inside App into frontend-agnostic types (LogicalRect, AppKeyEvent).
3. Verify that the existing TUI binary (vivivi) and all unit tests pass 100% via cargo test.
```

### Step 1: Styling & Font Initialization (PlemolJP Setup) [Recommended: Gemini 3.8 Flash]
```text
Create a new GUI crate (vivivi-gui) using egui / eframe for a Vi-like text editor.
Configure a modern Tokyo Night (or Catppuccin Mocha) theme.

Implement initialization code:
1. cc.egui_ctx.set_visuals:
   - Avoid pure pitch black; create 3 tiers of contrast across editor canvas, side panels, and statusline.
   - Eliminate harsh button strokes; set corner radiuses to 4px.
2. cc.egui_ctx.set_fonts:
   - Configure egui::FontDefinitions.
   - Load fonts/PlemolJP-*.ttf in executable directory or system-installed PlemolJP first.
   - Fallback to BIZ UD Gothic and system monospace fonts.
```

### Step 2: Workspace Layout & Panels [Recommended: Gemini 3.8 Flash]
```text
Construct the primary editor window layout inside eframe::App::update:
1. TopBottomPanel::top("menu_bar"): Compact menu bar & multi-tab bar
2. SidePanel::left("sidebar"): Collapsible sidebar for File Explorer / Outline Tree
3. TopBottomPanel::bottom("status_bar"): Mode indicator (-- NORMAL --), cursor position, update notification, ex command line
4. CentralPanel: Nested split editor views (WindowTreeNode) + right-side Minimap area (with Viewport Culling)
Ensure editor workspace maximizes dynamically on window resize.
```

### Step 3: Coupling with Shared Headless App Core & Japanese IME Integration (Crucial) [Recommended: Gemini 4 Argon]
```text
Connect the shared session layer (App struct in vivivi-app) to vivivi-gui.
※Important: Do not use egui's default TextEdit widget.

Requirements:
1. In CentralPanel, use ctx.input(|i| ...) to capture Event::Key, committed text Event::Text, and IME preedit Event::Ime(Preedit), dispatching to the shared input handler.
2. In CentralPanel::show, use ui.painter() to render visible line numbers, buffer text, and underlined IME preedit text with fixed-width spacing.
3. Render mode-appropriate cursors (solid block for NORMAL, vertical I-beam for INSERT), and emit ctx.output_mut(|o| o.ime = ...) with the cursor rectangle every frame during Insert mode.
```

### Step 4: Particle Animation & Energy-Saving Hybrid Loop [Recommended: Gemini 3.8 Flash]
```text
Implement a particle manager for falling character deletion animations:
1. Particle array tracking glyph character, initial position/velocity, and fade-out timer (0.5s).
2. While particles exist, update physics each frame, render with ui.painter().text(), and request ctx.request_repaint().
3. Once particles terminate, cease repaint requests, preserving egui's reactive idle mode (0% CPU).
```
