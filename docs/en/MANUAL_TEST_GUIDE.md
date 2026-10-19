# vivivi: Manual Acceptance Testing Checklist (MANUAL_TEST_GUIDE)

This document serves as the **Manual Acceptance Testing Checklist** for `vivivi` prior to releases (such as `v0.1.0-alpha`) and major refactoring cycles, guiding developers and testers through hands-on verification of UX, keybindings, and display rendering.

---

## 🚀 Setup & Launch

```powershell
# 1. Build release binary
cargo build --release

# 2. Launch
.\target\release\vivivi.exe
```

---

## 1. Startup, File I/O, and Status Display

| No. | Test Item               | Action / Key Sequence                            | Expected Result                                                  | Checked |
| :-- | :---------------------- | :----------------------------------------------- | :--------------------------------------------------------------- | :-----: |
| 1-1 | Untitled Launch         | Run without arguments (`vivivi.exe`)             | Opens an empty buffer with `[無題]` displayed on the status line |   [ ]   |
| 1-2 | Open Existing File      | Run with file argument (`vivivi.exe Cargo.toml`) | Correctly renders file contents with line numbers and filename   |   [ ]   |
| 1-3 | Encoding Auto-Detection | Open Shift_JIS or EUC-JP text file               | Japanese text renders clearly without encoding artifacts         |   [ ]   |
| 1-4 | Save File (`:w`)        | Edit text and run `:w <filename>`                | Buffer written to specified path; confirmation shown in status   |   [ ]   |
| 1-5 | Save and Quit (`:wq`)   | Edit text and run `:wq`                          | Buffer saved and editor gracefully exits back to the terminal    |   [ ]   |
| 1-6 | Unsaved Quit Guard      | Edit text and run `:q` without saving            | Blocks exit with warning: `E37: 前回の変更が保存されていません`  |   [ ]   |
| 1-7 | Force Quit (`:q!`)      | Run `:q!` on dirty buffer                        | Discards changes and exits immediately                           |   [ ]   |

---

## 2. Vi Basic Operations, Modes, and Cursor Shapes

| No. | Test Item              | Action / Key Sequence                                                   | Expected Result                                                | Checked |
| :-- | :--------------------- | :---------------------------------------------------------------------- | :------------------------------------------------------------- | :-----: |
| 2-1 | Motions                | In Normal mode, press `h`, `j`, `k`, `l`, `w`, `b`, `0`, `$`, `gg`, `G` | Smooth cursor motion without boundary crashes                  |   [ ]   |
| 2-2 | Insert Mode Entry/Exit | Type text after `i`, `a`, `I`, `A`, `o`, `O`, then press `Esc`          | Text inserted accurately; `Esc` returns to Normal mode         |   [ ]   |
| 2-3 | Cursor Shape Switching | Switch between Normal (`Esc`) $\to$ Insert (`i`) $\to$ Replace (`R`)    | Normal: █ (Block) / Insert: ▎ (Bar) / Replace:   (Underline)   |   [ ]   |
| 2-4 | Deletion               | Execute `x`, `dw`, `dd`                                                 | Characters/words/lines deleted and shifted cleanly             |   [ ]   |
| 2-5 | Yank and Put           | Press `yy` to copy line, then `p` or `P` to paste                       | Pastes line directly above or below cursor position            |   [ ]   |
| 2-6 | Undo and Redo          | Edit text, then press `u` to undo and `Ctrl-r` to redo                  | Restores previous states accurately per keystroke/typing burst |   [ ]   |
| 2-7 | Replace Mode (`R`)     | Press `R` in Normal mode, type over existing text                       | Overwrites existing characters; `Esc` exits to Normal mode     |   [ ]   |

---

## 3. Search, Navigation, and Highlighting

