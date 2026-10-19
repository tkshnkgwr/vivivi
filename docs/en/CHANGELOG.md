# vivivi: Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.9.0-beta] - 2026-10-19 (Testing / Dogfooding)

### Added
- **Major Vi Standard Editing Command Enhancements (Phase 1 & Phase 2)**:
  - **Full Text Objects Engine**:
    - Word & Big Word objects: `iw` / `aw`, `iW` / `aW`.
    - Quote objects: `i"` / `a"` (double quotes), `i'` / `a'` (single quotes), `` i` `` / `` a` `` (backticks).
    - Bracket & Parenthesis objects: `i(` / `a(` (`ib`/`ab`), `i{` / `a{` (`iB`/`aB`), `i[` / `a[`, `i<` / `a<`.
    - Operator combinations in Normal mode (`diw`, `ci"`, `ya(`, `guiw`, etc.).
    - Automatic selection expansion in Visual mode (`v` + `i"`, `v` + `iw`, etc.).
  - **Case Conversion Operators**:
    - `gu<motion>` / `gU<motion>`: Lowercase / uppercase target motion or text object range (`guw`, `gU$`, `gui"`, etc.).
    - `guu` / `gug`, `gUU` / `gUg`: Line-wise lowercase / uppercase conversions.
    - Visual mode conversions: `u` (lowercase), `U` (uppercase).
  - **Indentation Operators**:
    - Line-wise indentation: `>>` (shift right / 4 spaces), `<<` (shift left / unindent).
    - Motion-targeted indentation: `>` + motion, `<` + motion.
    - Visual mode selection indentation: `>`, `<`.
  - **Dot Repeat Engine (`.`)**:
    - Full repeatable change engine accurately repeating the last modification (single-command deletion `x`, operator changes `cw`/`ciw`, line changes `cc`, indentations `>>`/`<<`, case shifts, and character insertions).
  - **Screen Scrolling Adjustments**:
    - `zz`: Center cursor line vertically on screen.
    - `zt`: Scroll cursor line to the top of screen.
    - `zb`: Scroll cursor line to the bottom of screen.
  - **Paragraph Jump Motions**:
    - `{`: Jump backward to preceding blank line / paragraph start.
    - `}`: Jump forward to next blank line / paragraph end.
- **Dynamic Line Number & Line Wrap Control Commands**:
  - Implemented `:se nu` / `:se nonu` / `:set nu` / `:set nonu` (line numbering), `:se rnu` / `:se nornu` (relative line numbering).
  - Implemented `:se wrap` / `:se nowrap` / `:set wrap` / `:set nowrap` (line wrapping at terminal edge vs. default horizontal scrolling).
  - Implemented configuration query commands (`:se nu?`, `:se wrap?`, `:se rnu?`, etc.).
  - Added and persisted `wrap = false` option under `[editor]` in `config.toml`.
- **Tab Key Input in Insert & Replace Modes**:
  - Handled `KeyCode::Tab` in `handle_insert_mode` and `handle_replace_mode` to reliably insert and replace tab character `\t`.

### Fixed / Improved
- **PDF Export Duplicate File Creation Bug**:
  - Resolved bug in print preview (`p` key) and `:pdf` command where dual PDF files were generated; now reliably outputs a single complete PDF document.
- **Generational Backup DiffLog Naming Convention**:
  - Standardized edit session diff log file names to leading-dot `.<filename>.difflog` and registered in `.gitignore` to prevent repository pollution.
- **Multi-Line Visual Yank & Paste Bug**:
  - Fixed issue where pasting (`p`) after multi-line character visual yank (`v` + `y`) corrupted lines; now cleanly pastes as character/line-wise blocks.
- **Filer `d` Key Conflict Resolution**:
  - Fixed conflict between Vi delete operator `d` and 2-pane Filer delete prompt, ensuring the deletion confirmation modal immediately triggers in Filer.
- **Full-Width (Zenkaku) Space Visualization**:
  - Replaced ambiguous-width `□` substitution with native `\u{3000}` (preserving exact 2-column display width) highlighted with a refined wine-red background (`Color::Rgb(120, 35, 35)`) under `:set list`.
  - Fixed rendering order bug where current line background was overwriting full-width space highlight colors.
- **Tab Character Rendering & Visual Cursor Synchronization**:
  - Expanded `\t` into accurate space count matching `tab_width` setting in normal view mode.
  - Synchronized hardcoded tab width `4` in `visual_col_at` with dynamic runtime `tab_width` setting.
- **Feature Freeze & Transition to Beta Testing**:
  - Following the complete implementation of all planned features (Phases 1 through 4.8) and large-scale modularization refactoring, the development phase has officially transitioned from Alpha (feature addition) to Beta (validation, hardening, and dogfooding).
  - Started manual testing and real-world daily usage verification based on `MANUAL_TEST_GUIDE.md`.

---

## [0.9.0-alpha] - 2026-10-16 (Alpha Final)

### Added
- **Phase 4.8 Advanced Editing Power Tools & Hardening**:
  - **Multi-Tab Buffer Management (Tab Bar UI)**:
    - Implemented `TabPage` maintaining independent window split trees, active buffers, and cursor positions per tab.
    - Automatically render a top-line tab bar when 2+ tabs are open, highlighting the active tab.
    - Full support for `gt`, `gT`, `[count]gt`, `:tabnew [file]`, `:tabclose` (`:tabc`), `:tabnext`, `:tabprev`, `:tabonly`.
  - **Visual Block Column Operations**:
    - Supported multi-line columnar operations on `Ctrl-v` rectangular selections: `I<text>Esc` (prepend column), `A<text>Esc` (append column), `c<text>Esc` (replace block), `r<char>` (fill block), and `d` / `x` (delete block and shift text).
  - **Configuration File & Customization (`config.toml` / `vivivirc`)**:
    - Automatic loading of settings from `%USERPROFILE%\.vivivi\config.toml` (or `~/.vivivi/config.toml`).
    - Persistent options for editor settings (`tab_stop`, `number`, `relative_number`, `gutter`, `hlsearch`, `clipboard`, `theme`), invisible characters, and key remapping.
    - Added dynamic runtime `:config` reload command, and `:set number`, `:set relativenumber`, `:set tabstop`, `:set clipboard` commands.
  - **Explicit Encoding Reopening & Line Ending Format (:e ++enc, :set fenc, :set ff)**:
    - `:e ++enc=sjis [file]`, `:e ++enc=euc-jp [file]`: Force reload file using specified character encoding.
    - `:set fenc=utf-8 / sjis / euc-jp`: Dynamically change file encoding for next save.
    - `:set ff=unix / dos`: Convert line endings (LF vs CRLF) on save.
  - **Crash Recovery & Auto-Save Swap Files (.swp)**:
    - Periodic background swap file checkpoints every 20 keystrokes and on `Esc` (`.<filename>.swp` format).
    - Warning prompt upon launching if an existing swap file is detected after an unexpected exit or crash.
    - Implemented `:recover` (restore unsaved buffer) and `:swap` (inspect swap file metadata) commands.
    - Automatic cleanup of swap files upon successful save or clean exit.
  - **System Clipboard Integration ("+y, "*p)**:
    - Integrated `arboard` crate for cross-platform `SystemClipboard` provider.
    - Supported `"+y` / `"*y` (yank directly to OS clipboard) and `"+p` / `"*p` (paste from OS clipboard).
    - Added `:set clipboard=unnamed / unnamedplus / none` for transparent synchronization with standard Vi registers.
  - **Syntax Highlighting Expansion (18 Languages Supported)**:
    - Added syntax tokenizers and status bar badges for Go (`.go`), Ruby (`.rb`), TOML (`.toml`), PowerShell (`.ps1`), YAML (`.yaml`), XML (`.xml`), and Windows Batch (`.bat`, `.cmd`).
  - **Built-in Help System Expansion**:
    - Added `doc/help/tab.md`, `doc/help/config.md`, `doc/help/recovery.md`, `doc/help/clipboard.md`.
    - Synchronized `doc/help/index.md`, `doc/help/syntax.md`, `doc/help/visual.md`, and `doc/help/basic.md`.

