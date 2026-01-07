# Getting Started

Sage is in the middle of a restructuring. While stacked workflows are being rebuilt, the CLI already streamlines everyday Git tasks like switching branches and saving commits. This guide walks through the parts that work today so you can start using `sg` in real projects.

## What You Can Do Right Now

- Switch to an existing branch or create a new one with `sg work`.
- See where you are and what tracks behind/ahead with `sg list`.
- Stage changes, generate (optionally AI-assisted) messages, and create commits with `sg save`.
- Inspect the linear history with `sg log`.

Stack-aware operations—automatic restacking, submitting stacks, and share flows—are temporarily unavailable. They will return as soon as they are ready.

## 1. Verify Your Installation

1. Install Sage using the method from the README (script, local build, or package manager).
2. Confirm the binary is on your `PATH`:

   ```bash
   sg --version
   ```

If the command prints a version (for example `sg 0.4.0-dev`), you are ready to go.

## 2. Quick Demo: Branch → Change → Commit

Follow this short walkthrough inside any Git repository.

1. **Check your starting point.**

   ```bash
   sg list
   ```

   The current branch is highlighted, and arrows show how many commits you are ahead (`↑`) or behind (`↓`) the remote default branch.

2. **Create or switch to a feature branch.**

   ```bash
   sg work feature/getting-started-demo
   ```

   - If the branch already exists locally, `sg` switches to it.
   - If it does not exist, `sg` creates it and checks it out.
   - Add `--push` to set the upstream immediately, `--root` to force the new branch to base on the repository default branch, `--parent other-branch` to choose a different parent branch, or `--fuzzy` to fuzzy-match branch names when you are unsure of the exact spelling.

3. **Make your code changes.**

   Edit files as usual. You can optionally restrict the next step to certain paths by passing `--paths`.

4. **Save a commit.**

   ```bash
   sg save "Explain what changed"
   ```

   This stages unstaged changes, prompts for (or uses your provided) commit message, creates the commit, and prints the short SHA. Add `--amend` to fix up the previous commit, `--ai` to ask your configured model for a draft message (see the AI setup guides linked below), or `--push` to push immediately after the commit. For example, once AI is configured:

   ```bash
   sg save --ai
   ```

5. **Review recent history (optional).**

   ```bash
   sg log --limit 5
   ```

   This shows a concise, colorful summary of the last few commits on the current branch.

## 3. Next Steps

- Need the new branch to start from the default branch instead of your current feature branch? Use `sg work new-branch --root`.
- Want to anchor a feature branch to another local branch? Use `sg work new-branch --parent other-branch`.
- Configure AI-assisted commit messages by following [docs/USING_OPENAI.md](USING_OPENAI.md).

That is everything you need while we finish rebuilding stack support. Feedback is welcome—every `sg save` helps shape the next release.
