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
  - Implemented forward/backward search (`/`, `?`, `n`, `N`, `*`, `#`) with wraparound.
  - Implemented ex command executor (`:w`, `:q`, `:s`, `:%s/before/after/g` regex substitution, `:line` jump).
  - Stateful `CommandParser` (accumulating counts, register prefixes, 2-key commands in real time).
- **`vivivi-modes`**:
  - Defined Inline CSV/TSV mode specifications based on ViVi 4.
  - Ported Unicode box-drawing lookup table from OpenViVi's `keisen.cpp`.
  - Implemented keisen drawing/erasing/joining step logic (`step`) with automatic line expansion and perpendicular checking.
  - Implemented box-internal text alignment algorithms (Left, Center, Right).
  - Implemented inline CSV formatting (`format_line`) with quote hiding and active-line raw display.
  - Implemented raw buffer cell range resolution (`cell_range_in_raw_line`).
  - Implemented syntax highlighting tokenizer (`syntax.rs`) supporting Rust, C/C++, Python, JS/TS, JSON, and Markdown keywords, types, strings, numbers, and comments.
- **`vivivi-app`**:
  - Implemented full-screen Terminal UI (TUI) frontend powered by `ratatui` (v0.29) and `crossterm` (v0.28).
  - Terminal lifecycle management with Alternate Screen, Raw mode, and panic recovery hooks.
  - Rendered editor buffer with line numbers, cursor row highlighting, scrolling, and inline CSV alignment view.
  - Rendered real-time syntax highlighting and matching parenthesis highlighting.
  - Real-time status bar displaying Vi mode, keisen/CSV badges, filename, modified flag, encoding/newline/language badges, line/col, and percentage.
  - Preserved original charset and newline sequence upon file save.
  - Interactive command-line UI for ex commands (`:w`, `:q`, `:wq`, `:align`, `:%s`).
  - Native hardware blinking cursor integration.
  - Supported `--demo` console demo mode flag.
- **Agent & Development Infrastructure**:
  - Setup `.agents/` rules and skills, `GEMINI.md`, and `RULES.md`.
  - Established bilingual documentation system (`docs/ja/`, `docs/en/`).
