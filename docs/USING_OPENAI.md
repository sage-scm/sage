# Using Sage with OpenAI GPT

Sage can talk to OpenAI-compatible APIs to draft commit messages. The integration currently targets OpenAI's GPT family by default, but it also works with compatible services that expose the same REST surface.

## Defaults at a Glance

These values come from `crates/sage-ai/src/context.rs` and are applied when no explicit configuration is provided:

| Setting | Config key | Default |
| --- | --- | --- |
| API base URL | `ai.url` | `https://api.openai.com/v1` |
| Model name | `ai.model` | `gpt-5-nano` |
| Request timeout | `ai.timeout` | `60` seconds |
| Max output tokens | `ai.max_tokens` | `2048` |
| Retry attempts | `ai.max_retries` | `1` |
| Retry delay | `ai.retry_delay_ms` | `0` ms (no delay) |

You **must** provide an API key (`ai.api_key`) before Sage can reach OpenAI. The key is trimmed automatically, so you can paste values copied with quotes or leading `=` signs (common with CI secret injection).

## Configure the Integration

### If you have `sg` installed

```bash
sg config --key ai.api_key --value sk-your-openai-key
sg config --key ai.model --value gpt-4.1-mini      # optional override
sg config --key ai.timeout --value 120             # seconds, optional
```

To target a custom-compatible endpoint (e.g., Azure OpenAI), set:

```bash
sg config --key ai.url --value https://your-endpoint.openai.azure.com/v1
```

### If you are running from source

```bash
just try config --key ai.api_key --value sk-your-openai-key
just try config --key ai.model --value gpt-4.1-mini
just try config --key ai.url --value https://api.openai.com/v1
```

You can inspect the current values at any time:

```bash
sg config --key ai.model
sg config --key ai.url
# or
just try config --key ai.model
```

Running `sg config` (or `just try config`) with no flags prints every stored key/value pair.

## Using the AI Commit Flow

After staging changes:

```bash
sg save --ai
# or from source
just try save --ai
```

Sage gathers the staged diff, crafts a prompt, and asks OpenAI for a commit message. If the model responds with a valid Conventional Commit subject line (72 characters or fewer), Sage will offer it for review before finalising the commit.

## Tuning Behaviour

- **`ai.max_tokens`** controls the upper bound on tokens returned by OpenAI. Set it lower if you prefer shorter responses.
- **`ai.max_retries` / `ai.retry_delay_ms`** let you soften transient API failures. For example:

  ```bash
  sg config --key ai.max_retries --value 3
  sg config --key ai.retry_delay_ms --value 500
  ```

- Setting `ai.timeout` to `0` removes the HTTP timeout; otherwise Sage enforces the configured duration.

## Troubleshooting

- Ensure the `ai.api_key` has sufficient quota and access to the model you request.
- If the API base URL is wrong or unreachable, Sage reports `Failed to build OpenAI-compatible client`.
- Sage rejects responses that are empty or do not match our Conventional Commit lint checks. In those cases it falls back to manual commit message entry.
