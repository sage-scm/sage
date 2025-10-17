use anyhow::{Context, Result};
use sage_config::ConfigManager;

pub async fn commit_message(
    repo: &sage_git::Repo,
    console: &sage_fmt::Console,
    message: Option<String>,
    use_ai: bool,
) -> Result<String> {
    if use_ai {
        let progress = console.progress("Generating message with AI");
        let diff = repo.diff_ai()?;

        let config_manager = ConfigManager::load().context("Failed to load configuration")?;
        let config = config_manager.get();
        let additional_prompt = config.ai.additional_commit_prompt.as_deref();

        let generated = sage_ai::commit_message(&diff, additional_prompt)
            .await
            .context("AI failed to generate a commit message")?;
        progress.done();

        return Ok(generated);
    }

    if let Some(message) = message {
        console.message(sage_fmt::MessageType::Info, "Using provided message")?;
        return Ok(message);
    }

    Ok(String::new())
}
