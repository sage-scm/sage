use crate::helpers::ColorizeExt;
use anyhow::{Context, Result};
use chrono::Utc;
use colored::*;
use octocrab::Octocrab;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Error, ErrorKind},
    path::PathBuf,
    time::Duration,
};

const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours;
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
struct UpdateCheck {
    last_check: i64,
    latest_version: Option<String>,
}

impl Default for UpdateCheck {
    fn default() -> Self {
        Self {
            last_check: 0,
            latest_version: None,
        }
    }
}

fn get_update_check_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Could not find config directory"))?;
    path.push("sage");
    fs::create_dir_all(&path)?;
    path.push("update_check.json");
    Ok(path)
}

fn load_update_check() -> Result<UpdateCheck> {
    let path = get_update_check_path()?;
    if !path.exists() {
        return Ok(UpdateCheck::default());
    }

    let contents = fs::read_to_string(&path)?;
    serde_json::from_str(&contents).context("Failed to parse update check file")
}

fn save_update_check(check: &UpdateCheck) -> Result<()> {
    let path = get_update_check_path()?;
    let contents = serde_json::to_string_pretty(check)?;
    fs::write(path, contents)?;
    Ok(())
}

fn should_check_for_updates() -> Result<bool> {
    let check = load_update_check()?;
    let now = Utc::now().timestamp();
    Ok(now - check.last_check >= CHECK_INTERVAL.as_secs() as i64)
}

async fn get_latest_version() -> Result<Option<String>> {
    let octo = Octocrab::builder().build()?;
    let releases = octo
        .repos("sage-scm", "sage")
        .releases()
        .list()
        .per_page(1)
        .send()
        .await
        .context("Failed to fetch releases")?;

    if let Some(release) = releases.items.first() {
        // Remove 'v' prefix if present
        let version = release.tag_name.trim_start_matches('v').to_string();
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

fn show_update_notification(current: &str, latest: &str) {
    println!(
        "\n{}",
        "✨ A new version of Sage is available!".sage().bold()
    );
    println!("Current version: {}", current.yellow());
    println!("Latest version: {}", latest.green());
    println!("To update, run: {}", "cargo install sage-rs --force".cyan());
    println!();
}

pub async fn check_for_updates() -> Result<()> {
    if !should_check_for_updates()? {
        return Ok(());
    }

    let latest_version = get_latest_version().await?;
    let current_version = CURRENT_VERSION;

    if let Some(latest) = latest_version {
        let current = Version::parse(current_version)?;
        let latest = Version::parse(&latest)?;

        if latest > current {
            show_update_notification(current_version, &latest.to_string());
        }

        // Update the check file
        let mut check = load_update_check()?;
        check.last_check = Utc::now().timestamp();
        check.latest_version = Some(latest.to_string());
        save_update_check(&check)?;
    }

    Ok(())
}
