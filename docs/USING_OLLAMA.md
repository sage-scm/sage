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
sg config --key ai.api_url --value http://localhost:11434
sg config --key ai.model --value gemma3:2b
sg config --key ai.enabled --value true
```

### If you are running from source

```bash
just try config --key ai.api_url --value http://localhost:11434
just try config --key ai.model --value gemma3:2b
just try config --key ai.enabled --value true
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

## Troubleshooting

- Make sure `ollama serve` is reachable on `http://localhost:11434`.
- If Sage cannot contact Ollama, it falls back to manual commit messages.
- Toggle the feature off with `sg config --key ai.enabled --value false` (or `just try …`) if you want to opt out temporarily.
