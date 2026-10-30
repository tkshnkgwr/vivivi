# vivivi: Settings Management & Configuration GUI Design Specification (SETTINGS_DESIGN.md)

> [!IMPORTANT]
> This document specifies the configuration persistence architecture, standalone external theme file schema, auto-bootstrap behavior upon first run, and the design specification for both GUI and TUI settings modal dialogs across `vivivi`.

---

## 1. Overall Architecture & File Decoupling Design

Rather than storing settings in binary properties or Windows registry like legacy editors, `vivivi` persists human-readable **TOML** configurations.
In particular, **color themes (syntax highlighting, editor background, UI badges)** are strictly decoupled from general editor settings into standalone theme files to facilitate easy addition, customization, sharing, and distribution.

### 1.1 Directory Structure (`%USERPROFILE%\.vivivi\`)

```
%USERPROFILE%\.vivivi\
├── config.toml                # General editor behavior, file I/O, font, keymaps
├── themes\                    # Standalone color themes
│   ├── tokyo-night.toml       # Default dark theme (Tokyo Night)
│   ├── openvivi.toml          # Revived OpenViVi signature dark palette
│   ├── vivi-classic.toml      # ViVi 3/4 classic light/gray theme
│   ├── vivivi-dark.toml       # Modern dark palette
│   └── vivivi-light.toml      # Modern light palette
├── types\                     # (Future) Filetype-specific configs (Rust, Markdown, etc.)
│   ├── rust.toml
│   └── markdown.toml
└── cache\                     # Ephemeral caches (recent files, bookmarks, update check)
    ├── recent.json
    ├── bookmarks.json
    └── update_check.json
```

---

## 2. Auto-Bootstrap on First Launch

Upon launch (TUI and GUI alike), the configuration system performs the following initialization checks:

1. **Config Directory Existence**:
   - Creates `%USERPROFILE%\.vivivi\` (or `~/.vivivi/`, `~/.config/vivivi/` on Unix) if missing via `std::fs::create_dir_all`.
2. **`config.toml` Auto-Generation**:
   - If missing, generates a clean default `config.toml` complete with descriptive comments.
3. **`themes/` Directory & Builtin Theme Deployment**:
   - Creates `themes/` and dumps embedded standard presets (`tokyo-night.toml`, `openvivi.toml`, `vivi-classic.toml`, `vivivi-dark.toml`, `vivivi-light.toml`).
   - Users can drop any custom `<theme>.toml` file here to immediately make it selectable in the editor.

---

## 3. General Configuration Schema (`config.toml`)

Manages editor behaviors, view options, file I/O, fonts, and key mappings across 6 standard categories.

```toml
# vivivi Configuration File
version = "1.0"

# ==========================================
# 1. General
# ==========================================
[general]
default_encoding = "utf-8"     # "utf-8", "sjis", "euc-jp"
default_fileformat = "unix"    # "unix" (LF) or "dos" (CRLF)
clipboard = "unnamedplus"      # "none", "unnamed" (*), "unnamedplus" (+)
crash_recovery = true          # Background swap file recovery on crash
check_update = true            # Check for GitHub releases on launch

# ==========================================
# 2. Appearance
# ==========================================
[appearance]
theme = "tokyo-night"          # Active theme (themes/<name>.toml)
line_number = true             # Line numbers (nu)
relative_number = false        # Relative line numbers (rnu)
wrap = false                   # Line wrap (wrap)
gutter_markers = true          # Modified line gutter markers (modified: yellow / saved: green)

[appearance.gui]
font_family = "PlemolJP Console NF"
font_size = 15.0
line_height_multiplier = 1.35
show_minimap = true
minimap_width = 90.0
enable_gravity_drop = true     # Character deletion physics animation (OpenViVi)
reactive_idle_sleep = true     # 0% CPU reactive idle sleep control

# ==========================================
# 3. Invisibles
# ==========================================
[invisibles]
enabled = false                # Invisible characters display (:set list)
zenkaku_space = "　"           # Full-width whitespace highlight
space = "·"                    # Half-width space symbol
tab = "▸---"                   # Tab glyph
eol = "$"                      # End of line symbol

# ==========================================
# 4. Editing & Assists
# ==========================================
[editing]
tab_width = 4                  # Tab display width
expand_tab = false             # Expand Tab key press into spaces
ignorecase = true              # Case-insensitive search (ic)
smartcase = true               # Override ignorecase when query has uppercase (sc)
keisen_protect = true          # Keisen box-border protected editing
dabbrev_ignorecase = true      # Dynamic abbreviation case-insensitive search
match_paren = true             # Highlight matching parenthesis (%)

# ==========================================
# 5. Tools & Views
# ==========================================
[tools]
difflog_enabled = true         # Automated diff history (.difflog) logging
difflog_max_generations = 50   # Max generation retention count
outline_auto_open = false      # Auto-open outline pane on startup
outline_width = 30             # Outline sidebar width (character columns)
code_folding_level = 0         # Default fold level (0: all expanded)

# ==========================================
# 6. Keymaps & Presets
# ==========================================
[keymap]
preset = "vi"                  # "vi" or "windows" (Shift+Arrow, Ctrl+C/V)

[keymap.normal]
"<C-s>" = ":w<CR>"

[keymap.insert]
"jj" = "<Esc>"
```

---

## 4. Theme Configuration Schema (`themes/<name>.toml`)

Defines hexadecimal color values (`#RRGGBB` / `#RRGGBBAA`) for editor canvas, UI frames, status badges, and syntax tokens.

