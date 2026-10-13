# vivivi: Development Roadmap & TODOs (TODO)

## 1. Completed Tasks
- [x] Initial workspace environment & multi-crate architecture setup.
- [x] `vivivi-core`: Implemented OpenViVi-compatible `GapBuffer<T>`.
- [x] `vivivi-core`: Implemented transactional `UndoMgr` (nested blocks, typing merge, save point tracking).
- [x] `vivivi-core`: Implemented `TextBuffer` combining `GapBuffer` and `UndoMgr`.
- [x] Agent infrastructure setup (`.agents/`, `GEMINI.md`, `RULES.md`).
- [x] Bilingual documentation structure (`docs/ja/`, `docs/en/`).

---

## 2. Upcoming Roadmap

### Phase 1: Full Vi Command Engine Port (`vivivi-vi`)
- [x] Basic cursor motions (`h`, `j`, `k`, `l`, `w`, `b`, `e`, `0`, `$`, `^`, `gg`, `G`).
- [x] Operators and deletions/changes/yanks (`d`, `c`, `y` + motion / `dd`, `yy`, `cc`).
- [x] Paste (`p`, `P`) and register operations (`"a` - `"z`, clipboard).
- [x] Search commands (`/`, `?`, `n`, `N`).
- [x] Basic ex commands (`:w`, `:q`, `:wq`, `:s`, `:%s`).

### Phase 2: Advanced Editing Modes (`vivivi-modes`)
- [x] Keisen key handling (`Ctrl+Arrow` draw/join, `Shift+Arrow` erase).
- [x] Box-internal text alignment (left, center, right).
- [x] Inline CSV/TSV column calculation view.
- [x] Transparent quote display and active-line raw view in CSV mode.

### Phase 3: Frontend Implementation (`vivivi-app`)
- [x] Terminal UI (Ratatui + crossterm) frontend implementation (interactive Vi editing & realtime render).
- [x] File I/O and charset/newline auto-detection (UTF-8, Shift_JIS, EUC-JP / CRLF, LF).
- [x] Syntax highlighting (Rust, C++, Python, JS, JSON), matching paren highlight, and paren jump (`%`).

---

### Phase 4: Daily Driver ViVi Features & Advanced Extensions (`vivivi-vi`, `vivivi-modes`, `vivivi-app`)

Focusing on features essential for primary/daily driver use (Gvim-compatible workflows) and advanced features from ViVi 3, ViVi 4, and OpenViVi.

#### 4.1 Boss Top Priority: Daily Driver Features (Gvim-compatible & Layout)
- [x] **Split & Vertical Split Windows (`:split`, `:vsplit`, `Ctrl-w`)**
  - Split views (horizontal & vertical) for the same or multiple buffers.
  - Window focus switching (`Ctrl-w h/j/k/l`) and window resizing.
- [x] **Full Visual Block (BOX Selection) Support (`VisualBlock` / `Ctrl-v`)**
  - Block deletion (`d`, `x`) and yank (`y`).
  - Block multi-line insert (`I`, `A`).
  - Multi-line block paste (`p`).
- [x] **Outline Analysis & Side Pane Tree**
  - Markdown heading outline and function/struct symbol trees for code (Rust, C++, etc.).
  - Toggleable side pane with instantaneous jump to target lines.

#### 4.2 Signature ViVi Editing Assists (`vivivi-modes`)
- [ ] **Keisen Protected Edit Mode**
  - Automatic whitespace padding/trimming when inserting/deleting text inside box-drawing borders so that the outer frame stays intact.
- [ ] **Text Alignment**
  - Multi-line columnar alignment based on target characters (e.g., `=`, `:`).
- [ ] **Renumbering**
  - Automatic renumbering of numbered lists or lines.
- [x] **Character Conversion**
  - Batch conversions: Full-width ⇄ Half-width, Hiragana ⇄ Katakana, Uppercase ⇄ Lowercase.

#### 4.3 Advanced Tooling & Extensibility (ViVi 3 / ViVi 4 Compatible)
- [ ] **Project-wide Grep Search (`:grep`, ripgrep-powered)**
  - Adopt high-speed `ripgrep` engine (`grep-searcher` / `ignore` crates integration).
  - Search results pane (file, line number, matched line) and instantaneous jump to matched locations.
- [ ] **Side-by-Side File Comparison (DIFF)**
  - Dual split-view diff highlighting at line and word level (powered by `similar`).
- [ ] **Print Preview & PDF Printing (ViVi 3 / ViVi 4 Compatible)**
  - Direct PDF generation/printing rather than physical printer output (supports page sizing, line numbers, headers/footers).
  - **Markdown Print Preview**: Treats document preview as print preview for Markdown files, showing rendered headings, styles, and tables.
  - Paginated print preview for plain text and source code.
- [ ] **Desktop GUI (egui / eframe) Frontend**
  - Rescheduled from Phase 3. Rich graphic border drawing and font rendering.

---

### Phase 5: Future Enhancements (Future)
- [ ] **Binary / Hex Editor Mode**
  - Rescheduled from Phase 4. Hex dump + ASCII parallel view and byte-level editing.
- [ ] **Scripting Engine Integration (Modern ViViScript via Rhai / Lua)**
  - Rescheduled from Phase 4. Macro recording/playback and custom automation scripts.
- [ ] Gigabyte file (1GB+) handling via Piece Tree & Memory-Mapped Files (`memmap2`).
- [ ] LSP (Language Server Protocol) client integration.
- [ ] Multi-cursor editing.