| No. | Test Item            | Action / Key Sequence                          | Expected Result                                               | Checked |
| :-- | :------------------- | :--------------------------------------------- | :------------------------------------------------------------ | :-----: |
| 3-1 | Slash Search         | Press `/`, enter search pattern, press `Enter` | Real-time inverted color highlight on all matches in viewport |   [ ]   |
| 3-2 | Search Navigation    | Press `n` (next) and `N` (previous)            | Jumps sequentially between search matches                     |   [ ]   |
| 3-3 | Clear Highlights     | Run `:noh` or `:nohlsearch`                    | Viewport search highlights cleared immediately                |   [ ]   |
| 3-4 | Word Under Cursor    | Press `*` or `g*` over a word                  | Immediately searches word and jumps to next match             |   [ ]   |
| 3-5 | Migemo Romaji Search | Press `g/`, type Romaji query (e.g. `kensaku`) | Incrementally matches and jumps to corresponding Kanji/Kana   |   [ ]   |

---

## 4. OpenViVi / Modern Visual Enhancements

| No. | Test Item                       | Action / Key Sequence                     | Expected Result                                                                                                                 | Checked |
| :-- | :------------------------------ | :---------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------ | :-----: |
| 4-1 | Gutter Change Marker (Modified) | Edit a line (insert or delete characters) | Yellow indicator bar (`▌`) appears in the leftmost gutter                                                                       |   [ ]   |
| 4-2 | Gutter Change Marker (Saved)    | Save buffer via `:w` after editing        | Yellow indicator transitions to green bar (`▌`)                                                                                 |   [ ]   |
| 4-3 | Recent Files Dialog (`F7`)      | Press `F7` or run `:recent`               | Popup modal dialog lists recent files; select with `j`/`k` and `Enter`                                                          |   [ ]   |
| 4-4 | Show Invisible Characters       | Run `:set list`                           | Renders full-width space with 2-cell wine-red background highlight (preserving full character width), tab as `▸---`, EOL as `$` |   [ ]   |
| 4-5 | Hide Invisible Characters       | Run `:set nolist`                         | Reverts to standard blank rendering (background highlight removed)                                                              |   [ ]   |

---

## 5. ViVi Signature Editing Power Tools

| No. | Test Item                 | Action / Key Sequence                                 | Expected Result                                                          | Checked |
| :-- | :------------------------ | :---------------------------------------------------- | :----------------------------------------------------------------------- | :-----: |
| 5-1 | Keisen Box-Drawing Mode   | Press `Alt+k` or run `:keisen`, then press arrow keys | Draws line borders along arrow directions                                |   [ ]   |
| 5-2 | Auto Border Joining       | Cross lines or turn corners                           | Automatically transforms intersections into `┌`, `┐`, `┘`, `└`, `┼`, `┬` |   [ ]   |
| 5-3 | In-Box Text Alignment     | Inside box, run `:align center` or `:align left`      | Text centered or left-aligned within box margins                         |   [ ]   |
| 5-4 | Renumbering               | Select ordered list (1., 2.) and run `:renum 1 1`     | Automatically renumbers prefixes starting at 1 with step 1               |   [ ]   |
| 5-5 | Head/Tail Batch Insertion | Run `:insbeg // ` or `:insend ;`                      | Prepends or appends target string across range                           |   [ ]   |
| 5-6 | Character Conversion      | Run `:zenkaku`, `:hankaku`, `:hiragana`, `:katakana`  | Accurate bidirectional character case and script conversion              |   [ ]   |
| 5-7 | Zen-Coding (Emmet)        | Run `:zen ul>li*3`                                    | Expands to `<ul>\n  <li></li>\n  <li></li>\n  <li></li>\n</ul>`          |   [ ]   |
| 5-8 | Dynamic Completion        | Type word prefix and press `Ctrl-L`                   | Completes candidate word from buffer history                             |   [ ]   |

---

## 6. Window Splitting, Tools, and Workspaces

