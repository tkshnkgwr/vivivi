# vivivi: Development Roadmap & TODOs (TODO)

## 0. Lineage & Feature Architecture: OpenViVi vs. ViVi 3 / ViVi 4

`vivivi` is an ambitious, modern reconstruction in **Rust** that bridges two legendary lineages:
1. **ViVi (ViVi 3 / ViVi 4)** by Nobuhide Tsuda: Renowned for its rich Japanese editing assists, inline CSV mode, dynamic macros, Migemo search, and generational backup.
2. **OpenViVi** by vivisuke: The open-source C++ / Qt project focused on clean, high-performance data structures (`GapBuffer`, `UndoMgr`) and modern GUI concepts.

### 🏷️ Lineage Classification
- **[OpenViVi Lineage]**: High-efficiency, memory-safe data structures (`GapBuffer`, `UndoMgr`, `ViewLineMgr`) and modern GUI/UX features (minimap, gutter change markers, gravity drop animation).
- **[ViVi 3 / ViVi 4 Lineage]**: Battle-tested editing power tools (keisen protection, inline CSV, expression substitution, visual block math, renumbering), rapid input assistants (dabbrev, dynamic macros, LexEdit, Zen-Coding), custom Vi commands (Migemo Romaji search, fFtT full-width match, numeric increments), and document management (generational DiffLog, in-code ToDo extraction, 2-pane filer).
- **[Common / Integrated]**: Unicode box-drawing border joining, syntax highlighting, outline parsing, Grep search, side-by-side DIFF, and multi-view splitting.

### 📊 Feature Lineage Matrix

