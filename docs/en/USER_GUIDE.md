# vivivi: User Guide (USER_GUIDE)

This document explains basic operations, mode switching, and specialized features (CSV mode, Box-Drawing mode) in `vivivi`.

---

## 1. Basic Operations & Modes

| Mode Name      | Switch Key           | Responsibilities & Actions                                   |
| :------------- | :------------------- | :----------------------------------------------------------- |
| **Normal**     | `Escape`             | Navigation, deletion, yanking, and command entry             |
| **Insert**     | `i`, `a`, `o`, `O`   | Regular typing (continuous typing merged for 1-step undo)    |
| **Visual**     | `v` / `V` / `Ctrl-V` | Character-wise / Line-wise / Rectangular block selection     |
| **Command**    | `:`                  | Save (`:w`), quit (`:q`), replace (`:s`)                     |

---

## 2. Advanced Editing Modes

### (1) Inline CSV Mode
- Aligns columns inline based on the maximum cell width in each column.
- Hides enclosing quotes (`"` / `'`) during reading, while revealing raw text on the active cursor line for editing.

### (2) Box-Drawing (Keisen) Mode
- **Toggle Mode**: `Shift-F5`
- **Draw Line**: `Ctrl + Arrow keys` (auto-joins intersecting cross and T-junction characters)
- **Erase Line**: `Shift + Arrow keys`
