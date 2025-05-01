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
# build the CLI (core features only)
cargo run --bin sage-cli -- work my-feature

# enable extras
cargo run -F stack,tui --bin sage-cli -- ui
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

## Building

```bash
# check build
cargo check

# run tests across workspace
cargo test --all
```

---

## Contributing

1. Fork & clone  
2. `cargo run …` – hack away  
3. Make sure `cargo fmt` and `cargo clippy` are clean  
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
