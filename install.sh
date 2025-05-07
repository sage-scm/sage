#!/bin/bash
set -e

# ANSI color codes
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print banner
echo -e "${GREEN}"
echo "🌿 Sage Installer"
echo "================="
echo -e "${NC}"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo is not installed.${NC}"
    echo "Please install Rust and Cargo first: https://rustup.rs/"
    exit 1
fi

# Parse command line arguments
FEATURES=""
RELEASE=""

print_help() {
    echo -e "Usage: ./install.sh [OPTIONS]"
    echo
    echo "Options:"
    echo "  --help          Show this help message"
    echo "  --stack         Enable stack features"
    echo "  --ai            Enable AI features"
    echo "  --tui           Enable TUI features"
    echo "  --all           Enable all features"
    echo "  --release       Build in release mode (recommended for normal use)"
    echo
    echo "Example:"
    echo "  ./install.sh --release --stack --tui"
    echo
}

for arg in "$@"; do
    case $arg in
        --help)
            print_help
            exit 0
            ;;
        --stack)
            FEATURES="${FEATURES}stack,"
            ;;
        --ai)
            FEATURES="${FEATURES}ai,"
            ;;
        --tui)
            FEATURES="${FEATURES}tui,"
            ;;
        --all)
            FEATURES="stack,ai,tui"
            ;;
        --release)
            RELEASE="--release"
            ;;
        *)
            echo -e "${YELLOW}Warning: Unknown option '$arg'${NC}"
            ;;
    esac
done

# Remove trailing comma if present
FEATURES=$(echo $FEATURES | sed 's/,$//')

# Build the command
INSTALL_CMD="cargo install --path ./bins/sage-cli"

if [ -n "$FEATURES" ]; then
    INSTALL_CMD="$INSTALL_CMD --features $FEATURES"
fi

if [ -n "$RELEASE" ]; then
    INSTALL_CMD="$INSTALL_CMD $RELEASE"
fi

# Display what we're about to do
echo -e "${BLUE}Installing Sage CLI...${NC}"
if [ -n "$FEATURES" ]; then
    echo -e "With features: ${GREEN}$FEATURES${NC}"
fi
echo -e "Command: ${YELLOW}$INSTALL_CMD${NC}"
echo

# Run the installation
echo -e "${BLUE}Running installation...${NC}"
eval $INSTALL_CMD

# Check if installation was successful
if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}✅ Sage CLI installed successfully!${NC}"
    echo -e "You can now use the ${GREEN}sage${NC} command from anywhere."
    echo -e "\nTry running: ${YELLOW}sage --help${NC}"
else
    echo -e "\n${RED}❌ Installation failed.${NC}"
    exit 1
fi
