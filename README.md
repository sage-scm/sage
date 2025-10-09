<p align="center">
  <img src="./docs/image.png" width="400" alt="Sage logo" />
</p>

# 🌿 Sage

> Burning away Git complexity

Sage wraps everyday Git pain points behind a single, intuitive CLI (installed as `sg`) and an optional sync server for shared stacks. It keeps branch stacks tidy, automates the boring bits, and gives you a slick TUI when you feel fancy – **no magic, no yak shaving, just clean commits**.

👉 **Just getting started?** Follow the [Quick Setup](docs/CONTRIBUTING.md#quick-setup) section in the contributing guide while we draft a new dedicated onboarding doc.

> ⚠️ **Alpha in motion:** Sage is in the middle of a major restructuring. Commands, flags, and workflows are being iterated on rapidly and may change without notice while we stabilize the new architecture.

---

## Why Sage?

* 🌱 **Elegantly simple** – zero mental overhead, readable Rust, no hidden
  side‑effects
* ✨ **DX first** – colourful output, progress bars, AI‑assisted commit
  messages (opt‑in)
* 🪄 **Stack aware** – restack, navigate, submit for review in seconds
* 🔌 **Plugin hooks** – extend every lifecycle stage with your own Rust or
  shell plugins
* 🖥️ **TUI** – full‑screen dashboard

---

## Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/sage-scm/sage/main/install.sh | sh
```

This script automatically:
- Detects your platform (Linux/macOS/Windows)
- Downloads the appropriate binary
- Verifies checksums for security
- Installs to `/usr/local/bin` (or `~/.local/bin` if needed)

### Manual Download

Download pre-built binaries from [GitHub Releases](https://github.com/sage-scm/sage/releases):

- **Linux**: `sage-linux-amd64.tar.gz` (glibc) or `sage-linux-amd64-musl.tar.gz` (musl)
- **macOS**: `sage-macos-amd64.tar.gz` (Intel) or `sage-macos-arm64.tar.gz` (Apple Silicon)  
- **Windows**: `sage-windows-amd64.zip`

All downloads include SHA256 checksums for verification.

### Building from Source

```bash
# Quick developer setup (installs hooks, tools, and builds project)
git clone https://github.com/sage-scm/sage
cd sage
./setup-hooks.sh          # optional, installs shared git hooks
./install-local.sh        # build + install sg from source

# Or build manually:
cargo build --release
./target/release/sg --version

# Try commands without installing
just try work my-feature
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for full development setup including Git hooks and code quality tools.

### Package Managers

```bash
# Homebrew (coming soon)
brew install sage-scm/cask/sage

# Cargo
cargo install --git https://github.com/sage-scm/sage sage-cli
```

> **Note**: All features including stacked‑diff commands and AI commit message generation are enabled by default.

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

Now you can use the AI features with your local Ollama instance:

```bash
# Generate commit message using local Ollama
sg save --ai
```

---

## Layout

```
sage/
├── bin/                 # 🌿 main CLI crate (produces the `sg` binary)
├── crates/
│   ├── sage-ai/        # AI integrations
│   ├── sage-config/    # configuration handling
│   ├── sage-core/      # domain logic and workflows
│   ├── sage-fmt/       # user-facing output helpers
│   ├── sage-git/       # git plumbing helpers
│   └── sage-graph/     # stack graph modelling
├── docs/
│   ├── CONTRIBUTING.md
│   └── image.png      # project logo
└── install-local.sh   # helper for local installs
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

We welcome contributions! Everything you need—including setup steps, conventions, and release details—is now recorded in [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md).

---

## Disclaimer

Sage is **early‑stage and evolving rapidly**. Features are still landing, and behaviour may change without notice while we burn away the rough edges.

## License

Licensed under either of

* **MIT** – see [LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>
* **Apache 2.0** – see [LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>

at your option.
