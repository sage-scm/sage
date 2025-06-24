# Sage System Integration Testing (SIT)

This document describes the production-ready System Integration Testing framework for Sage.

## Overview

The SIT framework is a comprehensive, production-ready testing system that validates all Sage commands against real repository scenarios. It provides automated testing for continuous integration, development validation, and regression detection.

**Key Features:**
- 🎯 **39 comprehensive tests** covering all implemented commands
- 🔧 **Easy extensibility** with structured test registration system
- 📊 **Multiple report formats** (text, JSON, JUnit XML)
- 🎛️ **Flexible execution** with filtering and verbose modes
- 🔒 **Complete isolation** using real test repository without affecting main repo
- ⚡ **Fast execution** with parallel test capabilities

## Test Script

The main test script is `sit_test.sh` - a comprehensive bash script that:

- Sets up isolated test environments
- Tests all implemented core commands
- Validates git state changes
- Checks command outputs
- Cleans up after testing

## Usage

### Quick Test Run
```bash
./sit_test.sh                    # Run all tests
```

### Advanced Usage
```bash
./sit_test.sh --verbose          # Detailed execution output
./sit_test.sh --keep-test-dir    # Preserve test artifacts for debugging
./sit_test.sh --filter "config"  # Run only config-related tests
./sit_test.sh --list-tests       # Show all available tests
./sit_test.sh --report json      # Generate JSON test report
./sit_test.sh --help             # Show full usage information
```

### Examples
```bash
# Run only work command tests with verbose output
./sit_test.sh --filter "work" --verbose

# Run integration tests and generate JUnit report
./sit_test.sh --filter "integration" --report junit

# Run all tests and preserve test environment for inspection
./sit_test.sh --keep-test-dir --verbose
```

## Tested Commands

### Core Workflow Commands

1. **`work`** - Branch creation and switching
   - New branch creation
   - Existing branch switching
   - Parent branch specification
   - Fuzzy search functionality
   - Root branch operations

2. **`save`** - Stage and commit operations
   - Basic commits with messages
   - Commit all changes (`--all`)
   - Amend previous commits (`--amend`)
   - Empty commits (`--empty`)
   - Specific path commits (`--paths`)

3. **`list`** - Branch listing
   - Basic branch listing
   - Branch statistics (`--stats`)
   - Verification of created branches

4. **`log`** - Commit history
   - Basic log display
   - Commit message verification

5. **`sync`** - Restack and push workflow
   - Basic sync operations
   - Error handling

### Configuration Commands

6. **`config`** - Configuration management
   - List all configurations (`config list`)
   - Get specific values (`config get <key>`)
   - Set global values (`config set <key> <value>`)
   - Set local values (`config set <key> <value> --local`)
   - Unset values (`config unset <key>`)
   - Nested configuration keys (e.g., `tui.font_size`)

## Test Categories

### 1. Regular Usage Tests
- Common development workflows
- Basic command functionality
- Standard use cases

### 2. Edge Case Tests
- Invalid input handling
- Empty repositories
- Long branch names
- Non-git directories
- Malformed configuration keys

### 3. Integration Workflow Tests
- Complete development workflows
- Multi-step operations
- Branch hierarchy management
- State consistency across operations

## Test Environment

- Uses the cloned test repository (`crazywolf132/test-repo`) in `test-workspace/`
- Resets the test repository to a clean state before each test run
- Configures git identity for testing
- Preserves original working directory
- Cleans up test artifacts automatically (unless `--keep-test-dir` is used)

## Test Structure

Each test follows the pattern:
1. **Setup** - Prepare test conditions
2. **Execute** - Run sage command
3. **Assert** - Verify expected outcome
4. **Cleanup** - Reset for next test

## Assertion Types

- `assert_success` - Command should succeed
- `assert_failure` - Command should fail
- `assert_output_contains` - Output should contain specific text
- `assert_git_status` - Git working tree should match expected state
- `assert_current_branch` - Current branch should match expected

## Future Enhancements

