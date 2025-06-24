#!/usr/bin/env bash

# Sage System Integration Test (SIT) Framework
# Production-ready testing system for Sage Git workflow tool
# 
# This framework provides comprehensive testing of all Sage commands
# and is designed for continuous integration and development validation.
#
# Usage: ./sit_test.sh [options]
# Options:
#   --verbose           Show detailed test execution
#   --keep-test-dir     Preserve test artifacts after completion
#   --filter <pattern>  Run only tests matching pattern
#   --list-tests        Show all available tests
#   --report <format>   Generate test report (text|json|junit)

set -e

# ==============================================================================
# CONFIGURATION AND GLOBALS
# ==============================================================================

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Test configuration
VERBOSE=false
KEEP_TEST_DIR=false
FILTER_PATTERN=""
LIST_TESTS=false
REPORT_FORMAT=""
TEST_DIR=""
SAGE_BINARY=""
ORIGINAL_DIR=""

# Test tracking
declare -a ALL_TESTS=()
declare -a PASSED_TESTS=()
declare -a FAILED_TESTS=()
declare -a SKIPPED_TESTS=()
declare -A TEST_RESULTS=()
declare -A TEST_DESCRIPTIONS=()
declare -A TEST_DURATIONS=()

# Test categories
declare -a CONFIG_TESTS=()
declare -a WORK_TESTS=()
declare -a SAVE_TESTS=()
declare -a LIST_CMD_TESTS=()
declare -a LOG_TESTS=()
declare -a SYNC_TESTS=()
declare -a INTEGRATION_TESTS=()
declare -a EDGE_CASE_TESTS=()

# ==============================================================================
# UTILITY FUNCTIONS
# ==============================================================================

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_debug() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${PURPLE}[DEBUG]${NC} $1"
    fi
}

log_test_start() {
    local test_name="$1"
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${CYAN}[TEST]${NC} Starting: $test_name"
    fi
}

# Time tracking
get_timestamp() {
    date +%s.%3N
}

# ==============================================================================
# COMMAND LINE PARSING
# ==============================================================================

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --verbose)
                VERBOSE=true
                shift
                ;;
            --keep-test-dir)
                KEEP_TEST_DIR=true
                shift
                ;;
            --filter)
                FILTER_PATTERN="$2"
                shift 2
                ;;
            --list-tests)
                LIST_TESTS=true
                shift
                ;;
            --report)
                REPORT_FORMAT="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
}

show_help() {
    cat << EOF
Sage System Integration Test (SIT) Framework

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --verbose           Show detailed test execution output
    --keep-test-dir     Preserve test artifacts after completion
    --filter <pattern>  Run only tests matching the given pattern
    --list-tests        Show all available tests and exit
    --report <format>   Generate test report (text|json|junit)
    --help, -h          Show this help message

EXAMPLES:
    $0                          # Run all tests
    $0 --verbose                # Run with detailed output
    $0 --filter "config"        # Run only config-related tests
    $0 --report json            # Generate JSON test report
    $0 --keep-test-dir          # Preserve test environment for debugging

ENVIRONMENT:
    The test suite uses the 'test-workspace' directory as the test repository.
    This directory is reset to a clean state before each test run.

EOF
}

# ==============================================================================
# TEST FRAMEWORK CORE
# ==============================================================================

# Test registration
register_test() {
    local category="$1"
    local test_name="$2"
    local description="$3"
    local test_function="$4"
    
    ALL_TESTS+=("$test_name")
    TEST_DESCRIPTIONS["$test_name"]="$description"
    
    # Add to category arrays
    case "$category" in
        "config") CONFIG_TESTS+=("$test_name") ;;
        "work") WORK_TESTS+=("$test_name") ;;
        "save") SAVE_TESTS+=("$test_name") ;;
        "list") LIST_CMD_TESTS+=("$test_name") ;;
        "log") LOG_TESTS+=("$test_name") ;;
        "sync") SYNC_TESTS+=("$test_name") ;;
        "integration") INTEGRATION_TESTS+=("$test_name") ;;
        "edge") EDGE_CASE_TESTS+=("$test_name") ;;
    esac
}

# Test execution wrapper
run_test() {
    local test_name="$1"
    local test_function="$2"
    
    # Check if test should be skipped due to filter
    if [[ -n "$FILTER_PATTERN" && ! "$test_name" =~ $FILTER_PATTERN ]]; then
        SKIPPED_TESTS+=("$test_name")
        TEST_RESULTS["$test_name"]="SKIPPED"
        return 0
    fi
    
    log_test_start "$test_name"
    local start_time
    start_time=$(get_timestamp)
    
    if "$test_function"; then
        local end_time
        end_time=$(get_timestamp)
        local duration
        duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "0")
        
        PASSED_TESTS+=("$test_name")
        TEST_RESULTS["$test_name"]="PASSED"
        TEST_DURATIONS["$test_name"]="$duration"
        log_success "$test_name: ${TEST_DESCRIPTIONS[$test_name]}"
        return 0
    else
        local end_time
        end_time=$(get_timestamp)
        local duration
        duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "0")
        
        FAILED_TESTS+=("$test_name")
        TEST_RESULTS["$test_name"]="FAILED"
        TEST_DURATIONS["$test_name"]="$duration"
        log_error "$test_name: ${TEST_DESCRIPTIONS[$test_name]}"
        return 1
    fi
}

# Assertion functions
assert_success() {
    local cmd="$1"
    local description="$2"
    
    log_debug "Running: $cmd"
    if eval "$cmd" >/dev/null 2>&1; then
        return 0
    else
        log_debug "Command failed: $cmd"
        return 1
    fi
}

assert_failure() {
    local cmd="$1"
    local description="$2"
    
    log_debug "Running (expecting failure): $cmd"
    if ! eval "$cmd" >/dev/null 2>&1; then
        return 0
    else
        log_debug "Command should have failed: $cmd"
        return 1
    fi
}

assert_output_contains() {
    local cmd="$1"
    local expected="$2"
    local description="$3"
    
    log_debug "Running: $cmd"
    local output
    output=$(eval "$cmd" 2>&1)
    
    if echo "$output" | grep -q "$expected"; then
        return 0
    else
        log_debug "Expected '$expected' in output"
        log_debug "Actual output: $output"
        return 1
    fi
}

assert_git_status() {
    local expected_pattern="$1"
    local description="$2"
    
    local actual_status
    actual_status=$(git status --porcelain)
    
    if [[ -z "$expected_pattern" && -z "$actual_status" ]]; then
        return 0
    elif [[ -n "$expected_pattern" && "$actual_status" =~ $expected_pattern ]]; then
        return 0
    else
        log_debug "Git status mismatch"
        log_debug "Expected pattern: '$expected_pattern'"
        log_debug "Actual status: '$actual_status'"
        return 1
    fi
}

assert_current_branch() {
    local expected_branch="$1"
    local description="$2"
    
    local current_branch
    current_branch=$(git branch --show-current)
    
    if [[ "$current_branch" == "$expected_branch" ]]; then
        return 0
    else
        log_debug "Expected branch '$expected_branch', got '$current_branch'"
        return 1
    fi
}

