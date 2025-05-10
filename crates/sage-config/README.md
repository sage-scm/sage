# sage-config

A flexible, layered configuration library for Sage projects.
Supports both **global user config** (in `~/.config/sage/config.toml`) and **local per-repo config** (`./.sage/config.toml`).
Automatically merges configs, allowing local options to override global ones.

---

## Features

- **Load and merge** global (user) and local (repo-specific) configuration files.
- **Write/update** config files programmatically.
- Uses TOML for user-friendliness.
- Add your own custom key-value settings!
- **Supports nested sections for TUI, AI (LLM), and Pull Request configuration.**

---

## File Locations

| Type     | Path                                       | Purpose               |
|----------|--------------------------------------------|-----------------------|
| Global   | `~/.config/sage/config.toml`               | User-wide defaults    |
| Local    | `<repo>/.sage/config.toml`                 | Project override      |

---

## Full Configuration Schema

All config values are optional and layered (local overrides global). If a field is not provided in either config file, a built-in default is used.

### Root Config Fields

| Field           | Type              | Description                                           | Default                |
|-----------------|-------------------|-------------------------------------------------------|------------------------|
| `editor`        | `String`          | Preferred text editor                                 | `"vim"`                |
| `auto_update`   | `bool`            | Automatically check for updates                       | `true`                 |
| `plugin_dirs`   | `Array[String]`   | Plugin directory paths                                | `["~/.config/sage/plugins"]` |
| `tui`           | `TuiConfig`       | Terminal UI configuration (see below)                 | See below              |
| `ai`            | `AiConfig`        | AI/LLM integration/config (see below)                 | See below              |
| `pull_requests` | `PrConfig`        | Pull request system integration (see below)           | See below              |
| `extras`        | `Table`           | Custom key-value settings (for extensions/plugins)     | `{}`                   |

### `[tui]` Section (`TuiConfig`)

| Field           | Type              | Description                                  | Default      |
|-----------------|-------------------|----------------------------------------------|--------------|
| `font_size`     | `u32`             | Font size for the TUI                        | `14`         |
| `color_theme`   | `String`          | Named color theme for the TUI                | `"default"`  |
| `line_numbers`  | `bool`            | Show line numbers                            | `true`       |
| `extras`        | `Table`           | Plugin or extension-specific TUI options     | `{}`         |

### `[ai]` Section (`AiConfig`)

| Field         | Type      | Description                                         | Default      |
|---------------|-----------|-----------------------------------------------------|--------------|
| `model`       | `String`  | AI model name or family (e.g. `"gpt-4"` / `"claude-3"`) | `"gpt-o4-mini"`  |
| `api_url`     | `String`  | API endpoint for provider                           | *unset*      |
| `api_key`     | `String`  | API key for LLM/AI provider                         | *unset*      |
| `max_tokens`  | `u32`     | Request or budget token limit                       | `4096`       |
| `extras`      | `Table`   | Arbitrary nested options for AI/LLM                 | `{}`         |

### `[pull_requests]` Section (`PrConfig`)

| Field            | Type      | Description                                             | Default     |
|------------------|-----------|---------------------------------------------------------|-------------|
| `enabled`        | `bool`    | Enable Sage-integrated pull request support             | `false`     |
| `default_base`   | `String`  | Default base branch when creating PRs                   | `"main"`    |
| `provider`       | `String`  | Provider kind or endpoint (e.g. `"github"`, URL, etc.)  | *unset*     |
| `access_token`   | `String`  | Credential for PR system (access token/app key)         | *unset*     |
| `extras`         | `Table`   | Additional pull request or VCS settings                 | `{}`        |

### `[extras]`

A table for any extra custom settings. This is supported at the root and nested within each config section.

---

## Example `config.toml`

```toml
# ~/.config/sage/config.toml
editor = "vim"
auto_update = true
plugin_dirs = ["~/.config/sage/plugins", "~/sage-extra-plugins"]

[tui]
font_size = 16
color_theme = "dracula"
line_numbers = true
[tui.extras]
compact_mode = true

[ai]
model = "gpt-4-turbo"
api_url = "https://api.custom-llm.com/v1"
api_key = "sk-xxxx"
max_tokens = 8192
[ai.extras]
temperature = 0.2

[pull_requests]
enabled = true
default_base = "main"
provider = "github"
access_token = "ghp_yyy..."
[pull_requests.extras]
auto_rebase = true

[extras]
hotkey_save = "Ctrl+S"
custom_color = "#4466dd"
```

```toml
# ./.sage/config.toml (per repo, only overrides what's present)
editor = "emacs"

[tui]
font_size = 18

[ai]
model = "claude-3"

[pull_requests]
enabled = false

[extras]
project_key = "abc123"
```

---

## How Merging Works

- Fields from `./.sage/config.toml` take precedence over those from `~/.config/sage/config.toml`.
- Unspecified values fall back to built-in defaults.
- All nested sections (`tui`, `ai`, `pull_requests`) merge recursively in the same manner.

---

## Usage

Load and update config in your Rust application:

```rust
use sage_config::{ConfigManager, Config};

let manager = ConfigManager::new(None)?;
let cfg = manager.load()?;
// Access: cfg.editor, cfg.tui, cfg.ai, cfg.pull_requests

let mut update = Config::empty();
update.editor = Some("nano".into());
manager.update(&update, true)?; // update local config (set false for global)
```

---

## Extending

- Add fields or sub-sections to the `Config` struct; local/global merging and TOML nesting just works.
- Plugins/extensions should use `extras` for arbitrary settings; you can create additional nested tables.

---

## License

MIT OR Apache-2.0

---