| Category          | Feature Name                               | Lineage (Origin)         | Key Capabilities & Description                                     | vivivi Status       |
| :---------------- | :----------------------------------------- | :----------------------: | :----------------------------------------------------------------- | :-----------------: |
| Data Architecture | GapBuffer                                  | OpenViVi                 | O(1) local cursor editing with memory shifting algorithm           | Implemented (core)  |
| Data Architecture | UndoMgr (Transactional Undo)               | OpenViVi                 | Undo blocks, typing merge, and Save Point tracking                 | Implemented (core)  |
| Data Architecture | ViewLineMgr (Wrap Line Mgr)                | OpenViVi                 | Bi-directional mapping between logical & wrapped screen lines      | Implemented (core)  |
| Data Architecture | Encoding & BOM Auto-Detection              | OpenViVi                 | UTF-8, Shift_JIS, EUC-JP, BOM recognition & conversion             | Implemented (core)  |
| Editing Assists   | Unicode Keisen Mode (Box-Drawing)          | Common (ViVi / OpenViVi) | Ctrl+Arrow draw/join, Shift+Arrow erase box borders                | Implemented (modes) |
| Editing Assists   | Inline CSV/TSV Column Mode                 | ViVi 3 / ViVi 4          | Auto column alignment & transparent quote editing                  | Implemented (modes) |
| Editing Assists   | Batch Character Conversion                 | Common (ViVi / OpenViVi) | Full-width ⇄ Half-width, Kana, Upper ⇄ Lowercase                   | Implemented (modes) |
| Editing Assists   | Keisen Protected Edit Mode                 | ViVi 3 / ViVi 4          | Automatic whitespace adjust inside boxes to keep frames intact     | Implemented (modes) |
| Editing Assists   | Text Alignment                             | ViVi 3 / ViVi 4          | Multi-line columnar vertical alignment on target chars (=, :)      | Implemented (app)  |
| Editing Assists   | Batch Head/Tail Insertion                  | ViVi 4                   | Batch prefix/suffix insertion on selected lines or document        | Implemented (app)  |
| Editing Assists   | Renumbering                                | ViVi 3 / ViVi 4          | Automatic sequential renumbering for ordered lists (1., 2.)        | Implemented (app)  |
| Editing Assists   | Expression Substitute (:s/.../e)           | ViVi 3 / ViVi 4          | Dynamic evaluation of replacement string as arithmetic/script expr | Implemented (modes) |
| Editing Assists   | Box Selection Calculation                  | ViVi 3 / ViVi 4          | Instant Sum, Average, StdDev calculation on visual block numbers   | Implemented (app)   |
| Input & Macros    | Dynamic Completion (dabbrev / Ctrl-L)      | ViVi 3 / ViVi 4          | Dynamic forward/backward in-buffer word completion                 | Implemented (app)   |
| Input & Macros    | Static Abbreviation (abbrev / Ctrl-;)      | ViVi 3 / ViVi 4          | Boilerplate expansion from external abbreviation dictionary        | Implemented (app)  |
| Input & Macros    | Dynamic Macro (Ctrl-Alt-M)                 | ViVi 3 / ViVi 4          | Infer recurring keystrokes (XX, XYX) to repeat complex edits       | Implemented (app)  |
| Input & Macros    | Lexical Edit Prediction (LexEdit / Ctrl-J) | ViVi 4                   | Word-level edit history prediction in interactive popup menu       | Implemented (app)  |
| Input & Macros    | Zen-Coding (Emmet) Expansion (Ctrl-,)      | ViVi 4                   | Expand shorthand syntax (table>tr>td*3) into HTML/XML tags         | Implemented (app)  |
| Input & Macros    | Clipboard History (cbHist)                 | ViVi 3 / ViVi 4          | Interactive history list and insertion of past yanks/copies        | Implemented (app)   |
| Input & Macros    | Keyboard Macro Record & Playback           | Common (ViVi / OpenViVi) | Dynamic keystroke sequence recording and replay (@q)               | Implemented (app)  |
| Vi Commands       | Romaji Japanese Search (Migemo / g/)       | ViVi 3 / ViVi 4          | Incremental Japanese search directly via Romaji (SKK / Migemo)     | Implemented (app)   |
| Vi Commands       | fFtT Full-width Auto-Match                 | ViVi 3 / ViVi 4          | Type ASCII char to match both half-width & full-width variants     | Implemented (modes) |
| Vi Commands       | Word Search (*, g*) & Prefix/Suffix Match  | Common (ViVi / OpenViVi) | Instant cursor word search & prefix/suffix matching (g<, g=)       | Implemented (vi)    |
| Vi Commands       | Numeric/Alpha Increment & Toggle (#!))     | ViVi 3 / ViVi 4          | Increment/decrement numbers/alpha (#+, #-), toggle true/false      | Implemented (vi)    |
| Vi Commands       | Jump Last Edit (@, '@) & Line Replicate    | ViVi 3 / ViVi 4          | Instant return to previous edit, replicate text from above line    | Implemented (vi)    |
| Vi Commands       | Global Marks (mA - mZ, :marks)             | ViVi 3 / ViVi 4          | Cross-file workspace-persistent marks using uppercase letters      | Implemented (vi)    |
| Vi Commands       | Tag Jump (Ctrl-], Alt-])                   | Common (ViVi / OpenViVi) | Jump to declaration/definition via tags file & AST syntax analysis | Implemented (vi)    |
| Tools & Files     | Generational Backup (DiffLog / .difflog)   | ViVi 3 / ViVi 4          | Differential backups on save; version rollback & per-line blame    | Planned (Phase 4)   |
| Tools & Files     | Persistent Undo (undofile)                 | Common (Gvim / Custom)   | Preserve Undo/Redo history on disk to resume undos across sessions | Planned (Phase 4)   |
| Tools & Files     | In-Code ToDo Extraction (undone/TODO)      | ViVi 4                   | Scan member files for ToDo tags and interactive jump list          | Planned (Phase 4)   |
| Tools & Files     | Code Folding (zc, ze, zE)                  | ViVi 3 / ViVi 4          | Indentation & bracket scope line folding and unfolding             | Planned (Phase 4)   |
| Tools & Files     | Dual-Pane Directory View (Filer)           | ViVi 3 / ViVi 4          | 2-pane file manager, directory diff, and 1-key file actions        | Planned (Phase 4)   |
| Tools & Files     | Project Management (Project View)          | ViVi 3 / ViVi 4          | Multi-file workspace tree, batch tags generation & backup          | Planned (Phase 4)   |
| Tools & Files     | Template Auto-Loading (.template)          | ViVi 3 / ViVi 4          | Auto-populate new files with boilerplate & macro substitutions     | Planned (Phase 4)   |
| Tools & Files     | Side-by-Side Diff Comparison               | Common (ViVi / OpenViVi) | Split-view line and character diff highlighting                    | Planned (Phase 4)   |
| Tools & Files     | Project-wide Grep Search (:grep)           | Common (ViVi / OpenViVi) | ripgrep integration with interactive jump list pane                | Planned (Phase 4)   |
| Tools & Files     | Print Preview & PDF Printing               | ViVi 3 / ViVi 4          | Rendered Markdown preview and paginated line-numbered PDF print    | Planned (Phase 4)   |
| Tools & Files     | Help System (:help / F1 / Markdown)        | Custom Extension         | Topic Markdown doc collection & in-app viewer (:help / F1)        | Alpha Stage (app)   |
| UI/UX Visual      | Multi-View Split Windows (:split, :vsplit) | Common (Gvim / ViVi)     | Horizontal/vertical split views, focus switching & resizing        | Implemented (app)   |
| UI/UX Visual      | Outline Tree & Side Pane                   | Common (ViVi / OpenViVi) | Markdown headings and function/struct symbols tree sidebar         | Implemented (app)   |
| UI/UX Visual      | Modified/Saved Line Indicators             | OpenViVi                 | Gutter bar markers for unsaved modified (yellow) & saved (green)   | Planned (Phase 4)   |
| UI/UX Visual      | Minimap Overview                           | OpenViVi                 | Bird's-eye code preview with viewport and search highlights        | Planned (Phase 4)   |
| UI/UX Visual      | Recent Files & Favorite Files              | OpenViVi                 | Persistent tracking and quick access to recent and starred files   | Planned (Phase 4)   |
| UI/UX Visual      | Gravity Drop Deletion Animation            | OpenViVi                 | Playful physical falling character animation upon deletion         | Planned (Phase 4)   |
| UI/UX Visual      | Desktop GUI (egui / eframe)                | OpenViVi (Qt rewrite)    | Modern GUI frontend with rich borders and custom fonts             | Planned (Phase 4)   |
| Future            | Binary / Hex Editor Mode                   | ViVi 3 / ViVi 4          | Hex dump + ASCII parallel view and byte-level editing              | Planned (Phase 5)   |
| Future            | Scripting Engine (Modern ViViScript)       | ViVi 3 / ViVi 4          | Editor automation via Rhai / Lua integration                       | Planned (Phase 5)   |
| Future            | Project Visualizer Graph (Visualize)       | ViVi 4                   | 2D force-directed graph of file sizes, dependencies & freshness    | Planned (Phase 5)   |