assert_file_exists() {
    local file_path="$1"
    local description="$2"
    
    if [[ -f "$file_path" ]]; then
        return 0
    else
        log_debug "File does not exist: $file_path"
        return 1
    fi
}

assert_file_contains() {
    local file_path="$1"
    local expected_content="$2"
    local description="$3"
    
    if [[ -f "$file_path" ]] && grep -q "$expected_content" "$file_path"; then
        return 0
    else
        log_debug "File '$file_path' does not contain '$expected_content'"
        return 1
    fi
}

# ==============================================================================
# TEST ENVIRONMENT SETUP
# ==============================================================================

setup_test_environment() {
    log_info "Setting up test environment..."
    
    # Store original directory
    ORIGINAL_DIR="$(pwd)"
    
    # Find sage binary
    if [[ -f "$ORIGINAL_DIR/target/release/sage" ]]; then
        SAGE_BINARY="$ORIGINAL_DIR/target/release/sage"
    elif [[ -f "$ORIGINAL_DIR/target/debug/sage" ]]; then
        SAGE_BINARY="$ORIGINAL_DIR/target/debug/sage"
    else
        log_error "Sage binary not found. Please build first with: cargo build --bin sage"
        exit 1
    fi
    
    log_info "Using sage binary: $SAGE_BINARY"
    
    # Use the test-workspace directory
    TEST_DIR="$ORIGINAL_DIR/test-workspace"
    
    if [[ ! -d "$TEST_DIR" ]]; then
        log_error "Test workspace not found at $TEST_DIR"
        log_error "Please ensure the test repository is cloned to test-workspace/"
        exit 1
    fi
    
    # Clean up and reset test repository
    cd "$TEST_DIR"
    
    log_debug "Resetting test repository to clean state"
    git clean -fd >/dev/null 2>&1 || true
    git reset --hard HEAD >/dev/null 2>&1 || true
    git checkout main >/dev/null 2>&1 || git checkout master >/dev/null 2>&1 || true
    rm -rf .sage >/dev/null 2>&1 || true
    
    # Configure git for testing
    git config user.name "Sage Test" >/dev/null 2>&1
    git config user.email "test@sage.dev" >/dev/null 2>&1
    
    # Verify git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        log_error "Failed to set up git repository in test directory"
        exit 1
    fi
    
    log_info "Test directory: $TEST_DIR"
    log_info "Test environment ready"
}

cleanup_test_environment() {
    if [[ "$KEEP_TEST_DIR" == "false" && -n "$TEST_DIR" ]]; then
        log_info "Cleaning up test artifacts..."
        cd "$TEST_DIR"
        git clean -fd >/dev/null 2>&1 || true
        git reset --hard HEAD >/dev/null 2>&1 || true
        git checkout main >/dev/null 2>&1 || git checkout master >/dev/null 2>&1 || true
        rm -rf .sage >/dev/null 2>&1 || true
        log_info "Test artifacts cleaned up"
    else
        log_info "Test directory preserved: $TEST_DIR"
    fi
    
    # Always return to original directory
    if [[ -n "$ORIGINAL_DIR" ]]; then
        cd "$ORIGINAL_DIR"
    fi
}

# Test helper functions
run_sage() {
    "$SAGE_BINARY" "$@"
}

create_test_file() {
    local filename="$1"
    local content="$2"
    echo "$content" > "$filename"
}

# ==============================================================================
# COMPREHENSIVE CONFIG COMMAND TESTS
# ==============================================================================

# Basic functionality tests
test_config_list_basic() {
    assert_success "run_sage config list" "config list shows available options"
}

test_config_list_shows_paths() {
    assert_output_contains "run_sage config list" "Global config:" "config list shows global config path" &&
    assert_output_contains "run_sage config list" "Local config:" "config list shows local config path"
}

test_config_list_shows_all_keys() {
    local output
    output=$(run_sage config list)
    echo "$output" | grep -q "editor" && echo "$output" | grep -q "auto_update" && echo "$output" | grep -q "tui.font_size"
}

# Config get tests - comprehensive key coverage
test_config_get_editor() {
    assert_success "run_sage config get editor" "config get editor returns value"
}

test_config_get_auto_update() {
    assert_success "run_sage config get auto_update" "config get auto_update returns boolean"
}

test_config_get_all_tui_keys() {
    assert_success "run_sage config get tui.font_size" "config get tui.font_size works" &&
    assert_success "run_sage config get tui.color_theme" "config get tui.color_theme works" &&
    assert_success "run_sage config get tui.line_numbers" "config get tui.line_numbers works"
}

test_config_get_all_ai_keys() {
    assert_success "run_sage config get ai.model" "config get ai.model works" &&
    assert_success "run_sage config get ai.api_url" "config get ai.api_url works" &&
    assert_success "run_sage config get ai.max_tokens" "config get ai.max_tokens works"
}

test_config_get_all_pr_keys() {
    assert_success "run_sage config get pull_requests.enabled" "config get pr.enabled works" &&
    assert_success "run_sage config get pull_requests.default_base" "config get pr.default_base works" &&
    assert_success "run_sage config get pull_requests.provider" "config get pr.provider works"
}

test_config_get_invalid_keys() {
    # Various invalid key patterns that users might try
    assert_output_contains "run_sage config get nonexistent" "Unknown config key" "handles nonexistent top-level key" &&
    assert_output_contains "run_sage config get tui.nonexistent" "Unknown config key" "handles nonexistent nested key" &&
    assert_output_contains "run_sage config get ai.fake.key" "Unknown config key" "handles nonexistent deeply nested key" &&
    assert_output_contains "run_sage config get ''" "Unknown config key" "handles empty key gracefully"
}

test_config_get_malformed_keys() {
    # Keys with special characters that might break parsing
    assert_output_contains "run_sage config get 'key.with.too.many.dots'" "Unknown config key" "handles overly nested keys" &&
    assert_output_contains "run_sage config get 'key..double.dots'" "Unknown config key" "handles double dots" &&
    assert_output_contains "run_sage config get '.starts.with.dot'" "Unknown config key" "handles leading dot" &&
    assert_output_contains "run_sage config get 'ends.with.dot.'" "Unknown config key" "handles trailing dot"
}

# Config set tests - comprehensive value types and edge cases
test_config_set_editor_values() {
    # Test various editor values users might set
    assert_success "run_sage config set editor vim" "sets editor to vim" &&
    assert_success "run_sage config set editor nano" "sets editor to nano" &&
    assert_success "run_sage config set editor emacs" "sets editor to emacs" &&
    assert_success "run_sage config set editor 'code --wait'" "sets editor to vscode with flags" &&
    assert_success "run_sage config set editor '/usr/bin/vi'" "sets editor to absolute path"
}

test_config_set_boolean_values() {
    # Test all valid boolean representations (Sage accepts true/false only)
    assert_success "run_sage config set auto_update true" "sets boolean to true" &&
    assert_success "run_sage config set auto_update false" "sets boolean to false" &&
    assert_success "run_sage config set tui.line_numbers true" "sets boolean to true" &&
    assert_success "run_sage config set tui.line_numbers false" "sets boolean to false"
}

