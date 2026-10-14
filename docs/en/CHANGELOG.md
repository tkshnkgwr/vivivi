# vivivi: Changelog (CHANGELOG)

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

---

## [Unreleased]

### Added
- **`vivivi-core`**:
  - Implemented `GapBuffer<T>` ported from OpenViVi ($O(1)$ local edits, dynamic resizing, gap movement).
  - Implemented transactional `UndoMgr` (nested undo blocks, continuous typing merge, save point tracking).
  - Implemented `TextBuffer` integrating `GapBuffer` and `UndoMgr`.
  - Implemented character encoding and newline conversion module (`encoding.rs`) with BOM detection, UTF-8, Shift_JIS/CP932, EUC-JP, UTF-16LE/BE, and CRLF/LF/CR normalization.
  - Implemented matching parentheses detection module (`paren.rs`) for `()`, `{}`, `[]`, `<>` with comment/string skipping and cursor pair detection.
- **`vivivi-vi`**:
  - Defined Vi modes (Normal, Insert, Replace, Visual, VisualLine, VisualBlock, CommandLine).
  - Defined basic commands, motions, operators, and registers.
  - Implemented cursor motions (`h`, `j`, `k`, `l`, `w`, `b`, `e`, `W`, `B`, `E`, `0`, `^`, `$`, `gg`, `G`, and `%` matching paren jump).
  - Implemented operator execution engine (`d`, `y`, `c` + motion / `dd`, `yy`, `cc`, `D`, `C`, `Y`, `x`, `X`).
  - Implemented register management system (unnamed register, `"a` - `"z`, uppercase append, line/char paste `p`, `P`).
  - **Full Visual Block (BOX Selection) Support (`VisualBlock` / `Ctrl-v`)**:
    - Block selection, block yank (`y`), block delete (`d`, `x`), and `RegisterContent::BlockWise` storage.
    - Block multi-line insert (`I`, `A`): Enters insert mode at column boundary and batch-applies input across selected lines upon `Esc`.
    - Smart block paste (`p`, `P`): Automatic space padding for shorter lines during block pasting.
    - Visual selection case inversion (`~`).
  - Implemented forward/backward search (`/`, `?`, `n`, `N`, `*`, `#`) with wraparound.
  - Implemented ex command executor (`:w`, `:q`, `:s`, `:%s/before/after/g` regex substitution, `:line` jump).
  - Stateful `CommandParser` (accumulating counts, register prefixes, 2-key commands in real time).
  - **ViVi Vi Command Extensions & Navigation**:
    - `fFtT` full-width auto-matching (`f`, `F`, `t`, `T`, `;`, `,`) and operator integration (`df)`, `ct]`).
    - Word search under cursor (`*`, `#` exact match, `g*`, `g#` partial match, `g<` prefix match, `g=` suffix match).
    - Numeric and alphabet increment/decrement (`#+`, `##`, `#-`) and boolean toggle (`#!`).
    - Last edit location jump (`@`, `'@`, `` `@ ``) and above-line text replication (`=<char>`).
    - Local and global marks (`ma`-`mz`, `mA`-`mZ`, jump `'a`, list `:marks`).
    - Tag navigation (`Ctrl-]`, `Ctrl-t`, `Ctrl-[`, `:tag`, `:tags` with AST syntax analysis fallback).
- **`vivivi-modes`**:
  - **Expression Substitution Engine (`expr_subst.rs`)**:
    - Dynamic arithmetic/script evaluation for `:s/pattern/expression/e` (`+`, `-`, `*`, `/`, `%`, `$0`-`$9` capture groups, `$i`/`$n` counter, `$line`, string concatenation).
  - **Full-width Character Matching Mapping (`fftt.rs`)**:
    - Transparent mapping and forward/backward search matching ASCII characters to full-width variants.
  - Defined Inline CSV/TSV mode specifications based on ViVi 4.
  - Ported Unicode box-drawing lookup table from OpenViVi's `keisen.cpp`.
  - Implemented keisen drawing/erasing/joining step logic (`step`) with automatic line expansion and perpendicular checking.
  - Implemented box-internal text alignment algorithms (Left, Center, Right).
  - Implemented inline CSV formatting (`format_line`) with quote hiding and active-line raw display.
  - Implemented raw buffer cell range resolution (`cell_range_in_raw_line`).
  - Implemented syntax highlighting tokenizer (`syntax.rs`) supporting Rust, C/C++, Python, JS/TS, JSON, and Markdown keywords, types, strings, numbers, and comments.
  - **Character Conversion Module (`conv.rs`)**:
    - Batch conversions: Full-width ⇄ Half-width alphanumeric, Hiragana ⇄ Katakana, Half-width Katakana ⇄ Full-width Katakana (with voiced sound mark synthesis/decomposition), Uppercase ⇄ Lowercase, and SwapCase.
  - **Outline Analysis Module (`outline.rs`)**:
    - Symbol hierarchy extraction for Markdown (`#` - `######`), Rust (`fn`, `struct`, `impl`, `enum`, `trait`, `mod`), C/C++, and Python.
- **`vivivi-app`**:
  - Implemented full-screen Terminal UI (TUI) frontend powered by `ratatui` (v0.29) and `crossterm` (v0.28).
  - Terminal lifecycle management with Alternate Screen, Raw mode, and panic recovery hooks.
  - Rendered editor buffer with line numbers, cursor row highlighting, scrolling, and inline CSV alignment view.
  - Rendered real-time syntax highlighting and matching parenthesis highlighting.
  - **Split & Multi-Window Views (Split / Vsplit)**:
    - Horizontal split (`:split` / `:sp`), vertical split (`:vsplit` / `:vs`), window close (`:close` / `:c`), and single-window isolation (`:only`).
    - Gvim-compatible `Ctrl-w` window keybindings (`Ctrl-w w`, `Ctrl-w s`, `Ctrl-w v`, `Ctrl-w c`, `Ctrl-w o`, `Ctrl-w h/j/k/l`).
    - Multi-window layout rendering with active window borders and titles.
  - **Outline Side Pane View**:
    - Toggle outline display via `F8` key or `:outline` ex command.
    - Side pane navigation with `Ctrl-o` focus, `j`/`k` item navigation, and `Enter` instant jump to target line.
  - **Character Conversion Ex Commands**:
    - Batch whole-buffer conversion via `:%zenkaku`, `:%hankaku`, `:%hiragana`, `:%katakana`, `:%hankata`, `:%zenkata`, `:%toupper`, `:%tolower`.
  - **Text Alignment, Renumbering & Line Editing Assists**:
    - Multi-line columnar alignment via `:align <target>` (`=`, `:`, `,`, `//`).
    - Ordered list renumbering via `:renumber [start] [step]` / `:renum`.
    - Batch head/tail insertion via `:insbeg`, `:insend`, and `:begend`.
  - **High-Speed Input & Macro Assistance**:
    - In-buffer cyclic dynamic word completion (dabbrev / `Ctrl-l`, `Ctrl-m`).
    - Static abbreviation expansion (abbrev / `Ctrl-;`, `Alt-;`, `F4`, `:abbrev`).
    - Dynamic macro (Masui Repeat / `Alt-m`, `Ctrl-Alt-m`, `:dmacro`) recurring pattern detection and replay.
    - Lexical edit prediction (LexEdit / `Ctrl-j`, `:lexedit`) based on edit history.
    - Zen-Coding (Emmet) expansion (`Ctrl-,`, `Alt-,`, `:zen`) for HTML shorthand.
    - Full keyboard macro recording and replay (`Ctrl-Alt-r` / `Ctrl-Alt-p`, `F9` / `F10`, `:macro`).
  - **Advanced Navigation, Search & Math**:
    - Romaji Japanese incremental search (Migemo / `g/`) with SKK dictionary support.
    - Box selection numeric calculation (`:calc`, `:sum`, `:total`) for count, sum, avg, stddev.
    - Interactive clipboard history modal (`F6`, `:cbhist`).
    - Built-in documentation viewer (`F1`, `:help [topic]`) and newly added topics (`expr_subst.md`, `vivi_cmd.md`).
    - Expression substitution (`:s/.../.../e`, `:%s/.../.../e`) integrated into command-line UI.
    - Full-width character matching, word searching, incrementing, marks, and tag navigation integrated into TUI.
  - **AI Integration Architecture**:
    - Interactive command dispatch slots for `:ai <instruction>` and `:ai commit`.
    - Published zero-latency AI architecture specification (`docs/ja/AI_INTEGRATION_DESIGN.md`).
  - Real-time Visual / VisualLine / VisualBlock selection background highlighting.
  - Real-time status bar displaying Vi mode, keisen/CSV badges, filename, modified flag, encoding/newline/language badges, line/col, and percentage.
  - Preserved original charset and newline sequence upon file save.
  - Interactive command-line UI for ex commands (`:w`, `:q`, `:wq`, `:align`, `:%s`).
  - Native hardware blinking cursor integration.
  - Supported `--demo` console demo mode flag.
- **Repository, Licensing & Infrastructure**:
  - Placed LICENSE (MIT License) in repository root.
  - Setup `.agents/` rules and skills, `GEMINI.md`, and `RULES.md`.
  - Established bilingual documentation system (`docs/ja/`, `docs/en/`).

### Changed / Refactored
- **Code Modularization & Source Decomposition Refactoring**:
  - Decomposed oversized `vivivi-vi`'s `engine.rs` (1,441 lines) into focused submodules under `engine/` (`motion.rs`, `operate.rs`, `visual.rs`, `ex.rs`, `mod.rs`, `tests.rs`), keeping all files between 150-350 lines.
  - Decomposed `vivivi-app`'s `tui/ui.rs` (514 lines) into structured modules under `ui/` (`editor.rs`, `outline.rs`, `status.rs`, `mod.rs`), improving extensibility for future Grep panes and DIFF views.
  - Extracted editor window state (`SplitAxis`, `EditorWindow`) from `app.rs` into `tui/window.rs`.
  - All source files across the entire workspace now strictly remain well under 500 lines, dramatically improving readability and maintainability.

