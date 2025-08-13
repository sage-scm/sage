use anyhow::Result;
use sage_git::branch::get_current;
use sage_git::repo::name;

pub fn get_repo_info(ui: &crate::Ui) -> Result<()> {
    let repo_name = name().unwrap_or_else(|| "(repo)".to_string());
    let branch = get_current()?;
    let now = humantime::format_rfc3339(std::time::SystemTime::now()).to_string();

    ui.header(&[("repo", &repo_name), ("branch", &branch), ("time", &now)]);

    Ok(())
}
