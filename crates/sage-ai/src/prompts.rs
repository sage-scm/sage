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

/// Prompt for generating pull request title
pub async fn pr_title_prompt(commits: Vec<String>) -> String {
    // Join the commits into a block that can be pasted into the prompt.
    let commit_log = commits.join("\n");

    // The prompt that instructs the LLM to generate the title.
    format!(
        r#"You are an assistant that converts a list of git commit messages into a single Conventional Commit pull request title.

        The input is the following commit messages, one per line:
        ```
        {commit_log}
        ```

        Follow these rules to generate the title:

        1. **Choose the most relevant commit** – prefer the most recent commit that starts with a Conventional Commit type (`feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`).
        2. **Scope** – if any commit contains a JIRA ticket reference like `ABC‑123` or `PROJ‑4567`, use that ticket as the scope: `type(scope): subject`.
        3. **Breaking changes** – if any commit mentions `BREAKING CHANGE` (case‑insensitive), add an exclamation mark after the type: `type!:`.
        4. **Subject** – use the first line of the chosen commit after the type (and optional scope), trimmed.
        5. **Lower‑case** – the entire title must be lower‑cased.
        6. **Format** – output exactly one line, e.g. `feat(scope): subject` or `fix: subject`.
        7. **Fallback** – if no commit matches the pattern, output `chore: no conventional commit found`.
        8. **Length** - Keep the title between 32-50 characters max
        9. **Relevane** - Ensure the title captures all the changes / the core of them, and not just the first commit.

        Return **only** the pull request title, no additional explanation or formatting."#
    )
}

/// Prompt for generating pull request descriptions
pub fn pr_description_prompt(title: &str, commits: Vec<String>, template: &str) -> String {
    // Join the commits into a block that can be injected into the prompt.
    let commit_log = commits.join("\n");

    // The prompt text – it changes slightly depending on whether a template is supplied.
    if template.trim().is_empty() {
        format!(
            r#"You are an assistant that writes a concise, professional GitHub pull‑request body.

            The pull‑request title is: "{title}"

            The list of commits in this PR is:
            ```
            {commit_log}
            ```

            Write a well‑structured body in Markdown that includes the following sections:

            1. **Summary** – 1–2 sentences summarizing what the PR achieves.
            2. **Problem** – a short description of the issue or feature being addressed.
            3. **Solution** – the key changes or new features introduced.
            4. **Testing** – what tests were added or updated, and any areas that might need additional review.
            5. **Breaking changes** – list any breaking changes or deployment considerations.

            The body should be clean, no extraneous commentary, and ready to be pasted directly into the GitHub PR description field. Return only the Markdown body, nothing else."#
        )
    } else {
        // A non‑empty template was supplied – we ask the LLM to replace the
        // placeholders ({{summary}}, {{details}}, etc.) with appropriate content.
        format!(
            r#"You are an assistant that fills in a pull‑request template.

            The pull‑request title is: "{title}"

            The list of commits in this PR is:
            ```
            {commit_log}
            ```

            The template is:
            ```
            {template}
            ```

            Replace the placeholders in the template with content that matches the sections
            (you can assume the placeholders are {{summary}}, {{details}}, {{testing}},
            {{breaking_changes}}, etc.).  Keep the formatting exactly as in the template.
            If there are no placeholders, utilise the headers as markers for the different content.
            Ensure you only check boxes / fill in information you actually know. Don't guess anything.
            Return **only** the completed Markdown body, no additional explanation."#
        )
    }
}
