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
- [ ] Desktop GUI (egui / eframe) frontend implementation (rich graphic borders & font rendering).
