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
use tokio::time::timeout;

const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours;
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Serialize, Deserialize, Default)]
struct UpdateCheck {
    last_check: i64,
    latest_version: Option<String>,
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

fn update_check_record(latest: Option<&str>) -> Result<()> {
    let mut check = load_update_check()?;
    check.last_check = Utc::now().timestamp();

    if let Some(latest) = latest {
        check.latest_version = Some(latest.to_string());
    }

    save_update_check(&check)?;
    Ok(())
}

fn should_check_for_updates() -> Result<bool> {
    let check = load_update_check()?;
    let now = Utc::now().timestamp();
    Ok(now - check.last_check >= CHECK_INTERVAL.as_secs() as i64)
}

async fn get_latest_version() -> Result<Option<String>> {
    let octo = Octocrab::builder().build()?;

    // Get repository info from environment or use default
    let repo_owner = option_env!("SAGE_REPO_OWNER").unwrap_or("sage-scm");
    let repo_name = option_env!("SAGE_REPO_NAME").unwrap_or("sage");

    let releases = octo
        .repos(repo_owner, repo_name)
        .releases()
        .list()
        .per_page(1)
        .send()
        .await
        .context("Failed to fetch releases")?;

    if let Some(release) = releases.items.first() {
        // Skip pre-releases and drafts
        if release.prerelease || release.draft {
            return Ok(None);
        }

        // Remove 'v' prefix if present
        let version = release.tag_name.trim_start_matches('v').to_string();
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
enum InstallationMethod {
    Homebrew,
    Cargo,
    Manual,
}

fn detect_installation_method() -> InstallationMethod {
    // Check if installed via Homebrew
    if let Ok(output) = std::process::Command::new("brew")
        .args(["list", "sage-scm/sage/sage"])
        .output()
        && output.status.success()
    {
        return InstallationMethod::Homebrew;
    }

    // Check if binary is in a Homebrew path
    if let Ok(which_output) = std::process::Command::new("which").arg("sg").output()
        && which_output.status.success()
    {
        let path = String::from_utf8_lossy(&which_output.stdout);
        if path.contains("/opt/homebrew/") || path.contains("/usr/local/Cellar/") {
            return InstallationMethod::Homebrew;
        }
    }

    // Check if installed via Cargo
    if let Ok(output) = std::process::Command::new("cargo")
        .args(["install", "--list"])
        .output()
        && output.status.success()
    {
        let list = String::from_utf8_lossy(&output.stdout);
        if list.contains("sage-cli") {
            return InstallationMethod::Cargo;
        }
    }

    InstallationMethod::Manual
}

fn show_update_notification(current: &str, latest: &str) {
    let repo_owner = option_env!("SAGE_REPO_OWNER").unwrap_or("sage-scm");
    let repo_name = option_env!("SAGE_REPO_NAME").unwrap_or("sage");
    let installation_method = detect_installation_method();

    // Visual: compact, clear, and consistent spacing
    println!();
    println!(
        "{}",
        "────────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "{} {} {} {}",
        "✨ Sage update available".blue().bold(),
        "—".dimmed(),
        format!("v{}", current).yellow().bold(),
        format!("→ v{}", latest).green().bold()
    );

    let notes_url = format!("https://github.com/{repo_owner}/{repo_name}/releases/tag/v{latest}");
    println!(
        "{} {}",
        "Release notes:".bold(),
        notes_url.as_str().purple()
    );
    println!();

    match installation_method {
        InstallationMethod::Homebrew => {
            println!("{}", "Update via Homebrew:".bold());
            println!(
                "  {} {}",
                "•".dimmed(),
                "brew update && brew upgrade sage-scm/sage/sage".cyan()
            );
            println!("{}", "Other options:".bold());
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Quick install:".dimmed(),
                format!(
                    "curl -fsSL https://raw.githubusercontent.com/{repo_owner}/{repo_name}/main/install.sh | sh"
                )
                .cyan()
            );
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Manual download:".dimmed(),
                notes_url.as_str().cyan()
            );
        }
        InstallationMethod::Cargo => {
            println!("{}", "Update via Cargo:".bold());
            println!(
                "  {} {}",
                "•".dimmed(),
                format!(
                    "cargo install --git https://github.com/{repo_owner}/{repo_name} --tag v{latest} sage-cli --force"
                )
                .cyan()
            );
            println!("{}", "Other options:".bold());
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Quick install:".dimmed(),
                format!(
                    "curl -fsSL https://raw.githubusercontent.com/{repo_owner}/{repo_name}/main/install.sh | sh"
                )
                .cyan()
            );
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Homebrew:".dimmed(),
                "brew install sage-scm/sage/sage".cyan()
            );
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Manual download:".dimmed(),
                notes_url.as_str().cyan()
            );
        }
        InstallationMethod::Manual => {
            println!("{}", "Update options:".bold());
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Quick install:".dimmed(),
                format!(
                    "curl -fsSL https://raw.githubusercontent.com/{repo_owner}/{repo_name}/main/install.sh | sh"
                )
                .cyan()
            );
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Homebrew:".dimmed(),
                "brew install sage-scm/sage/sage".cyan()
            );
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "From source:".dimmed(),
                format!(
                    "cargo install --git https://github.com/{repo_owner}/{repo_name} --tag v{latest} sage-cli"
                )
                .cyan()
            );
            println!(
                "  {} {} {}",
                "•".dimmed(),
                "Manual download:".dimmed(),
                notes_url.as_str().cyan()
            );
        }
    }

    println!(
        "{}",
        "────────────────────────────────────────────────────────".dimmed()
    );
}

pub async fn check_for_updates() -> Result<()> {
    if !should_check_for_updates()? {
        return Ok(());
    }

    let latest_version = match timeout(UPDATE_CHECK_TIMEOUT, get_latest_version()).await {
        Ok(Ok(version)) => version,
        Ok(Err(err)) => {
            update_check_record(None)?;
            return Err(err);
        }
        Err(_) => {
            update_check_record(None)?;
            return Ok(());
        }
    };

    let current_version = Version::parse(CURRENT_VERSION)?;

    if let Some(ref latest) = latest_version {
        let latest_version = Version::parse(latest)?;

        if latest_version > current_version {
            show_update_notification(CURRENT_VERSION, latest);
        }
    }

    update_check_record(latest_version.as_deref())?;

    Ok(())
}
