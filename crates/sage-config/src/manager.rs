use crate::{Config, ConfigError};
use dirs::home_dir;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// The filename used for the config file.
const CONFIG_FILENAME: &str = "config.toml";

/// Directory in $HOME/.config for global configuration.
const GLOBAL_CONFIG_SUBDIR: &str = ".config/sage";

/// Directory at repo root (or current directory) for local configuration.
const LOCAL_CONFIG_SUBDIR: &str = ".sage";

/// Manages reading, merging, updating, and saving Sage configuration.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Path to the global config file (e.g. ~/.config/sage/config.toml)
    global_path: PathBuf,
    /// Path to the local config file (e.g. ./sage/config.toml)
    local_path: PathBuf,
}

impl ConfigManager {
    /// Constructs a new ConfigManager with default locations.
    /// - `repo_root` determines the starting directory for local (per-repo) config.
    ///   Defaults to current directory if None.
    pub fn new() -> Result<Self, ConfigError> {
        let home = home_dir().ok_or(ConfigError::ConfigPathNotFound)?;

        let global_path = home.join(GLOBAL_CONFIG_SUBDIR).join(CONFIG_FILENAME);

        let repo_root = sage_git::repo::get_repo_root()?;
        // We will get the local path from thje git module
        let local_path = PathBuf::from(repo_root)
            .join(LOCAL_CONFIG_SUBDIR)
            .join(CONFIG_FILENAME);

        Ok(ConfigManager {
            global_path,
            local_path,
        })
    }

    /// Loads and merges the global and local configurations, using defaults for missing values.
    pub fn load(&self) -> Result<Config, ConfigError> {
        let mut config = Config::default();
        if let Ok(global_cfg) = self.read_config(&self.global_path) {
            config.merge_from(&global_cfg);
        }
        if let Ok(local_cfg) = self.read_config(&self.local_path) {
            config.merge_from(&local_cfg);
        }
        Ok(config)
    }

    /// Returns the path for the current global config file.
    pub fn global_config_path(&self) -> &Path {
        &self.global_path
    }

    /// Returns the path for the current local config file.
    pub fn local_config_path(&self) -> &Path {
        &self.local_path
    }

    /// Updates the config, writing changes to either global or local file.
    ///
    /// If `local` is true, the changes update the local (repo) config, otherwise global.
    /// Only non-None values from `update` will be persisted.
    pub fn update(&self, update: &Config, local: bool) -> Result<(), ConfigError> {
        let path = if local {
            &self.local_path
        } else {
            &self.global_path
        };
        let mut config = self.read_config(path).unwrap_or_else(|_| Config::default());
        // Overwrite fields directly
        config.editor = update.editor.clone();
        config.auto_update = update.auto_update;
        config.plugin_dirs = update.plugin_dirs.clone();
        config.save = update.save.clone();
        config.tui = update.tui.clone();
        config.ai = update.ai.clone();
        config.pull_requests = update.pull_requests.clone();
        config.extras.extend(update.extras.clone());
        self.save_config(path, &config)
    }

    /// Save the entire config to a given location.
    /// Will create parent directories if needed.
    pub fn save_config(&self, target: &Path, config: &Config) -> Result<(), ConfigError> {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(config)?;
        let mut file = fs::File::create(target)?;
        file.write_all(toml_str.as_bytes())?;
        Ok(())
    }

    /// Reads a config from a file, returning an error if not present.
    fn read_config(&self, path: &Path) -> Result<Config, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::ConfigPathNotFound);
        }
        let mut buf = String::new();
        fs::File::open(path)?.read_to_string(&mut buf)?;
        let config: Config = toml::from_str(&buf)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn merge_and_save_load_cycle() {
        let dir = tempdir().unwrap();
        let global_dir = dir.path().join("user/.config/sage");
        let _ = fs::create_dir_all(&global_dir);

        let global_path = global_dir.join(CONFIG_FILENAME);

        // Setup initial global config.
        let mut global_cfg = Config::default();
        global_cfg.editor = "nano".into();
        let toml_str = toml::to_string_pretty(&global_cfg).unwrap();
        fs::write(&global_path, toml_str).unwrap();

        // Local config setup.
        let local_dir = dir.path().join("repo/.sage");
        let _ = fs::create_dir_all(&local_dir);
        let local_path = local_dir.join(CONFIG_FILENAME);
        let mut local_cfg = Config::default();
        local_cfg.editor = "emacs".into();
        let toml_str = toml::to_string_pretty(&local_cfg).unwrap();
        fs::write(&local_path, toml_str).unwrap();

        // Point manager at our test paths
        let manager = ConfigManager {
            global_path: global_path.clone(),
            local_path: local_path.clone(),
        };

        // Load config: local.editor overrides global.editor
        let cfg = manager.load().unwrap();
        assert_eq!(cfg.editor, "emacs".to_string());

        // Update local config
        let mut update = Config::default();
        update.editor = "vim".to_string();
        manager.update(&update, true).unwrap();
        let updated_cfg = manager.read_config(&local_path).unwrap();
        assert_eq!(updated_cfg.editor, "vim".to_string());
    }

    #[test]
    fn boolean_merge_preserves_global_config() {
        let dir = tempdir().unwrap();
        let global_dir = dir.path().join("user/.config/sage");
        let _ = fs::create_dir_all(&global_dir);

        let global_path = global_dir.join(CONFIG_FILENAME);

        // Setup global config with auto_update = false (non-default)
        let global_cfg_str = r#"
editor = "nano"
auto_update = false

[tui]
line_numbers = false
"#;
        fs::write(&global_path, global_cfg_str).unwrap();

        // Local config with only editor specified (auto_update will get default true)
        let local_dir = dir.path().join("repo/.sage");
        let _ = fs::create_dir_all(&local_dir);
        let local_path = local_dir.join(CONFIG_FILENAME);
        let local_cfg_str = r#"
editor = "vim"
"#;
        fs::write(&local_path, local_cfg_str).unwrap();

        // Point manager at our test paths
        let manager = ConfigManager {
            global_path: global_path.clone(),
            local_path: local_path.clone(),
        };

        // Load config: global auto_update = false should be preserved
        let cfg = manager.load().unwrap();
        assert_eq!(cfg.editor, "vim".to_string(), "Local editor should override global");
        assert_eq!(cfg.auto_update, false, "Global auto_update=false should be preserved");
        assert_eq!(cfg.tui.line_numbers, false, "Global line_numbers=false should be preserved");
    }

    #[test]
    fn global_config_without_local() {
        let dir = tempdir().unwrap();
        let global_dir = dir.path().join("user/.config/sage");
        let _ = fs::create_dir_all(&global_dir);

        let global_path = global_dir.join(CONFIG_FILENAME);

        // Setup global config with custom ai.api_url
        let global_cfg_str = r#"
[ai]
api_url = "wow"
"#;
        fs::write(&global_path, global_cfg_str).unwrap();

        // Local config path that doesn't exist
        let local_path = dir.path().join("repo/.sage").join(CONFIG_FILENAME);

        // Point manager at our test paths
        let manager = ConfigManager {
            global_path: global_path.clone(),
            local_path: local_path.clone(),
        };

        // Load config: global ai.api_url should be "wow"
        let cfg = manager.load().unwrap();
        assert_eq!(cfg.ai.api_url, "wow", "Global ai.api_url should be preserved when no local config exists");
    }
}
