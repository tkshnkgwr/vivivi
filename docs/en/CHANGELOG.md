# vivivi: Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.9.9-beta] - 2026-10-29

### Fixed / Improved
- **Comprehensive Workspace Bug Fixes, Robustness & Edge-Case Safeguards (34+ issues resolved)**:
  - **Core & Vi Engine Hardening**:
    - `vivivi-core`: Prevented buffer modified-flag freezing when editing immediately after saving, guaranteeing correct `is_modified` state transitions.
    - `undo`: Overhauled undofile path serialization with safe hex-encoding, fully supporting complex Windows/Unix paths and drive letters.
    - `encoding`: Fixed UTF-16 first-character truncation and improved line-ending detection for mixed CRLF/LF files.
    - `paren`: Strengthened nested parenthesis scan bounds checking.
    - `vi/text_object`: Enhanced boundary detection for quote text-objects (e.g. `ci"`) when multiple quotes reside on the same line.
    - `vi/ex`: Hardened Ex substitute command (`:s`) with atomic line-by-line application and accurate replacement counts across multi-line ranges.
    - `vi/motion`: Corrected word motions (`w`, `b`, `e`) boundary clamping on blank lines and line extremities.
  - **Modes Extended Editing Suite Hardening (22 fixes)**:
    - **CJK & Full-Width Handling**:
      - `line_edit`: Eliminated `split_at` UTF-8 byte boundary panic during auto-indent with full-width spaces.
      - `invisible`: Corrected tab-stop column calculation in invisible character rendering to respect full-width character display width (2 columns).
      - `keisen`: Resolved column alignment drift when boxed text contains full-width characters and prevented improper endpoint straightening.
      - `todo`: Fixed column calculation in full-width comment lines to count UTF-8 characters accurately.
      - `conv`: Added bidirectional conversion between Japanese wave dash (`〜` U+301C) and half-width tilde (`~`).
      - `dabbrev`: Excluded CJK punctuation marks (「」、。) from dynamic abbreviation word tokens.
    - **Safety, Zero-Division & OOM Safeguards**:
      - `diff`: Implemented common prefix/suffix pre-stripping for near-instant diff calculation and imposed a 1,000,000-cell safety limit against OOM.
      - `folding` / `invisible`: Added zero-division guards when `tab_width == 0`.
      - `print`: Prevented infinite page pagination loops and memory exhaustion when `lines_per_page <= 4`.
      - `filer`: Safeguarded against overwriting files when attempting to copy to identical paths, and added `EXDEV` copy-and-delete fallback for cross-device file moves.
    - **Formatting & Syntax Precision**:
      - `align`: Prevented split of comparison operators (`==`, `!=`, `<=`, `>=`, `=>`) and scope operators (`::`) with interior alignment spaces.
      - `csv`: Supported RFC 4180 standard `""` quote escapes and prevented cell mis-tokenization caused by English apostrophes (`'`).
      - `calc`: Prevented erroneous digit concatenation for spaceless comma-separated numbers in BOX calculations.
      - `renumber`: Preserved nested sub-list hierarchy when renumbering lists.
      - `difflog`: Overhauled patch-added line tracking in mini-blame for accurate commit generation tracking.
      - `expr_subst`: Retained `$0`, `$1` capture variable replacement via `re.captures_iter`.
      - `syntax`: Fixed numeric literal detection to avoid capturing method chains (e.g. `123.to_string()`).
      - `project`: Enhanced tags generation to recognize modifiers such as `async fn` and `pub(crate) fn`.
      - `template`: Removed multiple `%Cursor%` placeholders cleanly and sanitized macro identifiers.
  - **TUI Frontend Usability & Bug Fixes (12 fixes)**:
    - **Unsaved Changes Protection**: Enforced unsaved modifications guard across `:e`, filer, history, grep, `:tabclose`, `:tabonly`, `:q`, and `:qa`.
    - **Tab Management**: Prevented inadvertent editor termination when closing windows in multi-tab sessions via `:close`.
    - **Keybindings**: Fully supported `Ctrl+[` as an Esc replacement across all modes and modals; enabled `Ctrl+u`, `Ctrl+w`, and `Ctrl+h` in command-line input.
    - **Display & Scrolling**: Fixed cursor positioning when `:set nonumber` is active, adjusted Migemo search cursor width, prevented full-width character clipping on horizontal scroll, added scroll tracking to history/clipboard modals, synchronized jump navigation to center view, supported escaped slashes (`\/`) in substitution, and prevented statusline wrapping glitch on narrow terminals.
  - **GUI Frontend Optimization & Bug Fixes (11 fixes)**:
    - **Rendering Performance**: Integrated syntax token line cache, reused zero-allocation draw buffers, optimized smart font loading for PlemolJP, and enabled `opt-level=3` for external crates.
    - **UX & Robustness**: Accurate mouse-click character index resolution handling full-width characters and horizontal scrolling, added `Ctrl+[` to Esc mapping, protected inactive tabs and OS window close events with unsaved prompt (E37), blocked keyboard pass-through during modal dialogs, sanitized blank file paths, synchronized minimap/outline click viewports, clamped out-of-screen IME candidate rects, restored number keys (0-9) and `Ctrl+6` shortcuts, and reduced particle heap allocations.

