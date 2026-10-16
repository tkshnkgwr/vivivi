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

|  Category            |  Feature Name                                   |      Lineage (Origin)      |  Key Capabilities & Description                                      |            vivivi Status  |
|  :----------------   |  :-------------------------------------------   |  :----------------------:  |  :----------------------------------------------------------------   |  ----------------------:  |
|  Data Architecture   |  GapBuffer                                      |          OpenViVi          |  O(1) local cursor editing with memory shifting algorithm            |       Implemented (core)  |
|  Data Architecture   |  UndoMgr (Transactional Undo)                   |          OpenViVi          |  Undo blocks, typing merge, and Save Point tracking                  |       Implemented (core)  |
|  Data Architecture   |  ViewLineMgr (Wrap Line Mgr)                    |          OpenViVi          |  Bi-directional mapping between logical & wrapped screen lines       |       Implemented (core)  |
|  Data Architecture   |  Encoding & BOM Auto-Detection                  |          OpenViVi          |  UTF-8, Shift_JIS, EUC-JP, BOM recognition & conversion              |       Implemented (core)  |
|  Editing Assists     |  Unicode Keisen Mode (Box-Drawing)              |  Common (ViVi / OpenViVi)  |  Ctrl+Arrow draw/join, Shift+Arrow erase box borders                 |      Implemented (modes)  |
|  Editing Assists     |  Inline CSV/TSV Column Mode                     |      ViVi 3 / ViVi 4       |  Auto column alignment & transparent quote editing                   |      Implemented (modes)  |
|  Editing Assists     |  Batch Character Conversion                     |  Common (ViVi / OpenViVi)  |  Full-width ⇄ Half-width, Kana, Upper ⇄ Lowercase                    |      Implemented (modes)  |
|  Editing Assists     |  Keisen Protected Edit Mode                     |      ViVi 3 / ViVi 4       |  Automatic whitespace adjust inside boxes to keep frames intact      |      Implemented (modes)  |
|  Editing Assists     |  Text Alignment                                 |      ViVi 3 / ViVi 4       |  Multi-line columnar vertical alignment on target chars (=, :)       |        Implemented (app)  |
|  Editing Assists     |  Batch Head/Tail Insertion                      |           ViVi 4           |  Batch prefix/suffix insertion on selected lines or document         |        Implemented (app)  |
|  Editing Assists     |  Renumbering                                    |      ViVi 3 / ViVi 4       |  Automatic sequential renumbering for ordered lists (1., 2.)         |        Implemented (app)  |
|  Editing Assists     |  Expression Substitute (:s/.../e)               |      ViVi 3 / ViVi 4       |  Dynamic evaluation of replacement string as arithmetic/script expr  |      Implemented (modes)  |
|  Editing Assists     |  Box Selection Calculation                      |      ViVi 3 / ViVi 4       |  Instant Sum, Average, StdDev calculation on visual block numbers    |        Implemented (app)  |
|  Input & Macros      |  Dynamic Completion (dabbrev / Ctrl-L)          |      ViVi 3 / ViVi 4       |  Dynamic forward/backward in-buffer word completion                  |        Implemented (app)  |
|  Input & Macros      |  Static Abbreviation (abbrev / Ctrl-;)          |      ViVi 3 / ViVi 4       |  Boilerplate expansion from external abbreviation dictionary         |        Implemented (app)  |
|  Input & Macros      |  Dynamic Macro (Ctrl-Alt-M)                     |      ViVi 3 / ViVi 4       |  Infer recurring keystrokes (XX, XYX) to repeat complex edits        |        Implemented (app)  |
|  Input & Macros      |  Lexical Edit Prediction (LexEdit / Ctrl-J)     |           ViVi 4           |  Word-level edit history prediction in interactive popup menu        |        Implemented (app)  |
|  Input & Macros      |  Zen-Coding (Emmet) Expansion (Ctrl-,)          |           ViVi 4           |  Expand shorthand syntax (table>tr>td*3) into HTML/XML tags          |        Implemented (app)  |
|  Input & Macros      |  Clipboard History (cbHist)                     |      ViVi 3 / ViVi 4       |  Interactive history list and insertion of past yanks/copies         |        Implemented (app)  |
|  Input & Macros      |  Keyboard Macro Record & Playback               |  Common (ViVi / OpenViVi)  |  Dynamic keystroke sequence recording and replay (@q)                |        Implemented (app)  |
|  Vi Commands         |  Romaji Japanese Search (Migemo / g/)           |      ViVi 3 / ViVi 4       |  Incremental Japanese search directly via Romaji (SKK / Migemo)      |        Implemented (app)  |
|  Vi Commands         |  fFtT Full-width Auto-Match                     |      ViVi 3 / ViVi 4       |  Type ASCII char to match both half-width & full-width variants      |      Implemented (modes)  |
|  Vi Commands         |  Word Search (*, g*) & Prefix/Suffix Match      |  Common (ViVi / OpenViVi)  |  Instant cursor word search & prefix/suffix matching (g<, g=)        |         Implemented (vi)  |
|  Vi Commands         |  Numeric/Alpha Increment & Toggle (#!))         |      ViVi 3 / ViVi 4       |  Increment/decrement numbers/alpha (#+, #-), toggle true/false       |         Implemented (vi)  |
|  Vi Commands         |  Jump Last Edit (@, '@) & Line Replicate        |      ViVi 3 / ViVi 4       |  Instant return to previous edit, replicate text from above line     |         Implemented (vi)  |
|  Vi Commands         |  Global Marks (mA - mZ, :marks)                 |      ViVi 3 / ViVi 4       |  Cross-file workspace-persistent marks using uppercase letters       |         Implemented (vi)  |
|  Vi Commands         |  Tag Jump (Ctrl-], Alt-])                       |  Common (ViVi / OpenViVi)  |  Jump to declaration/definition via tags file & AST syntax analysis  |         Implemented (vi)  |
|  Tools & Files       |  Generational Backup (DiffLog / .difflog)       |      ViVi 3 / ViVi 4       |  Differential backups on save; version rollback & per-line blame     |      Implemented (modes)  |
|  Tools & Files       |  Persistent Undo (undofile)                     |   Common (Gvim / Custom)   |  Preserve Undo/Redo history on disk to resume undos across sessions  |       Implemented (core)  |
|  Tools & Files       |  In-Code ToDo Extraction (undone/TODO)          |           ViVi 4           |  Scan member files for ToDo tags and interactive jump list           |      Implemented (modes)  |
|  Tools & Files       |  Code Folding (zc, ze, zE)                      |      ViVi 3 / ViVi 4       |  Indentation & bracket scope line folding and unfolding              |      Implemented (modes)  |
|  Tools & Files       |  Dual-Pane Directory View (Filer)               |      ViVi 3 / ViVi 4       |  2-pane file manager, directory diff, and 1-key file actions         |      Implemented (modes)  |
|  Tools & Files       |  Project Management (Project View)              |      ViVi 3 / ViVi 4       |  Multi-file workspace tree, batch tags generation & backup           |      Implemented (modes)  |
|  Tools & Files       |  Template Auto-Loading (.template)              |      ViVi 3 / ViVi 4       |  Auto-populate new files with boilerplate & macro substitutions      |      Implemented (modes)  |
|  Tools & Files       |  Side-by-Side Diff Comparison                   |  Common (ViVi / OpenViVi)  |  Split-view line and character diff highlighting                     |      Implemented (modes)  |
|  Tools & Files       |  Project-wide Grep Search (:grep)               |  Common (ViVi / OpenViVi)  |  High-speed recursive scan, regex engine & interactive jump list     |      Implemented (modes)  |
|  Tools & Files       |  Print Preview & PDF Printing                   |      ViVi 3 / ViVi 4       |  Rendered Markdown preview and standalone PDF 1.4 output             |      Implemented (modes)  |
|  Tools & Files       |  Help System (:help / F1 / Markdown)            |      Custom Extension      |  Topic Markdown doc collection & in-app viewer (:help / F1)          |        Implemented (app)  |
|  UI/UX Visual        |  Multi-View Split Windows (:split, :vsplit)     |    Common (Gvim / ViVi)    |  Horizontal/vertical split views, focus switching & resizing         |        Implemented (app)  |
|  UI/UX Visual        |  Outline Tree & Side Pane                       |  Common (ViVi / OpenViVi)  |  Markdown headings and function/struct symbols tree sidebar          |        Implemented (app)  |
|  UI/UX Visual        |  Modified/Saved Line Indicators (Gutter)        |          OpenViVi          |  Gutter bar markers for unsaved modified (yellow) & saved (green)    |        Implemented (app)  |
|  UI/UX Visual        |  Invisible Characters Rendering (:set list)     |   Common (Gvim / Custom)   |  Render full-width spaces (□), tabs (▸---), and EOL marks ($)        |      Implemented (modes)  |
|  UI/UX Visual        |  Search Match Highlighting (:noh / :set hls)    |    Common (Gvim / ViVi)    |  Real-time visible match highlight & :noh clearing                   |        Implemented (app)  |
|  UI/UX Visual        |  Mode-Synchronized Cursor Shape (DECSCUSR)      |      Custom Extension      |  Block (Normal) / Bar (Insert) / Underline (Replace)                 |        Implemented (app)  |
|  UI/UX Visual        |  Recent Files & Favorite Files                  |          OpenViVi          |  Persistent tracking and quick access modal dialog for recent files  |        Implemented (app)  |
|  UI/UX Visual        |  Multi-Tab Buffer Management (Tab Bar UI)       |  Common (ViVi / OpenViVi)  |  Top tab bar for multi-buffer switching, per-tab split layouts       |  Implemented (Phase 4.8)  |
|  Editing Assists     |  Visual Block Operations (Ctrl-v + I/A/c/r)     |  Common (ViVi / OpenViVi)  |  Multi-line column text insertion, replacement, and blank fill       |  Implemented (Phase 4.8)  |
|  Config & Env        |  Configuration File (config.toml / vivivirc)    |  Common (ViVi / OpenViVi)  |  Persistent user keymaps, tab width, colors, and startup options     |  Implemented (Phase 4.8)  |
|  Encoding            |  Explicit Encoding Reload & Format (:e ++enc)   |  Common (ViVi / OpenViVi)  |  Force reopen with SJIS/EUC/UTF-8; convert CRLF/LF on save           |  Implemented (Phase 4.8)  |
|  Robustness          |  Auto-Save & Crash Recovery (.swp)              |  Common (ViVi / OpenViVi)  |  Periodic background swap dumps & unsaved recovery on startup        |  Implemented (Phase 4.8)  |
|  OS Integration      |  System Clipboard Integration ("+y, "*p)        |  Common (ViVi / OpenViVi)  |  Bidirectional transparent sync with OS clipboard registers          |  Implemented (Phase 4.8)  |
|  Languages & Syntax  |  Syntax Highlighting Expansion (Go, Ruby, etc)  |  Common (ViVi / OpenViVi)  |  Syntax highlighting for Go, Ruby, TOML, PowerShell, YAML, XML, BAT  |  Implemented (Phase 4.8)  |
|  UI/UX Visual        |  Minimap Overview                               |          OpenViVi          |  Bird's-eye code preview with viewport and search highlights         |        Planned (Phase 5)  |
|  UI/UX Visual        |  Gravity Drop Deletion Animation                |          OpenViVi          |  Playful physical falling character animation upon deletion          |        Planned (Phase 5)  |
|  UI/UX Visual        |  Desktop GUI (egui / eframe)                    |   OpenViVi (Qt rewrite)    |  Modern GUI frontend with rich borders and custom fonts              |        Planned (Phase 5)  |
|  Future              |  Tree-sitter Syntax Engine Integration          |      Modern Extension      |  Accurate AST-based syntax highlighting and AST node selection       |        Planned (Phase 5)  |
|  Future              |  Binary / Hex Editor Mode                       |      ViVi 3 / ViVi 4       |  Hex dump + ASCII parallel view and byte-level editing               |        Planned (Phase 5)  |
|  Future              |  Scripting Engine (Modern ViViScript)           |      ViVi 3 / ViVi 4       |  Editor automation via Rhai / Lua integration                        |        Planned (Phase 5)  |
|  Future              |  Project Visualizer Graph (Visualize)           |           ViVi 4           |  2D force-directed graph of file sizes, dependencies & freshness     |        Planned (Phase 5)  |
|  Distribution        |  Update Check, Notify & Auto-Update Prep        |      Modern Extension      |  GitHub Releases integration, version notification UI & self-update  |        Planned (Phase 5)  |

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
- [x] **[Custom] Help System Infrastructure (Markdown Topic Docs & In-App Viewer `:help` / `F1`)**
  - Topic-based Markdown help documentation organized in `doc/help/`, viewable directly in-app via `:help [topic]` or `F1` (with scrolling, Markdown styling, and full modal display).
