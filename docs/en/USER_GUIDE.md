# vivivi: User Guide (USER_GUIDE)

This document provides a comprehensive operational guide for `vivivi`, covering basic editing, multi-window navigation, signature ViVi assists, macros, and AI integration.

---

## 1. Basic Operations & Modes

| Mode Name      | Switch Key           |  Cursor Shape  | Responsibilities & Actions                                   |
| :------------- | :------------------- | :------------: | :----------------------------------------------------------- |
| **Normal**     | `Escape`             |   Block (█)    | Navigation, deletion, yanking, and command entry             |
| **Insert**     | `i`, `a`, `o`, `O`   |    Bar (|)     | Regular typing (continuous typing merged into single undo)   |
| **Replace**    | `R`                  | Underline (_)  | In-place character overwrite (returns to Normal on `Esc`)    |
| **Visual**     | `v` / `V` / `Ctrl-v` |   Block (█)    | Character-wise / Line-wise / Rectangular block selection     |
| **Command**    | `:`                  |   Block (█)    | Save (`:w`), quit (`:q`), substitute (`:s`), ex commands     |

> [!NOTE]
> **Dynamic Terminal Cursor Synchronization**: Following Vim/Gvim tradition, the native terminal cursor shape automatically changes (Block, Bar, or Underline) based on the active editing mode. It reliably resets to your terminal's default shape upon exit or abnormal termination.

> [!TIP]
> **Horizontal Scrolling for Long Lines**: For lines exceeding screen width (e.g. > 80 columns), the viewport automatically and smoothly scrolls horizontally as the cursor moves or jumps to line ends (`$`). It computes true visual widths (Visual Column via `UnicodeWidthChar`) for multi-byte characters and tabs, clipping text cleanly without visual artifacts.

---

## 2. Advanced Modes & Navigation

### (1) Inline CSV/TSV Mode
- Dynamically calculates maximum column widths to align comma- or tab-separated tables inline.
- Enclosing quotes (`"`, `'`) are transparently hidden during viewing, revealing raw text only on the active cursor line for editing.

### (2) Box-Drawing (Keisen) & Protected Edit
- **Toggle Mode**: `F5` or `:keisen`
- **Draw Borders**: `Ctrl + Arrow keys` or `Alt + Arrow keys`
  - Overhauled corner-joining algorithm eliminates dangling stubs, automatically forming clean, geometric corners (`┌`, `┐`, `└`, `┘`).
  - Drawing downwards past the buffer end automatically expands lines with newlines and padding.
- **Erase Borders**: `Shift + Arrow keys` (re-evaluates and repairs adjacent junctions).
- **Normal Cursor Navigation**: When Keisen mode is active, bare arrow keys and `h`/`j`/`k`/`l` move the cursor normally without drawing.
- **Protected Edit & Deletion**:
  - Typing inside boxes automatically contracts trailing spaces to preserve the right border (accounting for 1-column ASCII and 2-column wide characters).
  - Deleting text inside boxes—via `Backspace`/`Delete` in Insert mode or `x`/`X` in Normal mode—automatically inserts space before the right border to maintain dimensions.
- **Box Text Alignment**: `:align left`, `:align center`, `:align right`.

### (3) Multi-Window Splits (Nested Tree)
- **Horizontal Split**: `Ctrl-w s` or `:split` (`:sp`)
- **Vertical Split**: `Ctrl-w v` or `:vsplit` (`:vs`)
- **Vim-Compatible Nested Tree**: Window layouts are managed via a binary tree (`WindowTreeNode`). Splitting vertically after a horizontal split only divides the active pane rather than turning the whole screen into 3 columns.
- **Independent Scrolling & Cursor**: Each pane maintains its own independent scroll offsets (vertical & horizontal) and cursor coordinates without unwanted sync.
- **Geometric Focus Navigation**: Move focus via `Ctrl-w h/j/k/l` (Left, Down, Up, Right) based on true screen coordinates, or `Ctrl-w w` to cycle.
- **Close Window**: `Ctrl-w c` or `:close` (`:q`)
- **Maximize Active Window**: `Ctrl-w o` or `:only`