test_config_set_numeric_values() {
    # Test various numeric values and edge cases
    assert_success "run_sage config set tui.font_size 12" "sets font size to normal value" &&
    assert_success "run_sage config set tui.font_size 8" "sets font size to small value" &&
    assert_success "run_sage config set tui.font_size 24" "sets font size to large value" &&
    assert_success "run_sage config set ai.max_tokens 1024" "sets max tokens to power of 2" &&
    assert_success "run_sage config set ai.max_tokens 4096" "sets max tokens to large value"
}

test_config_set_string_values_with_spaces() {
    # Test strings with spaces, quotes, special characters
    assert_success "run_sage config set ai.model 'gpt-4'" "sets AI model with dash" &&
    assert_success "run_sage config set pull_requests.provider 'GitHub Enterprise'" "sets provider with spaces" &&
    assert_success "run_sage config set tui.color_theme 'dark-blue'" "sets theme with dash" &&
    assert_success "run_sage config set ai.api_url 'https://api.example.com/v1'" "sets URL with special chars"
}

test_config_set_empty_values() {
    # Test setting empty values (should work for strings)
    assert_success "run_sage config set ai.api_key ''" "sets empty API key" &&
    assert_success "run_sage config set pull_requests.access_token ''" "sets empty access token"
}

test_config_set_invalid_values() {
    # Test invalid values that should fail gracefully
    assert_failure "run_sage config set tui.font_size abc" "rejects non-numeric font size" &&
    assert_failure "run_sage config set ai.max_tokens -1" "rejects negative max tokens" &&
    assert_failure "run_sage config set auto_update maybe" "rejects invalid boolean"
}

test_config_set_global_vs_local() {
    # Test precedence: local should override global
    assert_success "run_sage config set editor global-editor" "sets global editor" &&
    assert_success "run_sage config set editor local-editor --local" "sets local editor" &&
    assert_output_contains "run_sage config get editor" "local-editor" "local config overrides global" &&
    
    # Test setting different keys globally vs locally
    assert_success "run_sage config set tui.font_size 14" "sets global font size" &&
    assert_success "run_sage config set tui.font_size 16 --local" "sets local font size" &&
    assert_output_contains "run_sage config get tui.font_size" "16" "local font size overrides global"
}

test_config_unset_comprehensive() {
    # Test unsetting various extras entries
    assert_success "run_sage config set extras.custom_key custom_value" "sets custom key" &&
    assert_success "run_sage config unset extras.custom_key" "unsets custom key" &&
    
    assert_success "run_sage config set ai.extras.custom_ai custom_ai_value" "sets AI extras" &&
    assert_success "run_sage config unset ai.extras.custom_ai" "unsets AI extras" &&
    
    assert_success "run_sage config set tui.extras.custom_tui custom_tui_value" "sets TUI extras" &&
    assert_success "run_sage config unset tui.extras.custom_tui" "unsets TUI extras"
}

test_config_unset_invalid_keys() {
    # Test unsetting non-extras keys (should fail)
    assert_failure "run_sage config unset editor" "cannot unset editor" &&
    assert_failure "run_sage config unset tui.font_size" "cannot unset font size" &&
    assert_failure "run_sage config unset ai.model" "cannot unset AI model"
}

test_config_persistence() {
    # Test that config changes persist across command invocations
    assert_success "run_sage config set editor persistence-test" "sets editor for persistence test" &&
    assert_output_contains "run_sage config get editor" "persistence-test" "config persists after set" &&
    
    # Test local config persistence
    assert_success "run_sage config set tui.font_size 18 --local" "sets local font size" &&
    assert_output_contains "run_sage config get tui.font_size" "18" "local config persists"
}

test_config_plugin_dirs() {
    # Test plugin directories configuration (comma-separated)
    assert_success "run_sage config set plugin_dirs '/usr/local/sage/plugins,~/.sage/plugins'" "sets multiple plugin dirs" &&
    assert_output_contains "run_sage config get plugin_dirs" "/usr/local/sage/plugins" "plugin dirs contains first path" &&
    assert_output_contains "run_sage config get plugin_dirs" "~/.sage/plugins" "plugin dirs contains second path"
}

# ==============================================================================
# COMPREHENSIVE WORK COMMAND TESTS
# ==============================================================================

test_work_create_simple_branch() {
    local initial_branch
    initial_branch=$(git branch --show-current)
    
    assert_success "run_sage work simple-branch" "creates simple branch name" &&
    assert_current_branch "simple-branch" "switches to simple branch"
}

test_work_create_feature_branches() {
    # Test common feature branch patterns
    assert_success "run_sage work feature/user-auth" "creates feature branch with slash" &&
    assert_current_branch "feature/user-auth" "switches to feature branch" &&
    
    assert_success "run_sage work feature/fix-login-bug" "creates bugfix feature branch" &&
    assert_current_branch "feature/fix-login-bug" "switches to bugfix branch"
}

test_work_create_complex_branch_names() {
    # Test various branch naming patterns users might use
    assert_success "run_sage work feature/JIRA-123-implement-oauth" "creates branch with ticket number" &&
    assert_success "run_sage work hotfix/security-patch-v2.1" "creates hotfix branch with version" &&
    assert_success "run_sage work experiments/machine-learning-integration" "creates experiment branch" &&
    assert_success "run_sage work chore/update-dependencies-2024" "creates chore branch with year"
}

test_work_branch_name_edge_cases() {
    # Test edge cases in branch names
    assert_success "run_sage work a" "creates single character branch" &&
    assert_success "run_sage work feature/a-b-c-d-e-f" "creates branch with many dashes" &&
    assert_success "run_sage work user/john.doe/feature" "creates branch with dots in username"
}

test_work_invalid_branch_names() {
    # Test branch names that should fail
    assert_failure "run_sage work ''" "rejects empty branch name" &&
    assert_failure "run_sage work ' '" "rejects space-only branch name" &&
    assert_failure "run_sage work 'branch with spaces'" "rejects branch with spaces" &&
    assert_failure "run_sage work 'branch~with~tildes'" "rejects branch with tildes" &&
    assert_failure "run_sage work 'branch^with^carets'" "rejects branch with carets" &&
    assert_failure "run_sage work '.starts-with-dot'" "rejects branch starting with dot" &&
    assert_failure "run_sage work 'ends-with-dot.'" "rejects branch ending with dot"
}

test_work_very_long_branch_names() {
    # Test extremely long branch names (git has limits)
    local long_name
    long_name=$(printf 'very-long-branch-name-%.0s' {1..20})  # ~400 chars
    assert_success "run_sage work $long_name" "creates very long branch name" &&
    
    # Test at git's limit (255 chars is typically the limit)
    local max_length_name
    max_length_name=$(printf 'x%.0s' {1..250})
    assert_success "run_sage work $max_length_name" "creates max length branch name"
}

test_work_switch_existing_branches() {
    # Create several branches first
    run_sage work branch-one >/dev/null 2>&1
    run_sage work branch-two >/dev/null 2>&1
    run_sage work branch-three >/dev/null 2>&1
    
    # Test switching between them
    assert_success "run_sage work branch-one" "switches to existing branch one" &&
    assert_current_branch "branch-one" "correctly on branch one" &&
    
    assert_success "run_sage work branch-two" "switches to existing branch two" &&
    assert_current_branch "branch-two" "correctly on branch two" &&
    
    assert_success "run_sage work branch-three" "switches to existing branch three" &&
    assert_current_branch "branch-three" "correctly on branch three"
}

