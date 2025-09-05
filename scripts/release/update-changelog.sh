#!/bin/bash
set -euo pipefail

# update-changelog.sh - Update CHANGELOG.md with new release information
# Follows Keep a Changelog format with semantic-release integration

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
CHANGELOG_FILE="CHANGELOG.md"
TEMP_CHANGELOG="/tmp/changelog_new.md"

log_info "Updating CHANGELOG.md for version $VERSION"

# Create new changelog entry
cat > "$TEMP_CHANGELOG" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [${VERSION}] - $(date +%Y-%m-%d)

EOF

# Add version type summary
case "$VERSION_TYPE" in
    "major")
        echo "### ⚠️ BREAKING CHANGES" >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        echo "This major release contains breaking changes that may require code updates." >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        ;;
    "minor")
        echo "### ✨ New Features" >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        echo "This minor release adds new features while maintaining backward compatibility." >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        ;;
    "patch")
        echo "### 🔧 Bug Fixes" >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        echo "This patch release includes bug fixes and minor improvements." >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        ;;
    "initial")
        echo "### 🎉 Initial Release" >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        echo "Welcome to Sage! This is the first release of our Git workflow automation tool." >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        ;;
esac

# Add breaking changes
if [ "$BREAKING_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/breaking.txt" ]; then
    echo "### Changed" >> "$TEMP_CHANGELOG"
    echo "" >> "$TEMP_CHANGELOG"
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$TEMP_CHANGELOG"
    done < "$OUTPUT_DIR/breaking.txt"
    echo "" >> "$TEMP_CHANGELOG"
fi

# Add features
if [ "$FEAT_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/feat.txt" ]; then
    echo "### Added" >> "$TEMP_CHANGELOG"
    echo "" >> "$TEMP_CHANGELOG"
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$TEMP_CHANGELOG"
    done < "$OUTPUT_DIR/feat.txt"
    echo "" >> "$TEMP_CHANGELOG"
fi

# Add bug fixes
if [ "$FIX_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/fix.txt" ]; then
    echo "### Fixed" >> "$TEMP_CHANGELOG"
    echo "" >> "$TEMP_CHANGELOG"
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$TEMP_CHANGELOG"
    done < "$OUTPUT_DIR/fix.txt"
    echo "" >> "$TEMP_CHANGELOG"
fi

# Add other changes
if [ "$OTHER_COUNT" -gt 0 ] && [ -s "$OUTPUT_DIR/other.txt" ] && [ $((BREAKING_COUNT + FEAT_COUNT + FIX_COUNT)) -gt 0 ]; then
    echo "### Other" >> "$TEMP_CHANGELOG"
    echo "" >> "$TEMP_CHANGELOG"
    while IFS= read -r commit; do
        [ -n "$commit" ] && echo "- $commit" >> "$TEMP_CHANGELOG"
    done < "$OUTPUT_DIR/other.txt"
    echo "" >> "$TEMP_CHANGELOG"
fi

# Merge with existing changelog if it exists
if [ -f "$CHANGELOG_FILE" ]; then
    # Check if existing changelog has the standard header
    if grep -q "# Changelog" "$CHANGELOG_FILE"; then
        # Skip the header and merge
        tail -n +5 "$CHANGELOG_FILE" >> "$TEMP_CHANGELOG"
    else
        # Append entire existing file
        echo "" >> "$TEMP_CHANGELOG"
        echo "## Previous Changes" >> "$TEMP_CHANGELOG"
        echo "" >> "$TEMP_CHANGELOG"
        cat "$CHANGELOG_FILE" >> "$TEMP_CHANGELOG"
    fi
fi

# Replace the original changelog
mv "$TEMP_CHANGELOG" "$CHANGELOG_FILE"

log_info "CHANGELOG.md updated successfully"

# Output for GitHub Actions (if needed)
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "changelog_updated=true" >> "$GITHUB_OUTPUT"
fi