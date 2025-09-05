#!/bin/bash
set -euo pipefail

# test-release-workflow.sh - Test the release workflow with various scenarios

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils.sh"

# Test configuration
TEST_REPO_DIR="/tmp/sage-release-test"
ORIGINAL_DIR="$(pwd)"

# Cleanup function
cleanup() {
    cd "$ORIGINAL_DIR"
    if [ -d "$TEST_REPO_DIR" ]; then
        rm -rf "$TEST_REPO_DIR"
    fi
}

trap cleanup EXIT

# Create test repository
setup_test_repo() {
    log_info "Setting up test repository"

    rm -rf "$TEST_REPO_DIR"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"

    git init
    git config user.name "Test User"
    git config user.email "test@example.com"

    # Create initial commit
    echo "# Test Repo" > README.md
    git add README.md
    git commit -m "chore: initial commit"

    # Copy scripts
    cp -r "$ORIGINAL_DIR/scripts" .
}

# Test scenario: Feature commits
test_feature_commits() {
    log_info "Testing feature commits scenario"

    # Add feature commits
    echo "Feature 1" >> features.txt
    git add features.txt
    git commit -m "feat: add new feature 1"

    echo "Feature 2" >> features.txt
    git add features.txt
    git commit -m "feat(ui): add new UI component"

    # Test analysis
    export GITHUB_OUTPUT=/tmp/test-github-output.txt
    > "$GITHUB_OUTPUT"
    ./scripts/release/analyze-commits.sh "0.0.0" "true" > /tmp/test-output.txt 2>&1

    if grep -q "should_release=true" "$GITHUB_OUTPUT"; then
        log_info "✓ Feature commits test passed (should_release=true)"
    else
        log_error "✗ Feature commits test failed"
        echo "Expected should_release=true in GITHUB_OUTPUT, got:"
        cat "$GITHUB_OUTPUT"
        echo "Script output:"
        cat /tmp/test-output.txt
        return 1
    fi
}

# Test scenario: Fix commits
test_fix_commits() {
    log_info "Testing fix commits scenario"

    # Reset to clean state and create a tag
    git reset --hard HEAD~2
    git tag v1.0.0

    # Add fix commits
    echo "Bug fix 1" >> fixes.txt
    git add fixes.txt
    git commit -m "fix: resolve critical bug"

    echo "Performance fix" >> fixes.txt
    git add fixes.txt
    git commit -m "perf: improve performance"

    # Test analysis
    export GITHUB_OUTPUT=/tmp/test-github-output-fix.txt
    > "$GITHUB_OUTPUT"
    ./scripts/release/analyze-commits.sh "1.0.0" "false" > /tmp/test-output.txt 2>&1

    if grep -q "should_release=true" "$GITHUB_OUTPUT" && grep -q "version_type=patch" "$GITHUB_OUTPUT"; then
        log_info "✓ Fix commits test passed"
    else
        log_error "✗ Fix commits test failed"
        echo "GITHUB_OUTPUT:"
        cat "$GITHUB_OUTPUT"
        echo "Script output:"
        cat /tmp/test-output.txt
        return 1
    fi
}

# Test scenario: Breaking changes
test_breaking_commits() {
    log_info "Testing breaking change commits scenario"

    # Reset to clean state
    git reset --hard v1.0.0

    # Add breaking change commit
    echo "Breaking change" >> breaking.txt
    git add breaking.txt
    git commit -m "feat!: major API change"

    # Test analysis
    export GITHUB_OUTPUT=/tmp/test-github-output-breaking.txt
    > "$GITHUB_OUTPUT"
    ./scripts/release/analyze-commits.sh "1.0.0" "false" > /tmp/test-output.txt 2>&1

    if grep -q "should_release=true" "$GITHUB_OUTPUT" && grep -q "version_type=major" "$GITHUB_OUTPUT"; then
        log_info "✓ Breaking change commits test passed"
    else
        log_error "✗ Breaking change commits test failed"
        echo "GITHUB_OUTPUT:"
        cat "$GITHUB_OUTPUT"
        echo "Script output:"
        cat /tmp/test-output.txt
        return 1
    fi
}

