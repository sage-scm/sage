#!/bin/bash
set -euo pipefail

# Sage Installation Script
# Downloads and installs pre-built binaries from GitHub releases
# Secure installation with checksum verification and platform detection
# Usage: curl -fsSL https://raw.githubusercontent.com/sage-scm/sage/main/install.sh | sh

# Configuration
REPO="sage-scm/sage"
BINARY_NAME="sg"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TEMP_DIR=$(mktemp -d)
VERSION="${VERSION:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    if [ "${DEBUG:-false}" = "true" ]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

# Cleanup function
cleanup() {
    if [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
        log_debug "Cleaned up temporary directory: $TEMP_DIR"
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Platform detection
detect_platform() {
    local os arch platform
    
    # Detect OS - handle Windows variants better
    if [ -n "${WINDIR:-}" ] || [ -n "${SYSTEMROOT:-}" ]; then
        os="windows"
    else
        case "$(uname -s 2>/dev/null || echo Unknown)" in
            Linux*)
                os="linux"
                ;;
            Darwin*)
                os="macos"
                ;;
            CYGWIN*|MINGW*|MSYS*|Windows_NT)
                os="windows"
                ;;
            *)
                log_error "Unsupported operating system: $(uname -s 2>/dev/null || echo Unknown)"
                exit 1
                ;;
        esac
    fi
    
    # Detect architecture - handle Windows variants
    if [ "$os" = "windows" ]; then
        # On Windows, check PROCESSOR_ARCHITECTURE
        case "${PROCESSOR_ARCHITECTURE:-$(uname -m 2>/dev/null || echo x86_64)}" in
            AMD64|x86_64|amd64)
                arch="amd64"
                ;;
            ARM64|aarch64|arm64)
                arch="arm64"
                ;;
            *)
                arch="amd64"  # Default to amd64 on Windows
                ;;
        esac
    else
        case "$(uname -m 2>/dev/null || echo x86_64)" in
            x86_64|amd64)
                arch="amd64"
                ;;
            aarch64|arm64)
                arch="arm64"
                ;;
            *)
                log_error "Unsupported architecture: $(uname -m 2>/dev/null || echo Unknown)"
                exit 1
                ;;
        esac
    fi
    
    # Determine platform string and archive type
    if [ "$os" = "linux" ]; then
        # Detect libc type for Linux
        if command -v ldd >/dev/null 2>&1 && ldd --version 2>&1 | grep -q musl; then
            platform="${os}-${arch}-musl"
        else
            platform="${os}-${arch}"
        fi
        archive_ext="tar.gz"
        binary_ext=""
    elif [ "$os" = "macos" ]; then
        platform="${os}-${arch}"
        archive_ext="tar.gz"
        binary_ext=""
    elif [ "$os" = "windows" ]; then
        platform="${os}-${arch}"
        archive_ext="zip"
        binary_ext=".exe"
    fi
    
    echo "$platform|$archive_ext|$binary_ext"
}