| No. | Test Item                                | Action / Key Sequence                              | Expected Result                                                             | Checked |
| :-- | :--------------------------------------- | :------------------------------------------------- | :-------------------------------------------------------------------------- | :-----: |
| 6-1 | Horizontal Split (`:sp`)                 | Run `:split` or `:sp`, scroll one window           | Splits horizontally; other window retains independent scroll position       |   [ ]   |
| 6-2 | Vertical Split (`:vs`)                   | Run `:vsplit` or `:vs`, scroll one window          | Splits vertically; other window retains independent scroll position         |   [ ]   |
| 6-3 | Close Window (`:close`)                  | In split view, run `:close` or `:c`                | Closes active window and expands remaining window                           |   [ ]   |
| 6-4 | Only Window (`:only`)                    | In multi-split view, run `:only`                   | Closes all other splits except the active window                            |   [ ]   |
| 6-5 | Outline View (`:outline`)                | In Markdown or Rust file, run `:outline`           | Sidebar tree lists headings/symbols for fast jumping                        |   [ ]   |
| 6-6 | Print Preview & PDF (`:preview`, `:pdf`) | Run `:preview` then `p`, or run `:pdf`             | Paginated modal opens; `p` or `:pdf` exports a valid PDF 1.4 file           |   [ ]   |
| 6-7 | 2-Pane Filer (`:filer`)                  | Run `:filer` and use `Tab`/`j`/`k`/`Enter`/`Space` | Dual-pane modal opens; `Tab` switches pane, `Enter` opens/enters, `q` exits |   [ ]   |
| 6-8 | Workspace Grep (`:grep`)                 | Run `:grep fn`                                     | Displays matches list; jump via `:cnext` and `:cprev`                       |   [ ]   |
| 6-9 | Help System (`F1`)                       | Press `F1` or run `:help`                          | Fullscreen modal help viewer displays topic docs (`j`/`k` scroll)           |   [ ]   |

---

## 7. Immediate Feedback Verification Items (Implemented)

| No.  | Test Item                               | Action / Key Sequence                        | Expected Result                                                   | Checked |
| :--- | :-------------------------------------- | :------------------------------------------- | :---------------------------------------------------------------- | :-----: |
| 7-1  | Keisen Protected Fullwidth Insert       | Type fullwidth char (e.g. 'あ') inside box   | Consumes 2-column trailing spaces; right border completely intact |   [ ]   |
| 7-2  | Keisen Protected Fullwidth Delete       | Delete fullwidth char via Backspace / Delete | Restores 2-column padding spaces; box width completely preserved  |   [ ]   |
| 7-3  | Search Highlight Retention (Single Esc) | Press `Esc` once in Normal mode after search | Highlight remains active (guards against accidental dismissal)    |   [ ]   |
| 7-4  | Search Highlight Dismissal (Double Esc) | Double-press `Esc` quickly within 500ms      | Search highlights cleared immediately with status notification    |   [ ]   |
| 7-5  | Nested Split Windows (Vim Behavior)     | Run `:vsplit` inside bottom window of split  | Top remains 1 pane, bottom divides into left/right (Vim layout)   |   [ ]   |
| 7-6  | Directional Window Focus Navigation     | Press `Ctrl-W h/j/k/l` across nested panes   | Seamlessly moves focus to adjacent left/down/up/right window      |   [ ]   |
| 7-7  | Keisen Protection with Normal 'x' Key   | Press `x` in Normal mode inside keisen box   | Character deleted and space padded; box boundary strictly kept    |   [ ]   |
| 7-8  | Outline Navigation (F2 / F8 / Tab)      | Open via `F2` or `F8`; press `j`/`k`/`Tab`   | Immediately selectable; `Tab`/`Esc` toggles editor/outline focus  |   [ ]   |
| 7-9  | Horizontal Scrolling over 80 Columns    | Move to end-of-line `$` on 80+ char line     | View smoothly scrolls horizontally; cursor tracks correctly       |   [ ]   |
| 7-10 | Romaji Japanese (Migemo) Search         | Run `g/` or `:migemo kensaku`                | Built-in dictionary directly matches Kanji candidates like 検索   |   [ ]   |

---

## 8. Phase 4.8 Verification Items (Advanced Power Tools & Hardening)

