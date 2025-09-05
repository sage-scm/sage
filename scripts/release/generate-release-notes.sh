#!/bin/bash
set -euo pipefail

# generate-release-notes.sh - Generate release notes from conventional commits
# Follows semantic-release standards for release note formatting

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils.sh"

# Parse arguments
VERSION="$1"
VERSION_TYPE="$2"
CURRENT_VERSION="$3"
BREAKING_COUNT="$4"
FEAT_COUNT="$5"
FIX_COUNT="$6"
OTHER_COUNT="$7"

OUTPUT_DIR="/tmp/release-analysis"
RELEASE_NOTES_FILE="/tmp/release-notes.md"

log_info "Generating release notes for version $VERSION"

# Create release notes header
cat > "$RELEASE_NOTES_FILE" << EOF
## What's Changed in v$VERSION

EOF

# Add version type summary with semantic-release style
case "$VERSION_TYPE" in
    "major")
        cat >> "$RELEASE_NOTES_FILE" << EOF
🚨 **Major Release** - Contains breaking changes that may require code updates

EOF
        ;;
    "minor")
        cat >> "$RELEASE_NOTES_FILE" << EOF
✨ **Minor Release** - New features and improvements

EOF
        ;;
    "patch")
        cat >> "$RELEASE_NOTES_FILE" << EOF
🔧 **Patch Release** - Bug fixes and minor improvements

EOF
        ;;
    "initial")
        cat >> "$RELEASE_NOTES_FILE" << EOF
🎉 **Initial Release** - Welcome to Sage!

This is the first release of Sage, a Git workflow automation tool that burns away complexity.

EOF
        ;;
esac

# Add breaking changes first (most important)
if [ "$BREAKING_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/breaking.txt" ]; then
    cat >> "$RELEASE_NOTES_FILE" << EOF
### ⚠️ Breaking Changes

EOF
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$RELEASE_NOTES_FILE"
    done < "$OUTPUT_DIR/breaking.txt"
    echo "" >> "$RELEASE_NOTES_FILE"
fi

# Add features
if [ "$FEAT_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/feat.txt" ]; then
    cat >> "$RELEASE_NOTES_FILE" << EOF
### 🚀 Features

EOF
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$RELEASE_NOTES_FILE"
    done < "$OUTPUT_DIR/feat.txt"
    echo "" >> "$RELEASE_NOTES_FILE"
fi

# Add bug fixes
if [ "$FIX_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/fix.txt" ]; then
    cat >> "$RELEASE_NOTES_FILE" << EOF
### 🐛 Bug Fixes

EOF
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$RELEASE_NOTES_FILE"
    done < "$OUTPUT_DIR/fix.txt"
    echo "" >> "$RELEASE_NOTES_FILE"
fi

# Add other changes (only if there are releasable changes)
if [ "$OTHER_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/other.txt" ] && [ $((BREAKING_COUNT + FEAT_COUNT + FIX_COUNT)) -gt 0 ]; then
    cat >> "$RELEASE_NOTES_FILE" << EOF
### 🔧 Other Changes

EOF
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$RELEASE_NOTES_FILE"
    done < "$OUTPUT_DIR/other.txt"
    echo "" >> "$RELEASE_NOTES_FILE"
fi

# Add installation instructions
cat >> "$RELEASE_NOTES_FILE" << EOF
## Installation

### Quick Install (Recommended)
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/sage-scm/sage/main/install.sh | sh
\`\`\`

### Manual Download
Download the appropriate binary for your platform from the assets below:

- **macOS**: \`sg-macos-amd64.tar.gz\` (Intel) or \`sg-macos-arm64.tar.gz\` (Apple Silicon)
- **Linux**: \`sg-linux-amd64.tar.gz\` (glibc) or \`sg-linux-amd64-musl.tar.gz\` (musl)
- **Windows**: \`sg-windows-amd64.zip\`

All binaries include SHA256 checksums for verification.

### Build from Source
\`\`\`bash
cargo install --git https://github.com/sage-scm/sage --tag v$VERSION sage-cli
\`\`\`

EOF

# Add changelog link (if not first release)
if [ "$VERSION_TYPE" != "initial" ] && [ "$CURRENT_VERSION" != "0.0.0" ]; then
    cat >> "$RELEASE_NOTES_FILE" << EOF
**Full Changelog**: https://github.com/sage-scm/sage/compare/v$CURRENT_VERSION...v$VERSION

EOF
fi

# Output for GitHub Actions
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
        echo 'release_notes<<EOF'
        cat "$RELEASE_NOTES_FILE"
        echo 'EOF'
    } >> "$GITHUB_OUTPUT"
fi

log_info "Release notes generated successfully"
cat "$RELEASE_NOTES_FILE"