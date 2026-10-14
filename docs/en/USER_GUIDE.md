# vivivi: User Guide (USER_GUIDE)

This document provides a comprehensive operational guide for `vivivi`, covering basic editing, multi-window navigation, signature ViVi assists, macros, and AI integration.

---

## 1. Basic Operations & Modes

| Mode Name      | Switch Key           | Responsibilities & Actions                                   |
| :------------- | :------------------- | :----------------------------------------------------------- |
| **Normal**     | `Escape`             | Navigation, deletion, yanking, and command entry             |
| **Insert**     | `i`, `a`, `o`, `O`   | Regular typing (continuous typing merged into single undo)   |
| **Visual**     | `v` / `V` / `Ctrl-v` | Character-wise / Line-wise / Rectangular block selection     |
| **Command**    | `:`                  | Save (`:w`), quit (`:q`), substitute (`:s`), ex commands     |

---

## 2. Advanced Modes & Navigation

### (1) Inline CSV/TSV Mode
- Dynamically calculates maximum column widths to align comma- or tab-separated tables inline.
- Enclosing quotes (`"`, `'`) are transparently hidden during viewing, revealing raw text only on the active cursor line for editing.

### (2) Box-Drawing (Keisen) & Protected Edit
- **Toggle Mode**: `F5` or `Shift-F5`
- **Draw Borders**: `Ctrl + Arrow keys` (automatic orthogonal junction joining).
- **Erase Borders**: `Shift + Arrow keys`.
- **Protected Edit**: Typing inside boxes automatically contracts trailing spaces to preserve the right border. Backspacing expands spaces to maintain box dimensions.
- **Box Text Alignment**: `:align left`, `:align center`, `:align right`.

### (3) Multi-Window Splits
- **Horizontal Split**: `Ctrl-w s` or `:split` (`:sp`)
- **Vertical Split**: `Ctrl-w v` or `:vsplit` (`:vs`)
- **Cycle Focus**: `Ctrl-w w` (or `Ctrl-w` followed by `h`/`j`/`k`/`l`)
- **Close Window**: `Ctrl-w c` or `:close`
- **Maximize Active Window**: `Ctrl-w o` or `:only`

### (4) Symbol Outline Pane (`F8`, `:outline`)
- Press `F8` or run `:outline` to toggle a 30-column code outline tree on the left pane.
- Extracts Markdown headers (`#`), Rust symbols (`fn`, `struct`, `impl`), C/C++, and Python functions.
- Navigate with `j`/`k` and press `Enter` to jump directly to the declaration line.

### (5) Romaji Japanese Search (Migemo / `g/`)
- In Normal mode, type `g/` followed by Romanized text (e.g. `g/kensaku`) to search directly for Japanese Kanji/Kana matches without toggling your IME.
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

- Press `F1` or execute `:help [topic]` (e.g. `:help keisen`, `:help calc`) to read built-in manuals.
- Scroll with `j`/`k` and dismiss with `q` or `Esc`.
