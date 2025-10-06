use anyhow::{Context, Result};

pub async fn commit_message(
    repo: &sage_git::Repo,
    console: &sage_fmt::Console,
    message: Option<String>,
    use_ai: bool,
) -> Result<String> {
    if use_ai {
        let progress = console.progress("Generating message with AI");
        let diff = repo.diff_ai()?;
        let generated = sage_ai::commit_message(&diff)
            .await
            .context("AI failed to generate a commit message")?;
        progress.done();

        return Ok(generated);
    }

    if message.is_some() {
        console.message(sage_fmt::MessageType::Info, "Using provided message")?;
        return Ok(message.unwrap());
    }

    Ok(String::new())
}
