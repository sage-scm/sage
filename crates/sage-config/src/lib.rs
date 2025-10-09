use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use toml::Value;
use toml::value::Table;

#[derive(Debug, Clone)]
pub struct Config {
    path: PathBuf,
    file_data: Table,
    merged_data: Table,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let file_data = Self::read_table(&path)?;
        let merged_data = Self::merge_with_env(file_data.clone());
        Ok(Self {
            path,
            file_data,
            merged_data,
        })
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let segments = Self::segments(key)?;
        let parts = segments.iter().map(String::as_str).collect::<Vec<_>>();
        Ok(
            Self::get_from_table(&self.merged_data, &parts).map(|value| match value {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            }),
        )
    }

    pub fn keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        Self::collect_keys("", &self.merged_data, &mut keys);
        keys
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let segments = Self::segments(key)?;
        let parts = segments.iter().map(String::as_str).collect::<Vec<_>>();
        Self::set_in_table(
            &mut self.file_data,
            &parts,
            Value::String(value.to_string()),
        );
        Self::write_table(&self.path, &self.file_data)?;
        self.merged_data = Self::merge_with_env(self.file_data.clone());
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> Result<()> {
        let segments = Self::segments(key)?;
        let parts = segments.iter().map(String::as_str).collect::<Vec<_>>();
        if Self::remove_from_table(&mut self.file_data, &parts) {
            Self::write_table(&self.path, &self.file_data)?;
            self.merged_data = Self::merge_with_env(self.file_data.clone());
        }
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        #[cfg(windows)]
        let config_dir = dirs::config_dir()
            .context("Failed to locate user config directory")?
            .join("sage");

        #[cfg(not(windows))]
        let config_dir = dirs::home_dir()
            .context("Failed to locate user home directory")?
            .join(".config")
            .join("sage");

        fs::create_dir_all(&config_dir).with_context(|| {
            format!(
                "Failed to create config directory at {}",
                config_dir.display()
            )
        })?;
        Ok(config_dir.join("config.toml"))
    }

    fn merge_with_env(mut table: Table) -> Table {
        for (name, value) in env::vars() {
            if let Some(segments) = Self::env_segments(&name) {
                let parts = segments.iter().map(String::as_str).collect::<Vec<_>>();
                Self::set_in_table(&mut table, &parts, Value::String(value));
            }
        }
        table
    }

    fn read_table(path: &Path) -> Result<Table> {
        if !path.exists() {
            return Ok(Table::new());
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {}", path.display()))?;
        if contents.trim().is_empty() {
            return Ok(Table::new());
        }

        let value: Value = contents
            .parse()
            .with_context(|| format!("Failed to parse TOML from {}", path.display()))?;
        match value {
            Value::Table(table) => Ok(table),
            other => Err(anyhow!(
                "Config file {} must contain a TOML table at the root, found {}",
                path.display(),
                other.type_str()
            )),
        }
    }

    fn write_table(path: &Path, table: &Table) -> Result<()> {
        if table.is_empty() {
            if path.exists() {
                fs::remove_file(path).with_context(|| {
                    format!("Failed to remove config file at {}", path.display())
                })?;
            }
            return Ok(());
        }

        let toml = Value::Table(table.clone());
        let contents = toml::to_string_pretty(&toml)
            .with_context(|| format!("Failed to serialize config to TOML at {}", path.display()))?;
        fs::write(path, contents)
            .with_context(|| format!("Failed to write config file at {}", path.display()))
    }

    fn get_from_table<'a>(table: &'a Table, segments: &[&str]) -> Option<&'a Value> {
        if segments.is_empty() {
            return None;
        }

        let mut current = table.get(segments[0])?;
        for segment in &segments[1..] {
            match current.as_table() {
                Some(next) => current = next.get(*segment)?,
                None => return None,
            }
        }
        Some(current)
    }

    fn set_in_table(table: &mut Table, segments: &[&str], value: Value) {
        if segments.is_empty() {
            return;
        }

        if segments.len() == 1 {
            table.insert(segments[0].to_string(), value);
            return;
        }

        let mut current = table;
        for segment in &segments[..segments.len() - 1] {
            let entry = current
                .entry((*segment).to_string())
                .or_insert_with(|| Value::Table(Table::new()));

            if !entry.is_table() {
                *entry = Value::Table(Table::new());
            }

            current = entry.as_table_mut().expect("entry must be table");
        }

        current.insert(segments[segments.len() - 1].to_string(), value);
    }

    fn remove_from_table(table: &mut Table, segments: &[&str]) -> bool {
        if segments.is_empty() {
            return false;
        }

        if segments.len() == 1 {
            return table.remove(segments[0]).is_some();
        }

        if let Some(child) = table.get_mut(segments[0])
            && let Some(child_table) = child.as_table_mut()
            && Self::remove_from_table(child_table, &segments[1..])
        {
            if child_table.is_empty() {
                table.remove(segments[0]);
            }
            return true;
        }

        false
    }

    fn collect_keys(prefix: &str, table: &Table, keys: &mut Vec<String>) {
        for (key, value) in table.iter() {
            let next = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match value {
                Value::Table(child) => {
                    if child.is_empty() {
                        continue;
                    }
                    Self::collect_keys(&next, child, keys);
                }
                _ => keys.push(next),
            }
        }
    }

    fn segments(key: &str) -> Result<Vec<String>> {
        let parts: Vec<String> = key
            .split('.')
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(str::to_string)
            .collect();

        if parts.is_empty() {
            Err(anyhow!(
                "Config key '{}' must contain at least one segment",
                key
            ))
        } else {
            Ok(parts)
        }
    }

    fn env_segments(name: &str) -> Option<Vec<String>> {
        let raw = name.strip_prefix("SAGE_")?;
        let parts: Vec<String> = raw
            .split("__")
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(|segment| segment.to_ascii_lowercase())
            .collect();

        if parts.is_empty() { None } else { Some(parts) }
    }
}
