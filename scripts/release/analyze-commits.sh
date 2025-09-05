#!/bin/bash
set -euo pipefail

# analyze-commits.sh - Analyze conventional commits for semantic versioning
# Follows semantic-release and conventional commits standards

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils.sh"

# Configuration
CONVENTIONAL_TYPES=(
    "feat:minor"
    "fix:patch"
    "perf:patch"
    "docs:none"
    "style:none"
    "refactor:none"
    "test:none"
    "build:none"
    "ci:none"
    "chore:none"
    "revert:patch"
)

# Parse command line arguments
LATEST_VERSION="${1:-0.0.0}"
IS_FIRST_RELEASE="${2:-true}"

log_info "Analyzing commits since version $LATEST_VERSION (first release: $IS_FIRST_RELEASE)"

# Get commits since last release
if [ "$IS_FIRST_RELEASE" = "true" ] || [ "$LATEST_VERSION" = "0.0.0" ]; then
    # For first release, get all commits
    COMMITS=$(git log --pretty=format:"%H|%s|%b|%an|%ae" --no-merges 2>/dev/null || echo "")
    log_debug "Getting all commits for first release"
else
    # For subsequent releases, get commits since last tag
    if git rev-parse "v$LATEST_VERSION" >/dev/null 2>&1; then
        COMMITS=$(git log "v$LATEST_VERSION..HEAD" --pretty=format:"%H|%s|%b|%an|%ae" --no-merges 2>/dev/null || echo "")
        log_debug "Getting commits since v$LATEST_VERSION"
    else
        # Fallback if tag doesn't exist
        COMMITS=$(git log --pretty=format:"%H|%s|%b|%an|%ae" --no-merges 2>/dev/null || echo "")
        log_warn "Tag v$LATEST_VERSION not found, analyzing all commits"
        IS_FIRST_RELEASE="true"
        LATEST_VERSION="0.0.0"
    fi
fi

# Handle case where git log fails or returns empty
if [ -z "$COMMITS" ]; then
    log_warn "No commits found or git log failed"
    COMMITS=""
fi

# Initialize counters
BREAKING_COUNT=0
FEAT_COUNT=0
FIX_COUNT=0
OTHER_COUNT=0
TOTAL_COUNT=0

# Create output files
mkdir -p /tmp/release-analysis
OUTPUT_DIR="/tmp/release-analysis"
> "$OUTPUT_DIR/breaking.txt"
> "$OUTPUT_DIR/feat.txt"
> "$OUTPUT_DIR/fix.txt"
> "$OUTPUT_DIR/other.txt"

# Analyze each commit
if [ -n "$COMMITS" ]; then
    while IFS='|' read -r hash subject body author email; do
        [ -z "$hash" ] && continue
        
        TOTAL_COUNT=$((TOTAL_COUNT + 1))
    
    # Format commit for output
    formatted_commit="$subject (@$author)"
    
    # Check for breaking changes (highest priority)
    if is_breaking_change "$subject" "$body"; then
        BREAKING_COUNT=$((BREAKING_COUNT + 1))
        echo "$formatted_commit" >> "$OUTPUT_DIR/breaking.txt"
        log_debug "Breaking change: $subject"
    # Check for features
    elif [[ "$subject" =~ ^feat(\(.+\))?: ]]; then
        FEAT_COUNT=$((FEAT_COUNT + 1))
        echo "$formatted_commit" >> "$OUTPUT_DIR/feat.txt"
        log_debug "Feature: $subject"
    # Check for fixes (including perf)
    elif [[ "$subject" =~ ^(fix|perf)(\(.+\))?: ]]; then
        FIX_COUNT=$((FIX_COUNT + 1))
        echo "$formatted_commit" >> "$OUTPUT_DIR/fix.txt"
        log_debug "Fix: $subject"
    # Check for other conventional commit types
    elif is_conventional_commit "$subject"; then
        OTHER_COUNT=$((OTHER_COUNT + 1))
        echo "$formatted_commit" >> "$OUTPUT_DIR/other.txt"
        log_debug "Other: $subject"
    # Non-conventional commits
    else
        OTHER_COUNT=$((OTHER_COUNT + 1))
        echo "$formatted_commit" >> "$OUTPUT_DIR/other.txt"
        log_debug "Non-conventional: $subject"
    fi
    done <<< "$COMMITS"
else
    log_info "No commits to analyze"
fi

# Determine version bump
VERSION_TYPE="none"
SHOULD_RELEASE="false"

if [ "$IS_FIRST_RELEASE" = "true" ]; then
    NEXT_VERSION="0.1.0"
    VERSION_TYPE="initial"
    SHOULD_RELEASE="true"
elif [ $BREAKING_COUNT -gt 0 ]; then
    NEXT_VERSION=$(increment_version "$LATEST_VERSION" "major")
    VERSION_TYPE="major"
    SHOULD_RELEASE="true"
elif [ $FEAT_COUNT -gt 0 ]; then
    NEXT_VERSION=$(increment_version "$LATEST_VERSION" "minor")
    VERSION_TYPE="minor"
    SHOULD_RELEASE="true"
elif [ $FIX_COUNT -gt 0 ]; then
    NEXT_VERSION=$(increment_version "$LATEST_VERSION" "patch")
    VERSION_TYPE="patch"
    SHOULD_RELEASE="true"
else
    NEXT_VERSION="$LATEST_VERSION"
    VERSION_TYPE="none"
    SHOULD_RELEASE="false"
fi

# Log analysis results
log_info "Commit analysis complete:"
log_info "  Total commits: $TOTAL_COUNT"
log_info "  Breaking changes: $BREAKING_COUNT"
log_info "  Features: $FEAT_COUNT"
log_info "  Fixes: $FIX_COUNT"
log_info "  Other: $OTHER_COUNT"
log_info "  Version type: $VERSION_TYPE"
log_info "  Should release: $SHOULD_RELEASE"
log_info "  Next version: $NEXT_VERSION"

# Output results for GitHub Actions
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
        echo "should_release=$SHOULD_RELEASE"
        echo "next_version=$NEXT_VERSION"
        echo "version_type=$VERSION_TYPE"
        echo "current_version=$LATEST_VERSION"
        echo "breaking_count=$BREAKING_COUNT"
        echo "feat_count=$FEAT_COUNT"
        echo "fix_count=$FIX_COUNT"
        echo "other_count=$OTHER_COUNT"
        echo "total_count=$TOTAL_COUNT"
    } >> "$GITHUB_OUTPUT"
fi

# Validate calculated version
if [ "$SHOULD_RELEASE" = "true" ] && ! is_valid_semver "$NEXT_VERSION"; then
    log_error "Invalid calculated version: $NEXT_VERSION"
    exit 1
fi

exit 0