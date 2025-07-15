#!/usr/bin/env just --working-directory .

# Sage - Git workflow automation that burns away complexity 🔥
# A smart Git CLI that understands your workflow

set shell := ["bash", "-c"]
set dotenv-load := true

# Colors for better output
export CYAN := '\033[0;36m'
export GREEN := '\033[0;32m'
export YELLOW := '\033[0;33m'
export RED := '\033[0;31m'
export NC := '\033[0m' # No Color

# ────────────────────────────────────────────────────────────────
# 🚀 Quick Start
# ────────────────────────────────────────────────────────────────

# Show this help message
@help:
    echo -e "${GREEN}Sage Development Commands${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    just --list --unsorted
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Tips:${NC}"
    echo "  • Run 'just' to build and install with all features"
    echo "  • Use 'just watch' for auto-rebuild during development"
    echo "  • Try 'just try <command>' to test without installing"

# Default: build and install with all features
@default: install

# ────────────────────────────────────────────────────────────────
# 🔨 Building & Installing
# ────────────────────────────────────────────────────────────────

# Quick check - verify code compiles
check:
    @echo -e "${CYAN}Checking code...${NC}"
    cargo check --workspace --all-features

# Build with default features (debug)
build:
    @echo -e "${CYAN}Building sage...${NC}"
    cargo build --workspace

# Build with specific features
build-with +features:
    @echo -e "${CYAN}Building with features: {{features}}${NC}"
    cargo build --workspace --features {{features}}

# Build optimized release version
release:
    @echo -e "${CYAN}Building release version...${NC}"
    cargo build --workspace --release --all-features

# Install sage with all features
install: check
    @echo -e "${GREEN}Installing sage with all features...${NC}"
    ./install.sh --all

# Install optimized version
install-release:
    @echo -e "${GREEN}Installing release build...${NC}"
    ./install.sh --all --release

# Install with specific features only
install-only +features:
    @echo -e "${GREEN}Installing with: {{features}}${NC}"
    ./install.sh {{features}}

# Quick reinstall (skip checks)
reinstall:
    @echo -e "${GREEN}Quick reinstall...${NC}"
    cargo install --path ./bins/sage-cli --force --all-features

# ────────────────────────────────────────────────────────────────
# 🧪 Testing & Quality
# ────────────────────────────────────────────────────────────────

# Run all tests
test:
    @echo -e "${CYAN}Running tests...${NC}"
    cargo test --workspace --all-features

# Run tests and capture output
test-verbose:
    cargo test --workspace --all-features -- --nocapture

# Run a specific test
test-one pattern:
    cargo test --workspace --all-features {{pattern}}

# Run tests continuously on file changes
test-watch:
    cargo watch -x "test --workspace --all-features"

# Run benchmarks (including ignored ones)
bench:
    @echo -e "${CYAN}Running benchmarks...${NC}"
    cargo bench --workspace -- --include-ignored

# Check code quality with clippy
lint:
    @echo -e "${CYAN}Running clippy...${NC}"
    cargo clippy --workspace --all-features -- -D warnings

# Fix lint issues automatically
lint-fix:
    cargo clippy --workspace --all-features --fix -- -D warnings

# Format code
fmt:
    @echo -e "${CYAN}Formatting code...${NC}"
    cargo fmt --all

# Check if code is formatted
fmt-check:
    cargo fmt --all -- --check

# ────────────────────────────────────────────────────────────────
# 🛠️ Development Workflow
# ────────────────────────────────────────────────────────────────

# Watch files and rebuild automatically
watch:
    @echo -e "${GREEN}Watching for changes...${NC}"
    ./watch.sh

# Full development build (check + install)
dev:
    @echo -e "${GREEN}Running development build...${NC}"
    ./build.sh

# Try a sage command without installing (cargo run)
try +args:
    @echo -e "${CYAN}Running: sage {{args}}${NC}"
    cargo run --bin sage-cli --all-features -- {{args}}

# Try with specific features
try-with features +args:
    cargo run --bin sage-cli --features {{features}} -- {{args}}

# Open sage TUI dashboard (requires installation)
tui:
    sage tui

# Run sage with debug logging
debug +args:
    RUST_LOG=sage=debug sage {{args}}

# Run sage with trace logging
trace +args:
    RUST_LOG=sage=trace sage {{args}}

