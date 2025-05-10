use thiserror::Error;
use std::io;

/// Errors that can occur when loading or saving a configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML parsing or serialization failed: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization failed: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Could not determine a valid config path")]
    ConfigPathNotFound,

    #[error("Local config is invalid at: {0}")]
    InvalidLocalConfig(String),

    #[error("Global config is invalid at: {0}")]
    InvalidGlobalConfig(String),

    #[error("Custom config error: {0}")]
    Custom(String),
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigError::TomlSer(e)
    }
}

impl From<anyhow::Error> for ConfigError {
    fn from(e: anyhow::Error) -> Self {
        ConfigError::Custom(format!("{:?}", e))
    }
}