### Changed / Refactored
- **Large-Scale Module Decomposition & Architecture Refactoring**:
  - Decomposed oversized source files (>1,000 lines) across all crates into focused submodules with clean separation of concerns, maintaining 100% backward compatibility of public APIs:
    - `vivivi-app/src/tui/app.rs` (3,440 lines) $\to$ `tui/app/` (`mod.rs`, `tabs.rs`, `windows.rs`, `files.rs`, `tools.rs`, `tests.rs`).
    - `vivivi-app/src/tui/event.rs` (1,696 lines) $\to$ `tui/event/` (`mod.rs`, `modes.rs`, `ex.rs`).
    - `vivivi-modes/src/syntax.rs` (1,563 lines) $\to$ `syntax/` (`mod.rs`, `keywords.rs`, `tests.rs`).
    - `vivivi-modes/src/keisen.rs` (1,240 lines) $\to$ `keisen/` (`mod.rs`, `draw.rs`, `protect.rs`, `tests.rs`).
    - `vivivi-vi/src/engine/mod.rs` (1,182 lines) $\to$ `engine/` (`mod.rs`, `exec.rs`).
    - `vivivi-core/src/undo.rs` (960 lines) $\to$ `undo/` (`mod.rs`, `serialize.rs`, `tests.rs`).
  - Every production source file (excluding test files) across the entire workspace is now strictly under 1,000 lines (most under 800 lines). All 354+ tests pass.

---

## [0.8.0-alpha] - 2026-10-15

### Added
- **Vim-Compatible Window Hierarchy Tree Layout (`window.rs`, `app.rs`)**:
  - Recursive tree structure `WindowTreeNode` for nested window splits.
  - Geometric proximity navigation for `Ctrl-W h / j / k / l` and arrow keys.
  - Independent scroll offsets and buffer views per window.
- **Visual Column-Based Horizontal Scrolling (`editor.rs`, `app.rs`)**:
  - Accurate scrolling calculation based on terminal visual columns rather than char counts, handling fullwidth East Asian characters seamlessly.
- **Outline Navigation Overhaul (`outline.rs`, `event.rs`)**:
  - Unified `F2` and `F8` shortcuts, automatic focus transfer on toggle, bidirectional `Tab`/`Esc` switching.
  - Multi-level header extraction (H1-H6), content-based language auto-detection, and auto-scroll tracking.
- **Vim-Compatible `Insert` Key Mode Toggle (`event.rs`)**:
  - Toggle seamlessly between Normal, Insert, and Replace modes via `Insert` key.
- **Migemo Romaji Japanese Search Built-in Clean MIT Dictionary (`migemo.rs`)**:
  - Built-in clean MIT-licensed dictionary covering punctuation and symbols (`kakko`, `yajirushi`, `maru`, `ten`, etc.).
  - Automatic discovery and transparent loading of external SKK dictionaries.
- **Manual Test Guide (`docs/ja/MANUAL_TEST_GUIDE.md`, `docs/en/MANUAL_TEST_GUIDE.md`)**:
  - Comprehensive guide for real-world verification of splits, keisen, outline, horizontal scroll, and Migemo.

---

## [0.7.0-alpha] - 2026-10-14

### Added
- **TUI/UX Visual Assistance (Phase 4.6)**:
  - **Change Markers (Gutter Highlighting)**: Visual indicator (`▌`) on line number gutter distinguishing unsaved vs saved edits (`:set gutter`).
  - **MRU & Favorites Modal**: `F7` / `:recent` dialog for opening recent and favorite files.
  - **Invisible Character Visualization**: `:set list` displaying fullwidth spaces, tabs, and newline symbols.