- [x] **[Common (ViVi / OpenViVi)] Project-wide Grep Search (`:grep`)**
  - High-speed recursive directory traversal and regex search engine.
  - Search results session model and jump navigation with `:cnext` (`:cn`) and `:cprev` (`:cp`).
- [x] **[Common (ViVi / OpenViVi)] Side-by-Side File Comparison (DIFF)**
  - LCS-based line/character diff engine and synchronized Side-by-Side data generation.
- [x] **[ViVi 3 / ViVi 4] Generational Backup (DiffLog / `.difflog`)**
  - Differential backups on save; version rollback & per-line generation history (blame).
- [x] **[Common (Gvim / Custom)] Persistent Undo (undofile)**
  - Serialize Undo/Redo stacks to undofile format on disk and deserialize on reopen.
  - Resume undoing past editing operations seamlessly with `u` across separate sessions.
- [x] **[ViVi 4] In-Code ToDo Extraction & Jump List (`undone` / `TODO` / `FIXME`)**
  - Scan member files for ToDo tags and interactive jump navigation.
- [x] **[ViVi 3 / ViVi 4] Code Folding (`zc`, `ze`, `zE`, `z<level>`, `z%`)**
  - Indentation-based line folding and unfolding (`zc`, `zo`, `za`, `zM`, `zR`, `z<lvl>`).
