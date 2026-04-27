# commit-audition

Interactive Git Commit Message generator and validator following [Conventional Commits](https://www.conventionalcommits.org/).

[中文文档](docs/README.zh-CN.md)

## Features

- **Interactive Generation** — Guided prompts collect type / title / body / issue step by step
- **Real-time Validation** — Title width ≤50, body width ≤72, no trailing periods
- **Three Editing Modes** — Terminal inline, system default editor, custom editor (VS Code / Vim etc.)
- **Vim Mode TUI** — Full keyboard-driven TUI interface with ratatui (lazygit-style)
- **Git Hook Integration** — One-command `commit-msg` hook install/uninstall
- **CJK Friendly** — Accurate display-width calculation for Chinese characters
- **Colored Preview** — Review commit message with syntax highlighting before output

## Demo

*Screenshots and GIFs will be added after i18n support (planned for v0.5.0).*

## Installation

### Cargo

```bash
cargo install commit-audition
```

### One-line Installer

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Orangex-position0/commit-audition/releases/latest/download/commit-audition-installer.sh | sh

# Windows PowerShell
powershell -c "irm https://github.com/Orangex-position0/commit-audition/releases/latest/download/commit-audition-installer.ps1 | iex"
```

### Build from Source

```bash
git clone https://github.com/Orangex-position0/commit-audition.git
cd commit-audition
cargo build --release
```

The binary is at `target/release/commit-audition`.

## Usage

### Interactive Generation

```bash
commit-audition
```

The tool guides you through:

1. Select commit type (feat / fix / docs / style / refactor / tests / chore)
2. Enter title (imperative mood, ≤50 display width)
3. Choose body editing mode → enter body
4. Optionally link an Issue number
5. Preview and confirm → output plain text

### Validate Existing Messages

```bash
# Validate a file
commit-audition validate .git/COMMIT_EDITMSG

# From stdin (CI scenarios)
echo "feat: Add feature" | commit-audition validate -
```

### Git Hook

```bash
# Install commit-msg hook
commit-audition hook install

# Uninstall
commit-audition hook uninstall
```

## Configuration

Config file: `~/.commit-audition/config.toml`

```toml
# Enable vim mode TUI interface
vim_mode = true

[editor]
command = "code --wait"    # Custom editor command
extension = "md"            # Temp file extension
```

See [Configuration Guide](docs/note/Configuration.md) for details.

## Recommended Alias

You can set a shorter alias for convenience:

| Shell | Config File | Command |
|---|---|---|
| bash | `~/.bashrc` | `alias cmt='commit-audition'` |
| zsh | `~/.zshrc` | `alias cmt='commit-audition'` |
| fish | `~/.config/fish/config.fish` | `alias cmt commit-audition` |
| PowerShell | `$PROFILE` | `Set-Alias -Name cmt -Value commit-audition` |

After setup, use `cmt` instead of `commit-audition`:

```bash
cmt          # Start interactive generation
cmt validate # Validate mode
```

## Commit Message Format

```text
<type>: <title>           ← Title line (≤50 display width)
                          ← Blank line
<body>                    ← Body (each line ≤72 display width, optional)
                          ← Blank line
#<issue>                  ← Issue reference (optional)
```

### Valid Types

| Type | Description |
|---|---|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Formatting (no code changes) |
| `refactor` | Code refactoring |
| `tests` | Adding tests |
| `chore` | Build or tooling changes |

## Architecture

| Layer | Directory | Responsibility |
|---|---|---|
| **CLI Layer** | `cli/` | Command-line argument parsing |
| **Logic Layer** | `logic/` | Pure business logic: models, validation, building |
| **Integration Layer** | `integration/` | Side-effect boundary (reserved) |
| **UI Layer** | `ui/` | Terminal interaction: prompts, editor, rendering |

## Documentation

| Document | Description |
|---|---|
| [Architecture](docs/note/architecture.md) | Four-layer architecture and data flow |
| [Configuration](docs/note/Configuration.md) | Config file format and usage |
| [Vim Mode Design](docs/note/vim-mode.md) | Vim mode TUI detailed design |
| [Release Guide](docs/release-guide.md) | dist + GitHub Actions release workflow |

## License

MIT OR Apache-2.0
