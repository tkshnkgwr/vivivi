# vivivi: Testing Strategy & Guide (TESTING)

This document defines testing strategies, unit test coverage, and verification guidelines for `vivivi`.

---

## 1. Testing Strategy

The editor core (buffer, undo, vi state machine, special modes) is decoupled from the UI, enabling comprehensive unit testing in headless environments.

### Core Testing Areas
1. **Buffer Operations (`vivivi-core`)**:
   - Gap movement, boundary insertion/deletion, dynamic reallocation, iterators.
2. **Undo/Redo (`vivivi-core`)**:
   - Typing merge, newline boundary splitting, `prohibit_merge`, transactional block rollback.
3. **Vi Engine (`vivivi-vi`)**:
   - Mode transitions, key parsing, register management.
4. **Special Modes (`vivivi-modes`)**:
   - Unicode box-drawing lookup table correctness, CSV column width calculation.

---

## 2. Test Commands

```powershell
# 1. Run all workspace tests
cargo test

# 2. Run specific crate tests
cargo test -p vivivi-core
cargo test -p vivivi-modes
cargo test -p vivivi-vi

# 3. Verbose test output
cargo test -- --nocapture
```