- [x] **[ViVi 3 / ViVi 4] Dual-Pane Directory View (Integrated Filer)**
  - 2-pane file manager with Tab switching and single-key file actions.
- [x] **[ViVi 3 / ViVi 4] Project Management (Project View)**
  - Multi-file/directory workspace tree, batch tags generation (`:maketags`).
- [x] **[ViVi 3 / ViVi 4] Template Auto-Loading (`.template.<ext>`)**
  - Automatic template boilerplate generation on new file creation with macro substitutions (`%FileName%`, `%Date%`, `%UserName%`, `%Cursor%`).
- [x] **[ViVi 3 / ViVi 4] Print Preview & PDF Printing**
  - Standalone PDF 1.4 direct generation (supports page sizing, line numbers, headers/footers).
  - **Markdown Print Preview**: Modal paginated print preview with headers and footers.

#### 4.6 TUI/UX Editing & Visual Enhancements (Phase 4.6)
- [x] **[OpenViVi] Modified / Saved Line Gutter Indicators (Change Markers)**
  - Visual gutter indicators and highlighting for unsaved modified lines (yellow) and saved lines (green) in TUI line number area.
- [x] **[OpenViVi] Recent Files & Favorite Files Management**
  - Persistent tracking and quick access to recently edited and bookmarked documents via TUI modal dialog (`F7` / `:recent` / `:fav`).
