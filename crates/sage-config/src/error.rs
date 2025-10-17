use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file at '{path}': {source}")]
    FileRead { path: PathBuf, source: io::Error },

    #[error("Failed to write config file at '{path}': {source}")]
    FileWrite { path: PathBuf, source: io::Error },

    #[error("Failed to parse config file at '{path}': {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("Failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("Invalid field path '{path}': {reason}")]
    InvalidFieldPath { path: String, reason: String },

    #[error("Field '{field}' not found. Available fields: {available}")]
    FieldNotFound { field: String, available: String },

    #[error("Invalid value '{value}' for field '{field}': {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },

    #[error("Failed to locate {directory} directory")]
    DirectoryNotFound { directory: String },

    #[error("Configuration directory creation failed at '{path}': {source}")]
    DirectoryCreation { path: PathBuf, source: io::Error },
}

impl ConfigError {
    pub fn file_read(path: PathBuf, source: io::Error) -> Self {
        Self::FileRead { path, source }
    }

    pub fn file_write(path: PathBuf, source: io::Error) -> Self {
        Self::FileWrite { path, source }
    }

    pub fn parse(path: PathBuf, source: toml::de::Error) -> Self {
        Self::Parse { path, source }
    }

    pub fn invalid_field_path(path: String, reason: String) -> Self {
        Self::InvalidFieldPath { path, reason }
    }

    pub fn field_not_found(field: String, available_fields: &[String]) -> Self {
        let available = if available_fields.is_empty() {
            "none".to_string()
        } else {
            available_fields.join(", ")
        };

        Self::FieldNotFound { field, available }
    }

    pub fn invalid_value(field: String, value: String, reason: String) -> Self {
        Self::InvalidValue {
            field,
            value,
            reason,
        }
    }

    pub fn directory_not_found(directory: String) -> Self {
        Self::DirectoryNotFound { directory }
    }

    pub fn directory_creation(path: PathBuf, source: io::Error) -> Self {
        Self::DirectoryCreation { path, source }
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
