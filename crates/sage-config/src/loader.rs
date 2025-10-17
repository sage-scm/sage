use std::env;
use std::fs;
use std::path::PathBuf;

use crate::config::SageConfig;
use crate::error::{ConfigError, Result};

#[derive(Default)]
struct PartialConfig {
    ai_provider: Option<String>,
    ai_api_key: Option<Option<String>>,
    ai_model: Option<String>,
    git_auto_stage: Option<bool>,
    git_commit_template: Option<String>,
    general_update_check: Option<bool>,
    general_telemetry: Option<bool>,
}

pub struct ConfigLoader {
    global_path: PathBuf,
    local_path: Option<PathBuf>,
}

impl ConfigLoader {
    pub fn new() -> Result<Self> {
        let global_path = Self::global_config_path()?;
        let local_path = Self::local_config_path();

        Ok(Self {
            global_path,
            local_path,
        })
    }

    pub fn global_path(&self) -> &PathBuf {
        &self.global_path
    }

    pub fn load(&self) -> Result<SageConfig> {
        let global_config = self.load_global_config()?;
        let local_config = self.load_local_config()?;
        let env_partial = Self::load_env_config();

        Ok(Self::merge_configs(
            global_config,
            local_config,
            env_partial,
        ))
    }

    fn load_global_config(&self) -> Result<Option<SageConfig>> {
        Self::load_config_from_path(&self.global_path)
    }

    fn load_local_config(&self) -> Result<Option<SageConfig>> {
        match &self.local_path {
            Some(path) => Self::load_config_from_path(path),
            None => Ok(None),
        }
    }

    fn load_config_from_path(path: &PathBuf) -> Result<Option<SageConfig>> {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(e) => {
                return Err(ConfigError::file_read(path.clone(), e));
            }
        };

        if contents.is_empty() || contents.trim().is_empty() {
            return Ok(None);
        }

        let config: SageConfig =
            toml::from_str(&contents).map_err(|e| ConfigError::parse(path.clone(), e))?;

        Ok(Some(config))
    }

    fn load_env_config() -> PartialConfig {
        let mut partial = PartialConfig::default();

        for (key, value) in env::vars() {
            if !key.starts_with("SAGE_") {
                continue;
            }

            let field_path = &key[5..];

            let separator_pos = match field_path.find("__") {
                Some(pos) => pos,
                None => continue,
            };

            let section = &field_path[..separator_pos];
            let field = &field_path[separator_pos + 2..];

            match section {
                "AI" | "ai" => match field {
                    "PROVIDER" | "provider" => partial.ai_provider = Some(value),
                    "API_KEY" | "api_key" => partial.ai_api_key = Some(Some(value)),
                    "MODEL" | "model" => partial.ai_model = Some(value),
                    _ => {}
                },
                "GIT" | "git" => match field {
                    "AUTO_STAGE" | "auto_stage" => {
                        partial.git_auto_stage = match value.as_str() {
                            "true" | "1" | "yes" => Some(true),
                            "false" | "0" | "no" => Some(false),
                            _ => value.parse::<bool>().ok(),
                        };
                    }
                    "COMMIT_TEMPLATE" | "commit_template" => {
                        partial.git_commit_template = Some(value)
                    }
                    _ => {}
                },
                "GENERAL" | "general" => match field {
                    "UPDATE_CHECK" | "update_check" => {
                        partial.general_update_check = match value.as_str() {
                            "true" | "1" | "yes" => Some(true),
                            "false" | "0" | "no" => Some(false),
                            _ => value.parse::<bool>().ok(),
                        };
                    }
                    "TELEMETRY" | "telemetry" => {
                        partial.general_telemetry = match value.as_str() {
                            "true" | "1" | "yes" => Some(true),
                            "false" | "0" | "no" => Some(false),
                            _ => value.parse::<bool>().ok(),
                        };
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        partial
    }

    fn merge_configs(
        global: Option<SageConfig>,
        local: Option<SageConfig>,
        env: PartialConfig,
    ) -> SageConfig {
        let mut result = SageConfig::default();

        if let Some(global_config) = global {
            result = global_config;
        }

        if let Some(local_config) = local {
            result.ai.provider = local_config.ai.provider;
            result.ai.api_key = local_config.ai.api_key;
            result.ai.model = local_config.ai.model;
            result.git.auto_stage = local_config.git.auto_stage;
            result.git.commit_template = local_config.git.commit_template;
            result.general.update_check = local_config.general.update_check;
            result.general.telemetry = local_config.general.telemetry;
        }

        if let Some(provider) = env.ai_provider {
            result.ai.provider = provider;
        }
        if let Some(api_key) = env.ai_api_key {
            result.ai.api_key = api_key;
        }
        if let Some(model) = env.ai_model {
            result.ai.model = model;
        }
        if let Some(auto_stage) = env.git_auto_stage {
            result.git.auto_stage = auto_stage;
        }
        if let Some(commit_template) = env.git_commit_template {
            result.git.commit_template = commit_template;
        }
        if let Some(update_check) = env.general_update_check {
            result.general.update_check = update_check;
        }
        if let Some(telemetry) = env.general_telemetry {
            result.general.telemetry = telemetry;
        }

        result
    }

    fn global_config_path() -> Result<PathBuf> {
        #[cfg(windows)]
        let config_dir = dirs::config_dir()
            .ok_or_else(|| ConfigError::directory_not_found("user config".to_string()))?
            .join("sage");

        #[cfg(not(windows))]
        let config_dir = dirs::home_dir()
            .ok_or_else(|| ConfigError::directory_not_found("user home".to_string()))?
            .join(".config")
            .join("sage");

        fs::create_dir_all(&config_dir)
            .map_err(|e| ConfigError::directory_creation(config_dir.clone(), e))?;

        Ok(config_dir.join("config.toml"))
    }

    fn local_config_path() -> Option<PathBuf> {
        let mut current_dir = env::current_dir().ok()?;

        loop {
            let config_path = current_dir.join(".sage").join("config.toml");
            if config_path.exists() {
                return Some(config_path);
            }

            if !current_dir.pop() {
                break;
            }
        }

        None
    }
}
