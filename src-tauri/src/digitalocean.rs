use crate::source::check_status;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoApp {
    pub name: String,
    /// ACTIVE | BUILDING | DEPLOYING | ERROR | ...
    pub phase: String,
    pub live_url: Option<String>,
    pub console_url: String,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
struct ApiApps {
    #[serde(default)]
    apps: Vec<ApiApp>,
}

#[derive(Deserialize)]
struct ApiApp {
    id: String,
    spec: ApiSpec,
    live_url: Option<String>,
    updated_at: Option<String>,
    active_deployment: Option<ApiDeployment>,
    in_progress_deployment: Option<ApiDeployment>,
}

#[derive(Deserialize)]
struct ApiSpec {
    name: String,
}

#[derive(Deserialize)]
struct ApiDeployment {
    phase: Option<String>,
}

/// Lists every App Platform app, since each running app costs money.
pub async fn apps(client: &reqwest::Client, token: &str) -> Result<Vec<DoApp>, String> {
    let response = client
        .get("https://api.digitalocean.com/v2/apps")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("Nätverksfel: {e}"))?;
    let body: ApiApps = check_status(response).await?.json().await.map_err(|e| e.to_string())?;

    Ok(body
        .apps
        .into_iter()
        .map(|a| {
            // A running deploy is more interesting than the last finished one.
            let phase = a
                .in_progress_deployment
                .and_then(|d| d.phase)
                .or_else(|| a.active_deployment.and_then(|d| d.phase))
                .unwrap_or_else(|| "UNKNOWN".into());
            DoApp {
                name: a.spec.name,
                phase,
                live_url: a.live_url,
                console_url: format!("https://cloud.digitalocean.com/apps/{}", a.id),
                updated_at: a.updated_at,
            }
        })
        .collect())
}