# ────────────────────────────────────────────────────────────────
# 📚 Documentation
# ────────────────────────────────────────────────────────────────

# Generate and open documentation
docs:
    @echo -e "${CYAN}Generating documentation...${NC}"
    cargo doc --workspace --all-features --no-deps --open

# Generate docs for all dependencies too
docs-all:
    cargo doc --workspace --all-features --open

# Check for documentation issues
doc-check:
    cargo doc --workspace --all-features --no-deps
    @echo -e "${GREEN}Documentation builds successfully!${NC}"

# Serve docs locally with live reload
docs-serve:
    @echo -e "${CYAN}Serving docs at http://localhost:8000${NC}"
    python3 -m http.server 8000 --directory target/doc

# ────────────────────────────────────────────────────────────────
# 🚢 Release & Distribution
# ────────────────────────────────────────────────────────────────

# Run full CI pipeline locally
ci: fmt-check lint test doc-check
    @echo -e "${GREEN}✓ All CI checks passed!${NC}"

# Prepare for release (run all checks)
pre-release: ci
    @echo -e "${CYAN}Checking for uncommitted changes...${NC}"
    git diff --exit-code
    @echo -e "${CYAN}Checking for untracked files...${NC}"
    git ls-files --others --exclude-standard | grep -q . && exit 1 || true
    @echo -e "${GREEN}✓ Ready for release!${NC}"

# Full release build and install
ship: pre-release release install-release
    @echo -e "${GREEN}🚀 Sage released successfully!${NC}"
    sage --version

# ────────────────────────────────────────────────────────────────
# 🔧 Maintenance
# ────────────────────────────────────────────────────────────────

# Update all dependencies
update:
    @echo -e "${CYAN}Updating dependencies...${NC}"
    cargo update
    @echo -e "${GREEN}Updated! Run 'just outdated' to see what changed.${NC}"

# Show outdated dependencies
outdated:
    cargo outdated

# Audit for security vulnerabilities
audit:
    cargo audit

# Deep clean (removes target/, Cargo.lock)
clean-all:
    @echo -e "${RED}Removing all build artifacts...${NC}"
    cargo clean
    rm -f Cargo.lock
    @echo -e "${GREEN}Clean!${NC}"

# Show project statistics
stats:
    @echo -e "${CYAN}Project Statistics:${NC}"
    @echo "Lines of Rust code:"
    @find . -name "*.rs" -type f -not -path "./target/*" | xargs wc -l | tail -n 1
    @echo ""
    @echo "Number of dependencies:"
    @cargo tree --no-dedupe | grep -v "└" | wc -l
    @echo ""
    @echo "Binary size (if built):"
    @ls -lh target/release/sage 2>/dev/null || echo "Not built yet"

# ────────────────────────────────────────────────────────────────
# 🐕 Dogfooding (Using Sage to develop Sage)
# ────────────────────────────────────────────────────────────────

# Start working on a new feature
work feature:
    sage work {{feature}}

# Save current work
save message:
    sage save "{{message}}"

# Save with AI-generated message
save-ai:
    sage save

# Sync current branch
sync:
    sage sync

# Share changes (create PR)
share:
    sage share

# Show current stack
stack:
    sage stack

# Show repository dashboard
dashboard:
    sage tui

# ────────────────────────────────────────────────────────────────
# 🧰 Utilities
# ────────────────────────────────────────────────────────────────

# Show dependency tree
tree:
    cargo tree

# Show dependency tree for a specific package
tree-pkg package:
    cargo tree -p {{package}}

# Find a dependency
find-dep pattern:
    cargo tree | grep -i {{pattern}}

# Print current sage version
version:
    @sage --version 2>/dev/null || echo "Sage not installed yet"

# Show feature list
features:
    @echo -e "${CYAN}Available features:${NC}"
    @echo "  • stack  - Advanced stacked-diff operations"
    @echo "  • ai     - AI-powered commit messages (Ollama)"
    @echo "  • tui    - Terminal UI dashboard"
    @echo ""
    @echo -e "${YELLOW}Currently enabled in default build: all${NC}"

# Open repository in browser
browse:
    @open https://github.com/$( git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/' ) 2>/dev/null || echo "Not a GitHub repo"

# Show recent git history with sage
history:
    sage log --graph --oneline -n 20