- **Geometric Keisen Corner Connection Algorithm**:
  - Reconstructed corner joins based on neighbor connectivity, completely eliminating frayed corner artifacts.
- **Terminal Cursor Style Synchronization**:
  - Dynamic cursor shape sync: SteadyBlock for Normal/Visual, SteadyBar for Insert, SteadyUnderScore for Replace.
- **Replace Mode (`R`) TUI Handling**:
  - In-place character overwrite, line auto-expansion, and undo block integration.
- **Search Highlighting (`hlsearch`) & Clearance (`:noh`, Double-Esc)**:
  - Real-time highlight of matching words, `:noh` command, and quick double `Esc` clearing.
- **Keisen Mode Modifier Key Guard**:
  - Keisen drawing/erasing restricted to `Ctrl`/`Alt`/`Shift` + arrows, keeping bare arrow keys and `hjkl` for normal navigation.

---

## [0.6.0-alpha] - 2026-10-14

### Added
- **Tooling & File Management TUI Integration (Phase 4.5)**:
  - **DiffLog (`.difflog`)**: Incremental backup on save, historical version restoration, and line-level blame.
  - **Persistent Undo (`undofile`)**: Multi-session undo stack serialization and transparent recovery.
  - **Print Preview & Standalone PDF 1.4 Direct Export (`print.rs`)**: A4 pagination, formatted preview, and zero-dependency PDF generation.
  - **Template Auto-Loading (`.template.<ext>`)**: Macro expansion on creating new files.
  - **In-Code ToDo Extraction (`todo.rs`)**: Scanning comments for `TODO`/`FIXME` and jumping to locations.
  - **Code Folding (`folding.rs`)**: Indentation-based folding hierarchy and commands (`zc`, `zo`, `za`, `zM`, `zR`, `z<lvl>`).
  - **Project-Wide Grep (`grep.rs`)**: Recursive fast scanning, regex matching, and QuickFix navigation (`:cnext`, `:cprev`).
  - **2-Pane Filer (`filer.rs`)**: Dual-pane directory tree, Tab switching, single-key operations.
  - **Project Management (`project.rs`)**: Integrated workspace tree and batch `:maketags` generation.

---

## [0.5.0-alpha] - 2026-10-14

### Added
- **Expression Substitution Engine (`expr_subst.rs`)**:
  - `:s/pattern/expression/e` arithmetic evaluation, capture variable `$0`-$9 expansion, `$i` numbering, and `$line` integration.
