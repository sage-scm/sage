use std::env;
use std::process::Command;

use anyhow::{Result, bail};

pub fn access_token() -> Result<String> {
    if let Some(token) = get_token_from_gh_cli() {
        return Ok(token);
    }

    if let Some(token) = get_token_from_env() {
        return Ok(token);
    }

    bail!("Cannot find an access token")
}

fn get_token_from_env() -> Option<String> {
    // Check for SAGE_GITHUB_TOKEN first (our custom env var)
    if let Ok(token) = env::var("SAGE_GITHUB_TOKEN") {
        return Some(token);
    }

    // Then check for standard GITHUB_TOKEN
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        return Some(token);
    }

    None
}

fn get_token_from_gh_cli() -> Option<String> {
    // Check if the gh CLI is installed and authenticated
    let result = Command::new("gh").arg("auth").arg("token").output();

    if let Ok(output) = result
        && output.status.success()
    {
        // Convert the output to a string and trim whitespace
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !token.is_empty() {
            return Some(token);
        }
    }
    // gh CLI not installed or other error

    None
}