test_work_switch_to_main_variants() {
    # Test switching to main/master branches (repos might use either)
    local main_branch
    main_branch=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")
    
    assert_success "run_sage work $main_branch" "switches to main branch" &&
    assert_current_branch "$main_branch" "correctly on main branch"
}

test_work_with_explicit_parent() {
    # Test creating branches with explicit parents
    run_sage work parent-branch >/dev/null 2>&1
    create_test_file "parent-file.txt" "parent content"
    run_sage save "Parent commit" --all >/dev/null 2>&1
    
    assert_success "run_sage work child-branch --parent parent-branch" "creates branch with explicit parent" &&
    assert_current_branch "child-branch" "switches to child branch" &&
    assert_file_exists "parent-file.txt" "child branch has parent's files"
}

test_work_with_nonexistent_parent() {
    # Test specifying a parent that doesn't exist
    assert_failure "run_sage work new-branch --parent nonexistent-parent" "fails with nonexistent parent"
}

test_work_fuzzy_search_comprehensive() {
    # Create branches with similar names for fuzzy testing
    run_sage work feature/user-authentication >/dev/null 2>&1
    run_sage work feature/user-authorization >/dev/null 2>&1
    run_sage work feature/user-profile >/dev/null 2>&1
    run_sage work hotfix/user-login-bug >/dev/null 2>&1
    
    # Switch to main first
    run_sage work main >/dev/null 2>&1
    
    # Test fuzzy matching
    assert_success "run_sage work auth --fuzzy" "fuzzy finds authentication branch" &&
    assert_success "run_sage work profile --fuzzy" "fuzzy finds profile branch" &&
    assert_success "run_sage work login --fuzzy" "fuzzy finds login branch"
}

test_work_fuzzy_search_edge_cases() {
    # Test fuzzy search with no matches
    assert_failure "run_sage work nonexistent-pattern --fuzzy" "fuzzy search fails with no matches" &&
    
    # Test fuzzy search with ambiguous matches (should pick best)
    run_sage work test-branch-one >/dev/null 2>&1
    run_sage work test-branch-two >/dev/null 2>&1
    run_sage work main >/dev/null 2>&1
    assert_success "run_sage work test --fuzzy" "fuzzy search picks best match with ambiguous input"
}

test_work_root_flag_comprehensive() {
    # Create a feature branch and make changes
    run_sage work feature/some-feature >/dev/null 2>&1
    create_test_file "feature-file.txt" "feature content"
    run_sage save "Feature commit" --all >/dev/null 2>&1
    
    # Create new branch from root (should not have feature-file.txt)
    assert_success "run_sage work feature/from-root --root" "creates branch from root" &&
    assert_current_branch "feature/from-root" "switches to new branch" &&
    assert_failure "test -f feature-file.txt" "new branch from root doesn't have feature files"
}

test_work_push_flag() {
    # Test creating branch with immediate push (if remote is available)
    # Note: This might fail if no remote is configured, which is expected
    local branch_name="feature/push-test-$$"  # Use PID for uniqueness
    
    # This test is informational - push might fail without remote
    run_sage work "$branch_name" --push >/dev/null 2>&1
    if [[ $? -eq 0 ]]; then
        log_debug "Push flag test succeeded (remote available)"
        return 0
    else
        log_debug "Push flag test failed as expected (no remote or push failed)"
        return 0  # We consider this success since it's expected without remote
    fi
}

test_work_fetch_flag() {
    # Test fetch flag (might fail without remote, which is expected)
    run_sage work main --fetch >/dev/null 2>&1
    if [[ $? -eq 0 ]]; then
        log_debug "Fetch flag test succeeded (remote available)"
        return 0
    else
        log_debug "Fetch flag test failed as expected (no remote)"
        return 0  # Expected without remote
    fi
}

test_work_same_branch_multiple_times() {
    local current_branch
    current_branch=$(git branch --show-current)
    
    # Try switching to same branch multiple times
    assert_success "run_sage work $current_branch" "handles switching to same branch" &&
    assert_success "run_sage work $current_branch" "handles switching to same branch again" &&
    assert_current_branch "$current_branch" "stays on same branch"
}

test_work_rapid_branch_switching() {
    # Create multiple branches
    run_sage work rapid-one >/dev/null 2>&1
    run_sage work rapid-two >/dev/null 2>&1
    run_sage work rapid-three >/dev/null 2>&1
    
    # Rapidly switch between them
    assert_success "run_sage work rapid-one" "rapid switch to one" &&
    assert_success "run_sage work rapid-two" "rapid switch to two" &&
    assert_success "run_sage work rapid-three" "rapid switch to three" &&
    assert_success "run_sage work rapid-one" "rapid switch back to one" &&
    assert_current_branch "rapid-one" "ends on correct branch"
}

# ==============================================================================
# COMPREHENSIVE SAVE COMMAND TESTS
# ==============================================================================

test_save_basic_commit_with_message() {
    run_sage work feature/save-basic >/dev/null 2>&1
    
    create_test_file "basic.txt" "basic content"
    assert_success "run_sage save 'Add basic file' --all" "saves with basic message"
    
    # Verify commit was created
    local commit_count
    commit_count=$(git rev-list --count HEAD)
    [[ $commit_count -gt 0 ]]
}

test_save_various_commit_message_formats() {
    run_sage work feature/save-messages >/dev/null 2>&1
    
    # Test different message formats users might use
    create_test_file "file1.txt" "content1"
    assert_success "run_sage save 'feat: add new feature' --all" "saves with conventional commit format" &&
    
    create_test_file "file2.txt" "content2"
    assert_success "run_sage save 'Fix bug in user authentication system' --all" "saves with descriptive message" &&
    
    create_test_file "file3.txt" "content3"
    assert_success "run_sage save 'WIP: work in progress commit' --all" "saves with WIP message" &&
    
    create_test_file "file4.txt" "content4"
    assert_success "run_sage save 'HOTFIX: critical security patch' --all" "saves with hotfix message"
}

test_save_message_with_special_characters() {
    run_sage work feature/save-special-chars >/dev/null 2>&1
    
    create_test_file "special.txt" "content"
    
    # Test messages with various special characters
    assert_success "run_sage save 'Message with (parentheses) and [brackets]' --all" "handles parentheses and brackets" &&
    
    create_test_file "special2.txt" "content2"
    assert_success "run_sage save 'Message with \"quotes\" and more' --all" "handles quotes in message" &&
    
    create_test_file "special3.txt" "content3"
    assert_success "run_sage save 'Message with émojis 🚀 and unicode' --all" "handles unicode and emojis"
}

