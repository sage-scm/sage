# Contributing to Sage

Thank you for your interest in contributing to Sage! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.75+ (latest stable recommended)
- Git
- Just (command runner)

### Getting Started

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/your-username/sage.git
   cd sage
   ```

2. Install dependencies and build:
   ```bash
   just install
   ```

3. Run tests to ensure everything works:
   ```bash
   just test
   ```

## Commit Message Guidelines

Sage uses [Conventional Commits](https://www.conventionalcommits.org/) for automated versioning and changelog generation. All commits must follow this format:

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature (triggers minor version bump)
- **fix**: A bug fix (triggers patch version bump)
- **perf**: A performance improvement (triggers patch version bump)
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Breaking Changes

To indicate a breaking change, add `!` after the type or include `BREAKING CHANGE:` in the footer:

```bash
feat!: change API structure
# or
feat: add new feature

BREAKING CHANGE: This changes the public API
```

### Examples

```bash
# Feature
feat: add branch stacking support
feat(cli): add new --interactive flag

# Bug fix
fix: resolve merge conflict detection
fix(git): handle empty repositories correctly

# Breaking change
feat!: redesign CLI interface
fix!: change default behavior for sync command

# Documentation
docs: update installation instructions
docs(api): add examples for new functions

# Maintenance
chore: update dependencies
ci: add automated release workflow
```

### Validation

Commit messages are automatically validated in CI. You can validate locally:

```bash
# Validate recent commits
./scripts/validate-commits.sh

# Validate specific range
./scripts/validate-commits.sh "HEAD~5..HEAD"
```

## Development Workflow

### Branch Naming

Use descriptive branch names:
- `feat/branch-stacking` - for new features
- `fix/merge-conflicts` - for bug fixes
- `docs/contributing-guide` - for documentation
- `chore/update-deps` - for maintenance

### Making Changes

1. Create a feature branch:
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. Make your changes following the coding standards
3. Add tests for new functionality
4. Ensure all tests pass:
   ```bash
   just test
   just lint
   just fmt-check
   ```

5. Commit with conventional commit format:
   ```bash
   git commit -m "feat: add your new feature"
   ```

6. Push and create a pull request

### Code Quality

We maintain strict coding standards to ensure a beautiful, high-quality codebase:

### Core Principles

1. **Purpose-driven code**: Every line must have a clear purpose - no unnecessary abstractions or over-engineering
2. **Self-documenting**: No code comments unless absolutely necessary - code should be self-documenting through clear naming and structure
3. **Beautiful and elegant**: Code should inspire other developers - prioritize clarity over cleverness
4. **Keep it simple (KISS)**: No advanced Rust tactics - avoid complex lifetime gymnastics, exotic type system features, or clever tricks
5. **Consistent error handling**: Use `anyhow::Result` consistently throughout the codebase

### Rust-Specific Guidelines

- **Function Design**: Keep functions small and focused on a single responsibility
- **Naming**: Use full words over abbreviations (`repository` not `repo`, `save_changes()` not `save()`)
- **Error Propagation**: Use the `?` operator rather than unwrapping
- **Module Organization**: Group related functionality together with minimal public APIs

Run quality checks:
```bash
just ci  # Runs all checks (format, lint, test, docs)
```

## Testing

### Running Tests

```bash
# All tests
just test

# Specific test
just test-one "test_name"

# With output
just test-verbose

# Continuous testing
just test-watch
```

### Writing Tests

- Write unit tests for all new functionality
- Use descriptive test names
- Test both success and error cases
- Mock external dependencies

## Release Process

Sage uses automated releases based on conventional commits:

### Version Bumps

- **Major** (1.0.0 → 2.0.0): Breaking changes (`feat!:`, `fix!:`, or `BREAKING CHANGE:`)
- **Minor** (1.0.0 → 1.1.0): New features (`feat:`)
- **Patch** (1.0.0 → 1.0.1): Bug fixes (`fix:`, `perf:`)

### Release Workflow

1. Commits are pushed to `main` branch
2. GitHub Actions analyzes commits since last release
3. If releasable changes are found:
   - Version is calculated based on commit types
   - Release notes are generated
   - Binaries are built for all platforms
   - GitHub release is created with assets
   - CHANGELOG.md is updated

### Manual Release

For manual releases or hotfixes:

```bash
# Trigger release workflow manually
gh workflow run auto-release.yml

# Or create a tag
git tag v1.2.3
git push origin v1.2.3
```

## Documentation

### Code Documentation

- Use `cargo doc` comments for public APIs
- Include examples in documentation
- Keep documentation up to date with code changes

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Update installation instructions if needed

### Generating Documentation

```bash
# Generate and open docs
just docs

# Check for documentation issues
just doc-check
```

## Getting Help

- **Issues**: Report bugs or request features via [GitHub Issues](https://github.com/sage-scm/sage/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/sage-scm/sage/discussions)
- **Documentation**: Check the [Release Workflow](RELEASE_WORKFLOW.md) for release-related questions

## Code of Conduct

Please note that this project is released with a Contributor Code of Conduct. By participating in this project you agree to abide by its terms.

## License

By contributing to Sage, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

## Recognition

Contributors are recognized in:
- Release notes (automatic via commit authors)
- CONTRIBUTORS.md file
- GitHub contributors page

Thank you for contributing to Sage! 🎉

## Quick Reference

- **Repository**: https://github.com/sage-scm/sage
- **Binary name**: `sg` (not `sage`)
- **Main crate**: `sage-cli` (installs as `sg`)
- **Rust version**: 1.75+ required
- **Commit format**: Conventional commits (required for automated releases)