---

## [0.9.7-beta] - 2026-10-27

### Changed
- **Project License Transition (GPL-3.0-or-later)**:
  - Transitioned the project license from MIT License to GNU General Public License v3.0 or later (GPL-3.0-or-later).
  - Updated `LICENSE` with the full text of GNU GPL v3.
  - Updated `Cargo.toml` package license metadata and documentation (`README.md`, `README_JA.md`) license notices and badges.

---

## [0.9.6-beta] - 2026-10-26

### Refactored
- **Modular Refactoring of Monolithic Source Files**:
  - Deconstructed large source files exceeding or nearing the 1,000-line modular design threshold into single-responsibility submodules while retaining 100% backward API compatibility:
    1. **`vivivi-gui` Main App Deconstruction (`crates/vivivi-gui/src/app/`)**:
       - Split the monolithic `app.rs` (2,074 lines) into 5 focused modules: `mod.rs` (struct definition & `eframe::App` trait impl), `input.rs` (event handling & particle emission), `panels.rs` (top menu/tabs, sidebar, bottom statusbar), `canvas.rs` (central editor split grid, custom painter, IME preedit underline, OS IME rect notification, minimap), and `modals.rs` (floating modal windows).
    2. **`vivivi-app` Tooling Deconstruction (`crates/vivivi-app/src/tui/app/tools/`)**:
       - Split `tools.rs` (1,000 lines) into `mod.rs` (outline parsing, text conversion, box calculation, clipboard history, Migemo search, help viewer) and `macros.rs` (text align, renumbering, insert beg/end, dabbrev, abbrev, Zen-Coding, Dynamic Macro, LexEdit, keyboard macro recording).
    3. **`vivivi-vi` Parser State Machine Deconstruction (`crates/vivivi-vi/src/parser/`)**:
       - Split `parser.rs` (983 lines) into `mod.rs` (state enum definition, command line & multi-stroke transitions) and `idle.rs` (Visual mode key handling, idle single-key dispatch, motion parsing).

---

## [0.9.5-beta] - 2026-10-26

