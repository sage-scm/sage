# Using Sage with Ollama

Sage can ask a locally running [Ollama](https://ollama.com) instance to draft commit messages for you. The feature is in active development, so expect things to change while the CLI settles.

## Prerequisites

1. Install Ollama and ensure the daemon is running (`ollama serve`).
2. Pull a model you want Sage to use, for example:

   ```bash
   ollama pull gemma3:2b
   ```

   (Any compatible chat model works—adjust the name below to match.)

## Configure Sage

### If you installed `sg`

```bash
sg config --key ai.provider --value ollama
sg config --key ai.api_url --value http://localhost:11434
sg config --key ai.model --value gemma3:2b
```

### If you are running from source

```bash
just try config --key ai.provider --value ollama
just try config --key ai.api_url --value http://localhost:11434
just try config --key ai.model --value gemma3:2b
```

To inspect the current values at any time:

```bash
sg config --key ai.model
# or
just try config --key ai.model
```

Running `sg config` (or `just try config`) with no flags prints every stored key/value pair.

## Generate a Commit Message

Once you have staged changes:

```bash
sg save --ai
# or from source
just try save --ai
```

Sage will call Ollama, suggest a commit message, and let you accept or edit it before finishing the save.

## Customizing Commit Message Generation

You can provide additional instructions to guide the AI when generating commit messages:

```bash
sg config --key ai.additional_commit_prompt --value "Always include ticket numbers in the format [PROJ-123]"
# or
sg config --key ai.additional_commit_prompt --value "Use present tense and focus on the business impact"
```

This custom prompt will be appended to the base commit message generation instructions, allowing you to enforce team-specific conventions or add context.

## Troubleshooting

- Make sure `ollama serve` is reachable on `http://localhost:11434`.
- If Sage cannot contact Ollama, it falls back to manual commit messages.
