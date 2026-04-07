# commit-audition

An interactive Git Commit Message generator and validator that enforces [Conventional Commits](https://www.conventionalcommits.org/) conventions.

[中文文档](README.md)

## Features

- **Interactive Generation** — Guided Q&A to collect type / title / body / issue step by step, zero memorization
- **Real-time Validation** — Title width ≤50, body line width ≤72, no trailing periods — validated as you type
- **Three Editing Modes** — Terminal inline input, system default editor, or custom editor (VS Code / Vim etc.)
- **Git Hook Integration** — One-command `commit-msg` hook installation, auto-rejects non-compliant messages on `git commit`
- **CJK Friendly** — Unicode display width calculation ensures accurate Chinese title/body validation
- **Colored Preview** — Preview commit message with colors before confirming

## Installation

```bash
git clone https://github.com/<your-username>/commit-audition.git
cd commit-audition
cargo build --release
```

The binary is at `target/release/commit-audition`. Add it to your `PATH` or copy to `/usr/local/bin`.

## Quick Start

### Interactive Commit Message Generation

```bash
commit-audition
```

The tool guides you through:

1. Select commit type (feat / fix / docs / style / refactor / tests / chore)
2. Enter title (imperative mood, capitalized first letter, ≤50 chars)
3. Optionally add body → choose editing mode → enter body
4. Optionally link an Issue
5. Preview and confirm → output plain text

### Validate Existing Commit Messages

```bash
# Validate a file
commit-audition validate .git/COMMIT_EDITMSG

# Read from stdin (CI pipeline)
echo "feat: Add feature" | commit-audition validate -
```

### Install Git Hook

```bash
# Install commit-msg hook
commit-audition hook install

# Uninstall
commit-audition hook uninstall
```

Once installed, every `git commit` will auto-validate the commit message. Non-compliant messages are rejected.

## Configuration

Config file: `~/.commit-audition/config.toml`

```toml
[editor]
command = "code --wait"    # Custom editor command
extension = "md"            # Temp file extension
```

See [Configuration Docs](docs/note/Configuration.md).

## Commit Message Format

```text
<type>: <title>           ← Title line (≤50 chars display width)
                          ← Blank line
<body>                    ← Body (each line ≤72 chars display width, optional)
                          ← Blank line
#<issue>                  ← Issue reference (optional)
```

### Allowed Types

| Type | Description |
|---|---|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Formatting (no code behavior change) |
| `refactor` | Code refactoring |
| `tests` | Adding tests |
| `chore` | Build process or auxiliary tool changes |

## Architecture

Four-layer minimalist CLI architecture:

| Layer | Directory | Responsibility |
|---|---|---|
| **CLI Layer** | `cli/` | Command-line argument parsing and normalization |
| **Logic Layer** | `logic/` | Pure business logic: models, validation, assembly |
| **Integration Layer** | `integration/` | Side-effect boundary (reserved) |
| **UI Layer** | `ui/` | Terminal interaction: prompts, editor, rendering |

See [Architecture Docs](docs/note/architecture.md).

## Documentation

| Document | Description |
|---|---|
| [Architecture Overview](docs/note/architecture.md) | Four-layer design and data flow |
| [CLI Layer](docs/note/Architecture-CLI%20Layer.md) | CLI argument definitions |
| [Logic Layer](docs/note/Architecture-Logic%20Layer.md) | Business logic and validation rules |
| [UI Layer](docs/note/Architecture-UI%20Layer.md) | Terminal interaction design |
| [Integration Layer](docs/note/Architecture-Integration%20Layer.md) | Side-effect boundary design |
| [Configuration](docs/note/Configuration.md) | Config file format and usage |
| [Dependencies](docs/note/dependencies.md) | Dependency selection rationale |
| [Validation Strategy](docs/note/Commit%20Message%20校验策略.md) | Commit message validation approach |
| [Usage Guide](docs/Usage-en.md) | Detailed usage documentation |

## License

MIT
