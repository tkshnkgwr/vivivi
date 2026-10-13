<div align="center">

# ⚡ vivivi

**Modern, Memory-Safe, and High-Performance Reconstruction of the Classic "OpenViVi" Text Editor in Rust**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-v0.1.0-brightgreen.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/tkshnkgwr/vivivi)

[English](./README.md) | [日本語](./README_JA.md)

</div>

---

## 📌 Overview

`vivivi` is a ground-up reimagining in **Rust** of the classic C++ / Qt text editor "**OpenViVi**", engineered for memory safety, concurrency, and uncompromising performance.

It faithfully ports the core Vi editing mechanics and data structures while integrating unique, high-productivity features from commercial editions (ViVi 3 & 4), such as **Inline CSV/TSV mode** and **Unicode Box-Drawing (Keisen) auto-joining mode**.

---

## ✨ Key Features

- 🚀 **$O(1)$ Amortized Local Edits via GapBuffer**:
  - Implements OpenViVi's `gap_buffer.h` design, maintaining an empty gap at the edit cursor for instantaneous character insertions and deletions.
- 🔄 **Transactional Undo / Redo with Typing Merging**:
  - Automatically merges contiguous typings into single undo steps.
  - Combines compound vi actions (e.g. `dd`, `cw`, `:%s`) into atomic undo blocks.
  - Tracks save points to accurately restore `is_modified == false`.
- ⌨️ **Faithful Vi Command Engine**:
  - State machine supporting Normal, Insert, Replace, Visual, VisualLine, VisualBlock, and CommandLine modes.
  - Full register support (unnamed, named `"a` - `"z`, system clipboard).
- 📊 **Inline CSV/TSV Alignment & Editing**:
  - Formats separated values into clean columnar tables with full CJK wide-character support.
  - Transparently hides quote marks while reading, exposing raw text on the cursor line for precision editing.
- 📐 **Unicode Box-Drawing (Keisen) Auto-Joining**:
  - Seamlessly draws and connects Unicode box characters (U+2500 - U+254B) using `Ctrl + Arrow keys`, automatically resolving corners, T-junctions, and cross intersections.
- 📦 **Decoupled Multi-Crate Architecture (Cargo Workspace)**:
  - Strict separation between editing engine and UI layers, enabling exhaustive headless unit test coverage.

---

## 🏗️ Workspace Architecture

```mermaid
graph TD
    App[vivivi-app (CLI / TUI Entrypoint)]
    Core[vivivi-core (GapBuffer, UndoMgr, Cursor, Encodings)]
    Vi[vivivi-vi (Vi State Machine, Command Parser, Registers, Ex)]
    Modes[vivivi-modes (Inline CSV, Keisen Box Drawing)]

    App --> Vi
    App --> Modes
    App --> Core
    Vi --> Core
    Modes --> Core
```

| Crate Name     | Responsibilities & Roles                                     | Dependencies  |
| :------------- | :----------------------------------------------------------- | :------------ |
| `vivivi-core`  | GapBuffer, line manager, cursor, Undo/Redo, encodings        | None          |
| `vivivi-vi`    | Vi state machine, command parsing, registers, ex commands    | `vivivi-core` |
| `vivivi-modes` | Inline CSV/TSV mode, Unicode box-drawing (keisen) mode       | `vivivi-core` |
| `vivivi-app`   | Application binary and execution entrypoint                  | All crates    |

---

## 🚀 Installation & Build

### Prerequisites
- Rust 1.80+ (Edition 2024 support)
- Cargo

### Local Build
```powershell
# Clone repository
git clone https://github.com/tkshnkgwr/vivivi.git
cd vivivi

# Build workspace
cargo build

# Run application
cargo run

# Build optimized release binary
cargo build --release
```

### Run Tests
```powershell
# Run unit tests across all crates (15 tests passed)
cargo test
```

---

## 📚 Documentation Index

Comprehensive documentation is available in both English and Japanese under the [`docs/`](./docs) directory:

| Document | Japanese (JA) | English (EN) | Description |
| :--- | :--- | :--- | :--- |
| **Architecture** | [ja/ARCHITECTURE.md](./docs/ja/ARCHITECTURE.md) | [en/ARCHITECTURE.md](./docs/en/ARCHITECTURE.md) | System design & multi-crate architecture |
| **Specifications**| [ja/SPECIFICATION.md](./docs/ja/SPECIFICATION.md) | [en/SPECIFICATION.md](./docs/en/SPECIFICATION.md) | Detailed specifications (Buffer, Undo, Vi, CSV, Keisen) |
| **Development** | [ja/DEVELOPMENT.md](./docs/ja/DEVELOPMENT.md) | [en/DEVELOPMENT.md](./docs/en/DEVELOPMENT.md) | Developer workflows & setup |
| **Testing** | [ja/TESTING.md](./docs/ja/TESTING.md) | [en/TESTING.md](./docs/en/TESTING.md) | Test strategy & test suites |
| **Test Report** | [ja/TEST_REPORT.md](./docs/ja/TEST_REPORT.md) | [en/TEST_REPORT.md](./docs/en/TEST_REPORT.md) | Test execution logs |
| **User Guide** | [ja/USER_GUIDE.md](./docs/ja/USER_GUIDE.md) | [en/USER_GUIDE.md](./docs/en/USER_GUIDE.md) | Keybindings & operating manual |
| **Roadmap** | [ja/TODO.md](./docs/ja/TODO.md) | [en/TODO.md](./docs/en/TODO.md) | Upcoming tasks & progress |

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
