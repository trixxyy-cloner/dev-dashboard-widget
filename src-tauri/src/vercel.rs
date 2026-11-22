use crate::source::check_status;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VercelDeploy {
    /// READY | BUILDING | ERROR | QUEUED | CANCELED | INITIALIZING
    pub state: String,
    pub url: String,
    pub inspector_url: Option<String>,
    pub created_ms: i64,
    pub commit_message: Option<String>,
}

#[derive(Deserialize)]
struct ApiDeployments {
    deployments: Vec<ApiDeployment>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiDeployment {
    url: String,
    state: Option<String>,
    ready_state: Option<String>,
    inspector_url: Option<String>,
    created: i64,
    meta: Option<ApiMeta>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiMeta {
    github_commit_message: Option<String>,
}

pub async fn latest_production(
    client: &reqwest::Client,
    token: &str,
    team_id: Option<&str>,
    project: &str,
) -> Result<Option<VercelDeploy>, String> {
    let mut query = vec![("projectId", project), ("target", "production"), ("limit", "1")];
    if let Some(team) = team_id {
        query.push(("teamId", team));
    }
    let response = client
        .get("https://api.vercel.com/v6/deployments")
        .bearer_auth(token)
        .query(&query)
        .send()
        .await
        .map_err(|e| format!("Nätverksfel: {e}"))?;
    let body: ApiDeployments = check_status(response).await?.json().await.map_err(|e| e.to_string())?;

    Ok(body.deployments.into_iter().next().map(|d| VercelDeploy {
        state: d.state.or(d.ready_state).unwrap_or_else(|| "UNKNOWN".into()),
        url: format!("https://{}", d.url),
        inspector_url: d.inspector_url,
        created_ms: d.created,
        commit_message: d.meta.and_then(|m| m.github_commit_message),
    }))
}