```toml
[meta]
name = "Tokyo Night"
author = "folke / vivivi port"
variant = "dark" # "dark" or "light"

[ui]
# 3-tier background contrast
bg_editor = "#1a1b26"       # Main editor canvas
bg_panel = "#16161e"        # Sidebars, tab bar, menu bar
bg_statusline = "#1f2335"   # Status lines and separators
bg_selection = "#283457"    # Visual selection
bg_hover = "#292e42"        # Widget hover state
bg_active = "#414868"       # Active / pressed state

fg_text = "#c0caf5"         # Standard text foreground
fg_dimmed = "#565f89"       # Comments and inactive elements
fg_linenumber = "#3b4261"   # Gutter line numbers
fg_cursor = "#c0caf5"       # Monospace cursor foreground
bg_cursor = "#7aa2f7"       # Monospace cursor block background

[modes]
# Vi mode status badges
normal = "#7aa2f7"
insert = "#9ece6a"
visual = "#bb9af7"
command = "#e0af68"
replace = "#f7768e"
keisen = "#7dcfff"

[gutter]
change_modified = "#e0af68" # Unsaved change bar (Yellow)
change_saved = "#9ece6a"    # Saved change bar (Green)

[syntax]
# Syntax highlighting tokens
keyword = "#bb9af7"
type = "#7dcfff"
function = "#7aa2f7"
string = "#9ece6a"
number = "#ff9e64"
comment = "#565f89"
macro = "#7dcfff"
operator = "#89ddff"
```

---

## 5. Settings GUI / TUI Interface Specification

### 5.1 GUI Preferences Panel (egui: 2-Pane Modal Dialog)

```
+--------------------------------------------------------------------------+
|  Preferences - [Tokyo Night]                                    [X] Close|
+-------------------+------------------------------------------------------+
| [Search: _______] | [Appearance & Themes]                                |
|                   |                                                      |
| ▶ 1. General      |  Color Theme                   : [Tokyo Night       ▼]|
| ▼ 2. Appearance   |  Line Numbers                  : [x] Enabled         |
|    - Themes       |  Relative Line Numbers         : [ ] Disabled        |
|    - Gutter       |  Word Wrap                     : [ ] Disabled        |
| ▶ 3. Invisibles   |  Change Gutter Markers         : [x] Enabled         |
| ▶ 4. Editing      |                                                      |
| ▶ 5. Tools & Views| [GUI & Fonts]                                        |
| ▶ 6. Keymaps      |  Font Family                   : [PlemolJP Console NF▼|
|                   |  Font Size                     : [ 15.0 ] [▲][▼]     |
|                   |  Gravity-Drop Particle FX      : [x] Enabled         |
|                   |  Minimap Overview              : [x] Enabled (90px)  |
|                   |                                                      |
+-------------------+------------------------------------------------------+
| [Reset to Defaults]                         [Cancel]  [Apply]  [Save & Close]|
+--------------------------------------------------------------------------+
```

### 5.2 Dynamic Theme Switching & Hot Reload

- **Menu Switcher**: `View(V) -> Color Theme(T) -> OpenViVi / ViViClassic / TokyoNight / ViviviDark / ViviviLight`
- **Hot Reload**: Editing and saving any `.toml` theme file instantly triggers reactive repainting without requiring an editor restart.

### 5.3 TUI Settings Screen (Ratatui Modal)

- Opened via `:preferences` or `:set`.
- Fully navigable via Vi keybindings (`j`/`k` to navigate, `h`/`l` to expand/collapse categories, `Space` to toggle, `Enter` to edit value).

---

## 6. Command-Line (`:set`, `:colorscheme`) Execution & Persistence Boundary

Defines clear boundaries between in-session dynamic mutation and persistent configuration:

1. **Ex Command Mutations Are Ephemeral (In-Session Only)**:
   - **Editor Options**: Temporary `:set nonumber` (`:se nonu`) for clean screenshots, temporary `:set wrap` for inspecting long logs, or temporary `:set tabstop=2` (`:se ts=2`).
   - **Color Themes**: Temporary `:colorscheme high-contrast` (`:colo ...`) or `:set theme=openvivi` for changing mood or high-glare environments.
   - These commands **only modify the in-memory editor state for the active session** and never overwrite `config.toml`.
   - Restarting the editor reliably restores the clean baseline settings and default theme defined in `config.toml`.
2. **How to Persist Configuration Permanently**:
   - **Method A (Ex Command Persistence)**: Execute the Vim-compatible **`:mkvivivirc`** (or `:wconfig`) command.
     - Assembles the current in-memory editor configuration and active theme, dumping them directly into `%USERPROFILE%\.vivivi\config.toml` in one shot.
   - **Method B (Preferences Dialog)**: Open the GUI or TUI settings dialog, adjust settings, and click `[Save & Close]`.
3. **Configuration Reloading**:
   - Execute **`:source`** (or `:so`) to re-read and apply the latest `config.toml` from disk immediately without restarting the editor.

---

## 7. Implementation Roadmap

1. **Step 1: External Theme TOML Parser & Loader (`vivivi-gui::theme`)**
   - Implement `ThemeConfig` struct with `serde`. Load custom `.toml` palettes from `%USERPROFILE%\.vivivi\themes\`.
2. **Step 2: Startup Bootstrapping (`vivivi-app::config`)**
   - Automatically initialize `%USERPROFILE%\.vivivi\` and dump standard themes if directory is missing.
3. **Step 3: Interactive Theme Switching in GUI Menu Bar**
   - Connect menu items to mutate active theme and refresh visuals dynamically.
4. **Step 4: GUI / TUI Interactive Preferences Dialogs**
   - Build categorical settings panels with bidirectional live binding to `AppConfig`.