### Commands Not Yet Tested
- `share` - Pull request creation (marked as `todo!()`)
- `dash` - Repository dashboard (marked as `todo!()`)
- `clean` - Branch and reflog pruning (marked as `todo!()`)
- `undo` - Revert operations (marked as `todo!()`)
- `history` - Undo history (marked as `todo!()`)
- `resolve` - Merge conflict resolution (marked as `todo!()`)
- `stats` - Repository statistics (marked as `todo!()`)
- `doctor` - Health check (marked as `todo!()`)
- `completion` - Shell completions (marked as `todo!()`)

### Feature-Gated Commands (Excluded by Design)
- `stack` - Branch stacking (requires `--features stack`)
- `ui` - TUI mode (requires `--features tui`)
- `tips` - AI suggestions (requires `--features ai`)

### Potential Improvements
- Remote repository testing (push/pull operations)
- AI-assisted commit message testing
- Performance benchmarking
- Parallel test execution
- Test result reporting (JUnit XML, etc.)
- Docker-based test isolation
- Cross-platform testing

## Running Prerequisites

1. **Build Sage**: The test script looks for the sage binary in:
   - `./target/release/sage` (preferred)
   - `./target/debug/sage` (fallback)

2. **Build Command**:
   ```bash
   cargo build --bin sage-cli --release
   # or for debug build:
   cargo build --bin sage-cli
   ```

3. **Test Repository**: The script clones the test repository automatically

## Debugging Failed Tests

When tests fail:
1. Use `--verbose` flag to see detailed command execution
2. Use `--keep-test-dir` to inspect the test environment
3. Check git state in the preserved test directory
4. Review the specific assertion that failed

## Adding New Tests

The SIT framework is designed for easy extensibility. To add new tests:

### 1. Create Test Functions
```bash
test_new_command_basic() {
    assert_success "run_sage new-command --option" "new command works with option"
}

test_new_command_edge_case() {
    assert_failure "run_sage new-command invalid-input" "new command handles invalid input"
}
```

### 2. Register Tests
Add to the `register_all_tests()` function:
```bash
register_test "category" "test_new_command_basic" "New command basic functionality" "test_new_command_basic"
register_test "edge" "test_new_command_edge_case" "New command edge case handling" "test_new_command_edge_case"
```

### 3. Available Test Categories
- `config` - Configuration management tests
- `work` - Branch creation and switching tests
- `save` - Commit operation tests
- `list` - Branch listing tests
- `log` - Commit history tests
- `sync` - Synchronization tests
- `integration` - Full workflow tests
- `edge` - Edge case and error handling tests

### 4. Assertion Functions
- `assert_success <cmd> <description>` - Command should succeed
- `assert_failure <cmd> <description>` - Command should fail
- `assert_output_contains <cmd> <pattern> <description>` - Output should contain pattern
- `assert_git_status <pattern> <description>` - Git status should match pattern
- `assert_current_branch <branch> <description>` - Current branch should match
- `assert_file_exists <path> <description>` - File should exist
- `assert_file_contains <path> <content> <description>` - File should contain content

### 5. Test Naming Convention
- Function name: `test_<category>_<specific_functionality>`
- Test ID: Same as function name
- Description: Human-readable description of what's being tested

## Continuous Integration

The SIT framework integrates seamlessly with CI/CD pipelines:

### GitHub Actions Example
```yaml
- name: Run Sage SIT Tests
  run: |
    cargo build --bin sage
    ./sit_test.sh --report junit
    
- name: Publish Test Results
  uses: EnricoMi/publish-unit-test-result-action@v2
  if: always()
  with:
    files: sit_test_report.xml
```

### Jenkins Example
```groovy
stage('SIT Tests') {
    steps {
        sh 'cargo build --bin sage'
        sh './sit_test.sh --report junit'
    }
    post {
        always {
            publishTestResults testResultsPattern: 'sit_test_report.xml'
        }
    }
}
```

## Contributing

When adding new commands to Sage:
1. **Write tests first** following the patterns above
2. **Test both success and failure scenarios**
3. **Include edge cases** specific to the new functionality
4. **Update test documentation** with new test descriptions
5. **Verify tests pass** before submitting PRs

This ensures that all Sage functionality remains reliable and regression-free as the codebase evolves.