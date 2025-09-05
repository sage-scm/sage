#!/usr/bin/env bash
set -euo pipefail

YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}${BOLD}        Sage Developer Environment Setup         ${NC}"
echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo

# Check Rust installation
echo -e "${YELLOW}Checking Rust installation...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Rust is not installed${NC}"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
else
    rust_version=$(rustc --version | cut -d' ' -f2)
    echo -e "${GREEN}✓ Rust $rust_version installed${NC}"
fi

# Ask about nightly for better performance
echo
echo -e "${YELLOW}Want to use Rust nightly for faster builds? (recommended for development)${NC}"
echo "Nightly has better build caching and incremental compilation."
read -p "Install and use nightly? (Y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    echo -e "${YELLOW}Installing Rust nightly...${NC}"
    rustup install nightly
    rustup override set nightly
    rustup component add rustfmt clippy --toolchain nightly
    echo -e "${GREEN}✓ Rust nightly installed and set for this project${NC}"
else
    # Update stable
    echo -e "${YELLOW}Updating Rust stable toolchain...${NC}"
    rustup update stable
    rustup component add rustfmt clippy
    echo -e "${GREEN}✓ Rust stable toolchain updated${NC}"
fi

# Platform-specific optimizations
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo
    echo -e "${BLUE}${BOLD}macOS Performance Optimization${NC}"
    echo -e "${YELLOW}The lld linker can significantly speed up builds.${NC}"

    if ! command -v lld &> /dev/null; then
        echo "Install with: brew install llvm"
        read -p "Install lld now? (Y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            if command -v brew &> /dev/null; then
                brew install llvm
                echo -e "${GREEN}✓ lld installed via Homebrew${NC}"
                echo -e "${YELLOW}Note: .cargo/config.toml is already configured to use lld${NC}"
            else
                echo -e "${YELLOW}Homebrew not found. Install from https://brew.sh/ then run: brew install llvm${NC}"
            fi
        fi
    else
        echo -e "${GREEN}✓ lld already installed${NC}"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo
    echo -e "${BLUE}${BOLD}Linux Performance Optimization${NC}"
    echo -e "${YELLOW}The mold linker is even faster than lld on Linux.${NC}"

    if ! command -v mold &> /dev/null; then
        echo "mold can cut link times by 50-70%"
        echo
        echo "Install with:"
        echo "  Ubuntu/Debian: sudo apt install mold"
        echo "  Arch: sudo pacman -S mold"
        echo "  Fedora: sudo dnf install mold"
        echo
        echo -e "${YELLOW}After installing, the project will automatically use it${NC}"
    else
        echo -e "${GREEN}✓ mold already installed${NC}"
    fi
fi

# Install required development tools
echo
echo -e "${BLUE}${BOLD}Installing Development Tools${NC}"

install_tool() {
    local tool=$1
    local crate=$2
    local description=$3

    if ! command -v "$tool" &> /dev/null; then
        echo -e "${YELLOW}Installing $description...${NC}"
        cargo install "$crate"
        echo -e "${GREEN}✓ $description installed${NC}"
    else
        echo -e "${GREEN}✓ $description already installed${NC}"
    fi
}

# Core development tools
install_tool "cargo-watch" "cargo-watch" "cargo-watch (auto-rebuild on changes)"
install_tool "cargo-nextest" "cargo-nextest" "cargo-nextest (faster test runner)"
install_tool "cargo-audit" "cargo-audit" "cargo-audit (security scanner)"
install_tool "cargo-outdated" "cargo-outdated" "cargo-outdated (dependency checker)"
install_tool "cargo-edit" "cargo-edit" "cargo-edit (add/rm/upgrade dependencies)"

# Workspace optimization (optional but recommended)
if [ -f ".config/hakari.toml" ]; then
    install_tool "cargo-hakari" "cargo-hakari" "cargo-hakari (workspace optimizer)"
fi

# Install Git hooks
echo
echo -e "${BLUE}${BOLD}Setting up Git Hooks${NC}"
if [ -f "scripts/install-hooks.sh" ]; then
    echo -e "${YELLOW}Installing Git hooks...${NC}"
    # Auto-accept symlink method for dev setup
    echo "1" | bash scripts/install-hooks.sh
else
    echo -e "${YELLOW}⚠️  Git hooks installer not found${NC}"
fi

# Build the project
echo
echo -e "${BLUE}${BOLD}Building Project${NC}"
echo -e "${YELLOW}Building sage in debug mode...${NC}"
cargo build
echo -e "${GREEN}✓ Build successful${NC}"

# Run tests
echo
echo -e "${BLUE}${BOLD}Running Tests${NC}"
echo -e "${YELLOW}Running test suite...${NC}"
# Use nextest if available, fallback to cargo test
if command -v cargo-nextest &> /dev/null; then
    if cargo nextest run --workspace; then
        echo -e "${GREEN}✓ All tests passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Some tests failed (this might be expected for WIP features)${NC}"
    fi
else
    if cargo test --workspace; then
        echo -e "${GREEN}✓ All tests passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Some tests failed (this might be expected for WIP features)${NC}"
    fi
fi

# Setup local installation
echo
echo -e "${BLUE}${BOLD}Local Installation${NC}"
echo -e "${YELLOW}Would you like to install sage locally for testing? (y/N)${NC}"
read -p "" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -f "./install-local.sh" ]; then
        ./install-local.sh
        echo -e "${GREEN}✓ Sage installed locally${NC}"
    else
        cargo install --path bins/sage-cli
        echo -e "${GREEN}✓ Sage installed via cargo${NC}"
    fi
fi

# Development tips
echo
echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo -e "${GREEN}${BOLD}✅ Dev environment ready!${NC}"
echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo
echo -e "${YELLOW}${BOLD}Quick Commands:${NC}"
echo -e "  ${BOLD}./watch.sh${NC}              - Auto-rebuild on file changes"
echo -e "  ${BOLD}cargo build --release${NC}   - Build optimized binary"
echo -e "  ${BOLD}cargo nextest run${NC}      - Run tests (fast!)"
echo -e "  ${BOLD}cargo clippy${NC}            - Lint code"
echo -e "  ${BOLD}cargo fmt${NC}               - Format code"
echo
echo -e "${YELLOW}${BOLD}Performance Tips:${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "  • lld linker is configured for faster builds"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "  • Install mold for fastest possible builds"
fi
echo -e "  • Use ${BOLD}cargo build --release${NC} for benchmarking"
echo -e "  • Run ${BOLD}./watch.sh${NC} in a terminal for live rebuilds"
echo
echo -e "${YELLOW}${BOLD}Git Workflow:${NC}"
echo -e "  ${BOLD}sg work feature/name${NC}   - Start new feature"
echo -e "  ${BOLD}sg save${NC}                 - Commit changes"
echo -e "  ${BOLD}sg sync${NC}                 - Restack and push"
echo -e "  ${BOLD}sg share${NC}                - Create PR"
echo
echo -e "Ready to hack! 🚀"
