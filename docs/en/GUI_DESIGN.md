# ViViVi: Next-Generation Desktop GUI Architecture & Design Specification (GUI_DESIGN)

> [!NOTE]
> This document specifies the technical selection, rendering architecture, design principles, and implementation roadmap for the future desktop GUI frontend (Phase 5).
> (Consolidated and synthesized from design discussions conducted via smartphone / Web Gemini).

---

## 1. Development Environment & AI Model Orchestration Strategy (Gemini 3.8 Flash vs Gemini 4 Argon)

Optimization strategy based on task characteristics in agentic AI development environments (e.g., Antigravity).

### 1.1 Model Characteristics & Task Allocation

| Development Phase / Problem               | Recommended Model    | Rationale & Responsibility                                                                                                     |
| :---------------------------------------- | :------------------- | :----------------------------------------------------------------------------------------------------------------------------- |
| **GUI Boundary & Async Architecture**     | **Gemini 4 Argon**   | UI thread & worker thread channel design; lifecycle and concurrency consistency.                                               |
| **Rust Borrow Checker / Lifetime Errors** | **Gemini 4 Argon**   | Resolving mutable borrow conflicts in immediate mode (`eframe::App::update`) and thread boundary constraints (`Send + 'Sync`). |
| **Multi-Crate Boundary Refactoring**      | **Gemini 4 Argon**   | Redefining dependency graphs across `vivivi-core`, `vivivi-vi`, `vivivi-modes`, `vivivi-app`, and `vivivi-gui`.                |
| **Widget Placement & Layout Tuning**      | **Gemini 3.8 Flash** | High-velocity trial-and-error leveraging low latency and rapid code generation.                                                |
| **Local Syntax Fixes & Minor Refactors**  | **Gemini 3.8 Flash** | Fast turnarounds to maintain high coding rhythm without blocking flow.                                                         |

### 1.2 Pragmatic Switching Protocol
1. **Default Routine on 3.8 Flash**: Basic component construction, widget addition, styling tweaks.
2. **"Two-Round Rule" Switch to 4 Argon**: If 3.8 Flash cannot resolve compilation issues within two consecutive attempts (often creating cascading borrow errors), escalate immediately to 4 Argon with full `cargo check` logs to review fundamental architectural design.
3. **Revert to 3.8 Flash upon Stability**: Once crate boundaries and types stabilize, return to 3.8 Flash to maximize development speed.

---

## 2. GUI Infrastructure: Division of Labor between eframe and egui

### 2.1 Distinct Roles

* **egui (GUI Library)**:
  * Manages UI logic, layout calculation, input event handling, and rendering mesh generation.
  * Completely platform-agnostic; cannot display windows on its own (pure rendering instruction generator).
* **eframe (egui framework / Official Framework)**:
  * Manages OS windows (winit), GPU rendering backend initialization (wgpu / OpenGL), and OS event abstractions.
  * Provides the `eframe::App` trait, running the main event loop and invoking `update` per frame.

### 2.2 Architectural Relationship

```text
+-------------------------------------------------------------------+
| OS (Windows / macOS / Linux / WebAssembly)                        |
+-------------------------------------------------------------------+
                                ▲
                                │ Window Management / Input / GPU
+-------------------------------▼-----------------------------------+
| eframe (NativeOptions, run_native, Event Loop)                     |
+-------------------------------------------------------------------+
                                │ Context per frame (ctx)
+-------------------------------▼-----------------------------------+
| egui (CentralPanel, Layout, Painter, Visuals, Interaction State)   |
+-------------------------------------------------------------------+
                                ▲
                                │ Invocation
+-------------------------------▼-----------------------------------+
| vivivi-core / vivivi-modes (GapBuffer, Undo, Box/CSV Logic, etc.)  |
+-------------------------------------------------------------------+
```

---

## 3. Rust Native GUI Framework Comparison

Evaluation of pure native rendering stacks, excluding web technologies (React, HTML/CSS, Tauri / WebView).