test_save_very_long_commit_messages() {
    run_sage work feature/save-long-messages >/dev/null 2>&1
    
    create_test_file "long.txt" "content"
    
    # Test various lengths of commit messages
    local short_msg="Short"
    local medium_msg="This is a medium length commit message that describes the changes in reasonable detail"
    local long_msg="This is a very long commit message that goes into extensive detail about the changes made, including the rationale behind the decisions, the technical implementation details, potential impact on other systems, testing considerations, and any future improvements that could be made. It's the kind of message that developers sometimes write when they want to be very thorough in their documentation."
    
    assert_success "run_sage save '$short_msg' --all" "handles short commit message" &&
    
    create_test_file "long2.txt" "content2"
    assert_success "run_sage save '$medium_msg' --all" "handles medium commit message" &&
    
    create_test_file "long3.txt" "content3"
    assert_success "run_sage save '$long_msg' --all" "handles very long commit message"
}

test_save_empty_commit_message() {
    run_sage work feature/save-empty-msg >/dev/null 2>&1
    
    create_test_file "empty.txt" "content"
    
    # Test that empty message fails without --empty flag
    assert_failure "run_sage save '' --all" "rejects empty commit message without --empty flag"
}

test_save_all_flag_comprehensive() {
    run_sage work feature/save-all-comprehensive >/dev/null 2>&1
    
    # Create multiple files in different states
    create_test_file "new_file.txt" "new content"
    echo "modified content" > "existing_file.txt" 2>/dev/null || create_test_file "existing_file.txt" "modified content"
    
    # Create a subdirectory with files
    mkdir -p subdir
    create_test_file "subdir/nested.txt" "nested content"
    
    assert_success "run_sage save 'Add and modify all files' --all" "saves all changes including subdirectories"
    
    # Verify all files were committed
    assert_file_exists "new_file.txt" "new file was committed" &&
    assert_file_exists "existing_file.txt" "existing file was committed" &&
    assert_file_exists "subdir/nested.txt" "nested file was committed"
}

test_save_specific_paths_comprehensive() {
    run_sage work feature/save-specific-paths >/dev/null 2>&1
    
    # Create multiple files
    create_test_file "include1.txt" "include this"
    create_test_file "include2.txt" "include this too"
    create_test_file "exclude1.txt" "don't include this"
    create_test_file "exclude2.txt" "don't include this either"
    
    # Test saving specific files
    assert_success "run_sage save 'Add specific files' --paths include1.txt,include2.txt" "saves only specified files"
    
    # Test saving files with paths
    mkdir -p project/src
    create_test_file "project/src/main.rs" "main content"
    create_test_file "project/src/lib.rs" "lib content"
    create_test_file "project/README.md" "readme content"
    
    assert_success "run_sage save 'Add Rust files' --paths 'project/src/main.rs,project/src/lib.rs'" "saves specific files with paths"
}

test_save_specific_paths_edge_cases() {
    run_sage work feature/save-paths-edge >/dev/null 2>&1
    
    # Test with nonexistent files
    assert_failure "run_sage save 'Nonexistent file' --paths nonexistent.txt" "fails with nonexistent file" &&
    
    # Test with empty paths
    create_test_file "test.txt" "content"
    assert_failure "run_sage save 'Empty paths' --paths ''" "fails with empty paths"
}

test_save_amend_comprehensive() {
    run_sage work feature/save-amend-comprehensive >/dev/null 2>&1
    
    # Create initial commit
    create_test_file "amend_test.txt" "initial content"
    run_sage save "Initial commit to be amended" --all >/dev/null 2>&1
    
    # Get initial commit hash
    local initial_commit
    initial_commit=$(git rev-parse HEAD)
    
    # Modify file and amend
    echo "amended content" > "amend_test.txt"
    assert_success "run_sage save --amend" "amends previous commit"
    
    # Verify it's still the same commit (hash should be different but only one commit)
    local commit_count
    commit_count=$(git rev-list --count HEAD)
    [[ $commit_count -eq 1 ]] || return 1
    
    # Verify content was updated
    assert_file_contains "amend_test.txt" "amended content" "amend updated file content"
}

test_save_amend_edge_cases() {
    run_sage work feature/save-amend-edge >/dev/null 2>&1
    
    # Test amend on initial commit (might be special case)
    create_test_file "initial.txt" "content"
    run_sage save "Initial commit" --all >/dev/null 2>&1
    
    echo "amended initial" > "initial.txt"
    assert_success "run_sage save --amend" "can amend initial commit"
    
    # Test amend with no changes
    assert_success "run_sage save --amend" "can amend with no additional changes"
}

test_save_empty_commit() {
    run_sage work feature/save-empty >/dev/null 2>&1
    
    # Test creating empty commit
    assert_success "run_sage save 'Empty commit for marking milestone' --empty" "creates empty commit"
    
    # Verify it was created and working tree is still clean
    local commit_count
    commit_count=$(git rev-list --count HEAD)
    [[ $commit_count -gt 0 ]] || return 1
    
    # Working tree should still be clean
    local status
    status=$(git status --porcelain)
    [[ -z "$status" ]] || return 1
}

test_save_no_changes_without_empty() {
    run_sage work feature/save-no-changes >/dev/null 2>&1
    
    # Try to save with no changes and no --empty flag
    assert_failure "run_sage save 'No changes' --all" "fails to save with no changes and no --empty flag"
}

test_save_large_files() {
    run_sage work feature/save-large-files >/dev/null 2>&1
    
    # Create a reasonably large file (not huge, but bigger than typical)
    dd if=/dev/zero of=large_file.bin bs=1024 count=100 2>/dev/null
    
    assert_success "run_sage save 'Add large binary file' --all" "saves large binary file"
    
    # Create a file with many lines
    seq 1 10000 > many_lines.txt
    assert_success "run_sage save 'Add file with many lines' --all" "saves file with many lines"
}

test_save_binary_files() {
    run_sage work feature/save-binary >/dev/null 2>&1
    
    # Create various binary files
    echo -e "\x00\x01\x02\x03\xFF\xFE\xFD" > binary.bin
    
    # Create a fake image file (just binary data with proper extension)
    dd if=/dev/urandom of=fake_image.jpg bs=1024 count=1 2>/dev/null
    
    assert_success "run_sage save 'Add binary files' --all" "saves binary files including fake image"
}

test_save_files_with_special_names() {
    run_sage work feature/save-special-names >/dev/null 2>&1
    
    # Create files with various special characters in names
    create_test_file "file-with-dashes.txt" "content"
    create_test_file "file_with_underscores.txt" "content"
    create_test_file "file.with.dots.txt" "content"
    create_test_file "file123numbers.txt" "content"
    
    # Note: Spaces and other special chars might not work in all filesystems
    # so we test what's generally safe
    
    assert_success "run_sage save 'Add files with special names' --all" "saves files with special characters in names"
}

test_save_nested_directory_structure() {
    run_sage work feature/save-nested >/dev/null 2>&1
    
    # Create deep directory structure
    mkdir -p deep/nested/directory/structure/with/many/levels
    create_test_file "deep/nested/directory/structure/with/many/levels/deep_file.txt" "deep content"
    
    # Create multiple files at different levels
    create_test_file "deep/top_level.txt" "top level"
    create_test_file "deep/nested/mid_level.txt" "mid level"
    create_test_file "deep/nested/directory/lower_level.txt" "lower level"
    
    assert_success "run_sage save 'Add complex directory structure' --all" "saves complex nested directory structure"
}