---

## 1. Completed Tasks
- [x] Initial workspace environment & multi-crate architecture setup.
- [x] **[OpenViVi]** `vivivi-core`: Implemented OpenViVi-compatible `GapBuffer<T>`.
- [x] **[OpenViVi]** `vivivi-core`: Implemented transactional `UndoMgr` (nested blocks, typing merge, save point tracking).
- [x] **[OpenViVi]** `vivivi-core`: Implemented `TextBuffer` combining `GapBuffer` and `UndoMgr`.
- [x] Agent infrastructure setup (`.agents/`, `GEMINI.md`, `RULES.md`).
- [x] Bilingual documentation structure (`docs/ja/`, `docs/en/`).

---

## 2. Upcoming Roadmap

### Phase 1: Full Vi Command Engine Port (`vivivi-vi`)
- [x] **[Common]** Basic cursor motions (`h`, `j`, `k`, `l`, `w`, `b`, `e`, `0`, `$`, `^`, `gg`, `G`).
- [x] **[Common]** Operators and deletions/changes/yanks (`d`, `c`, `y` + motion / `dd`, `yy`, `cc`).
- [x] **[Common]** Paste (`p`, `P`) and register operations (`"a` - `"z`, clipboard).
- [x] **[Common]** Search commands (`/`, `?`, `n`, `N`).
- [x] **[Common]** Basic ex commands (`:w`, `:q`, `:wq`, `:s`, `:%s`).