### (4) Symbol Outline Pane (`F2`, `F8`, `:outline`)
- Press `F2`, `F8`, or run `:outline` to toggle a 30-column code outline tree on the left pane.
- Focus automatically moves to the outline tree upon opening. Navigate with `j`/`k` and press `Enter` to jump immediately to the target line in the editor.
- Press `Tab` or `Esc` to return focus smoothly to the editor while keeping the outline pane open.
- Extracts Markdown headers (`#`), Rust symbols (`fn`, `struct`, `impl`), C/C++, and Python functions.

### (5) Search, Highlighting (`hlsearch`) & Romaji Search (Migemo / `g/`)
- **Romaji Search (Migemo)**:
  - In Normal mode, type `g/` (or run `:migemo <query>`) followed by Romanized text (e.g. `g/kensaku`) to search directly for Japanese Kanji/Kana matches without toggling your IME.
  - **Built-in SKK Dictionary**: Standard common Japanese vocabulary is embedded in the binary; works out of the box with zero external configuration.
  - The viewport automatically scrolls to bring matched lines into view.
- **Standard Search**: `/pattern` (forward), `?pattern` (backward), `*`/`#` (exact word match under cursor).
- **Accurate Head Landing**: Cursor always positions precisely on the very first character of the matched token.
- **Real-Time Highlighting (`hlsearch`)**: All in-viewport match occurrences are highlighted in bold black on yellow-orange.
- **Clearing Highlights**: Run `:noh`, `:nohl`, `:nohlsearch`, or double-press `Esc` (`Esc` twice in quick succession) in Normal mode to dismiss highlights.
- Navigate matches with `n` and `N`.

### (6) Clipboard History Dialog (`F6`, `:cbhist`)
- Press `F6` or execute `:cbhist` to open an interactive history list of past yanks and copies.
- Browse with `j`/`k` and press `Enter` to paste into the cursor position.

### (7) Batch Character Conversion (`conv`)
- Full-width $\to$ Half-width: `:hankaku`
- Half-width $\to$ Full-width: `:zenkaku`
- Hiragana $\to$ Katakana: `:katakana`
- Katakana $\to$ Hiragana: `:hiragana`
- Half-width Kana $\to$ Full-width: `:zenkata`
- Full-width Kana $\to$ Half-width: `:hankata`
- Case Conversion: `:toupper` / `:tolower`

### (8) Box Selection Math (`calc`)
- Select numbers in a visual block (`Ctrl-v`) and run `:calc` or `:sum` / `:total`.
- Automatically calculates Count, Sum, Average, and Standard Deviation, copying metrics to the clipboard.

### (9) ViVi-Exclusive Vi Command Extensions & Navigation
- **fFtT Full-width Character Matching**: Type ASCII alphanumeric or symbols in `f`, `F`, `t`, `T` to transparently match both ASCII and full-width Japanese characters (digits, brackets `（）`, punctuation `：！`). Repeat with `;` (forward) and `,` (reverse).
- **Word Search Under Cursor & Matching**:
  - `*` / `#`: Exact word match (`\bword\b`) forward / backward.
  - `g*` / `g#`: Partial word match without word boundaries forward / backward.
  - `g<` / `g=`: Prefix / suffix word match forward.
- **Numeric & Alpha Increment/Decrement & Toggle**:
  - `#+` / `##`: Increment number (`+1`) or advance letter (`a` $\to$ `b`).
  - `#-`: Decrement number (`-1`) or reverse letter (`b` $\to$ `a`).
  - `#!`: Toggle boolean identifier (`true` ⇄ `false`, `yes` ⇄ `no`, `on` ⇄ `off`).
- **Last Edit Coordinate Jump & Text Duplication**:
  - `@`: Instantly jump back to the exact coordinate of the last text modification.
  - `'@` / `` `@ ``: Jump to the line where the last edit occurred.
  - `=<char>`: Replicate characters from the line above up to `<char>`.
- **Marks**:
  - `ma` - `mz`: Set local buffer mark.
  - `mA` - `mZ`: Set global workspace mark preserving file path.
  - `'a` / `` `a ``: Jump to mark coordinate.
  - `:marks`: View list of active marks.
