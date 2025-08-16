<p align="center">
  <img src="./docs/image.png" width="400" alt="Sage logo" />
</p>

# 🌿 Sage

> Burning away Git complexity

Sage wraps everyday Git pain points behind a single, intuitive CLI (installed as `sg`) and an optional sync server for shared stacks. It keeps branch stacks tidy, automates the boring bits, and gives you a slick TUI when you feel fancy – **no magic, no yak shaving, just clean commits**.

---

## Why Sage?

* 🌱 **Elegantly simple** – zero mental overhead, readable Rust, no hidden
  side‑effects
* ✨ **DX first** – colourful output, progress bars, AI‑assisted commit
  messages (opt‑in)
* 🪄 **Stack aware** – restack, navigate, submit for review in seconds
* 🔌 **Plugin hooks** – extend every lifecycle stage with your own Rust or
  shell plugins
* 🖥️ **TUI** – full‑screen dashboard (enable with `--features tui`)

---

## Quick start

### Install via Homebrew (macOS/Linux)

```bash
# Add the tap and install Sage
brew tap sage-scm/sage
brew install sage
```

### Install from source

```bash
# Install sage with all features
just install

# Or install with specific features
just install-only --stack --ai

# Try commands without installing
just try work my-feature
just try-with stack,tui dash
```

> **Note**: `--features stack` turns on advanced stacked‑diff commands,
> `--features ai` enables AI commit message generation.

---

## Commands

### Core Commands

| Command | Aliases | Description | Flags |
|---------|---------|-------------|-------|
| `work [branch]` | `w` | Smart create/checkout a branch | `--parent`, `--fetch`, `--root`, `--push`, `--fuzzy` |
| `save [message]` | `s` | Stage and commit changes | `--ai`, `--all`, `--paths`, `--amend`, `--push`, `--empty` |
| `sync` | `ss` | Restack and push branches | `--continue`, `--abort` |
| `share` | - | Create or update a PR | `--draft`, `--ready` |
| `list` | - | List local branches | - |
| `log` | - | Show previous commits | - |
| `undo [id]` | - | Revert an item | - |
| `history` | - | Alias for `undo --list` | - |

### Configuration

| Command | Aliases | Description | Flags |
|---------|---------|-------------|-------|
| `config list` | `c l` | List all configuration options | - |
| `config get <key>` | `c g` | Get a configuration value | - |
| `config set <key> <value>` | `c s` | Set a configuration value | `--local` |
| `config unset <key>` | `c u` | Unset a configuration value | - |
| `config edit` | `c e` | Open config in editor | - |

### Feature-Gated Commands

#### Stack Commands (requires `--features stack`)
| Command | Description | Flags |
|---------|-------------|-------|
| `stack init <name>` | Initialize a new stack | - |
| `stack branch <name>` | Create a branch in the stack | `--parent` |
| `stack log` | Show stack structure | - |
| `stack next` | Navigate to next branch | - |
| `stack prev` | Navigate to previous branch | - |
| `stack goto <branch>` | Jump to specific branch | - |
| `stack restack` | Rebase the stack | `--continue`, `--abort`, `--onto`, `--autosquash` |
| `stack submit` | Submit stack for review | `--ready` |
| `stack update` | Update stack | - |
| `stack status` | Show stack status | - |
| `stack clean` | Clean up stack | - |

#### Other Features
- **TUI** (`--features tui`): `ui` - Terminal UI dashboard
- **AI** (`--features ai`): `tips` - AI-powered tips (not yet implemented)

### Not Yet Implemented

The following commands are defined but not yet implemented:
- `dash [--watch]` - Repo dashboard
- `clean [--remote] [--dry-run]` - Prune branches and reflog
- `resolve` - Launch mergetool
- `stats [--since]` - Repo statistics
- `doctor [--fix]` - Environment/toolchain health check
- `completion <shell>` - Generate shell completions
- `plugin` - Plugin management commands

---

## Using Ollama for AI Features

To configure Sage to use your locally running Ollama API for AI-powered commit messages:

```bash
# Set the API endpoint (default Ollama port is 11434)
sg config set ai.api_url http://localhost:11434

# Set your preferred model (e.g., llama2, codellama, mistral)
sg config set ai.model gemma3n:latest

# Verify your configuration
sg config get ai
```

Now you can use AI features with your local Ollama instance:

```bash
# Generate commit message using local Ollama
sg save --ai
```

---

## Layout

```
sage/
├── bins/
│   ├── sage-cli/     # 🌿 main binary (installs as `sg`)
│   └── sage-server/  # stack‑sharing sync service (optional)
├── crates/
│   ├── sage-core/    # domain logic
│   ├── sage-git/     # git2 wrappers
│   ├── sage-tui/     # TUI widgets (optional)
│   ├── sage-plugin/  # plugin SDK
│   └── sage-utils/   # misc helpers
├── docs/
│   └── image.png     # project logo
```

---

## Development

```bash
# Quick check if code compiles
just check

# Run tests
just test

# Watch for changes and auto-rebuild
just watch

# Run full CI pipeline locally
just ci

# Build release version
just release
```

For more development commands, run `just help`.

---

## Dog‑fooding

We believe in using our own medicine. Throughout development **Sage manages its own repository**—every branch, save, sync, and PR is executed with the `sg` CLI you see taking shape here. Expect real‑world polish to land fast because we feel the pain first.

---

## Contributing

1. Fork & clone
2. `just watch` – auto-rebuild on changes
3. `just ci` – ensures code passes all checks
4. Submit a PR 🚀

---
---

## Disclaimer

Sage is **early‑stage and evolving rapidly**. The commands, feature flags, and server behaviour described above are aspirational; not everything is implemented (yet) and details may change without notice. Use at your own risk, and expect breaking changes while we burn away the rough edges.

## License

Licensed under either of

* **MIT** – see [LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>
* **Apache 2.0** – see [LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>

at your option.