### Phase 2: Advanced Editing Modes (`vivivi-modes`)
- [x] **[Common (ViVi / OpenViVi)]** Keisen key handling (`Ctrl+Arrow` draw/join, `Shift+Arrow` erase).
- [x] **[Common (ViVi / OpenViVi)]** Box-internal text alignment (left, center, right).
- [x] **[ViVi 3 / ViVi 4]** Inline CSV/TSV column calculation view.
- [x] **[ViVi 3 / ViVi 4]** Transparent quote display and active-line raw view in CSV mode.

### Phase 3: Frontend Implementation (`vivivi-app`)
- [x] **[Original]** Terminal UI (Ratatui + crossterm) frontend implementation (interactive Vi editing & realtime render).
- [x] **[OpenViVi]** File I/O and charset/newline auto-detection (UTF-8, Shift_JIS, EUC-JP / CRLF, LF).
- [x] **[Common]** Syntax highlighting (Rust, C++, Python, JS, JSON), matching paren highlight, and paren jump (`%`).

---

### Phase 4: Daily Driver ViVi Features & Advanced Extensions (`vivivi-vi`, `vivivi-modes`, `vivivi-app`)

Focusing on features essential for primary/daily driver use (Gvim-compatible workflows) and advanced features from ViVi 3, ViVi 4, and OpenViVi.

#### 4.1 Boss Top Priority: Daily Driver Features (Gvim-compatible & Layout)
- [x] **[Common (Gvim / ViVi)] Split & Vertical Split Windows (`:split`, `:vsplit`, `Ctrl-w`)**
  - Split views (horizontal & vertical) for the same or multiple buffers.
  - Window focus switching (`Ctrl-w h/j/k/l`) and window resizing.
- [x] **[Common (Gvim / ViVi)] Full Visual Block (BOX Selection) Support (`VisualBlock` / `Ctrl-v`)**
  - Block deletion (`d`, `x`) and yank (`y`).
  - Block multi-line insert (`I`, `A`).
  - Multi-line block paste (`p`).
- [x] **[Common (ViVi / OpenViVi)] Outline Analysis & Side Pane Tree**
  - Markdown heading outline and function/struct symbol trees for code (Rust, C++, etc.).
  - Toggleable side pane with instantaneous jump to target lines.

#### 4.2 Signature ViVi Editing Assists (`vivivi-modes`)
- [x] **[ViVi 3 / ViVi 4] Keisen Protected Edit Mode**
  - Automatic whitespace padding/trimming when inserting/deleting text inside box-drawing borders so that the outer frame stays intact.
- [x] **[ViVi 3 / ViVi 4] Text Alignment**
  - Multi-line columnar alignment based on target characters (e.g., `=`, `:` via `:align <target>`).
- [x] **[ViVi 3 / ViVi 4] Renumbering**
  - Automatic renumbering of numbered lists or lines (`:renumber [start] [step]`).
- [x] **[Common (ViVi / OpenViVi)] Character Conversion**
  - Batch conversions: Full-width ⇄ Half-width, Hiragana ⇄ Katakana, Uppercase ⇄ Lowercase.
- [x] **[ViVi 3 / ViVi 4] Expression Substitute (`:s/pattern/expression/e`)**
  - Dynamic evaluation of substitution replacement string as an arithmetic/scripting expression (arithmetic, `$var` variables, sequential renumbering).
- [x] **[ViVi 3 / ViVi 4] BOX Selection Numeric Calculations (Sum, Average, StdDev)**
  - Automatic columnar calculation on visual block selections with status bar display and clipboard copying (`:calc`, `:sum`, `:total`).
- [x] **[ViVi 4] Batch Line Head/Tail String Insertion (`insBegEndLine`)**
  - Batch insertion of prefixes/suffixes across selected lines or entire document (`:insbeg`, `:insend`, `:begend`).

#### 4.3 High-Speed Input & Macro Assistance (ViVi 3 / ViVi 4 Compatible)
- [x] **[ViVi 3 / ViVi 4] Dynamic Completion (dabbrev / `Ctrl-l`, `Ctrl-m`)**
  - Forward/backward cyclic in-buffer word dynamic completion based on typed prefix.
