use crate::source::{check_status, command};
use serde::{Deserialize, Serialize};

const API: &str = "https://api.github.com";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubStatus {
    pub url: String,
    pub last_commit: Option<Commit>,
    pub open_prs: usize,
    pub prs_url: String,
    pub ci: Option<CiRun>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub date: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CiRun {
    pub name: String,
    /// queued | in_progress | completed
    pub status: String,
    /// success | failure | cancelled | ... (only when completed)
    pub conclusion: Option<String>,
    pub url: String,
}

#[derive(Deserialize)]
struct ApiCommit {
    sha: String,
    html_url: String,
    commit: ApiCommitInner,
}

#[derive(Deserialize)]
struct ApiCommitInner {
    message: String,
    committer: ApiCommitter,
}

#[derive(Deserialize)]
struct ApiCommitter {
    date: String,
}

#[derive(Deserialize)]
struct ApiRuns {
    workflow_runs: Vec<ApiRun>,
}

#[derive(Deserialize)]
struct ApiRun {
    name: Option<String>,
    status: String,
    conclusion: Option<String>,
    html_url: String,
}

/// Falls back to the logged-in GitHub CLI so no extra token has to be created.
pub async fn resolve_token(configured: Option<&str>) -> Option<String> {
    if let Some(token) = configured {
        return Some(token.to_string());
    }
    let output = command("gh").args(["auth", "token"]).output().await.ok()?;
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (output.status.success() && !token.is_empty()).then_some(token)
}

async fn get<T: for<'de> Deserialize<'de>>(client: &reqwest::Client, token: &str, path: &str) -> Result<T, String> {
    let response = client
        .get(format!("{API}{path}"))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("Nätverksfel: {e}"))?;
    check_status(response).await?.json().await.map_err(|e| e.to_string())
}

pub async fn status(client: &reqwest::Client, token: &str, repo: &str) -> Result<GithubStatus, String> {
    let commits_path = format!("/repos/{repo}/commits?per_page=1");
    let prs_path = format!("/repos/{repo}/pulls?state=open&per_page=50");
    let runs_path = format!("/repos/{repo}/actions/runs?per_page=1");
    let (commits, prs, runs) = tokio::join!(
        get::<Vec<ApiCommit>>(client, token, &commits_path),
        get::<Vec<serde_json::Value>>(client, token, &prs_path),
        get::<ApiRuns>(client, token, &runs_path),
    );

    // Commits decide whether the repo is reachable at all; PRs and CI are best effort.
    let last_commit = commits?.into_iter().next().map(|c| Commit {
        sha: c.sha.chars().take(7).collect(),
        message: c.commit.message.lines().next().unwrap_or_default().to_string(),
        date: c.commit.committer.date,
        url: c.html_url,
    });
    let ci = runs.ok().and_then(|r| r.workflow_runs.into_iter().next()).map(|r| CiRun {
        name: r.name.unwrap_or_else(|| "CI".into()),
        status: r.status,
        conclusion: r.conclusion,
        url: r.html_url,
    });

    Ok(GithubStatus {
        url: format!("https://github.com/{repo}"),
        last_commit,
        open_prs: prs.map(|p| p.len()).unwrap_or(0),
        prs_url: format!("https://github.com/{repo}/pulls"),
        ci,
    })
}