- [x] **[Gvim / Custom] Invisible / Whitespace Character Visualization (`:set list` / `listchars`)**
  - Toggle visual markers for fullwidth spaces (`□`), tabs (`▸---`), and end-of-line symbols (`$`).

#### 4.7 Immediate Feedback Issues
- [x] **Fix keisen protected edit when inserting Japanese / full-width characters**
  - In keisen protected edit mode, inserting a 2-column wide Japanese/full-width character consumes 2 trailing spaces (or 1 full-width space) to prevent pushing the right box border.
- [x] **Keisen protected edit for Normal mode `x` / `X` deletion**
  - Extended protected editing to Normal mode single-character deletions (`x` and `X`), automatically padding spaces before the right border to maintain box geometry without corruption.
- [x] **Clear search highlights on double Esc keypress instead of single Esc**
  - Changed search highlight clearance (`hlsearch` off) from single `Esc` to double `Esc` (rapid double-press within ~400ms), reserving single `Esc` for regular command cancellation.
- [x] **Nested Window Layout Tree & Vim-compatible Directional Navigation**
  - Upgraded flat split axis into hierarchical `WindowTreeNode` (Window Layout Tree). Splitting vertically inside the lower pane of a horizontal split keeps the top pane intact while dividing only the bottom pane into left and right, matching Vim's behavior.
  - Added spatial geometric focus navigation via `Ctrl-W h/j/k/l` (Left/Down/Up/Right) based on actual screen coordinates.
