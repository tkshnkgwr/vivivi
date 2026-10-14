# vivivi: System Architecture Design (ARCHITECTURE)

## 1. Project Purpose
This project is a modern, memory-safe, and high-performance reconstruction in **Rust** of the classic text editor "OpenViVi" (originally implemented in C++ / Qt).
Furthermore, advanced features from the commercial editions (ViVi 3 and ViVi 4), such as inline CSV/TSV mode and box-drawing (keisen) mode, are faithfully ported and integrated.

---

## 2. Reference Resources
- **OpenViVi Source Code**: `%USERPROFILE%\Documents\参考元アプリ\openViVi`
  - Text buffer algorithms (`vivi/buffer/`)
  - Vi command engine (`vivi/vivi64/ViEngine.*`, `vivi/vivi64/TestViCommands.*`)
  - Box-drawing implementation (`vivi/vivi64/keisen.cpp`)
- **Commercial Editions (ViVi 3 & 4)**:
  - ViVi 3: `%USERPROFILE%\Documents\個人ツール\vivi3`
  - ViVi 4: `%USERPROFILE%\Documents\個人ツール\vivi4`
  - Help documentation (`vivihelp.CHM`): Decompiled into `docs/references/vivi4_help/` for specification lookups.

---

## 3. Multi-Crate Architecture (Cargo Workspace)
The editor core is decoupled from frontends (CLI, TUI, or future GUI) to enable high unit test coverage and modularity.

```mermaid
graph TD
    App["vivivi-app (CLI / TUI Entrypoint)"]
    Core["vivivi-core (GapBuffer, UndoMgr, Cursor, Encodings)"]
    Vi["vivivi-vi (Vi State Machine, Command Parser, Registers, Ex)"]
    Modes["vivivi-modes (Inline CSV, Keisen Box Drawing)"]

    App --> Vi
    App --> Modes
    App --> Core
    Vi --> Core
    Modes --> Core
```

| Crate Name     | Responsibilities & Roles                                                                                                     | Dependencies  |
| :------------- | :--------------------------------------------------------------------------------------------------------------------------- | :------------ |
| `vivivi-core`  | GapBuffer, line manager, cursor, transactional Undo/Redo (`UndoMgr`), encoding & newline conversion, bracket search         | None          |
| `vivivi-vi`    | Vi state machine, command parser, register history, motions, visual block, ex commands, custom vi extensions (fFtT, tags)  | `vivivi-core` |
| `vivivi-modes` | Signature editing assists: keisen protection, inline CSV, expr subst, fFtT, align, renumber, conv, calc, macros, migemo    | `vivivi-core` |
| `vivivi-app`   | Ratatui TUI frontend, multi-window splits, keyboard macro engine, clipboard history modal, AI command slots, entrypoint    | All crates    |