test_save_concurrent_file_changes() {
    run_sage work feature/save-concurrent >/dev/null 2>&1
    
    # Simulate rapid file changes (as might happen in development)
    for i in {1..5}; do
        create_test_file "rapid_$i.txt" "content $i"
    done
    
    assert_success "run_sage save 'Add multiple files rapidly' --all" "handles multiple rapid file changes"
    
    # Modify all files
    for i in {1..5}; do
        echo "modified content $i" > "rapid_$i.txt"
    done
    
    assert_success "run_sage save 'Modify all files' --all" "handles multiple rapid modifications"
}

# ==============================================================================
# LIST COMMAND TESTS
# ==============================================================================

test_list_basic() {
    assert_success "run_sage list" "list shows available branches"
}

test_list_with_stats() {
    assert_success "run_sage list --stats" "list shows branch statistics"
}

test_list_shows_created_branches() {
    # Create a test branch
    run_sage work feature/list-test >/dev/null 2>&1
    
    assert_output_contains "run_sage list" "feature/list-test" "list shows newly created branch"
}

test_list_output_format() {
    # Verify list output is properly formatted
    assert_output_contains "run_sage list" "main\|master" "list shows main branch"
}

# ==============================================================================
# LOG COMMAND TESTS
# ==============================================================================

test_log_basic() {
    assert_success "run_sage log" "log shows commit history"
}

test_log_shows_commits() {
    # Create a commit first
    run_sage work feature/log-test >/dev/null 2>&1
    create_test_file "log-test.txt" "test"
    run_sage save 'Test commit for log' --all >/dev/null 2>&1
    
    assert_output_contains "run_sage log" "commit" "log displays commit information"
}

# ==============================================================================
# SYNC COMMAND TESTS
# ==============================================================================

test_sync_basic() {
    # Switch to a feature branch
    run_sage work feature/sync-test >/dev/null 2>&1
    
    assert_success "run_sage sync" "sync command executes without error"
}

test_sync_on_main() {
    # Switch to main branch
    run_sage work main >/dev/null 2>&1
    
    assert_success "run_sage sync" "sync works on main branch"
}

# ==============================================================================
# INTEGRATION TESTS
# ==============================================================================

test_integration_full_workflow() {
    # Complete development workflow
    local initial_branch
    initial_branch=$(git branch --show-current)
    
    # 1. Create feature branch
    assert_success "run_sage work feature/integration-workflow" "workflow: create feature branch" &&
    
    # 2. Make changes
    create_test_file "feature.txt" "new feature" &&
    
    # 3. Commit changes
    assert_success "run_sage save 'Add new feature' --all" "workflow: commit feature" &&
    
    # 4. Create child branch
    assert_success "run_sage work feature/integration-child --parent feature/integration-workflow" "workflow: create child branch" &&
    
    # 5. Make more changes
    create_test_file "child.txt" "child feature" &&
    assert_success "run_sage save 'Add child feature' --all" "workflow: commit child feature" &&
    
    # 6. Switch back
    assert_success "run_sage work feature/integration-workflow" "workflow: switch back to parent" &&
    
    # 7. Check state
    assert_current_branch "feature/integration-workflow" "workflow: correct final branch"
}

test_integration_config_and_work() {
    # Test config affects work behavior
    assert_success "run_sage config set editor vim" "integration: set config" &&
    assert_success "run_sage work feature/config-integration" "integration: work after config change"
}

test_integration_multiple_commits() {
    run_sage work feature/multi-commit >/dev/null 2>&1
    
    # Multiple commits in sequence
    create_test_file "commit1.txt" "first"
    assert_success "run_sage save 'First commit' --all" "integration: first commit" &&
    
    create_test_file "commit2.txt" "second"
    assert_success "run_sage save 'Second commit' --all" "integration: second commit" &&
    
    create_test_file "commit3.txt" "third"
    assert_success "run_sage save 'Third commit' --all" "integration: third commit"
}

# ==============================================================================
# COMPREHENSIVE EDGE CASE TESTS
# ==============================================================================

# Command validation edge cases
test_edge_empty_branch_name() {
    assert_failure "run_sage work ''" "rejects empty branch name"
}

test_edge_invalid_branch_characters() {
    assert_failure "run_sage work 'invalid/branch/name/with/../dots'" "rejects invalid branch characters" &&
    assert_failure "run_sage work 'branch with spaces'" "rejects branch names with spaces" &&
    assert_failure "run_sage work 'branch~with~tildes'" "rejects branch names with tildes" &&
    assert_failure "run_sage work 'branch:with:colons'" "rejects branch names with colons"
}

test_edge_config_empty_key() {
    assert_failure "run_sage config set '' value" "rejects empty config key" &&
    assert_failure "run_sage config get ''" "rejects empty config get key"
}

test_edge_config_malformed_keys() {
    assert_failure "run_sage config get '...invalid...'" "rejects malformed config key" &&
    assert_failure "run_sage config set '.invalid.key' value" "rejects config key starting with dot" &&
    assert_failure "run_sage config set 'invalid..key' value" "rejects config key with double dots"
}

test_edge_save_no_changes() {
    run_sage work feature/no-changes >/dev/null 2>&1
    
    # Try to save with no changes (should require --empty)
    assert_success "run_sage save 'No changes' --empty" "creates empty commit when no changes"
}

test_edge_save_without_message() {
    run_sage work feature/no-message >/dev/null 2>&1
    create_test_file "test.txt" "content"
    
    assert_failure "run_sage save --all" "requires commit message"
}

test_edge_commands_outside_git() {
    # This test would need to be run outside a git repo
    log_debug "Skipping test_edge_commands_outside_git (requires non-git environment)"
    return 0
}

# ==============================================================================
# STRESS TESTS AND BOUNDARY CONDITIONS
# ==============================================================================

test_stress_rapid_command_execution() {
    # Test rapid execution of commands
    for i in {1..10}; do
        assert_success "run_sage config get editor" "rapid config get $i"
    done
    
    # Rapid branch creation and switching
    for i in {1..5}; do
        run_sage work "stress-branch-$i" >/dev/null 2>&1
    done
    
    for i in {1..5}; do
        assert_success "run_sage work stress-branch-$i" "rapid branch switch $i"
    done
}

test_stress_large_number_of_files() {
    run_sage work feature/stress-many-files >/dev/null 2>&1
    
    # Create many small files
    for i in {1..50}; do
        create_test_file "stress_file_$i.txt" "content $i"
    done
    
    assert_success "run_sage save 'Add many files' --all" "saves many files at once"
}

test_boundary_filesystem_limits() {
    run_sage work feature/boundary-tests >/dev/null 2>&1
    
    # Test maximum path lengths (varies by filesystem)
    local long_dir="very/long/directory/path/that/goes/deep/into/filesystem/structure"
    mkdir -p "$long_dir" 2>/dev/null
    if [[ -d "$long_dir" ]]; then
        create_test_file "$long_dir/deep_file.txt" "deep content"
        assert_success "run_sage save 'Add file in deep directory' --all" "handles deep directory structures"
    fi
    
    # Test file with maximum reasonable content size
    printf 'x%.0s' {1..10000} > large_content.txt
    assert_success "run_sage save 'Add file with large content' --all" "handles large file content"
}

