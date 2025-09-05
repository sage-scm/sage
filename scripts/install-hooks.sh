#!/usr/bin/env bash
set -euo pipefail

YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}${BOLD}       Sage Git Hooks Installation              ${NC}"
echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}✗ Error: Not in a git repository${NC}"
    echo "Please run this script from the repository root."
    exit 1
fi

# Check if hooks directory exists
if [ ! -d ".githooks" ]; then
    echo -e "${RED}✗ Error: .githooks directory not found${NC}"
    echo "Please ensure you're in the sage repository root."
    exit 1
fi

echo -e "${YELLOW}This script will install Git hooks to:${NC}"
echo -e "  • Auto-format code on commit"
echo -e "  • Run clippy checks and auto-fix issues"
echo -e "  • Validate commit message format"
echo -e "  • Run tests before push"
echo -e "  • Check build and documentation"
echo

# Backup existing hooks if they exist
backup_dir=".git/hooks.backup.$(date +%Y%m%d_%H%M%S)"
hooks_to_install=("pre-commit" "pre-push" "commit-msg" "prepare-commit-msg")
existing_hooks=()

for hook in "${hooks_to_install[@]}"; do
    if [ -f ".git/hooks/$hook" ] && [ ! -L ".git/hooks/$hook" ]; then
        existing_hooks+=("$hook")
    fi
done

if [ ${#existing_hooks[@]} -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Found existing hooks: ${existing_hooks[*]}${NC}"
    read -p "Do you want to backup existing hooks? (Y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        mkdir -p "$backup_dir"
        for hook in "${existing_hooks[@]}"; do
            mv ".git/hooks/$hook" "$backup_dir/$hook"
            echo -e "${GREEN}✓ Backed up $hook to $backup_dir${NC}"
        done
    fi
fi

# Install hooks method selection
echo -e "${YELLOW}Choose installation method:${NC}"
echo "  1) Symlink (recommended - auto-updates with repository)"
echo "  2) Copy (standalone - won't update automatically)"
echo
read -p "Select method [1-2] (default: 1): " -n 1 -r
echo

if [[ $REPLY =~ ^[2]$ ]]; then
    # Copy method
    echo -e "${YELLOW}Installing hooks (copy method)...${NC}"
    for hook in "${hooks_to_install[@]}"; do
        if [ -f ".githooks/$hook" ]; then
            cp ".githooks/$hook" ".git/hooks/$hook"
            chmod +x ".git/hooks/$hook"
            echo -e "${GREEN}✓ Installed $hook${NC}"
        fi
    done
else
    # Symlink method (default)
    echo -e "${YELLOW}Installing hooks (symlink method)...${NC}"

    # Get absolute path to hooks directory
    hooks_abs_path="$(cd .githooks && pwd)"

    for hook in "${hooks_to_install[@]}"; do
        if [ -f ".githooks/$hook" ]; then
            ln -sf "$hooks_abs_path/$hook" ".git/hooks/$hook"
            chmod +x ".githooks/$hook"
            echo -e "${GREEN}✓ Linked $hook${NC}"
        fi
    done

    # Alternative: Configure git to use .githooks directory
    echo
    echo -e "${YELLOW}Setting Git to use .githooks directory...${NC}"
    git config core.hooksPath .githooks
    echo -e "${GREEN}✓ Git configured to use .githooks directory${NC}"
fi

# Install optional tools
echo
echo -e "${BLUE}${BOLD}Optional Tools${NC}"
echo -e "${YELLOW}The following tools enhance the git hooks experience:${NC}"
echo

# Check for cargo-audit
if ! command -v cargo-audit &> /dev/null; then
    echo -e "  ${YELLOW}•${NC} cargo-audit - Security vulnerability scanner"
    echo -e "    Install with: ${BOLD}cargo install cargo-audit${NC}"
else
    echo -e "  ${GREEN}✓${NC} cargo-audit - Already installed"
fi

# Check for cargo-hakari
if ! command -v cargo-hakari &> /dev/null; then
    echo -e "  ${YELLOW}•${NC} cargo-hakari - Workspace dependency optimizer"
    echo -e "    Install with: ${BOLD}cargo install cargo-hakari${NC}"
else
    echo -e "  ${GREEN}✓${NC} cargo-hakari - Already installed"
fi

echo
echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo -e "${GREEN}${BOLD}✅ Git hooks installation complete!${NC}"
echo -e "${BLUE}${BOLD}════════════════════════════════════════════════${NC}"
echo
echo -e "${YELLOW}Hooks are now active for:${NC}"
echo -e "  • pre-commit: Format & lint checks"
echo -e "  • commit-msg: Message format validation"
echo -e "  • pre-push: Full test suite & build verification"
echo
echo -e "${YELLOW}To temporarily skip hooks, use:${NC}"
echo -e "  ${BOLD}git commit --no-verify${NC}  or  ${BOLD}git push --no-verify${NC}"
echo
echo -e "${YELLOW}To uninstall hooks:${NC}"
echo -e "  ${BOLD}git config --unset core.hooksPath${NC}"
echo -e "  ${BOLD}rm .git/hooks/{pre-commit,pre-push,commit-msg,prepare-commit-msg}${NC}"