- **Tag Jump**:
  - `Ctrl-]` / `:tag <name>`: Jump to declaration/definition via `tags` file or AST outline analysis.
  - `Ctrl-t` / `Ctrl-[`: Pop tag stack and return to previous location.
  - `:tags`: Display list of tag jump history stack.

---

## 3. Signature ViVi Editing Assists & Macros

### (1) Text Alignment (`:align <target>`)
- **Commands**: `:align =`, `:align :`, `:align ,`, `:align //`
- Aligns multiple lines or visual selection vertically at the target separator character.

### (2) Renumbering (`:renumber [start] [step]`)
- **Commands**: `:renumber [start] [step]` or `:renum`
- Automatically renumbers numbered lists (`1.`, `2)`, `(1)`, `[1]`) with customizable starting index and step increment while preserving indent levels.

### (3) Batch Head/Tail String Insertion (`:insbeg`, `:insend`, `:begend`)
- **Insert at Head**: `:insbeg // ` (prepends string after indent, skipping blank lines)
- **Insert at Tail**: `:insend ;` (appends string to line ends)
- **Enclose Lines**: `:begend <li> </li>` (encloses lines in matching tags/brackets)

### (4) Dynamic Word Completion (dabbrev / `Ctrl-l`, `Ctrl-m`)
- In Insert mode, type a prefix and press `Ctrl-l` to cycle through in-buffer word candidates forward (`Ctrl-m` backward).

### (5) Static Abbreviation Expansion (abbrev / `Ctrl-;`, `Alt-;`, `F4`)
- In Insert mode, type a keyword (`fn`, `struct`, `hw`) and press `Ctrl-;` (or `Alt-;` / `F4`) to expand boilerplate snippets.
- Register custom keywords on the fly: `:abbrev <key> <value>`.

### (6) Dynamic Macro (Masui Repeat / `Alt-m`, `Ctrl-Alt-m`)
- Requires zero prior recording setup.
- Repeat any edit twice, then press `Alt-m`: the editor automatically infers recurring keystroke patterns (XX) and re-executes the edit.

### (7) Lexical Edit Prediction (LexEdit / `Ctrl-j`)
- Tracks word substitutions and renames.
- Press `Ctrl-j` on a matching variable to infer and apply the next edit automatically.

### (8) Zen-Coding (Emmet) Expansion (`Ctrl-,`, `Alt-,`, `:zen`)
- In Insert mode, type HTML abbreviations (`ul#nav>li*3>a`) and press `Ctrl-,` to expand into structured HTML tags.

### (9) Keyboard Macro Record & Playback (`Ctrl-Alt-r`, `Ctrl-Alt-p`, `F9`, `F10`)
- **Record**: Toggle recording with `F9` or `Ctrl-Alt-r`.
- **Replay**: Replay recorded sequence with `F10` or `Ctrl-Alt-p`.
- Fully compatible with standard Vi register macros (`qa` ... `q`, `@a`).

### (10) Expression Substitute (`:s/pattern/expression/e`, `:%s/pattern/expression/e`)
- **Command**: `:s/pattern/expression/e` or `:%s/pattern/expression/e`
- **Description**: Dynamically evaluates the replacement expression as arithmetic or script operations.
- **Examples**:
  - **Calculate Tax / Markup (+10%)**:
    `:%s/price: (\d+)/"price: " + int($1 * 1.1)/e`
  - **Sequential Renumbering**:
    `:%s/item/"item_" + $i/e` (replaces with `item_1`, `item_2`, ...)
  - **Line Number Prefixing**:
    `:%s/^/"Line " + $line + ": "/e`

---

## 4. AI Integration (`:ai`)

- **Natural Language Transforms**: `:ai <instruction>` (refactors selected code or active block).
- **Commit Message Generation**: `:ai commit` (synthesizes conventional commit messages from `git diff`).
- **Architecture**: See [`docs/ja/AI_INTEGRATION_DESIGN.md`](../ja/AI_INTEGRATION_DESIGN.md).