| Framework                        | Rendering Architecture                         | Performance & Latency                                     | Design & Aesthetic Quality                                                                    | AI (LLM) Accuracy                                    | Fit for ViViVi                            |
| :------------------------------- | :--------------------------------------------- | :-------------------------------------------------------- | :-------------------------------------------------------------------------------------------- | :--------------------------------------------------- | :---------------------------------------- |
| **egui + eframe**<br>*(Adopted)* | **Immediate Mode**<br>Re-evaluates every frame | **Extremely Fast**<br>(Direct lightweight GPU)            | Pure Rust. Fully customizable with modern theme (`Visuals`) overrides.                        | **Extremely High**<br>(Abundant knowledge & samples) | **Optimal**<br>(Direct link to TUI core)  |
| **GPUI**<br>*(Zed base)*         | **Retained + GPU Direct**<br>(Tailwind-like)   | **Ultra Fast / Lowest Latency**<br>(Metal/DirectX/Vulkan) | Utility-first syntax. Lacks off-the-shelf widgets; requires building primitives from scratch. | **Low**<br>(Rapidly changing API)                    | **Premature**<br>(Excessive dev overhead) |
| **Slint**                        | **Retained Mode**<br>(Declarative DSL + Rust)  | **Top Tier**<br>(Runs on MCUs)                            | Uses dedicated `.slint` DSL. QML-like, polished defaults.                                     | **High**<br>(Clear DSL grammar)                      | **On Hold**<br>(Requires DSL sync layer)  |
| **Iced**                         | **Retained / Reactive**<br>(Elm Architecture)  | **Fast**<br>(wgpu-based)                                  | Robust state management (Model-Update-View). Verbose code structure.                          | **Moderate to High**                                 | **On Hold**<br>(Key latency impedance)    |

---

## 4. Why Zed Developed & Chose GPUI

Key reasons why Nathan Sobo and the Zed team opted for custom GPUI over existing libraries:

1. **Overcoming Atom (Electron/Web) Bottlenecks**:
   Zero input latency is structurally impossible while traversing DOM, CSS engines, and JS runtimes.
2. **ProMotion (120Hz) Smooth Tracking**:
   Render full frames within 8.33ms by binding directly to low-level platform APIs (Metal on macOS) rather than generic abstraction layers.
3. **Immediate Mode Battery & CPU Constraints**:
   Avoid recalculating tens of thousands of line layouts on the CPU per frame when scrolling large files; GPUI relies on diff-based updates and aggressive GPU caching.
4. **Non-Blocking Asynchronous Core**:
   Architected so Tree-sitter parsing, LSP communication, and async I/O never block UI rendering by even a single frame.

---

## 5. Modernizing egui: Eliminating "Utility/Debug Look"

egui's default look stems from game debugging tools. Overriding `Visuals` at startup transforms it into an elegant modern editor.

### 5.1 Modern Theme Comparison

| Dimension            | **Tokyo Night**                             | **Catppuccin (Mocha)**                        |
| :------------------- | :------------------------------------------ | :-------------------------------------------- |
| **Base Tone**        | Cool deep-sea blue-violet & dark navy       | Warm espresso mocha & soft dark charcoal      |
| **Accents**          | Neon cyan, light blue, magenta              | Lavender, mint, peach, rosewater              |
| **Aesthetic Feel**   | Sharp, futuristic cyber aesthetic           | Gentle on eyes, calm and cozy                 |
| **Target Use Cases** | Log viewers, terminal integration, monitors | Text editor, documentation reader, daily tool |

### 5.2 Four Rules to Prevent Design Rework
1. **Remove White Outlines (Stroke)**: Eliminate harsh white borders around rectangular buttons; represent states through hover brightness contrast.
2. **Apply Rounded Corners (Rounding)**: Use 4px to 6px radii for panels and buttons.
3. **Three-Tier Background Contrast**:
   - Base background (darkest editor canvas)
   - Panels / Sidebars (medium dark)
   - Selected items / Active lines (subtly highlighted)
4. **Monospace Typography & Clean Line Icons (Phosphor Icons)**:
   Avoid plain text buttons; maintain generous `item_spacing`.

---

## 6. GUI Font Strategy: Standard Adoption of PlemolJP & Strict 1:2 Half/Full-width Ratio

While the TUI edition relies on the host terminal's configured font (Windows Terminal, Alacritty, WezTerm, etc.), **the GUI edition (`vivivi-gui`) officially adopts "PlemolJP" as its standard built-in font**.

### 6.1 Decisive Rationale for Adopting PlemolJP

1. **Aesthetic Parity with Zed (Derived from IBM Plex Mono)**:
   PlemolJP uses **IBM Plex Mono** (the parent of `Zed Mono`) for Latin alphanumeric glyphs, delivering an identical intellectual, polished, and modern visual tone.
2. **Strict "1 : 2" Monospace Aspect Ratio (East Asian Width)**:
   Synthesized with "IBM Plex Sans JP" so full-width CJK characters precisely occupy double the width of half-width characters.
   - **Box Drawing Mode**: Horizontal and vertical intersecting lines (`┌ ─ ┐ │ ┼`) meet orthogonally with zero pixel offset.
   - **CSV Mode & Markdown Tables**: Column borders align vertically with mathematical precision, eliminating visual jitter.
3. **Clean Licensing (SIL Open Font License)**:
   Free for both commercial and personal use, and safe to bundle within standalone portable application releases.

### 6.2 Font Loading & Fallback Hierarchy

