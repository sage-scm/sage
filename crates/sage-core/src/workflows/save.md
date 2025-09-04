# Save

This command is used to easily and quicky save the changes in the current branch.
It has the ability to use AI to generate a commit message.

## List of scenarios

### Commit message
- No commit message and no --ai flag, or --amend or --empty. --- Exit with error.
- No commit message, but --ai flag. --- Generate ai commit message.
- No commit message, but --empty flag. --- Create empty commit.
- No commit message, but --amend flag. --- Amend previous commit.
- Commit message provided. --- Use provided commit message.
- Commit message provided with --amend flag. --- Ammend previous commit with provided message.

### Changes
- No changes to commit. --- Exit with error.
- Paths option provided, no staged changes. --- Stage provided paths.
- Paths option provided, staged changes. --- Unstage current staged changes, then stage provided paths.
- Staged changes, no unstaged or untracked changes. --- Use staged changes.
- Staged changes, unstaged or untracked changes, no --silent preference. --- Ask user to select changes to stage. 
- Staged changes, unstaged or untracked changes, with --silent preference. --- Use staged changes.
- No staged changes, has unstaged changes. --- Stage unstaged changes.
- No staged changes, has untracked files. --- Stage untracked files.
- No staged changes, has unstaged changes and untracked files. --- Ask user to select changes to stage.
- No staged changes, has unstaged changes and untracked files, with --silent preference. --- Stage unstaged
- No staged changes, has unstaged changes and untracked files, with no --silent preference. --- Ask user to select changes to stage.

### Ai Commit message

- User approves commit message. --- Use commit message.
- User rejects commit message. --- Exit with error.
- User edits commit message. --- Use edited commit message.

### Push

- No --push flag. --- Do not push.
- --push flag. --- Push to remote.

