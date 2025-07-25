<p align="center">
  <img src="./docs/image.png" width="400" alt="Sage logo" />
</p>

# 🌿 Sage

> Burning away Git complexity

Sage wraps everyday Git pain points behind a single, intuitive CLI (and an optional sync server for shared stacks). It keeps branch stacks tidy, automates the boring bits, and gives you a slick TUI when you feel fancy – **no magic, no yak shaving, just clean commits**.

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

## Commands (core set)

| Command | Purpose |
|---------|---------|
| `work <branch>` | Smart create/checkout |
| `save [-m\|--ai] [--all] <…>` | Stage → commit |
| `sync [--continue\|--abort]` | Restack + push |
| `share [--draft\|--ready]` | Create/update PR |
| `dash [--watch]` | Repo dashboard |
| `clean [--remote] [--dry-run]` | Prune branches/reflog |
| `undo [id]` / `history` | Revert + log |
| `resolve` | Launch mergetool |
| `stats [--since <date>]` | Repo statistics |
| `doctor [--fix]` | Env/toolchain check |
| `config <op>` | Manage config |
| `completion <shell>` | Shell completions |

Enable feature flags for extra **stack**, **ai**, and **tui** commands.

---

## Using Ollama for AI Features

To configure Sage to use your locally running Ollama API for AI-powered commit messages:

```bash
# Set the API endpoint (default Ollama port is 11434)
sage config set ai.api_url http://localhost:11434

# Set your preferred model (e.g., llama2, codellama, mistral)
sage config set ai.model gemma3n:latest

# Verify your configuration
sage config get ai
```

Now you can use AI features with your local Ollama instance:

```bash
# Generate commit message using local Ollama
sage save --ai
```

---

## Layout

```
sage/
├── bins/
│   ├── sage-cli/     # 🌿 main binary
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

We believe in using our own medicine. Throughout development **Sage manages its own repository**—every branch, save, sync, and PR is executed with the CLI you see taking shape here. Expect real‑world polish to land fast because we feel the pain first.

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