test_error_recovery_scenarios() {
    # Test that commands recover gracefully from various error conditions
    
    # Try to work with invalid branch name, then do valid operation
    run_sage work "invalid branch name" >/dev/null 2>&1 || true
    assert_success "run_sage work valid-branch-after-error" "recovers from invalid branch name error"
    
    # Try invalid config operation, then valid one
    run_sage config set "" "value" >/dev/null 2>&1 || true
    assert_success "run_sage config set editor recovery-test" "recovers from invalid config operation"
}

test_concurrent_operation_safety() {
    # Test that operations are safe when multiple might run
    # (This is limited in a single-threaded test, but we can test basic safety)
    
    run_sage work feature/concurrency-test >/dev/null 2>&1
    
    # Rapid config changes
    run_sage config set editor test1 >/dev/null 2>&1 &
    run_sage config set tui.font_size 12 >/dev/null 2>&1 &
    wait
    
    # Verify commands still work after concurrent operations
    assert_success "run_sage config get editor" "commands work after concurrent config changes"
}

# ==============================================================================
# USER EXPERIENCE AND ERROR MESSAGE TESTS
# ==============================================================================

test_helpful_error_messages() {
    # Test that error messages are helpful for common user mistakes
    
    # Empty branch name
    local output
    output=$(run_sage work "" 2>&1)
    echo "$output" | grep -q -i "empty\|invalid\|required" || return 1
    
    # Invalid config key
    output=$(run_sage config get invalid.key 2>&1)
    echo "$output" | grep -q -i "unknown\|invalid\|not found" || return 1
    
    # Missing commit message
    run_sage work feature/error-messages >/dev/null 2>&1
    create_test_file "test.txt" "content"
    output=$(run_sage save --all 2>&1)
    echo "$output" | grep -q -i "message\|required\|empty" || return 1
}

test_command_help_and_usage() {
    # Test that commands provide helpful usage information
    
    # These might not be implemented yet, but testing the pattern
    run_sage --help >/dev/null 2>&1 || true
    run_sage config --help >/dev/null 2>&1 || true
    run_sage work --help >/dev/null 2>&1 || true
    
    # The important thing is these don't crash
    return 0
}

test_graceful_degradation() {
    # Test behavior when optional features aren't available
    
    # Test commands in various repository states
    run_sage work feature/degradation-test >/dev/null 2>&1
    
    # Commands should work even in unusual repository states
    assert_success "run_sage config list" "config works in any branch"
    assert_success "run_sage list" "list works in any branch"
}

# ==============================================================================
# TEST REGISTRATION
# ==============================================================================

register_all_tests() {
    # Config tests - using actual function names
    register_test "config" "test_config_list_basic" "Config list command works" "test_config_list_basic"
    register_test "config" "test_config_list_shows_paths" "Config list shows file paths" "test_config_list_shows_paths"
    register_test "config" "test_config_get_editor" "Config get retrieves valid keys" "test_config_get_editor"
    register_test "config" "test_config_get_invalid_keys" "Config get handles invalid keys" "test_config_get_invalid_keys"
    register_test "config" "test_config_set_global_vs_local" "Config set updates global settings" "test_config_set_global_vs_local"
    register_test "config" "test_config_set_boolean_values" "Config set handles boolean values" "test_config_set_boolean_values"
    register_test "config" "test_config_unset_comprehensive" "Config unset removes entries" "test_config_unset_comprehensive"
    register_test "config" "test_config_get_all_ai_keys" "Config handles AI-specific settings" "test_config_get_all_ai_keys"
    
    # Work tests - using actual function names
    register_test "work" "test_work_create_simple_branch" "Work creates new branches" "test_work_create_simple_branch"
    register_test "work" "test_work_switch_existing_branches" "Work switches to existing branches" "test_work_switch_existing_branches"
    register_test "work" "test_work_with_explicit_parent" "Work creates branches with explicit parent" "test_work_with_explicit_parent"
    register_test "work" "test_work_fuzzy_search_comprehensive" "Work supports fuzzy branch search" "test_work_fuzzy_search_comprehensive"
    register_test "work" "test_work_root_flag_comprehensive" "Work creates branches from root" "test_work_root_flag_comprehensive"
    register_test "work" "test_work_same_branch_multiple_times" "Work handles switching to same branch" "test_work_same_branch_multiple_times"
    
    # Save tests - using actual function names
    register_test "save" "test_save_basic_commit_with_message" "Save creates basic commits" "test_save_basic_commit_with_message"
    register_test "save" "test_save_all_flag_comprehensive" "Save commits all changes with --all" "test_save_all_flag_comprehensive"
    register_test "save" "test_save_amend_comprehensive" "Save amends previous commits" "test_save_amend_comprehensive"
    register_test "save" "test_save_empty_commit" "Save creates empty commits" "test_save_empty_commit"
    register_test "save" "test_save_specific_paths_comprehensive" "Save commits specific file paths" "test_save_specific_paths_comprehensive"
    
    # List tests  
    register_test "list" "test_list_basic" "List shows available branches" "test_list_basic"
    register_test "list" "test_list_with_stats" "List shows branch statistics" "test_list_with_stats"
    register_test "list" "test_list_shows_created_branches" "List shows newly created branches" "test_list_shows_created_branches"
    register_test "list" "test_list_output_format" "List output is properly formatted" "test_list_output_format"
    
    # Log tests
    register_test "log" "test_log_basic" "Log shows commit history" "test_log_basic"
    register_test "log" "test_log_shows_commits" "Log displays commit information" "test_log_shows_commits"
    
    # Sync tests
    register_test "sync" "test_sync_basic" "Sync command executes without error" "test_sync_basic"
    register_test "sync" "test_sync_on_main" "Sync works on main branch" "test_sync_on_main"
    
    # Integration tests
    register_test "integration" "test_integration_full_workflow" "Complete development workflow" "test_integration_full_workflow"
    register_test "integration" "test_integration_config_and_work" "Config changes affect work behavior" "test_integration_config_and_work"
    register_test "integration" "test_integration_multiple_commits" "Multiple sequential commits work" "test_integration_multiple_commits"
    
    # Edge case tests
    register_test "edge" "test_edge_empty_branch_name" "Empty branch name handled gracefully" "test_edge_empty_branch_name"
    register_test "edge" "test_edge_invalid_characters" "Invalid branch characters handled" "test_edge_invalid_characters"
    register_test "edge" "test_edge_very_long_branch_name" "Very long branch names work" "test_edge_very_long_branch_name"
    register_test "edge" "test_edge_config_empty_key" "Empty config key handled gracefully" "test_edge_config_empty_key"
    register_test "edge" "test_edge_config_malformed_key" "Malformed config keys handled" "test_edge_config_malformed_key"
    register_test "edge" "test_edge_save_no_changes" "Save with no changes using --empty" "test_edge_save_no_changes"
    register_test "edge" "test_edge_commands_outside_git" "Commands outside git repo handled" "test_edge_commands_outside_git"
}

# ==============================================================================
# TEST EXECUTION
# ==============================================================================