- [x] **Eliminate synchronized scrolling across split windows (Independent Scroll & Cursor)**
  - Each split window now retains its own independent scroll offsets (vertical & horizontal) and cursor coordinates, completely preventing unwanted sync scrolling and allowing simultaneous parallel viewing of separate sections.
- [x] **Outline navigation & help key unification**
  - Unified toggle shortcuts on both `F2` and `F8`. Focus automatically transfers to the outline tree upon opening (`j`/`k` to select, `Enter` to jump). Pressing `Tab` or `Esc` seamlessly switches focus back to the editor buffer.
- [x] **Horizontal scrolling for lines exceeding 80 columns**
  - Replaced character-count scrolling with true Visual Column display-width tracking (`UnicodeWidthChar`). Multi-byte East Asian characters and tabs smoothly follow cursor and `$` motions without clipping artifacts or line corruption.
- [x] **Romaji search (Migemo) Kanji matching with embedded SKK dictionary**
  - Embedded standard SKK dictionary entries (`BUILTIN_SKK_ENTRIES`) directly into the binary for zero-configuration, instant offline Kanji matching. Added `:migemo <query>` ex command and automatic viewport scroll tracking to matched occurrences.

#### 4.8 ViVi / OpenViVi Advanced Editing Power Tools & Hardening (Phase 4.8)
- [x] **[Common (ViVi / OpenViVi)] Multi-Tab Buffer Management (Tab Bar UI)**
  - Top tab bar displaying active buffers with independent per-tab split window layouts (`:split`, `:vsplit`)
  - Key controls & commands: `gt`, `gT`, `[count]gt`, `:tabnew [file]`, `:tabclose` (`:tabc`), `:tabnext`, `:tabprev`
- [x] **[Common (ViVi / OpenViVi)] Visual Block Column Operations**
  - Advanced multi-line operations on `Ctrl-v` visual block selections:
  - `I<text>Esc`: Prepend text across all selected lines simultaneously
  - `A<text>Esc`: Append text to the end of all selected block lines
  - `c<text>Esc`: Replace rectangular block with replacement text
  - `r<char>`: Fill entire block area with specified character
  - `d` / `x`: Delete block and shift columns leftward
- [x] **[Common (ViVi / OpenViVi)] Configuration File & Customization (`config.toml` / `vivivirc`)**
  - Automatic settings loading from `%USERPROFILE%\.vivivi\config.toml` (or `~/.vivivi/config.toml`)
  - User remapping for keys (Normal, Insert, Visual), tab widths, line numbers, color schemes, and invisible character symbols
- [x] **[Common (ViVi / OpenViVi)] Explicit Encoding Reopening & Line Ending Format (:e ++enc, :set fenc, :set ff)**
  - `:e ++enc=sjis [file]`, `:e ++enc=euc-jp [file]`: Force reload file with specified character encoding
  - `:set fenc=utf-8 / sjis / euc-jp`: Target file encoding on save
  - `:set ff=unix / dos`: Convert line endings (LF vs CRLF) on save
- [x] **[Common (ViVi / OpenViVi)] Auto-Save & Crash Recovery (.swp)**
  - Periodic background swap file checkpoints triggered by keystroke counts or idle timer
  - Unsaved buffer recovery dialog prompt upon restarting after unexpected crash or disconnect
- [x] **[Common (ViVi / OpenViVi)] System Clipboard Integration ("+y, "*p)**
  - Transparent bidirectional sync with OS system clipboard (Windows API & OSC 52 ANSI escape sequences)
- [x] **[Common (ViVi / OpenViVi)] Syntax Highlighting Expansion (Go, Ruby, TOML, PowerShell, YAML, XML, bat/cmd)**
  - **Go (`.go`)**: Keywords (`func`, `package`, `import`, `go`, `chan`, `defer`, `type`, `struct`, `interface`), primitive & struct types, `//` & `/* */` comments, raw string literals
  - **Ruby (`.rb`)**: Keywords (`def`, `end`, `class`, `module`, `do`, `yield`, `attr_accessor`), symbols (`:symbol`), `#` comments, strings, heredocs
  - **TOML (`.toml`)**: `[...]` / `[[...]]` table headers, key names, strings, numbers, booleans, datetimes, `#` comments
  - **PowerShell (`.ps1`, `.psm1`, `.psd1`)**: Cmdlets (`Get-...`, `Set-...`), control flow, variables (`$var`), `#` and `<# #>` comments, strings
  - **YAML (`.yaml`, `.yml`)**: Keys, list hyphens (`-`), scalar values, strings, numbers, booleans, anchors/aliases (`&`, `*`), `#` comments
  - **XML (`.xml`)**: Tag names, attribute names, attribute values, `<?xml ...?>` declarations, `<!-- ... -->` comments, `<![CDATA[...]]>`
  - **bat / cmd (`.bat`, `.cmd`)**: Batch commands (`echo`, `set`, `goto`, `call`, `if`, `for`, `pause`, `rem`), env variables (`%VAR%`), labels (`:label`), comments (`rem`, `::`)
  - Status bar language badges (`[Go]`, `[Ruby]`, `[TOML]`, `[PowerShell]`, `[YAML]`, `[XML]`, `[Batch]`)

---

### Phase 5: Future Enhancements & Desktop GUI (egui) / High-Res Graphics (Future)
- [ ] **[OpenViVi (Qt Rewrite)] Desktop GUI (egui / eframe) Frontend**
  - Cross-platform desktop GUI frontend powered by OpenGL/wgpu backend alongside existing terminal TUI.
- [ ] **[OpenViVi] Minimap Overview (egui-backed)**
  - High-level bird's-eye code overview displaying viewport rectangle and search matches (requires high-res graphical rendering).
- [ ] **[OpenViVi] Gravity Drop Character Deletion Animation (egui-backed)**
  - Playful physical falling character animation upon deletions (requires frame-by-frame 2D physics animation).
- [ ] **[ViVi 3 / ViVi 4] Binary / Hex Editor Mode**
  - Hex dump + ASCII parallel view and byte-level editing.
- [ ] **[ViVi 3 / ViVi 4] Scripting Engine Integration (Modern ViViScript via Rhai / Lua)**
  - Macro recording/playback and custom automation scripts.
- [ ] **[ViVi 4] Project Visualizer Graph (Visualize / Force-Directed Graph)**
  - 2D force-directed layout graph visualizing file sizes, include dependencies, and edit freshness.
- [ ] **[Original]** Gigabyte file (1GB+) handling via Piece Tree & Memory-Mapped Files (`memmap2`).
- [ ] **[Original]** LSP (Language Server Protocol) client integration.
- [ ] **[Original]** Multi-cursor editing.
- [ ] **[Platform Expansion] macOS Support (Low priority)**
  - Validate and ensure compatibility for terminal TUI keybindings and future GUI under macOS.
