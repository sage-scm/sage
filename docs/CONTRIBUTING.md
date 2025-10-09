# Contributing to Sage

Hey! Want to help make Sage better? Sweet. Here's what you need to know.

## Quick Setup

```bash
git clone https://github.com/sage-scm/sage.git
cd sage
./setup-hooks.sh
just install
```

The hook script configures the repo to use our shared git hooks, and `just install` builds + installs `sg` from source using the local toolchain.

## Pro Performance Tips

### Use Rust Nightly
Nightly tends to build Sage faster:
```bash
rustup install nightly
rustup override set nightly
```

### macOS? Install the lld Linker
It cuts link times dramatically:
```bash
brew install lld
```

### Linux? Try mold
Even faster than lld on many setups:
```bash
# Ubuntu/Debian
sudo apt install mold

# Arch
sudo pacman -S mold

# Fedora
sudo dnf install mold
```

Switch the linker in `.cargo/config.toml` if you go with mold:
```toml
# [target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

### Install cargo-nextest
Blazingly fast test runner with great output:
```bash
cargo install cargo-nextest --locked
```

## Git Hooks (the good kind)

`./setup-hooks.sh` wires in project hooks that keep things tidy:

- **Pre-commit:** formats staged Rust files and warns on clippy issues while keeping your staged set intact
- **Commit-msg:** nudges you toward Conventional Commits
- **Pre-push:** runs a focused safety check before you send patches upstream

Need to bypass in a pinch?
```bash
git commit --no-verify
git push --no-verify
```

## Commit Messages

Stick to Conventional Commits:
```
type(scope): what you changed

feat(cli): add interactive branch switcher
fix(save): handle empty commits properly
docs: update installation guide
```

Valid types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`

## Everyday Dev Commands

```bash
just watch      # rebuild + reinstall on change
just fmt        # rustfmt
just lint       # cargo clippy -- -D warnings
just test       # cargo test --workspace
just try work   # run `sg work` without installing
just ci         # run the same checks CI expects
```

Tip: `just help` lists every recipe with a short description.

## Project Layout Snapshot

```
bin/                 # sg binary crate (installs as `sg`)
crates/              # core libraries (core, git, graph, fmt, ai, ...)
docs/                # public docs (README assets, guides, contributing)
hack-workspace/      # workspace hack crate for MSRV dependencies
.githooks/           # shared git hooks managed by setup-hooks.sh
```

## Submitting Changes

1. Fork + clone the repo
2. Start a branch (`sg work feature/awesome-thing` or `git switch -c feature/awesome-thing`)
3. Make it great and keep commits focused
4. `just ci` before you push so you see what CI will see
5. Push and open a PR with context (screenshots for UX tweaks help!)

## Reporting Bugs

Open an issue with:
- What you expected
- What actually happened
- Steps to reproduce
- Your environment (OS, Rust version, sg version)

## The Vibe

We dogfood Sage to build Sage. Aim for code that's obvious, dependency-light, and pleasant to read. Comments explain "why" when the code can't, not "what". When in doubt, ask in an issue or PR thread — we're friendly.

Welcome aboard! 🚀
