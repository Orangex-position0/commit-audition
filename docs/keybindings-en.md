# Vim Mode Keybindings

Vim mode uses a lazygit-style **view/edit mode** split for text input steps:

- **View mode**: vim keys (h/j/k/l/q/y/n) act as shortcuts
- **Edit mode**: all keys type as regular text; press `Esc` to return to view mode

## General Navigation (View Mode)

| Key | Action |
|-----|--------|
| `h` / `←` | Previous step |
| `l` / `→` | Next step |
| `1`-`5` | Jump directly to step 1-5 |
| `Ctrl+s` | Advance to next step (resets edit mode) |
| `q` | Quit |

## List Selection (Type / Body Editor Mode)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down (wraps around) |
| `k` / `↑` | Move cursor up (wraps around) |
| `Enter` | Confirm selection and advance |
| `/` | Enter search mode |
| `Esc` | Exit search → Quit (cascading) |
| `q` | Quit |

## Text Input — View Mode (Title / Issue Number)

| Key | Action |
|-----|--------|
| `Enter` | Enter edit mode |
| `h` / `l` | Navigate steps |
| `1`-`5` | Jump to step |
| `q` | Quit |
| `Esc` | Clear input (if non-empty) → Quit (if empty) |

## Text Input — Edit Mode (Title / Issue Number)

| Key | Action |
|-----|--------|
| Any character | Type character (including h/j/k/l/q/y/n) |
| `Backspace` | Delete last character |
| `Enter` | Confirm and advance to next step |
| `Esc` | Exit edit mode (stay on current step, keep content) |

The border turns **cyan** to indicate edit mode is active.

## Search Mode (activated with `/` in list steps)

| Key | Action |
|-----|--------|
| Any character | Append to search query |
| `Backspace` | Delete last search character |
| `Enter` | Confirm current selection and exit search |
| `Esc` | Exit search mode |

## Preview & Confirm

| Key | Action |
|-----|--------|
| `y` / `Enter` | Confirm and output commit message |
| `n` | Return to first step to re-edit |
| `1`-`5` | Jump to step |
| `q` / `Esc` | Cancel and quit |

## Step 3 Dual-Block Layout

Step 3 (SelectBody) shows two blocks:

- **Upper block**: Editor mode selection list (navigable with j/k)
- **Lower block**: Read-only preview of body content (shows "尚未编辑正文" when empty)

## Esc Cascading Behavior

Pressing `Esc` responds in the following priority order:

1. **Edit mode** (any step): Exit edit mode, keep content
2. **Search mode** (list steps): Exit search → Then quit
3. **Input steps** (view mode): Clear input (if non-empty) → Then quit (if empty)
4. **Preview step**: Quit immediately
