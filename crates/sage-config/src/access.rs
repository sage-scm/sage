use std::path::PathBuf;

use crate::config::{SECRET_KEYS, SageConfig};
use crate::error::{ConfigError, Result};
use crate::secret::SecretString;
use crate::toml_utils::{insert_value, parse_scalar_value, remove_value};

use toml::value::{Table, Value};

const UPDATE_PATH: &str = "<config update>";

#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub raw_value: Option<String>,
    pub display_value: Option<String>,
    pub is_secret: bool,
}

pub fn list_entries(config: &SageConfig) -> Result<Vec<ConfigEntry>> {
    let table = config_to_table(config)?;
    let mut entries = Vec::new();
    flatten_table("", &table, &mut entries);

    let mut flat_entries: Vec<(String, Option<Value>)> =
        entries.into_iter().map(|(k, v)| (k, Some(v))).collect();

    for &secret_key in SECRET_KEYS {
        if !flat_entries
            .iter()
            .any(|(existing, _)| existing == secret_key)
        {
            flat_entries.push((secret_key.to_string(), None));
        }
    }

    flat_entries.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(flat_entries
        .into_iter()
        .map(|(key, value)| to_entry(key, value))
        .collect())
}

pub fn get_entry(config: &SageConfig, key: &str) -> Result<Option<ConfigEntry>> {
    let segments = parse_key(key)?;
    let value = toml::Value::try_from(config.clone()).map_err(ConfigError::Serialize)?;

    match find_value(&value, &segments) {
        Some(value) => Ok(Some(to_entry(key.to_string(), Some(value.clone())))),
        None => Ok(None),
    }
}

pub fn set_value(config: &mut SageConfig, key: &str, value: Option<&str>) -> Result<()> {
    let segments = parse_key(key)?;
    let mut table = config_to_table(config)?;

    if let Some(raw) = value {
        insert_value(&mut table, &segments, parse_scalar_value(raw));
    } else {
        remove_value(&mut table, &segments);
    }

    let updated: SageConfig = Value::Table(table)
        .try_into()
        .map_err(|e| ConfigError::parse(PathBuf::from(UPDATE_PATH), e))?;

    *config = updated;
    Ok(())
}

fn config_to_table(config: &SageConfig) -> Result<Table> {
    match Value::try_from(config.clone()).map_err(ConfigError::Serialize)? {
        Value::Table(table) => Ok(table),
        _ => unreachable!("Serialized config should always be a table"),
    }
}

fn flatten_table(prefix: &str, table: &Table, entries: &mut Vec<(String, Value)>) {
    for (key, value) in table {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };

        match value {
            Value::Table(child) => flatten_table(&path, child, entries),
            other => entries.push((path, other.clone())),
        }
    }
}

fn to_entry(key: String, value: Option<Value>) -> ConfigEntry {
    let is_secret = SECRET_KEYS.contains(&key.as_str());

    match value {
        Some(value) => {
            let raw_value = Some(value_to_string(&value));
            let display_value = Some(if is_secret {
                mask_secret(raw_value.as_ref().unwrap())
            } else {
                raw_value.as_ref().unwrap().clone()
            });

            ConfigEntry {
                key,
                raw_value,
                display_value,
                is_secret,
            }
        }
        None => ConfigEntry {
            key,
            raw_value: None,
            display_value: Some("<unset>".to_string()),
            is_secret,
        },
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(dt) => dt.to_string(),
        Value::Array(items) => {
            let joined = items
                .iter()
                .map(value_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{joined}]")
        }
        Value::Table(_) => "<table>".to_string(),
    }
}

fn mask_secret(raw: &str) -> String {
    SecretString::from(raw).to_string()
}

fn parse_key(key: &str) -> Result<Vec<String>> {
    let segments: Vec<String> = key
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect();

    if segments.len() < 2 {
        return Err(ConfigError::invalid_field_path(
            key.to_string(),
            "Expected section.field".to_string(),
        ));
    }

    Ok(segments)
}

fn find_value<'a>(value: &'a Value, path: &[String]) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }

    match value {
        Value::Table(table) => {
            let (head, tail) = path.split_first()?;
            table.get(head).and_then(|child| find_value(child, tail))
        }
        _ => None,
    }
}