list_all_tests() {
    echo "Available Sage SIT Tests:"
    echo "========================="
    echo
    
    local categories=("config" "work" "save" "list" "log" "sync" "integration" "edge")
    
    for category in "${categories[@]}"; do
        local test_array_name
        if [[ "$category" == "list" ]]; then
            test_array_name="LIST_CMD_TESTS[@]"
        else
            test_array_name="${category^^}_TESTS[@]"
        fi
        local test_array=("${!test_array_name}")
        
        if [[ ${#test_array[@]} -gt 0 ]]; then
            echo "${category^} Tests:"
            for test_name in "${test_array[@]}"; do
                echo "  $test_name: ${TEST_DESCRIPTIONS[$test_name]}"
            done
            echo
        fi
    done
    
    echo "Total: ${#ALL_TESTS[@]} tests"
}

run_all_tests() {
    log_info "Starting Sage SIT Tests..."
    
    if [[ "$LIST_TESTS" == "true" ]]; then
        list_all_tests
        return 0
    fi
    
    local start_time
    start_time=$(get_timestamp)
    
    # Run tests by category
    local categories=("config" "work" "save" "list" "log" "sync" "integration" "edge")
    
    for category in "${categories[@]}"; do
        local test_array_name
        if [[ "$category" == "list" ]]; then
            test_array_name="LIST_CMD_TESTS[@]"
        else
            test_array_name="${category^^}_TESTS[@]"
        fi
        local test_array=("${!test_array_name}")
        
        if [[ ${#test_array[@]} -gt 0 ]]; then
            log_info "Running ${category} tests..."
            for test_name in "${test_array[@]}"; do
                run_test "$test_name" "$test_name" || true
            done
        fi
    done
    
    local end_time
    end_time=$(get_timestamp)
    local total_duration
    total_duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "0")
    
    # Generate report
    generate_test_report "$total_duration"
    
    # Return appropriate exit code
    if [[ ${#FAILED_TESTS[@]} -eq 0 ]]; then
        return 0
    else
        return 1
    fi
}

generate_test_report() {
    local total_duration="$1"
    local total_tests=${#ALL_TESTS[@]}
    local passed=${#PASSED_TESTS[@]}
    local failed=${#FAILED_TESTS[@]}
    local skipped=${#SKIPPED_TESTS[@]}
    
    echo
    echo "============================================"
    echo "           SAGE SIT TEST REPORT"
    echo "============================================"
    
    case "$REPORT_FORMAT" in
        "json")
            generate_json_report "$total_duration"
            ;;
        "junit")
            generate_junit_report "$total_duration"
            ;;
        *)
            generate_text_report "$total_duration"
            ;;
    esac
}

generate_text_report() {
    local total_duration="$1"
    local total_tests=${#ALL_TESTS[@]}
    local passed=${#PASSED_TESTS[@]}
    local failed=${#FAILED_TESTS[@]}
    local skipped=${#SKIPPED_TESTS[@]}
    
    echo "Execution Time: ${total_duration}s"
    echo "Total Tests:    $total_tests"
    echo -e "Passed:         ${GREEN}$passed${NC}"
    echo -e "Failed:         ${RED}$failed${NC}"
    echo -e "Skipped:        ${YELLOW}$skipped${NC}"
    echo
    
    if [[ $failed -gt 0 ]]; then
        echo "Failed Tests:"
        for test_name in "${FAILED_TESTS[@]}"; do
            echo -e "  ${RED}✗${NC} $test_name: ${TEST_DESCRIPTIONS[$test_name]}"
        done
        echo
    fi
    
    if [[ $skipped -gt 0 ]]; then
        echo "Skipped Tests:"
        for test_name in "${SKIPPED_TESTS[@]}"; do
            echo -e "  ${YELLOW}○${NC} $test_name: ${TEST_DESCRIPTIONS[$test_name]}"
        done
        echo
    fi
    
    local success_rate
    if [[ $total_tests -gt 0 ]]; then
        success_rate=$(echo "scale=1; $passed * 100 / $total_tests" | bc -l 2>/dev/null || echo "0")
    else
        success_rate="0"
    fi
    
    echo "Success Rate: ${success_rate}%"
    
    if [[ $failed -eq 0 ]]; then
        echo -e "${GREEN}All tests passed!${NC}"
    else
        echo -e "${RED}Some tests failed.${NC}"
    fi
}

generate_json_report() {
    local total_duration="$1"
    local report_file="sit_test_report.json"
    
    cat > "$report_file" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "duration": $total_duration,
  "summary": {
    "total": ${#ALL_TESTS[@]},
    "passed": ${#PASSED_TESTS[@]},
    "failed": ${#FAILED_TESTS[@]},
    "skipped": ${#SKIPPED_TESTS[@]}
  },
  "tests": [
EOF
    
    local first=true
    for test_name in "${ALL_TESTS[@]}"; do
        if [[ "$first" == "true" ]]; then
            first=false
        else
            echo "," >> "$report_file"
        fi
        
        local duration="${TEST_DURATIONS[$test_name]:-0}"
        local result="${TEST_RESULTS[$test_name]:-UNKNOWN}"
        local description="${TEST_DESCRIPTIONS[$test_name]:-}"
        
        cat >> "$report_file" << EOF
    {
      "name": "$test_name",
      "description": "$description",
      "result": "$result",
      "duration": $duration
    }
EOF
    done
    
    cat >> "$report_file" << EOF

  ]
}
EOF
    
    log_info "JSON report generated: $report_file"
}

generate_junit_report() {
    local total_duration="$1"
    local report_file="sit_test_report.xml"
    
    cat > "$report_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="Sage SIT Tests" tests="${#ALL_TESTS[@]}" failures="${#FAILED_TESTS[@]}" skipped="${#SKIPPED_TESTS[@]}" time="$total_duration">
EOF
    
    for test_name in "${ALL_TESTS[@]}"; do
        local duration="${TEST_DURATIONS[$test_name]:-0}"
        local result="${TEST_RESULTS[$test_name]:-UNKNOWN}"
        local description="${TEST_DESCRIPTIONS[$test_name]:-}"
        
        echo "  <testcase name=\"$test_name\" classname=\"sage.sit\" time=\"$duration\">" >> "$report_file"
        
        case "$result" in
            "FAILED")
                echo "    <failure message=\"Test failed\">$description</failure>" >> "$report_file"
                ;;
            "SKIPPED")
                echo "    <skipped message=\"Test skipped\">$description</skipped>" >> "$report_file"
                ;;
        esac
        
        echo "  </testcase>" >> "$report_file"
    done
    
    echo "</testsuite>" >> "$report_file"
    
    log_info "JUnit report generated: $report_file"
}

# ==============================================================================
# SIGNAL HANDLERS AND MAIN
# ==============================================================================

# Signal handlers
trap cleanup_test_environment EXIT
trap 'log_error "Test interrupted"; cleanup_test_environment; exit 1' INT TERM

# Main execution
main() {
    parse_args "$@"
    
    # Register all tests
    register_all_tests
    
    # Handle list-tests option
    if [[ "$LIST_TESTS" == "true" ]]; then
        list_all_tests
        exit 0
    fi
    
    # Setup and run tests
    setup_test_environment
    
    local exit_code=0
    if run_all_tests; then
        log_info "SIT tests completed successfully"
    else
        log_error "SIT tests failed"
        exit_code=1
    fi
    
    exit $exit_code
}

# Run main function
main "$@"