- **ViVi-Specific Vi Extensions (Phase 4.4)**:
  - **Transparent Fullwidth Character Matching (`fftt.rs`)**: Semi-automatic mapping between ASCII and fullwidth symbols.
  - **Word Boundary Search**: `*`, `#` exact word matches; `g*`, `g#` partial matches; `g<`, `g=` prefix/suffix matches.
  - **Number / Letter Increment & Decrement**: `#+`, `##`, `#-` increments, and `#!` boolean toggles.
  - **Jump to Last Edit Location**: `@`, `'@`, `` `@ `` instant jump, and `=<char>` copy from line above.
  - **Mark Navigation**: Local marks `ma`-`mz`, persistent global marks `mA`-`mZ`, and `:marks` dialog.
  - **Tag Navigation**: `Ctrl-]`, `Ctrl-t`, `Ctrl-[`, `:tag`, `:tags` with tags file and syntax fallback.

---

## [0.4.0-alpha] - 2026-10-14

### Added
- **Gvim Compatibility & Production Layout (Phase 4.1 - 4.3)**:
  - **Split Screen Multi-View**: `:split`, `:vsplit`, `:close`, `:only`, and `Ctrl-w` window keybindings.
  - **Visual Block Operations**: `Ctrl-v` rectangular selection, yank (`y`), delete (`d`, `x`), prepend (`I`), append (`A`), and smart block paste.
  - **Outline Analysis Pane**: Markdown header trees, Rust function/struct trees, and instant jump.
  - **Character Conversion Ex Commands**: `:%zenkaku`, `:%hankaku`, `:%hiragana`, `:%katakana`, `:%toupper`, `:%tolower`.
  - **Text Alignment, Renumbering, & Line Operations**: `:align <target>`, `:renumber [start] [step]`, `:insbeg`, `:insend`, `:begend`.
  - **Typing Assistance & Macros**: Dynamic word completion (dabbrev), static abbreviations (abbrev), dynamic macros (Masui repeat), LexEdit, Zen-Coding (Emmet), keyboard macros.
  - **Box Calculation**: Sum, average, and standard deviation over rectangular blocks (`:calc`, `:sum`, `:total`).
  - **Clipboard History**: `F6` / `:cbhist` dialog.
  - **Built-in Help Viewer**: `F1`, `:help [topic]`.

---

## [0.3.0-alpha] - 2026-10-13

### Added
- **TUI Frontend Implementation (`vivivi-app`)**:
  - Fullscreen terminal management via `ratatui` (v0.29) and `crossterm` (v0.28).
  - Alternate screen, raw mode, and panic-safe recovery hooks.
  - Real-time editor buffer rendering, status line, and command-line prompt.
- **Encoding & Line Ending Detection (`encoding.rs`)**:
  - BOM recognition, UTF-8, Shift_JIS/CP932, EUC-JP, UTF-16LE/BE.
  - Automatic detection and conversion between CRLF, LF, and CR line endings.
- **Matching Parenthesis Search (`paren.rs`)**:
  - Comment/string-skipping search for `()`, `{}`, `[]`, `<>`, and `%` jump.
- **Syntax Highlighting Engine (`syntax.rs`)**:
  - Colorized keywords, types, strings, and comments for Rust, C/C++, Python, JS, and JSON.

---

## [0.2.0-alpha] - 2026-10-13

### Added
- **Vi Command Engine Foundation (`vivivi-vi`)**:
  - Mode state machine (Normal, Insert, Visual, VisualLine, CommandLine).
  - Motions (`h/j/k/l`, `w/b/e`, `0/^/$`, `gg/G`).
  - Operators and actions (`d/c/y/p/P`, `dd/yy/cc`, `D/C/Y`, `x/X`).
  - Register management (unnamed, `"a`-`"z`, uppercase append).
  - Search (`/`, `?`, `n`, `N`) and basic ex commands (`:w`, `:q`, `:wq`, `:s`, `:%s`).
  - Stateful `CommandParser`.
- **Extended Modes Foundation (`vivivi-modes`)**:
  - Unicode box-drawing mode (`Ctrl+arrow` to draw/join, `Shift+arrow` to erase).
  - Text alignment inside box frames (left, center, right).
  - Inline CSV/TSV mode (auto-aligned columns, transparent quote handling).

---

## [0.1.0-alpha] - 2026-10-13

### Added
- **Core Data Structures (`vivivi-core`)**:
  - OpenViVi-compliant `GapBuffer<T>` ($O(1)$ local edits, dynamic capacity doubling, gap sliding).
  - Transactional `UndoMgr` (atomic undo blocks, continuous typing merge, save point tracking).
  - Unified `TextBuffer` integrating `GapBuffer` and `UndoMgr`.

---

## [0.0.1-alpha] - 2026-10-13

### Added
- **Initial Environment Setup**:
  - Multi-crate workspace setup (`vivivi-core`, `vivivi-vi`, `vivivi-modes`, `vivivi-app`).
  - AI pair programming rules and skills infrastructure (`.agents/`, `GEMINI.md`, `RULES.md`).
  - Bilingual documentation structure (`docs/ja/`, `docs/en/`).