```text
[Priority 1: Official Standard]
PlemolJP / PlemolJP Console (Bundled or System Installed)
       │ (If not found)
       ▼
[Priority 2: Windows High-Quality Standard]
BIZ UD Gothic (MORISAWA) / MS Gothic
       │ (Non-Windows / Fallback)
       ▼
[Priority 3: OS System Default]
System Monospace Font (Monospace / Courier)
```

---

## 7. Project Architecture: The Purpose of Multi-Crate Structure

Rationale for Cargo Workspace separation in vivivi:

1. **Complete Decoupling of Core Logic**:
   The editor core (`GapBuffer`, `UndoMgr`, Vi engine, Box Drawing, CSV logic) remains headless and 100% shared across TUI and GUI.
2. **Preventing Dependency Bloat**:
   Isolates heavy GUI dependencies (`wgpu`, `eframe`) from the streamlined TUI crate (`vivivi-app`, `crossterm`, `ratatui`).
3. **Minimizing Incremental Compile Times**:
   Modifications to GUI layouts never trigger recompilation of `vivivi-core` or TUI modules.

```text
vivivi/
├── Cargo.toml               (Workspace Manifest)
├── crates/
│   ├── vivivi-core          (Headless Buffer & Undo Engine)
│   ├── vivivi-vi            (Vi Modal Engine & Ex Parser)
│   ├── vivivi-modes         (Box Drawing, CSV, Migemo, Outline, Diff)
│   ├── vivivi-app           (Active Ratatui TUI Frontend)
│   └── vivivi-gui           (Planned egui / eframe GUI Frontend - Phase 5)
```

---

## 8. Historical Legacy of ViVi / openViVi & Purpose of ViViVi

### 8.1 Nobuhide Tsuda's Pioneering Work
* **Windows Pioneer**:
  Built ViVi as a native Win32 Vi clone during Windows 95/NT, setting the benchmark for stability and responsiveness.
* **Flawless Japanese IME Integration**:
  Achieved inline IME composition and Vi mode switching long before overseas editors handled CJK inputs properly.
* **Qt Port & Open Source**:
  Migrated extensive C++ codebases to Qt as openViVi for cross-platform support.

### 8.2 Why ViViVi Coexists with Zed
Zed is a heavy-duty IDE. ViViVi is a nimble, razor-sharp craftsman tool:

1. **Box Drawing Mode**:
   Draw ASCII/Unicode boxes (`┌ ─ ┐ │ └ ┘ ┼`) along cursor motion with automatic intersection resolution. Zero-latency diagramming in plain text.
2. **CSV Column Mode**:
   Tabular column alignment and editing without launching heavy spreadsheet suites.
3. **TUI & GUI Dual Capability**:
   Use `vivivi` over SSH/terminal and `vivivi-gui` on the desktop with identical muscle memory and configurations.

---

## 9. Prompt Template for Phase 5 egui Implementation

Ready-to-use prompt blueprints when commencing Phase 5:

### Step 1: Style & Font Setup (Prioritize PlemolJP)
```text
Create a new GUI crate (vivivi-gui) using egui / eframe for our Vi-like text editor.
Configure a modern editor look based on Tokyo Night (or Catppuccin Mocha).

Requirements:
1. cc.egui_ctx.set_visuals:
   - Avoid pitch black; use 3 levels of brightness across editor background, panel background, and status line.
   - Remove button strokes and set rounding to 4px.
2. cc.egui_ctx.set_fonts (Prioritize PlemolJP in font definitions):
   - Configure egui::FontDefinitions.
   - Register "PlemolJP" (or "PlemolJP Console") TTF/OTF data into font definitions.
   - Place "PlemolJP" at the front of FontFamily::Monospace and FontFamily::Proportional priority lists.
   - Set "BIZ UD Gothic" and system monospace as subsequent fallback options.
```


### Step 2: Screen Wireframe & Layout
```text
Implement the basic layout inside eframe::App::update.
Use mock data for internal logic to ensure proper container partitions:
1. TopBottomPanel::top("menu_bar"): Compact top menu bar.
2. TopBottomPanel::bottom("status_bar"): Mode indicator (-- NORMAL --), cursor coordinates, command line.
3. CentralPanel: Text canvas (line numbers + editor text).
Ensure the central editor canvas maximizes dynamically on resize.
```

### Step 3: Key Interception & Custom Painting (Crucial)
```text
Connect the existing vivivi-core / vivivi-vi crates to the GUI.
Important: Do NOT use egui's default TextEdit widget.

Requirements:
1. Intercept raw key events (Esc, h, j, k, l, i, colon, etc.) in CentralPanel via ctx.input(|i| ...) and feed into vivivi-vi handler.
2. Render line numbers and buffer contents using ui.painter() with monospace font metrics.
3. Render modal cursor shapes (Block vs I-beam) according to current Vi state.
```
