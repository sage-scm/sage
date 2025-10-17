use std::fs;
use std::io::Write;

use crate::config::SageConfig;
use crate::error::{ConfigError, Result};
use crate::loader::ConfigLoader;

pub struct ConfigManager {
    config: SageConfig,
    loader: ConfigLoader,
}

impl ConfigManager {
    pub fn load() -> Result<Self> {
        let loader = ConfigLoader::new()?;
        let config = loader.load()?;

        Ok(Self { config, loader })
    }

    pub fn get(&self) -> &SageConfig {
        &self.config
    }

    pub fn get_mut(&mut self) -> &mut SageConfig {
        &mut self.config
    }

    pub fn save(&self) -> Result<()> {
        let config_path = self.loader.global_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::directory_creation(parent.to_path_buf(), e))?;
        }

        let toml_content = toml::to_string_pretty(&self.config)?;
        let temp_path = config_path.with_extension("toml.tmp");

        let mut temp_file = fs::File::create(&temp_path)
            .map_err(|e| ConfigError::file_write(temp_path.clone(), e))?;

        temp_file
            .write_all(toml_content.as_bytes())
            .map_err(|e| ConfigError::file_write(temp_path.clone(), e))?;

        temp_file
            .sync_all()
            .map_err(|e| ConfigError::file_write(temp_path.clone(), e))?;

        drop(temp_file);

        fs::rename(&temp_path, config_path)
            .map_err(|e| ConfigError::file_write(config_path.clone(), e))?;

        Ok(())
    }
}