### Added / Improved
- **Full Native Desktop GUI Implementation (`vivivi-gui` Crate / Steps 0–4 Completed)**:
  - **Step 0 (Decoupling `vivivi-app` into a Shared Headless Library)**:
    - Added `[lib]` target (`src/lib.rs`, `src/types.rs`) to `vivivi-app`, exposing `App`, `Tab`, `WindowTreeNode`, `EditorConfig`, `handle_key_event`, and `execute_ex_command` to both TUI and GUI frontends.
    - Introduced frontend-agnostic `LogicalRect` and `AppKeyEvent` types and expanded Ex command aliases (`:undo` / `:u`, `:redo`, `:vsp`).
  - **Step 1 (Tokyo Night Visual Theme & PlemolJP Priority Font Loader / `theme.rs`, `fonts.rs`)**:
    - Configured 3-tier contrast dark palette (`#1A1B26` editor canvas, `#16161E` side panels, `#292E42` statusline), eliminated harsh widget strokes (`Stroke::NONE`), and applied `4px` corner rounding.
    - Implemented multi-stage font loader prioritizing `PlemolJP` from `./fonts/`, `%LOCALAPPDATA%\Microsoft\Windows\Fonts`, or `C:\Windows\Fonts`, falling back to `BIZ UD Gothic` and `Consolas`.
  - **Step 2 (4-Pane Skeletal Layout & Viewport-Culled Minimap / `app.rs`)**:
    - Built top menu bar & multi-tab bar, collapsible left sidebar (File Explorer with `..` parent traversal & directory navigation / Symbol Outline toggle), 2-row bottom status bar & ex command line, and central nested split editor canvas (`WindowTreeNode`) with right-side minimap.
  - **Step 3 (Shared `App` Core Coupling & Inline Japanese IME Integration / `app.rs`)**:
    - Bypassed `egui::TextEdit` entirely; captured `Event::Key`, `Event::Text`, and `Event::Ime(Preedit/Commit)` via `ctx.input` and dispatched directly to the shared `App` core, guarding IME composition keys (`Backspace`, `Enter`, etc.) while preedit is active.
    - Rendered monospaced character grid via `ui.painter()` supporting line numbers, gutter change markers (yellow unsaved / green saved), code folding, CSV alignment, invisible characters & full-width space highlight, matching parentheses, visual selection, search highlight, and underlined inline IME preedit preview.
    - Rendered mode-aware cursors (Block for Normal/Visual, I-beam for Insert, Underline for Replace) and emitted `ctx.output_mut(|o| o.ime = ...)` cursor rects to the OS IME every frame.
    - Integrated floating `egui::Window` modals for `:help` (`F1`), `:preview`, `F6` (clipboard history), `:recent` (`F7`), and `:filer`.
  - **Step 4 (Gravity-Drop Deletion Animation & 0% CPU Hybrid Loop / `particles.rs`)**:
    - Implemented `ParticleManager` and `extract_deleted_line_segment` to spawn falling 2D physics glyphs (`720.0 px/s²` gravity, 0.5s alpha fade-out) whenever characters or lines are deleted (`x`, `dd`, `dw`, `Backspace`).
    - Requested `ctx.request_repaint()` strictly while particles are active, immediately returning to `egui` reactive idle mode (`0% CPU` on input wait) once all particles expire.
- **Ultra-Fast Early Encoding & Line Ending Auto-Detection (`vivivi-core::encoding`)**:
  - Overhauled the detection engine to immediately return upon discovering exclusive signature bytes rather than scanning entire large buffers.
  - **BOM Early Return**: Instantly identifies UTF-8 BOM (`EF BB BF`), UTF-16LE (`FF FE`), and UTF-16BE (`FE FF`) from the leading bytes.
  - **UTF-8 Validity Check**: Leverages SIMD-accelerated `std::str::from_utf8`. Valid UTF-8 text with multi-byte characters immediately resolves to UTF-8.
  - **Shift_JIS Deterministic Early Return (1st Character Match)**:
    - 1st byte in `0x81..=0x9F` (hiragana `0x82`, katakana `0x83`, JIS level 1 kanji): Immediately resolves to Shift_JIS as this range does not exist in standard EUC-JP or lead UTF-8.
    - 2nd byte in `0x40..=0x7E` (e.g. katakana "Te" `83 65` or 0x5C-containing characters like "Hyou" `95 5C`): Strictly impossible in EUC-JP/UTF-8 multi-byte trails, instantly resolving to Shift_JIS.
  - **EUC-JP Deterministic Early Return**: Identifies EUC-JP hiragana (`0xA4` + `0xA1..=0xF3`) and katakana (`0xA5` + `0xA1..=0xF6`) on first appearance.
  - **Line Ending 3-Line Early Return**: Immediately resolves newline format when 3 consecutive identical line endings (CRLF, LF, CR) appear, skipping unnecessary buffer scanning.
