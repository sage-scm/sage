# Contributing to Sage

Hey! Want to help make Sage better? Sweet. Here's what you need to know.

## Quick Setup

```bash
git clone https://github.com/sage-scm/sage.git
cd sage
./scripts/dev-setup.sh
```

That's it. The setup script handles everything - Rust toolchain, dev tools, git hooks, the works.

## Pro Performance Tips

### Use Rust Nightly
Nightly has a faster build system. Worth it for development:
```bash
rustup install nightly
rustup override set nightly
```

### macOS? Get the lld Linker
Cuts build times significantly:
```bash
brew install lld
# Add to .cargo/config.toml (already configured in this repo)
```

### Linux? mold is Your Friend
Even faster than lld:
```bash
# Ubuntu/Debian
sudo apt install mold

# Arch
sudo pacman -S mold

# Fedora
sudo dnf install mold
```

To use mold, change this line in `.cargo/config.toml`:
```toml
# Find the [target.x86_64-unknown-linux-gnu] section
rustflags = ["-C", "link-arg=-fuse-ld=mold"]  # instead of lld
```

### Use cargo-nextest
Way faster test runner with better output:
```bash
cargo install cargo-nextest
cargo nextest run  # instead of cargo test
```

### Config Troubles?
If builds are slow or failing, check `.cargo/config.toml`:

**Linker issues?** Comment out the rustflags lines:
```toml
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**Cranelift issues on nightly?** Comment out:
```toml
# codegen-backend = "cranelift"
```

**Want max speed?** Make sure you have:
```toml
[profile.dev]
codegen-units = 256  # Fast compilation
[profile.dev.package."*"]
opt-level = 2  # Fast dependencies
```

## Git Hooks (The Good Kind)

We've got hooks that run automatically to keep the codebase clean. No config needed - the setup script handles it.

**Pre-commit**: Formats your code and fixes simple issues
**Pre-push**: Runs the full test suite so you don't break main
**Commit format**: We use conventional commits (feat/fix/docs/etc)

Need to bypass in an emergency?
```bash
git commit --no-verify  # Skip hooks
git push --no-verify    # YOLO mode
```

## Our Code Philosophy

Look, we're building something beautiful here. Every line of code should make you proud. Here's what we care about:

### 1. Code as Art
Your code should make other developers stop and think "damn, that's clean." We're not just solving problems - we're crafting solutions that are a joy to read.

### 2. Every Line Has Purpose
No fluff. No "maybe we'll need this later" abstractions. If you can't explain why a line exists in 5 seconds, it probably shouldn't.

### 3. Smart Dependencies
We don't pull in a whole framework to format a date. Every dependency should earn its place. And stick to popular, well-maintained libraries - no one wants supply chain drama.

### 4. Keep It Stupid Simple
If a junior dev can't understand your code, you're being too clever. We write Rust that reads like a book, not an academic paper.

### 5. No Comment Spam
Your code should tell the story. Comments are for "why", not "what". If you need a comment to explain what's happening, rewrite the code to be clearer.

### 6. Logical Structure
Finding a function shouldn't be a treasure hunt. Put things where they make sense. If you're wondering where something goes, ask yourself: "Where would I look for this?"

## Commit Messages

Keep them clean:
```
type(scope): what you did

feat(cli): add interactive branch switcher
fix(save): handle empty commits properly
docs: update installation guide
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`

## Dev Commands

```bash
# Watch mode - rebuilds on save
./watch.sh
# or if you prefer just
just watch

# Run tests (with nextest for speed)
cargo nextest run           # All tests
cargo nextest run -E 'test(name_of_test)'  # Specific test

# Check your work
cargo fmt      # Format
cargo clippy   # Lint
cargo nextest run  # Test (or cargo test if no nextest)

# Build for speed
cargo build --release

# Try without installing
cargo run --bin sage-cli -- work feature
```

## Project Layout

```
bins/
├── sage-cli/     # The main CLI (what users run as 'sg')
└── sage-server/  # Sync server (still cooking)

crates/
├── sage-core/    # Where the magic happens
├── sage-git/     # Git operations wrapper
├── sage-tui/     # Pretty terminal output
└── ...           # Other specialized crates
```

## Submitting Changes

1. Fork & clone
2. Branch from main: `sg work feature/your-thing`
3. Make it awesome
4. Push and PR
5. Describe what and why (screenshots for UI stuff)

## Found a Bug?

Open an issue with:
- What you expected
- What actually happened
- Steps to break it again
- Your setup (OS, Rust version)

## The Vibe

We're dog-fooding hard - using Sage to build Sage. Lots of `todo!()`s in the code - perfect spots to jump in and contribute.

The codebase should be a place you enjoy spending time. Clean, elegant, obvious. No magic, no wizardry, just solid engineering.

Questions? Hit us up in the issues. We're friendly.

Welcome aboard! 🚀