# Get latest release version
get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    
    log_debug "Fetching latest release from: $api_url"
    
    # Try multiple methods to get the version
    if command -v curl >/dev/null 2>&1; then
        version=$(curl -s "$api_url" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        version=$(wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    else
        log_error "Neither curl nor wget is available"
        exit 1
    fi
    
    if [ -z "$version" ] || [ "$version" = "null" ]; then
        log_error "Failed to get latest version"
        exit 1
    fi
    
    # Remove 'v' prefix if present
    version=${version#v}
    echo "$version"
}

# Download file with checksum verification
download_and_verify() {
    local url="$1"
    local output_file="$2"
    local checksum_url="$3"
    
    log_info "Downloading: $(basename "$output_file")"
    log_debug "URL: $url"
    
    # Download the file with better error handling
    if command -v curl >/dev/null 2>&1; then
        if ! curl -fsSL "$url" -o "$output_file"; then
            log_error "Failed to download from $url"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q "$url" -O "$output_file"; then
            log_error "Failed to download from $url"
            exit 1
        fi
    elif command -v powershell >/dev/null 2>&1; then
        # Windows PowerShell fallback
        if ! powershell -Command "Invoke-WebRequest -Uri '$url' -OutFile '$output_file'"; then
            log_error "Failed to download from $url"
            exit 1
        fi
    else
        log_error "No download utility available (curl, wget, or powershell)"
        exit 1
    fi
    
    # Download and verify checksum
    log_info "Verifying checksum..."
    local checksum_file="${output_file}.sha256"
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$checksum_url" -o "$checksum_file" 2>/dev/null || log_warn "Could not download checksum file"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$checksum_url" -O "$checksum_file" 2>/dev/null || log_warn "Could not download checksum file"
    elif command -v powershell >/dev/null 2>&1; then
        powershell -Command "Invoke-WebRequest -Uri '$checksum_url' -OutFile '$checksum_file'" 2>/dev/null || log_warn "Could not download checksum file"
    fi
    
    # Verify checksum if available
    if [ -f "$checksum_file" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            if ! (cd "$(dirname "$output_file")" && echo "$(cat "$checksum_file")" | sha256sum -c - >/dev/null 2>&1); then
                log_error "Checksum verification failed"
                exit 1
            fi
        elif command -v shasum >/dev/null 2>&1; then
            if ! (cd "$(dirname "$output_file")" && shasum -a 256 -c "$checksum_file" >/dev/null 2>&1); then
                log_error "Checksum verification failed"
                exit 1
            fi
        elif command -v powershell >/dev/null 2>&1; then
            # Windows PowerShell checksum verification
            local expected_hash=$(cat "$checksum_file" | cut -d' ' -f1)
            local actual_hash=$(powershell -Command "(Get-FileHash '$output_file' -Algorithm SHA256).Hash.ToLower()")
            if [ "$expected_hash" != "$actual_hash" ]; then
                log_error "Checksum verification failed"
                exit 1
            fi
        else
            log_warn "No SHA256 utility found, skipping checksum verification"
        fi
        
        if command -v sha256sum >/dev/null 2>&1 || command -v shasum >/dev/null 2>&1 || command -v powershell >/dev/null 2>&1; then
            log_info "Checksum verification passed"
        fi
    else
        log_warn "Checksum file not available, skipping verification"
    fi
}

# Extract archive
extract_archive() {
    local archive_file="$1"
    local extract_dir="$2"
    local archive_ext="$3"
    
    log_info "Extracting archive..."
    
    case "$archive_ext" in
        "tar.gz")
            if command -v tar >/dev/null 2>&1; then
                tar -xzf "$archive_file" -C "$extract_dir"
            else
                log_error "tar is required to extract .tar.gz archives"
                exit 1
            fi
            ;;
        "zip")
            if command -v unzip >/dev/null 2>&1; then
                unzip -q "$archive_file" -d "$extract_dir"
            elif command -v powershell >/dev/null 2>&1; then
                # Windows PowerShell fallback
                powershell -Command "Expand-Archive -Path '$archive_file' -DestinationPath '$extract_dir' -Force"
            else
                log_error "unzip or powershell is required to extract zip archives"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported archive format: $archive_ext"
            exit 1
            ;;
    esac
}

# Install binary
install_binary() {
    local binary_path="$1"
    local install_path="$2"
    
    log_info "Installing $BINARY_NAME to $install_path"
    
    # Create install directory if it doesn't exist
    if [ ! -d "$(dirname "$install_path")" ]; then
        log_debug "Creating directory: $(dirname "$install_path")"
        mkdir -p "$(dirname "$install_path")"
    fi
    
    # Copy binary
    cp "$binary_path" "$install_path"
    chmod +x "$install_path"
    
    log_info "Installation completed successfully!"
}

# Check if binary is in PATH
check_installation() {
    local install_path="$1"
    
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local installed_version
        installed_version=$("$BINARY_NAME" --version 2>/dev/null | head -n1 || echo "unknown")
        log_info "✓ $BINARY_NAME is available in PATH"
        log_info "Version: $installed_version"
    else
        log_warn "$BINARY_NAME is not in PATH"
        log_info "You may need to add $(dirname "$install_path") to your PATH"
        log_info "Or run: export PATH=\"$(dirname "$install_path"):\$PATH\""
    fi
}

# Main installation function
main() {
    log_info "Starting Sage installation..."
    
    # Detect platform
    local platform_info
    platform_info=$(detect_platform)
    IFS='|' read -r platform archive_ext binary_ext <<< "$platform_info"
    log_info "Detected platform: $platform"
    
    # Get version
    local version
    if [ "$VERSION" = "latest" ]; then
        version=$(get_latest_version)
        log_info "Latest version: $version"
    else
        version="$VERSION"
        log_info "Installing version: $version"
    fi
    
    # Construct URLs
    local asset_name="${BINARY_NAME}-${platform}.${archive_ext}"
    local download_url="https://github.com/$REPO/releases/download/v${version}/${asset_name}"
    local checksum_url="${download_url}.sha256"
    
    # Download and verify
    local archive_path="$TEMP_DIR/$asset_name"
    download_and_verify "$download_url" "$archive_path" "$checksum_url"
    
    # Extract
    local extract_dir="$TEMP_DIR/extract"
    mkdir -p "$extract_dir"
    extract_archive "$archive_path" "$extract_dir" "$archive_ext"
    
    # Find binary
    local binary_path="$extract_dir/${BINARY_NAME}${binary_ext}"
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found in archive: $binary_path"
        exit 1
    fi
    
    # Install
    local install_path="$INSTALL_DIR/${BINARY_NAME}${binary_ext}"
    
    # Check if we need elevated permissions
    if [ ! -w "$(dirname "$install_path")" ]; then
        if command -v sudo >/dev/null 2>&1; then
            log_info "Installing with sudo (directory not writable)"
            sudo cp "$binary_path" "$install_path"
            sudo chmod +x "$install_path"
        elif [ -n "${WINDIR:-}" ] || [ -n "${SYSTEMROOT:-}" ]; then
            # On Windows, try to install anyway (might work if running as admin)
            log_info "Attempting installation (may require administrator privileges)"
            if ! cp "$binary_path" "$install_path" 2>/dev/null; then
                log_error "Cannot write to $install_path"
                log_info "Try running as administrator or set INSTALL_DIR to a writable location"
                log_info "Example: set INSTALL_DIR=%USERPROFILE%\\bin && $0"
                exit 1
            fi
        else
            log_error "Cannot write to $install_path and sudo is not available"
            log_info "Try running with: INSTALL_DIR=\$HOME/.local/bin $0"
            exit 1
        fi
    else
        install_binary "$binary_path" "$install_path"
    fi
    
    # Verify installation
    check_installation "$install_path"
    
    log_info "🎉 Sage installation completed!"
    log_info ""
    log_info "Get started with: $BINARY_NAME --help"
    log_info "Documentation: https://github.com/$REPO"
}

# Run main function
main "$@"
