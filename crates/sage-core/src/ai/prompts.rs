//! Prompts used for AI-powered features

/// Maximum tokens that can be processed in a single request
pub const MAX_TOKENS: usize = 1_048_576;

/// Improved prompt for generating conventional commit messages, focused and streamlined for LLM reliability
pub fn commit_message_prompt(diff: &str) -> String {
    let prefix = r#"Craft concise, developer-friendly git commit messages following Conventional Commits. Analyze the code diff provided between two `---` fence markers to:
- Select ONE type: feat (new feature), fix (bug fix), docs (documentation), style (formatting), refactor (restructuring), test (tests), ci (CI/CD), or chore (maintenance).
- Include a specific, concise scope in parentheses (e.g., module, file, feature) when relevant, avoiding vague terms.
- Write a single, imperative-mood first line (30-50 characters, max 72) describing WHAT changed and WHY, clear for maintainers.
- For breaking changes, append `!` after the scope/type (e.g., `feat(api)!:` or `feat!:`) AND add a `BREAKING CHANGE:` footer after a blank line describing the breaking change.
- For generated files or unclear diffs, use `chore` with a specific description (e.g., `chore(gen): update auto-generated config`).
- For multi-file diffs, summarize the core change succinctly.
- Output only the commit message, no formatting or explanations.

Good examples:
- feat(api): add user authentication endpoint
- fix(db): correct index for faster queries
- feat(auth)!: replace token system with JWT
- refactor(sage-git)!: migrate to Git wrapper API

BREAKING CHANGE: sage-git public API now uses GitResult instead of Result; update callers accordingly
- chore(gen): update auto-generated API bindings

Bad examples (avoid):
- chore: initial commit (except first repo commit)
- feat: add stuff
- fix: bug fixed
- fix!(auth): breaking change (missing BREAKING CHANGE footer)
- refactor!(sage-git): migrate API (too vague, needs BREAKING CHANGE footer)

---
"#;

    let footer = "\n---\nOutput:";

    format!("{prefix}{diff}{footer}")
}

/// Prompt for generating pull request descriptions
pub fn pr_description_prompt(title: &str, commit_log: &str) -> String {
    format!(
        r#"You are writing a GitHub pull request description for a change with the title: "{title}".

        Here's information about the commits in this PR:
        ```
        {commit_log}
        ```

        Follow these guidelines for an effective PR description:

        1. Start with a brief summary of what this PR achieves (1-2 sentences).
        2. Explain the problem this PR solves and why it's important.
        3. Highlight key changes or new features introduced.
        4. If applicable, mention any testing performed or areas that would benefit from extra review.
        5. If there are any breaking changes, dependencies, or deployment considerations, note them.

        Format your description professionally, using proper Markdown formatting with headers and lists where appropriate.
        Be concise yet thorough - aim for clarity and completeness.

        Your response should ONLY include the PR description text, no additional explanations or comments."#
    )
}
