#!/bin/bash
set -euo pipefail

# get-latest-release.sh - Get latest release version with proper error handling

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils.sh"

log_info "Fetching latest release information"

# Get latest release with retry logic
LATEST_VERSION=$(get_latest_release)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ] && [ "$LATEST_VERSION" != "0.0.0" ]; then
    IS_FIRST_RELEASE="false"
    log_info "Found latest release: $LATEST_VERSION"
else
    IS_FIRST_RELEASE="true"
    LATEST_VERSION="0.0.0"
    log_info "No previous releases found, treating as first release"
fi

# Output for GitHub Actions
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
        echo "latest_version=$LATEST_VERSION"
        echo "is_first_release=$IS_FIRST_RELEASE"
    } >> "$GITHUB_OUTPUT"
fi

echo "Latest version: $LATEST_VERSION"
echo "Is first release: $IS_FIRST_RELEASE"