- [x] **[ViVi 3 / ViVi 4] Static Abbreviation (`Ctrl-;`, `Alt-;`, `F4`, `:abbrev`)**
  - File-based boilerplate abbreviation expansion and header file completion for `#include`.
- [x] **[ViVi 3 / ViVi 4] Dynamic Macro (Dynamic Macro / Masui Repeat / `Alt-m`, `Ctrl-Alt-m`, `:dmacro`)**
  - Automatically infer recurring keystroke patterns (XX, XYX) from keyboard event log to repeat complex editing tasks.
- [x] **[ViVi 4] Lexical Edit Prediction UI (LexEdit / `Ctrl-j`, `:lexedit`)**
  - Predict upcoming edit patterns based on word-level insertion/deletion/replacement history.
- [x] **[ViVi 4] Zen-Coding (Emmet) Expansion (`Ctrl-,`, `Alt-,`, `:zen`)**
  - Instantaneous expansion of HTML/XML shorthand expressions (e.g., `table>tr>td*3`, `ul.menu>li*3>a`) into full tags.
- [x] **[ViVi 3 / ViVi 4] Clipboard History (`cbHist` / `F6`, `:cbhist`)**
  - Maintain recent yank/copy history with interactive modal preview and insertion.
- [x] **[Common (ViVi / OpenViVi)] Keyboard Macro Record & Playback (`Ctrl-Alt-r` / `Ctrl-Alt-p`, `F9` / `F10`, `:macro`, `@q`)**
  - Interactive keystroke sequence recording and replay.

#### 4.4 ViVi-Exclusive Vi Command Extensions & Navigation (ViVi 3 / ViVi 4 Compatible)
- [x] **[ViVi 3 / ViVi 4] Romaji Japanese Search (Migemo / `g/` Command)**
  - Incremental Japanese search directly via Romaji keyboard input without toggling IME (SKK dictionary / Migemo integration, `n`/`N` navigation).
- [x] **[ViVi 3 / ViVi 4] fFtT Full-width Character Matching (`fFtT.txt` Mapping)**
  - Automatic matching of full-width counterparts (digits, brackets, punctuation) when typing ASCII characters in `f`/`F`/`t`/`T`.
- [x] **[Common (ViVi / OpenViVi)] Cursor Word Forward/Backward Search (`*`, `g*`) & Prefix/Suffix Match (`g<`, `g=`)**
  - Instantaneous word search under cursor or matching prefix/suffix for selected text.
- [x] **[ViVi 3 / ViVi 4] Numeric / Alphabet Increment & Decrement (`#+`, `##`, `#-`) & Boolean Toggle (`#!`)**
  - In-place numeric adjustment, alphabetical cycling (`a`⇄`b`...`z`), and boolean negation (`true` ⇄ `false`).
- [x] **[ViVi 3 / ViVi 4] Last Edit Location / Line Jump (`@`, `'@`) & Above Line Text Replication (`=<char>`)**
  - Instant jump to previous edit coordinate/line, and selective character duplication from the line above.
- [x] **[ViVi 3 / ViVi 4] Global Marks (`mA` - `mZ`, `:marks`)**
  - Cross-file, workspace-persistent marks using uppercase letters.
- [x] **[Common (ViVi / OpenViVi)] Tag Jump (`Ctrl-]`, `Ctrl-[`, `Alt-]`, `Alt-[`)**
  - Code navigation to function/variable/local declarations via `tags` files and AST analysis with history stack.

#### 4.5 Tooling, File Management & Projects (ViVi 3 / ViVi 4 Compatible)
- [x] **[Custom] Help System Infrastructure (Markdown Topic Docs & In-App Viewer `:help` / `F1`) [Alpha]**
  - Topic-based Markdown help documentation organized in `doc/help/`, viewable directly in-app via `:help [topic]` or `F1` (with scrolling, Markdown styling, and compiled binary fallback). Currently operating as an Alpha release.