- **Comprehensive Encoding & Line Ending Test Suite (`tests/encoding_api.rs`)**:
  - Added dedicated test cases covering Shift_JIS katakana/hiragana/0x5C characters (`表構十申能`), EUC-JP, UTF-8 with emoji, quick newline resolution, ASCII-only files, and mixed line endings.
  - Implemented 12-way roundtrip tests across all combinations (UTF-8, UTF-8 BOM, Shift_JIS, EUC-JP $\times$ LF, CRLF, CR) verifying 100% fidelity upon save and reload.

---

## [0.9.2-beta] - 2026-10-19

### Fixed / Improved
- **Complete Resolution of Japanese (CJK) Text Mojibake in PDF Output**:
  - Replaced legacy ASCII-only font (`Type 1 / Courier`) and non-ASCII fallback `?` substitution with standard **Adobe CIDFont (`Type 0` / `HeiseiKakuGo-W5` / `UniJIS-UTF16-H`)**, rendering beautiful Japanese Gothic typography across major PDF viewers (Windows Edge / Chrome / Acrobat Reader, macOS Preview, Linux, etc.) without inflating binary size.
  - Applied `/DW 1000 /W [ 1 230 500 ]` to enforce an exact **2:1 monospaced grid** (500 width for half-width ASCII CIDs 1..230, 1000 width for full-width CJK).
  - Transitioned content stream line drawing to **UTF-16BE Hexadecimal string format (`<HEX> '`)**, eliminating syntax breakage and special character escaping pitfalls.
- **Accurate Display-Width Wrapping & Centering via `unicode-width`**:
  - Upgraded pagination (`paginate_text`) and footer centering (`center_text`) from character counts (`chars().count()`) to character display widths (2 columns for full-width, 1 column for half-width). Source code containing mixed Japanese and ASCII now wraps cleanly at the exact column boundary (default 80 columns).

---

## [0.9.1-beta] - 2026-10-19

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

### Fixed / Improved
- **PDF Export Duplicate File Creation Bug**:
  - Resolved bug in print preview (`p` key) and `:pdf` command where dual PDF files were generated; now reliably outputs a single complete PDF document.
- **Generational Backup DiffLog Naming Convention**:
  - Standardized edit session diff log file names to leading-dot `.<filename>.difflog` and registered in `.gitignore` to prevent repository pollution.
- **Multi-Line Visual Yank & Paste Bug**:
  - Fixed issue where pasting (`p`) after multi-line character visual yank (`v` + `y`) corrupted lines; now cleanly pastes as character/line-wise blocks.
- **Filer `d` Key Conflict Resolution**:
  - Fixed conflict between Vi delete operator `d` and 2-pane Filer delete prompt, ensuring the deletion confirmation modal immediately triggers in Filer.

---

## [0.9.0-beta] - 2026-10-19 (Initial Beta)

### Added
- **Dynamic Line Number & Line Wrap Control Commands**:
  - Implemented `:se nu` / `:se nonu` / `:set nu` / `:set nonu` (line numbering), `:se rnu` / `:se nornu` (relative line numbering).
  - Implemented `:se wrap` / `:se nowrap` / `:set wrap` / `:set nowrap` (line wrapping at terminal edge vs. default horizontal scrolling).
  - Implemented configuration query commands (`:se nu?`, `:se wrap?`, `:se rnu?`, etc.).
  - Added and persisted `wrap = false` option under `[editor]` in `config.toml`.
- **Tab Key Input in Insert & Replace Modes**:
  - Handled `KeyCode::Tab` in `handle_insert_mode` and `handle_replace_mode` to reliably insert and replace tab character `\t`.

### Fixed / Improved
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
