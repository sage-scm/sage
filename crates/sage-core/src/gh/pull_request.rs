use anyhow::Result;
use octocrab::{models::pulls::PullRequest, Octocrab};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NewPullRequest {
    pub title: String,
    pub head: String,
    pub base: String,
    pub body: Option<String>,
}

/// List all pull requests for a repository.
pub async fn list_pull_requests(owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
    let octo = Octocrab::builder().build()?;
    let prs = octo.pulls(owner, repo).list().send().await?.items;
    Ok(prs)
}

/// Get a pull request by its number.
pub async fn get_pull_request(owner: &str, repo: &str, number: u64) -> Result<PullRequest> {
    let octo = Octocrab::builder().build()?;
    let pr = octo.pulls(owner, repo).get(number).await?;
    Ok(pr)
}

/// Create a new pull request.
pub async fn create_pull_request(owner: &str, repo: &str, new_pr: &NewPullRequest) -> Result<PullRequest> {
    let octo = Octocrab::builder().build()?;
    let pr = octo
        .pulls(owner, repo)
        .create(&new_pr.title, &new_pr.head, &new_pr.base)
        .body(new_pr.body.clone().unwrap_or_default())
        .send()
        .await?;
    Ok(pr)
}

/// Find pull requests by head branch name.
pub async fn find_pull_requests_by_branch(owner: &str, repo: &str, branch: &str) -> Result<Vec<PullRequest>> {
    let prs = list_pull_requests(owner, repo).await?;
    Ok(prs.into_iter().filter(|pr| pr.head.ref_field == branch).collect())
}

/// Get the status/state of a pull request (e.g. "open", "closed").
pub async fn get_pull_request_status(owner: &str, repo: &str, number: u64) -> Result<String> {
    let pr = get_pull_request(owner, repo, number).await?;
    let state_str = match pr.state {
        Some(state) => format!("{:?}", state),
        None => "unknown".to_string(),
    };
    Ok(state_str)
}