| No. | Test Item                                            | Action / Key Sequence                                     | Expected Result                                                                   | Checked |
| :-- | :--------------------------------------------------- | :-------------------------------------------------------- | :-------------------------------------------------------------------------------- | :-----: |
| 8-1 | Multi-Tab Management (`:tabnew`, `gt`)               | Create tab via `:tabnew`, switch with `gt`/`gT`           | Top tab bar is rendered; window split layouts remain independent per tab          |   [ ]   |
| 8-2 | Visual Block Column Ops (`Ctrl-v` + `I`/`A`/`c`/`r`) | Select column block via `Ctrl-v`, run `I// <Esc>` or `r*` | Batch inserts, replaces, or fills across all selected lines at column position    |   [ ]   |
| 8-3 | Config Persistence (`config.toml`, `:config`)        | Place settings in `~/.vivivi/config.toml`, run `:config`  | Reloads keymaps, tab width, and theme dynamically into running editor             |   [ ]   |
| 8-4 | Encoding & Line Ending Format (`:e ++enc=`)          | Run `:e ++enc=sjis` or `:set ff=unix`                     | Reopens buffer with specified encoding; converts LF/CRLF line endings on save     |   [ ]   |
| 8-5 | Crash Recovery & Swap Files (`.swp`, `:recover`)     | Terminate unsaved buffer and restart; run `:recover`      | Startup warning warns of existing swap; `:recover` restores unsaved changes       |   [ ]   |
| 8-6 | System Clipboard Sync (`"+y`, `"*p`)                 | Yank with `"+y`, paste in Notepad; `:set cb=unnamed`      | Transparent bidirectional synchronization between OS clipboard and registers      |   [ ]   |
| 8-7 | Syntax Expansion (Go, Ruby, TOML, PS, etc.)          | Open `.go`, `.rb`, `.toml`, `.ps1`, etc.                  | Status bar shows language badge; keywords, types, and comments syntax highlighted |   [ ]   |

---

## 9. Vi Standard Editing Enhancements (Phase 1 & Phase 2 Implemented)

| No.  | Test Item                                            | Action / Key Sequence                                     | Expected Result                                                                   | Checked |
| :--- | :--------------------------------------------------- | :-------------------------------------------------------- | :-------------------------------------------------------------------------------- | :-----: |
| 9-1  | Text Objects Word (diw, daw, ciw)                   | Run `diw`, `daw`, `ciw` on a target word                  | Deletes target word inner/around; ciw transitions to Insert mode                  |   [ ]   |
| 9-2  | Text Objects Quotes (di", da", ci')                 | Run `di"`, `da"`, `ci'` inside target quotes              | Deletes quote content inner only or around quotes                                 |   [ ]   |
| 9-3  | Text Objects Brackets (di(, da(, di{)               | Run `di(`, `da(`, `di{` inside target brackets            | Deletes bracket content inner only or around bracket symbols                      |   [ ]   |
| 9-4  | Visual Selection Text Objects (v + i")              | Press `v` then `i"` or `a"` inside quotes                 | Expands visual selection anchor/pos to target object; allows y and d              |   [ ]   |
| 9-5  | Case Conversion (guw, gU$, guu, gUU, u, U)          | Run `guw`, `gU$`, `guu`, `gUU`, or press `u`/`U` in Visual| Target range or line is cleanly converted to lowercase or uppercase               |   [ ]   |
| 9-6  | Indentation Operators (>>, <<, >, <)                | Run `>>` or `<<` at line start; run `>` or `<` on visual  | Indents or unindents by 4 spaces across target lines                              |   [ ]   |
| 9-7  | Dot Repeat (x then ., ciw then ., >> then .)        | After `x`, `ciw`, or `>>`, press `.` at other lines       | Accurately repeats previous change with identical semantics                       |   [ ]   |
| 9-8  | Screen Scrolling Adjust (zz, zt, zb)                | Press `zz` (center), `zt` (top), or `zb` (bottom)         | Smoothly adjusts vertical screen viewport relative to cursor line                 |   [ ]   |
| 9-9  | Paragraph Jump Motions ({, })                       | Press `{` or `}` in document with blank lines             | Accurately jumps cursor forward or backward across paragraph blank lines          |   [ ]   |
| 9-10 | Multi-Line Visual Yank & Paste (v + y -> p)         | Select multiple lines via `v`, yank with `y`, paste `p`   | Pastes clean multi-line text without line-wise formatting corruption              |   [ ]   |
| 9-11 | PDF Standalone Output Bug Fix (:pdf / preview p)    | Press `p` inside `:preview` or run `:pdf`                 | Outputs only a single clean .pdf document without duplicate files                 |   [ ]   |
| 9-12 | Filer d Key File Deletion Dialog                    | Highlight file inside `:filer` and press `d`              | Triggers file deletion confirmation modal without Vi operator conflict            |   [ ]   |

