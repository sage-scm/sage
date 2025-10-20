use std::env;
use std::fs;
use std::path::PathBuf;

use config::{Environment, File, FileFormat};

use crate::config::SageConfig;
use crate::error::{ConfigError, Result};

const ENV_PREFIX: &str = "SAGE";
const ENV_SEPARATOR: &str = "__";

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
        let mut builder = config::Config::builder().add_source(
            File::from(self.global_path.clone())
                .format(FileFormat::Toml)
                .required(false),
        );

        if let Some(local_path) = &self.local_path {
            builder = builder.add_source(
                File::from(local_path.clone())
                    .format(FileFormat::Toml)
                    .required(false),
            );
        }

        builder = builder.add_source(
            Environment::with_prefix(ENV_PREFIX)
                .separator(ENV_SEPARATOR)
                .try_parsing(true),
        );

        let config = builder.build()?.try_deserialize::<SageConfig>()?;
        Ok(config)
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
