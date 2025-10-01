#!/bin/bash
set -e

# Sage Local Development Install Script
# Install sage from source with all features enabled

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="sg"
BUILD_MODE="debug"

# Print colored output
print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Parse arguments
RELEASE_MODE=false
for arg in "$@"; do
    case $arg in
        --release)
            RELEASE_MODE=true
            BUILD_MODE="release"
            shift
            ;;
        --help)
            echo "Usage: $0 [--release]"
            echo ""
            echo "Options:"
            echo "  --release    Build and install optimized release version"
            echo ""
            echo "Note: All features (AI, TUI, stack) are now always enabled"
            exit 0
            ;;
        *)
            print_warning "Unknown option: $arg (ignored)"
            ;;
    esac
done

# Build sage
print_info "Building sage with all features..."

if [ "$RELEASE_MODE" = true ]; then
    print_info "Building in release mode (optimized)..."
    cargo build --release --bin $BINARY_NAME
    BINARY_PATH="target/release/$BINARY_NAME"
else
    print_info "Building in debug mode (faster compilation)..."
    cargo build --bin $BINARY_NAME
    BINARY_PATH="target/debug/$BINARY_NAME"
fi

# Check if build succeeded
if [ ! -f "$BINARY_PATH" ]; then
    print_error "Build failed: Binary not found at $BINARY_PATH"
    exit 1
fi

# Install to cargo bin directory
INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"

if [ ! -d "$INSTALL_DIR" ]; then
    print_info "Creating install directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi

print_info "Installing $BINARY_NAME to $INSTALL_DIR..."
temp_path="$(mktemp "$INSTALL_DIR/${BINARY_NAME}.tmp.XXXXXX")"
trap 'rm -f "$temp_path"' EXIT
cp "$BINARY_PATH" "$temp_path"
chmod +x "$temp_path"
mv "$temp_path" "$INSTALL_DIR/$BINARY_NAME"
trap - EXIT

# Verify installation
if command -v $BINARY_NAME >/dev/null 2>&1; then
    INSTALLED_VERSION=$($BINARY_NAME --version 2>/dev/null | head -n1 || echo "unknown")
    print_success "Successfully installed sage!"
    print_info "Version: $INSTALLED_VERSION"
    print_info "Path: $(which $BINARY_NAME)"
    echo ""
    echo "All features are now enabled by default:"
    echo "  ✓ AI-powered commit messages"
    echo "  ✓ Terminal UI (TUI)"
    echo "  ✓ Stack operations"
    echo ""
    echo "Get started with: $BINARY_NAME --help"
else
    print_warning "$BINARY_NAME installed but not in PATH"
    print_info "Add $INSTALL_DIR to your PATH:"
    print_info "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi
