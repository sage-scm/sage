# Release Process

This document describes the automated release process for Sage.

## Automated Releases

The release workflow is triggered by:
1. **Tag push**: Push a tag like `v1.0.0` to trigger a release
2. **Manual dispatch**: Use GitHub Actions UI to manually trigger with a version

## Features

The release workflow automatically:
- ✅ Builds binaries for all platforms (Linux, macOS, Windows)
- ✅ Creates GitHub releases with all artifacts
- ✅ Generates SHA256 checksums
- ✅ Updates Homebrew formula (if configured)

## Setting Up Homebrew Auto-Updates

To enable automatic Homebrew formula updates:

### 1. Create the Homebrew Tap Repository

Create a new repository called `sage-scm/homebrew-sage` with the following structure:
```
homebrew-sage/
└── Formula/
    └── sage.rb
```

### 2. Service Account Token

The workflow uses the organization's `SERVICE_ACCOUNT_TOKEN` secret for authentication.

Ensure the service account has:
- Write access to `sage-scm/homebrew-sage` repository
- Permission to create pull requests

If `SERVICE_ACCOUNT_TOKEN` is not available, you can create a Personal Access Token:
1. Go to [GitHub Settings > Tokens](https://github.com/settings/tokens/new)
2. Create a token with `repo` or `public_repo` scope
3. Add it as `SERVICE_ACCOUNT_TOKEN` in repository secrets

### 3. How It Works

When configured properly:
1. Each release automatically generates a Homebrew formula
2. Creates a PR to `sage-scm/homebrew-sage` with the update
3. Users can install via: `brew tap sage-scm/sage && brew install sage`

## Manual Release Process

If automatic releases fail, you can:

1. Download the `homebrew-formula` artifact from the workflow
2. Manually update the formula in `sage-scm/homebrew-sage`
3. Create a PR with the changes

## Platform Support

The workflow builds for:
- **Linux**: x86_64, ARM64 (both glibc and musl)
- **macOS**: x86_64 (Intel), ARM64 (Apple Silicon)  
- **Windows**: x86_64, ARM64

All builds exclude the AI feature by default to avoid OpenSSL dependencies.

## Troubleshooting

### Permission Denied Errors

If you see "Resource not accessible by integration":
- Ensure the workflow has `contents: write` permission
- Check that `GITHUB_TOKEN` has appropriate permissions

### Homebrew PR Creation Fails

If the Homebrew update fails:
- Verify `HOMEBREW_TAP_TOKEN` is set correctly
- Ensure the token has write access to the tap repository
- Check that `sage-scm/homebrew-sage` exists

### Build Failures

For platform-specific build issues:
- Linux musl: Requires `musl-tools`
- Linux ARM: Requires `gcc-aarch64-linux-gnu`
- Windows: Uses MSVC, no special requirements