- [ ] **[Common (ViVi / OpenViVi)] Project-wide Grep Search (`:grep`, ripgrep-powered)**
  - Adopt high-speed `ripgrep` engine (`grep-searcher` / `ignore` crates integration).
  - Search results pane (file, line number, matched line) and instantaneous jump to matched locations.
- [ ] **[Common (ViVi / OpenViVi)] Side-by-Side File Comparison (DIFF)**
  - Dual split-view diff highlighting at line and word level (powered by `similar`).
- [ ] **[ViVi 3 / ViVi 4] Generational Backup (DiffLog / `.difflog`)**
  - Accumulate forward/backward diffs on every save to conserve disk space while enabling instant historical version rollbacks and per-line edit history (blame).
- [ ] **[Common (Gvim / Custom)] Persistent Undo (undofile)**
  - Serialize Undo/Redo stacks to cache directory on save/exit and deserialize on reopen.
  - Resume undoing past editing operations seamlessly with `u` even across separate editing sessions.
- [ ] **[ViVi 4] In-Code ToDo Extraction & Jump List (`undone` / `TODO` / `FIXME`)**
  - Scan member files for ToDo tags and display interactive jump list in project view.
- [ ] **[ViVi 3 / ViVi 4] Code Folding (`zc`, `ze`, `zE`, `z<level>`, `z%`)**
  - Selective fold/unfold of lines based on indentation levels and bracket scopes.
- [ ] **[ViVi 3 / ViVi 4] Dual-Pane Directory View (Integrated Filer)**
  - 2-pane file manager with Tab switching, directory comparison, and single-key copy/move/delete operations.
- [ ] **[ViVi 3 / ViVi 4] Project Management (Project View)**
  - Multi-file/directory workspace tree, batch tags generation, and member backup.
- [ ] **[ViVi 3 / ViVi 4] Template Auto-Loading (`.template.<ext>`)**
  - Automatic template boilerplate generation on new file creation with macro substitutions (`%FileName%`, `%Date%`, `%UserName%`).
- [ ] **[ViVi 3 / ViVi 4] Print Preview & PDF Printing**
  - Direct PDF generation/printing rather than physical printer output (supports page sizing, line numbers, headers/footers).
  - **Markdown Print Preview**: Treats document preview as print preview for Markdown files, showing rendered headings, styles, and tables.
  - Paginated print preview for plain text and source code.
- [ ] **[OpenViVi (Qt Rewrite)] Desktop GUI (egui / eframe) Frontend**
  - Rescheduled from Phase 3. Rich graphic border drawing and font rendering.

#### 4.6 OpenViVi UI/UX & Visual Enhancements
- [ ] **[OpenViVi] Modified / Saved Line Gutter Indicators (Change Markers)**
  - Visual gutter indicators and highlighting for unsaved modified lines (yellow) and saved lines (green).
- [ ] **[OpenViVi] Minimap Overview**
  - High-level bird's-eye code overview displaying viewport rectangle and search matches.
- [ ] **[OpenViVi] Recent Files & Favorite Files Management**
  - Persistent tracking and quick access to recently edited and bookmarked documents.
- [ ] **[OpenViVi] Gravity Drop Character Deletion Animation**
  - Playful physical falling character animation upon deletions.

---

### Phase 5: Future Enhancements (Future)
- [ ] **[ViVi 3 / ViVi 4] Binary / Hex Editor Mode**
  - Rescheduled from Phase 4. Hex dump + ASCII parallel view and byte-level editing.
- [ ] **[ViVi 3 / ViVi 4] Scripting Engine Integration (Modern ViViScript via Rhai / Lua)**
  - Rescheduled from Phase 4. Macro recording/playback and custom automation scripts.
- [ ] **[ViVi 4] Project Visualizer Graph (Visualize / Force-Directed Graph)**
  - 2D force-directed layout graph visualizing file sizes, include dependencies, and edit freshness.
- [ ] **[Original]** Gigabyte file (1GB+) handling via Piece Tree & Memory-Mapped Files (`memmap2`).
- [ ] **[Original]** LSP (Language Server Protocol) client integration.
- [ ] **[Original]** Multi-cursor editing.
