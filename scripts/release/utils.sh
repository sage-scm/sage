#!/bin/bash
# utils.sh - Utility functions for release automation

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_debug() {
    if [ "${DEBUG:-false}" = "true" ]; then
        echo -e "${BLUE}[DEBUG]${NC} $1" >&2
    fi
}

# Check if commit is a breaking change
is_breaking_change() {
    local subject="$1"
    local body="$2"
    
    # Check for ! in type (e.g., feat!: or fix!:)
    if [[ "$subject" =~ ^[a-zA-Z]+(\(.+\))?!: ]]; then
        return 0
    fi
    
    # Check for BREAKING CHANGE in body
    if [[ "$body" =~ BREAKING[[:space:]]CHANGE ]]; then
        return 0
    fi
    
    return 1
}

# Check if commit follows conventional commit format
is_conventional_commit() {
    local subject="$1"
    
    # Standard conventional commit types
    local types="feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert"
    
    if [[ "$subject" =~ ^($types)(\(.+\))?!?: ]]; then
        return 0
    fi
    
    return 1
}

# Increment semantic version
increment_version() {
    local version="$1"
    local bump_type="$2"
    
    # Extract version components (handle pre-release and build metadata)
    local version_core=$(echo "$version" | cut -d'-' -f1 | cut -d'+' -f1)
    local major=$(echo "$version_core" | cut -d'.' -f1)
    local minor=$(echo "$version_core" | cut -d'.' -f2)
    local patch=$(echo "$version_core" | cut -d'.' -f3)
    
    case "$bump_type" in
        "major")
            echo "$((major + 1)).0.0"
            ;;
        "minor")
            echo "$major.$((minor + 1)).0"
            ;;
        "patch")
            echo "$major.$minor.$((patch + 1))"
            ;;
        *)
            echo "$version"
            ;;
    esac
}

# Validate semantic version format
is_valid_semver() {
    local version="$1"
    
    if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
        return 0
    fi
    
    return 1
}

# Get latest release version with retry logic
get_latest_release() {
    local max_retries=3
    local retry_delay=2
    
    for i in $(seq 1 $max_retries); do
        local latest=$(gh release list --limit 1 --json tagName --jq '.[0].tagName' 2>/dev/null || echo "")
        
        if [ $? -eq 0 ] && [ -n "$latest" ] && [ "$latest" != "null" ]; then
            # Clean version string (remove 'v' prefix if present)
            local clean_version=${latest#v}
            
            # Validate version format
            if is_valid_semver "$clean_version"; then
                echo "$clean_version"
                return 0
            else
                log_warn "Invalid version format: $clean_version"
            fi
        fi
        
        if [ $i -lt $max_retries ]; then
            log_warn "Attempt $i failed, retrying in ${retry_delay}s..."
            sleep $retry_delay
            retry_delay=$((retry_delay * 2))
        fi
    done
    
    log_info "No valid releases found, treating as first release"
    echo "0.0.0"
    return 1
}

# Check if release should be created
should_create_release() {
    local breaking_count="$1"
    local feat_count="$2"
    local fix_count="$3"
    
    if [ $((breaking_count + feat_count + fix_count)) -gt 0 ]; then
        return 0
    fi
    
    return 1
}

# Format commit for release notes
format_commit_for_release() {
    local subject="$1"
    local author="$2"
    local hash="$3"
    
    # Extract scope if present (simplified)
    local scope=""
    # Skip scope extraction for now to avoid regex issues
    
    # Format with author and optional hash
    if [ -n "$hash" ]; then
        echo "- $subject (@$author) [$hash]"
    else
        echo "- $subject (@$author)"
    fi
}