---

## 5. Help Viewer (`F1`, `:help`)

- Press `F1` or execute `:help [topic]` (e.g. `:help keisen`, `:help calc`, `:help tools`) to read built-in manuals.
- Scroll with `j`/`k` and dismiss with `q` or `Esc`.

---

## 6. Tools & File Management User Guide (ViVi 3 / ViVi 4 Compatible)

### (1) Persistent Undo
- **Automatic Behavior**: Undo history is automatically saved to `.<filename>.undofile` upon write (`:w`) and restored when reopened, allowing edits to be undone across editor sessions.
- **Manual Commands**:
  - `:undofile save [path]`: Manually save current undo history.
  - `:undofile load [path]`: Manually restore undo history from file.

### (2) Differential Revision Backup & Lightweight Blame (DiffLog)
- **Automatic Behavior**: Every overwrite save appends a Unified Diff and timestamp to `.<filename>.difflog`.
- **Revision History**: Run `:difflog` to view timestamps and line delta counts for all saved revisions.
- **Past Revision Restore**: Run `:difflog restore <generation>` to restore buffer content to any previous revision.
- **Lightweight Blame**: Run `:blame` to inspect which revision/timestamp each line originated from.

### (3) Template Auto-Loading
- When opening a new non-existent file, automatically searches for `.template.<ext>` in the target and parent directories.
- Automatically expands macros like `%FileName%`, `%FileBase%`, `%Ext%`, `%Date%`, `%Time%`, `%UserName%`, and `%Cursor%`.

### (4) In-Code ToDo Extractor (`:todo`, `:undone`)
- **Commands**: `:todo [dir]` or `:undone [dir]`
- Scans current buffer or directory for tags like `TODO`, `FIXME`, `BUG`, `UNDONE`, `HACK`, `NOTE`, and `XXX`.
- Lists filenames, line numbers, and comments for quick navigation.

### (5) Code Folding (`z` commands)
Folds code blocks based on indentation hierarchy.
- **Keybindings**:
  - `zc`: Close fold under cursor.
  - `zo`: Open fold under cursor.
  - `za`: Toggle fold open/close.
  - `zM`: Close all folds across entire buffer.
  - `zR`: Open all folds across entire buffer.
  - `z<lvl>`: Fold blocks deeper than specified level (e.g., `z2`).
- **Ex Command**: `:fold <toggle|close|open|all|none>`

### (6) Fast Project Grep Search (`:grep`)
- **Command**: `:grep <pattern> [dir]`
- Recursively searches project text files (skipping `.git`, `target`, node modules, and binaries).
- **Navigation**:
  - `:cnext` or `:cn`: Jump to next match.
  - `:cprev` or `:cp`: Jump to previous match.

### (7) 2-Pane Directory Browser (Filer / `:filer`)
- **Command**: `:filer [path]`
- Dual independent directory panes.
- **Controls**:
  - `Tab`: Switch focus between left and right panes.
  - `j` / `k`: Move cursor.
  - `Enter`: Navigate directory (or `..` to go up) / open selected file.
  - `Space`: Multi-select items.
  - `c`: Copy selected items to opposite pane directory.
  - `m`: Move selected items to opposite pane directory.
  - `d`: Delete selected items.

### (8) Project Management & Batch Tags (`:maketags`)
- **Command**: `:maketags [dir]`
- Scans workspace files and generates ctags-compatible `tags` files for instant symbol navigation with `Ctrl-]`.

### (9) Print Preview & Standalone PDF Export (`:preview`, `:pdf`)
- **Print Preview**: Run `:preview` or `:print` for a full-screen paginated A4 preview modal with line numbers, headers, and footers. Scroll with `j`/`k` or `d`/`u`, and exit with `Esc` or `q`.
- **Direct PDF Export**: Run `:pdf [path]` to export standard PDF 1.4 documents directly to disk without any external tools or libraries.
- *(Note: Desktop GUI (egui) and GUI-dependent features were migrated to Phase 5 per user instruction, focusing current capabilities on TUI and core tooling)*.
