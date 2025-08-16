#!/bin/bash
set -euo pipefail

# validate-commits.sh - Validate conventional commit format
# Can be used in CI or as a git hook

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils.sh"

# Configuration
CONVENTIONAL_TYPES="feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert"
SCOPE_PATTERN="\([a-zA-Z0-9_-]+\)"
BREAKING_PATTERN="!"

# Parse arguments
COMMIT_RANGE="${1:-HEAD~1..HEAD}"
STRICT_MODE="${2:-false}"

log_info "Validating commits in range: $COMMIT_RANGE"

# Get commits to validate
if [ -z "$COMMIT_RANGE" ] || [ "$COMMIT_RANGE" = "" ]; then
    # If no range specified, validate all commits
    COMMITS=$(git log --pretty=format:"%H|%s" --no-merges 2>/dev/null || echo "")
else
    COMMITS=$(git log --pretty=format:"%H|%s" --no-merges "$COMMIT_RANGE" 2>/dev/null || echo "")
fi

# Handle case where no commits are found
if [ -z "$COMMITS" ]; then
    log_warn "No commits found to validate"
    COMMITS=""
fi

# Validation counters
TOTAL_COMMITS=0
VALID_COMMITS=0
INVALID_COMMITS=0
WARNINGS=0

# Arrays to store results
declare -a INVALID_COMMIT_MESSAGES=()
declare -a WARNING_MESSAGES=()

# Validate a single commit message
validate_commit_message() {
    local subject="$1"
    local hash="$2"
    local is_valid=true
    local warnings=()
    
    # Check basic conventional commit format
    if [[ ! "$subject" =~ ^($CONVENTIONAL_TYPES)($SCOPE_PATTERN)?$BREAKING_PATTERN?: ]]; then
        is_valid=false
        INVALID_COMMIT_MESSAGES+=("$hash: $subject")
    else
        # Additional validation checks
        
        # Check subject length (recommended max 50 chars for subject)
        if [ ${#subject} -gt 72 ]; then
            warnings+=("Subject line too long (${#subject} chars, max 72 recommended)")
        fi
        
        # Check for proper capitalization after colon
        if [[ "$subject" =~ :[[:space:]]*[a-z] ]]; then
            warnings+=("Description should start with lowercase letter")
        fi
        
        # Check for trailing period
        if [[ "$subject" =~ \.$  ]]; then
            warnings+=("Subject should not end with a period")
        fi
        
        # Check scope format if present
        if [[ "$subject" =~ \([^)]*[[:space:]][^)]*\) ]]; then
            warnings+=("Scope should not contain spaces")
        fi
        
        # Store warnings
        if [ ${#warnings[@]} -gt 0 ]; then
            WARNING_MESSAGES+=("$hash: $subject")
            for warning in "${warnings[@]}"; do
                WARNING_MESSAGES+=("  - $warning")
            done
            WARNINGS=$((WARNINGS + 1))
        fi
    fi
    
    if [ "$is_valid" = true ]; then
        VALID_COMMITS=$((VALID_COMMITS + 1))
        log_debug "✓ Valid: $subject"
    else
        INVALID_COMMITS=$((INVALID_COMMITS + 1))
        log_debug "✗ Invalid: $subject"
    fi
}

# Process all commits
if [ -n "$COMMITS" ]; then
    while IFS='|' read -r hash subject; do
        [ -z "$hash" ] && continue
        TOTAL_COMMITS=$((TOTAL_COMMITS + 1))
        validate_commit_message "$subject" "$hash"
    done <<< "$COMMITS"
else
    log_info "No commits to validate"
fi

# Report results
log_info "Validation complete:"
log_info "  Total commits: $TOTAL_COMMITS"
log_info "  Valid commits: $VALID_COMMITS"
log_info "  Invalid commits: $INVALID_COMMITS"
log_info "  Warnings: $WARNINGS"

# Show invalid commits
if [ $INVALID_COMMITS -gt 0 ]; then
    echo ""
    log_error "Invalid commit messages found:"
    for msg in "${INVALID_COMMIT_MESSAGES[@]}"; do
        echo "  $msg"
    done
    echo ""
    log_info "Conventional commit format: type(scope): description"
    log_info "Valid types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert"
    log_info "Examples:"
    log_info "  feat: add new feature"
    log_info "  fix(auth): resolve login issue"
    log_info "  feat!: breaking change"
    log_info "  docs: update README"
fi

# Show warnings
if [ $WARNINGS -gt 0 ] && [ "$STRICT_MODE" = "true" ]; then
    echo ""
    log_warn "Commit message warnings:"
    for msg in "${WARNING_MESSAGES[@]}"; do
        echo "  $msg"
    done
fi

# Output for GitHub Actions
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
        echo "total_commits=$TOTAL_COMMITS"
        echo "valid_commits=$VALID_COMMITS"
        echo "invalid_commits=$INVALID_COMMITS"
        echo "warnings=$WARNINGS"
        echo "validation_passed=$( [ $INVALID_COMMITS -eq 0 ] && echo "true" || echo "false" )"
    } >> "$GITHUB_OUTPUT"
fi

# Exit with error if validation failed
if [ $INVALID_COMMITS -gt 0 ]; then
    exit 1
fi

# Exit with error on warnings in strict mode
if [ "$STRICT_MODE" = "true" ] && [ $WARNINGS -gt 0 ]; then
    log_error "Validation failed due to warnings in strict mode"
    exit 1
fi

log_info "✓ All commits follow conventional commit format"
exit 0