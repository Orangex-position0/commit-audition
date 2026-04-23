# Vim Mode Keybindings

## General Navigation

| Key | Action |
|-----|--------|
| `h` / `←` | Previous step |
| `l` / `→` | Next step |
| `Ctrl+s` | Skip current step, advance to next |

## List Selection (Type / Body Editor Mode)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `Enter` | Confirm selection and advance |
| `/` | Enter search mode |
| `Esc` | Exit search → Quit (cascading) |
| `q` | Quit |

## Text Input (Title / Issue Number)

| Key | Action |
|-----|--------|
| Any character | Type character |
| `Backspace` | Delete last character |
| `Enter` | Confirm and advance |
| `Esc` | Clear input → Quit (cascading) |

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
| `q` / `Esc` | Cancel and quit |

## Esc Cascading Behavior

Pressing `Esc` responds in the following priority order:

1. **List steps**: Exit search first → Then quit
2. **Input steps**: Clear input first → Then quit
3. **Preview step**: Quit immediately
