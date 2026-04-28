# Configuration

Config file path: `~/.commit-audition/config.toml`

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `language` | `string` | `"en"` | UI language, `"en"` for English, `"zh"` for Chinese |
| `vim_mode` | `bool` | `false` | Enable vim mode (full-screen TUI interface) |
| `editor.command` | `string?` | `null` | Custom editor command, e.g. `"code --wait"` |
| `editor.extension` | `string` | `"md"` | File extension for the temporary file |

## Example

```toml
# Set UI language
language = "zh"

# Enable vim mode
vim_mode = true

[editor]
command = "code --wait"
extension = "md"
```

## Option Details

### `language`

Sets the UI language. Supported values: `"en"` (English, default), `"zh"` (Chinese). Affects all user-facing text including prompts, type descriptions, error messages, vim mode labels, and hook command output.

### `vim_mode`

When set to `true`, running `commit-audition` launches a full-screen TUI interface with vim-style keybindings. When `false` or omitted, the default interactive prompt mode is used.

### `editor.command`

The command used to launch a custom editor. When the "Custom Editor" mode is selected, this command opens a temporary file for editing.

Common editor examples:

| Editor | Command |
|--------|---------|
| VS Code | `"code --wait"` |
| Vim | `"vim"` |
| Neovim | `"nvim"` |
| Helix | `"hx"` |
| Sublime Text | `"subl -w"` |

If not configured and "Custom Editor" mode is selected, the program prints an error and exits.

### `editor.extension`

File extension for the temporary file, which affects syntax highlighting in the editor. Defaults to `"md"`. Can also be set to `"txt"`, etc.