# Test scenario: Non-releasable commits
test_non_releasable_commits() {
    log_info "Testing non-releasable commits scenario"

    # Reset to clean state
    git reset --hard v1.0.0

    # Add non-releasable commits
    echo "Documentation" >> docs.txt
    git add docs.txt
    git commit -m "docs: update documentation"

    echo "Chore" >> chore.txt
    git add chore.txt
    git commit -m "chore: update dependencies"

    # Test analysis
    export GITHUB_OUTPUT=/tmp/test-github-output-nonrel.txt
    > "$GITHUB_OUTPUT"
    ./scripts/release/analyze-commits.sh "1.0.0" "false" > /tmp/test-output.txt 2>&1

    if grep -q "should_release=false" "$GITHUB_OUTPUT"; then
        log_info "✓ Non-releasable commits test passed"
    else
        log_error "✗ Non-releasable commits test failed"
        echo "GITHUB_OUTPUT:"
        cat "$GITHUB_OUTPUT"
        echo "Script output:"
        cat /tmp/test-output.txt
        return 1
    fi
}

# Test commit validation
test_commit_validation() {
    log_info "Testing commit validation"

    # Reset to clean state
    git reset --hard HEAD~2

    # Add valid commits
    echo "Valid feature" >> valid.txt
    git add valid.txt
    git commit -m "feat: add valid feature"

    # Add invalid commit
    echo "Invalid" >> invalid.txt
    git add invalid.txt
    git commit -m "invalid commit message format"

    # Test validation (should fail)
    if ./scripts/release/validate-commits.sh "HEAD~2..HEAD" 2>/dev/null; then
        log_error "✗ Commit validation test failed (should have detected invalid commit)"
        return 1
    else
        log_info "✓ Commit validation test passed (correctly detected invalid commit)"
    fi
}

# Test release notes generation
test_release_notes() {
    log_info "Testing release notes generation"

    # Reset and add mixed commits
    git reset --hard v1.0.0

    echo "Feature" >> feature.txt
    git add feature.txt
    git commit -m "feat: add awesome feature"

    echo "Fix" >> fix.txt
    git add fix.txt
    git commit -m "fix: resolve issue"

    echo "Breaking" >> breaking.txt
    git add breaking.txt
    git commit -m "feat!: breaking change"

    # Run analysis first
    export GITHUB_OUTPUT=/tmp/test-github-output-relnotes.txt
    > "$GITHUB_OUTPUT"
    ./scripts/release/analyze-commits.sh "1.0.0" "false" > /tmp/analysis-output.txt 2>&1

    # Extract values for release notes
    NEXT_VERSION=$(grep "next_version=" "$GITHUB_OUTPUT" | cut -d'=' -f2)
    VERSION_TYPE=$(grep "version_type=" "$GITHUB_OUTPUT" | cut -d'=' -f2)
    BREAKING_COUNT=$(grep "breaking_count=" "$GITHUB_OUTPUT" | cut -d'=' -f2)
    FEAT_COUNT=$(grep "feat_count=" "$GITHUB_OUTPUT" | cut -d'=' -f2)
    FIX_COUNT=$(grep "fix_count=" "$GITHUB_OUTPUT" | cut -d'=' -f2)
    OTHER_COUNT=$(grep "other_count=" "$GITHUB_OUTPUT" | cut -d'=' -f2)

    # Generate release notes
    ./scripts/release/generate-release-notes.sh "$NEXT_VERSION" "$VERSION_TYPE" "1.0.0" "$BREAKING_COUNT" "$FEAT_COUNT" "$FIX_COUNT" "$OTHER_COUNT" > /tmp/release-notes.txt

    # Check release notes content
    if grep -q "Breaking Changes" /tmp/release-notes.txt && grep -q "Features" /tmp/release-notes.txt && grep -q "Bug Fixes" /tmp/release-notes.txt; then
        log_info "✓ Release notes generation test passed"
    else
        log_error "✗ Release notes generation test failed"
        cat /tmp/release-notes.txt
        return 1
    fi
}

# Test version calculation
test_version_calculation() {
    log_info "Testing version calculation"

    # Test increment_version function
    if [ "$(increment_version "1.0.0" "major")" = "2.0.0" ] && \
       [ "$(increment_version "1.2.3" "minor")" = "1.3.0" ] && \
       [ "$(increment_version "1.2.3" "patch")" = "1.2.4" ]; then
        log_info "✓ Version calculation test passed"
    else
        log_error "✗ Version calculation test failed"
        return 1
    fi
}

# Main test runner
main() {
    log_info "Starting release workflow tests"

    setup_test_repo

    # Run all tests
    test_version_calculation
    test_feature_commits
    test_fix_commits
    test_breaking_commits
    test_non_releasable_commits
    test_commit_validation
    test_release_notes

    log_info "🎉 All tests passed!"
}

# Run tests if script is executed directly
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi
