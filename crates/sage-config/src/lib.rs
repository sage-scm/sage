//! `sage-config` - Flexible configuration handling for Sage projects.
//!
//! Supports global (user) and local (repo-specific) configuration files.
//! Provides loading, merging, updating, and saving utilities.
//!
//! Configuration is loaded from a global location (`~/.config/sage/config.toml`),
//! and optionally merged with a local configuration file (`./.sage/config.toml`).
//!
//! # Example
//! ```rust
//! use sage_config::{Config, ConfigManager};
//! let manager = ConfigManager::new()?;
//! let config = manager.load()?;
//! println!("Current theme: {:?}", config.theme);
//! ```

mod defaults;
mod error;
mod manager;
mod merge;
mod model;

pub use error::ConfigError;
pub use manager::ConfigManager;
pub use model::Config;
