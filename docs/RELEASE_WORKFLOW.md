# Release Workflow Documentation

This document describes Sage's automated release system, which creates releases automatically based on conventional commits.

## Overview

Sage uses a fully automated release system that:
- Analyzes conventional commits to determine version bumps
- Generates release notes automatically
- Builds multi-platform binaries
- Creates GitHub releases with assets
- Updates the changelog
- Provides secure installation scripts

## How It Works

### Trigger Conditions

Releases are triggered automatically when:
1. Code is pushed to the `main` branch
2. Commits contain releasable changes (feat, fix, perf, or breaking changes)

### Version Calculation

Version bumps follow [Semantic Versioning](https://semver.org/):

| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `feat!:` or `BREAKING CHANGE:` | Major | 1.0.0 → 2.0.0 |
| `feat:` | Minor | 1.0.0 → 1.1.0 |
| `fix:`, `perf:` | Patch | 1.0.0 → 1.0.1 |
| `docs:`, `chore:`, etc. | None | No release |

### Precedence Rules

When multiple commit types are present, the highest precedence wins:
1. **Breaking changes** (major bump)
2. **Features** (minor bump)
3. **Fixes** (patch bump)

## Workflow Steps

### 1. Commit Analysis

The workflow analyzes all commits since the last release:

```bash
# Get commits since last release
git log v1.0.0..HEAD --pretty=format:"%H|%s|%b|%an|%ae" --no-merges

# Categorize by type
- Breaking changes: feat!:, fix!:, BREAKING CHANGE
- Features: feat:
- Fixes: fix:, perf:
- Other: docs:, chore:, style:, refactor:, test:, build:, ci:
```

### 2. Version Calculation

```bash
# Examples
Current: 1.2.3
+ feat: commit     → 1.3.0 (minor)
+ fix: commit      → 1.2.4 (patch)
+ feat!: commit    → 2.0.0 (major)
+ chore: only      → No release
```

### 3. Release Notes Generation

Release notes are automatically generated with:
- Version summary (major/minor/patch)
- Categorized changes with emojis
- Author attribution
- Installation instructions
- Changelog links

### 4. Multi-Platform Builds

Binaries are built for:
- **Linux**: x86_64 (glibc/musl), aarch64 (glibc/musl)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64, aarch64

Each binary includes:
- Compressed archive (tar.gz/zip)
- SHA256 checksum for verification
- Consistent naming: `sage-{platform}-{arch}.{ext}`

### 5. Asset Distribution

All assets are attached to the GitHub release:
```
sage-linux-amd64.tar.gz
sage-linux-amd64.tar.gz.sha256
sage-macos-arm64.tar.gz
sage-macos-arm64.tar.gz.sha256
sage-windows-amd64.zip
sage-windows-amd64.zip.sha256
...
```

## Configuration

### Release Configuration

The release system is configured via `.github/release-config.yml`:

```yaml
conventional_commits:
  types:
    feat: minor
    fix: patch
    perf: patch
  breaking_change:
    indicators: ["!", "BREAKING CHANGE"]
    bump_type: major

release_notes:
  sections:
    - title: "⚠️ Breaking Changes"
      types: ["BREAKING"]
    - title: "🚀 Features"
      types: ["feat"]
    # ...
```

### Workflow Configuration

Key workflow settings:

```yaml
# Timeouts
analyze: 10 minutes
build: 30 minutes
release: 15 minutes

# Retry logic
max_attempts: 3
backoff_factor: 2

# Triggers
- push to main
- manual dispatch
```

## Manual Operations

### Dry Run

Test the release process without creating a release:

```bash
gh workflow run auto-release.yml -f dry_run=true
```

### Manual Release

Trigger a release manually:

```bash
# Via GitHub CLI
gh workflow run auto-release.yml

# Via tag (triggers existing release workflow)
git tag v1.2.3
git push origin v1.2.3
```

### Debug Mode

Enable debug logging:

```bash
gh workflow run auto-release.yml -f debug=true
```

## Installation Methods

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/sage-scm/sage/main/install.sh | sh
```

Features:
- Platform detection (Linux/macOS/Windows)
- Architecture detection (x86_64/aarch64)
- Checksum verification
- Automatic installation to `/usr/local/bin`
- Fallback to user directory if needed

### Manual Download

1. Go to [Releases](https://github.com/sage-scm/sage/releases)
2. Download the appropriate binary for your platform
3. Verify checksum: `sha256sum -c sage-*.sha256`
4. Extract and install: `tar -xzf sage-*.tar.gz && sudo mv sage /usr/local/bin/`

### From Source

```bash
cargo install --git https://github.com/sage-scm/sage --tag v1.2.3 sage-cli
```

## Troubleshooting

### Common Issues

#### Release Not Created

**Symptoms**: Push to main doesn't trigger release

**Causes**:
- Only non-releasable commits (docs, chore, etc.)
- Invalid conventional commit format
- Workflow disabled

**Solutions**:
```bash
# Check commit format
./scripts/release/validate-commits.sh "HEAD~5..HEAD"

# Check workflow status
gh workflow list

# Manual trigger
gh workflow run auto-release.yml
```

#### Build Failures

**Symptoms**: Release created but assets missing

**Causes**:
- Cross-compilation issues
- Network timeouts
- Dependency problems

**Solutions**:
- Check workflow logs: `gh run list --workflow=auto-release.yml`
- Re-run failed jobs: `gh run rerun <run-id>`
- Manual asset upload if needed

#### Version Conflicts

**Symptoms**: Version already exists error

**Causes**:
- Concurrent releases
- Manual tag conflicts
- Workflow re-runs

**Solutions**:
```bash
# Delete conflicting tag
git tag -d v1.2.3
git push origin :refs/tags/v1.2.3

# Re-run workflow
gh workflow run auto-release.yml
```

### Validation Scripts

Test the release workflow locally:

```bash
# Test commit analysis
./scripts/release/analyze-commits.sh "1.0.0" "false"

# Test release notes generation
./scripts/release/generate-release-notes.sh "1.1.0" "minor" "1.0.0" "0" "2" "1" "3"

# Test full workflow
./scripts/release/test-release-workflow.sh

# Validate commits
./scripts/release/validate-commits.sh "HEAD~10..HEAD"
```

### Monitoring

Monitor release health:

```bash
# Check recent releases
gh release list --limit 10

# Check workflow runs
gh run list --workflow=auto-release.yml --limit 10

# Check for failed releases
gh run list --workflow=auto-release.yml --status=failure
```

## Security Considerations

### Checksum Verification

All binaries include SHA256 checksums:
```bash
# Verify download
sha256sum -c sage-linux-amd64.tar.gz.sha256
```

### Supply Chain Security

- All GitHub Actions are pinned to specific versions
- Dependencies are regularly audited
- Build process is reproducible
- Checksums prevent tampering

### Token Permissions

The workflow uses minimal required permissions:
```yaml
permissions:
  contents: write    # Create releases and tags
  actions: read      # Access workflow artifacts
  packages: read     # Access container registry
```

## Future Enhancements

Planned improvements:
- GPG signing of releases
- Container image releases
- Package manager integration (Homebrew, Chocolatey)
- Release notifications (Discord, Slack)
- Automated security scanning

## Support

For release-related issues:
1. Check this documentation
2. Search existing [Issues](https://github.com/sage-scm/sage/issues)
3. Create a new issue with the `release` label
4. Include workflow run ID and relevant logs