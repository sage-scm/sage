# Getting Started

> **Heads up:** This guide describes the legacy stacked-diff workflow. During the current restructuring some commands may be temporarily unavailable or behave differently.

This guide walks you through the basics of creating, saving, syncing, and sharing a stack with Sage.

## 1. Create a Stack

Begin on the branch you want to stack off of (usually `main`). Then initialise a stack:

```bash
sg stack init my-stack
```

Create a branch for your first change within the stack:

```bash
sg stack branch feature-part
```

## 2. Save Your Work

Edit files as needed. When you're ready to commit:

```bash
sg save "Describe your change"
```

Repeat edits and `sg save` as required. Each branch in the stack should contain focused commits.

## 3. Sync the Stack

To rebase your stack onto the latest `main` and push updates to the remote, run:

```bash
sg sync
```

`sync` ensures each branch is rebased and pushed in the correct order.

## 4. Share for Review

When the stack is ready, create or update pull requests:

```bash
sg share
```

Sage will open or update PRs for all branches in the stack.

## Common Pitfalls

- **History Rewrites**: `sg sync` uses rebases and force pushes to keep stacks linear. This rewrites commit hashes and may require collaborators to fetch and rebase their work.
- **Protected Branches**: Force pushes may be rejected on protected branches. Ensure your remote allows it or work on unprotected branches.
- **Existing Commits**: Initialising a stack on a branch with existing commits includes those commits in the stack. Start from a clean branch if you want only new work in your stack.
- **Diverged History**: If the remote branch has diverged, `sg sync` may fail. Fetch and resolve conflicts before syncing again.

---

With these basics, you're ready to harness Sage for streamlined stacked development.
