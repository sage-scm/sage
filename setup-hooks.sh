#!/usr/bin/env bash
set -euo pipefail

echo "Setting up Sage Git hooks..."

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "This script must be run inside a Git repository." >&2
    exit 1
fi

chmod +x .githooks/pre-commit .githooks/pre-push .githooks/commit-msg .githooks/prepare-commit-msg

git config --local core.hooksPath .githooks

echo "✅ Hooks installed"
cat <<'MSG'

• Pre-commit: formats staged Rust files, surfaces clippy warnings, and keeps your staged set intact.
• Commit message helpers: nudge you toward Conventional Commit wording without blocking by default.
• Pre-push: runs fast safety checks; you stay in control of whether to continue.

Re-run this script whenever the hooks change in the repo.
MSG
