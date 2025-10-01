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
    echo "  • Run 'just' to build and install sage"
    echo "  • Use 'just watch' for auto-rebuild during development"
    echo "  • Try 'just try <command>' to test without installing"

# Default: build and install
@default: install

# ────────────────────────────────────────────────────────────────
# 🔨 Building & Installing
# ────────────────────────────────────────────────────────────────

# Quick check - verify code compiles
check:
    @echo -e "${CYAN}Checking code...${NC}"
    cargo check --workspace

# Build workspace
build:
    @echo -e "${CYAN}Building sage...${NC}"
    cargo build --workspace

# Build optimized release version
release:
    @echo -e "${CYAN}Building release version...${NC}"
    cargo build --workspace --release

# Install sage locally from source
install: check
    @echo -e "${GREEN}Installing sage from source...${NC}"
    ./install-local.sh

# Install optimized version from source
install-release:
    @echo -e "${GREEN}Installing release build from source...${NC}"
    ./install-local.sh --release

# Quick reinstall (skip checks)
reinstall:
    @echo -e "${GREEN}Quick reinstall...${NC}"
    cargo install --path ./bin --force

# ────────────────────────────────────────────────────────────────
# 🧪 Testing & Quality
# ────────────────────────────────────────────────────────────────

# Run all tests
test:
    @echo -e "${CYAN}Running tests...${NC}"
    cargo test --workspace

# Run tests and capture output
test-verbose:
    cargo test --workspace -- --nocapture

# Run a specific test
test-one pattern:
    cargo test --workspace {{pattern}}

# Run tests continuously on file changes
test-watch:
    cargo watch -x "test --workspace"

# Run benchmarks (including ignored ones)
bench:
    @echo -e "${CYAN}Running benchmarks...${NC}"
    cargo bench --workspace -- --include-ignored

# Check code quality with clippy
lint:
    @echo -e "${CYAN}Running clippy...${NC}"
    cargo clippy --workspace -- -D warnings

# Fix lint issues automatically
lint-fix:
    cargo clippy --workspace --fix -- -D warnings

# Format code
fmt:
    @echo -e "${CYAN}Formatting code...${NC}"
    cargo fmt

# Check if code is formatted
fmt-check:
    cargo fmt -- --check

# ────────────────────────────────────────────────────────────────
# 🛠️ Development Workflow
# ────────────────────────────────────────────────────────────────

# Watch files and rebuild automatically
watch:
    @echo -e "${GREEN}Watching for changes...${NC}"
    watchexec -re rs ./install-local.sh

# Try a sg command without installing (cargo run)
try +args:
    @echo -e "${CYAN}Running: sg {{args}}${NC}"
    cargo run --bin sg -- {{args}}

# Run sg with debug logging
debug +args:
    RUST_LOG=sage=debug sg {{args}}

# Run sg with trace logging
trace +args:
    RUST_LOG=sage=trace sg {{args}}

# ────────────────────────────────────────────────────────────────
# 📚 Documentation
# ────────────────────────────────────────────────────────────────

# Generate and open documentation
docs:
    @echo -e "${CYAN}Generating documentation...${NC}"
    cargo doc --workspace --no-deps --open

# Generate docs for all dependencies too
docs-all:
    cargo doc --workspace --open

# Check for documentation issues
doc-check:
    cargo doc --workspace --no-deps
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
    sg --version

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
    @ls -lh target/release/sg 2>/dev/null || echo "Not built yet"

# ────────────────────────────────────────────────────────────────
# 🐕 Dogfooding (Using Sage to develop Sage)
# ────────────────────────────────────────────────────────────────

# Start working on a new feature
work feature:
    sg work {{feature}}

# Save current work
save message:
    sg save "{{message}}"

# Save with AI-generated message
save-ai:
    sg save -a

# Sync current branch
sync:
    sg sync

# Share changes (create PR)
share:
    sg share

# Show current stack
stack:
    sg stack

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

# Print current sg version
version:
    @sg --version 2>/dev/null || echo "Sage not installed yet"

# Show built-in features
features:
    @echo -e "${CYAN}Built-in features (always enabled):${NC}"
    @echo "  • Advanced stacked-diff operations"
    @echo "  • AI-powered commit messages (Ollama)"
    @echo "  • Terminal UI dashboard"

# Open repository in browser
browse:
    @open https://github.com/$( git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/' ) 2>/dev/null || echo "Not a GitHub repo"

# Show recent git history with sg
history:
    sg log --graph --oneline -n 20