- [ ] **[Platform Expansion] Linux Support (Low priority)**
  - Validate and ensure compatibility for terminal TUI and future GUI under Linux (X11/Wayland).
- [ ] **[Packaging & Distribution] Windows Package Managers Support (Scoop / Winget) (Medium priority)**
  - **Scoop**: Create custom bucket (`scoop-bucket`) and `vivivi.json` manifest for portable, non-admin CLI/TUI distribution.
  - **Winget**: Submit package manifest to `microsoft/winget-pkgs` using `wingetcreate` for Windows 11 default package manager integration.
  - Establish release automation workflow tied to GitHub Releases (portable zip).
- [ ] **[Update Management & Operations] Update Check, Notification & Auto-Update Pipeline Preparation (Medium priority)**
  - **1. GitHub Actions CI/CD Release Pipeline (`.github/workflows/release.yml`)**:
    - Automated cross-compilation & release workflow triggered on `git push origin v*` (version tags).
    - Windows x86_64 (`x86_64-pc-windows-msvc`) as primary priority, matrix support prepared for macOS / Linux.
    - Standard release asset naming: `vivivi-v<X.Y.Z>-<target-triple>.zip` (e.g., `vivivi-v0.1.0-x86_64-pc-windows-msvc.zip`).
    - Automated checksum generation: Compute `SHA256SUMS` for all artifacts and upload alongside release assets.
  - **2. Release Standards & Versioning Conventions**:
    - Strict semantic versioning (`v<Major>.<Minor>.<Patch>`). Pre-releases use `v0.1.0-alpha.1` with automated pre-release flag.
    - Zip bundle contents: `vivivi.exe`, `README.md`, `LICENSE`, and `doc/help/` (self-contained offline help documentation).
  - **3. In-App Client Integration Protocol (`vivivi` Application Specifications)**:
    - **Asynchronous Endpoint Query**: Query `GET https://api.github.com/repos/<owner>/vivivi/releases/latest` in a detached background thread after startup (0ms startup latency impact).
    - **Throttling & Cache**: Cache query results in `%USERPROFILE%\.vivivi\cache\update_check.json` with a 24-hour rate-limit threshold to respect GitHub API limits (60 unauth req/hr).
    - **Dynamic Asset Selection & SHA256 Verification**: Identify matching asset URLs for target OS / architecture, download, and verify against `SHA256SUMS` for integrity and tamper prevention.
    - **Windows In-Place Binary Replacement (Rename Trick)**: Evade Windows file locking by renaming current `vivivi.exe` to `vivivi.exe.old`, extracting new binary into original location, and cleaning up `.old` on next launch/shutdown.
    - **Notification UX**: Subtly show `[⬆ v0.2.0 available (:upgrade)]` on the right side of status bar. Only perform download and binary replacement when explicitly approved via `:upgrade` or menu command (never terminate dirty buffers abruptly).
  - **4. Package Manager (Scoop / Winget) Coexistence & Detection**:
    - Detect executable path (e.g. within `scoop\apps\vivivi\` or `WindowsApps`) to determine if installed via a package manager.
    - When managed by a package manager, inhibit in-app binary overwrites and guide users to execute `scoop update vivivi` or `winget upgrade vivivi`.
- [ ] **[Quality & Extreme Performance] Benchmarking Suite (`benches/`) (Medium priority)**
  - Establish statistical benchmarking harness and HTML report generation using `criterion`.
  - Measure $O(1)$ vs $O(N)$ insertion/deletion speedup comparing GapBuffer against standard Vec on 1M chars.
  - Latency profiling for inline CSV/TSV width calculation and Migemo incremental search on 100k lines.
- [ ] **[Developer Ergonomics] Standalone Executable Examples (`examples/`) (Low priority)**
  - Minimal CLI demos verifying `vivivi-core` and `vivivi-modes` logic without launching the entire TUI.
  - Place `gap_buffer_demo.rs` (buffer locality visualization), `keisen_combine.rs` (Unicode box drawing intersection test), `inline_csv_align.rs` (instant table formatter).

