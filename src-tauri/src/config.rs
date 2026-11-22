use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User config, stored outside the repo so tokens never end up in git.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub refresh_seconds: u64,
    pub start_with_windows: bool,
    pub lock_position: bool,
    /// Optional. When empty, the token from the GitHub CLI (`gh auth token`) is used.
    pub github_token: Option<String>,
    pub digitalocean_token: Option<String>,
    pub projects: Vec<ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    pub name: String,
    /// "owner/repo" on GitHub.
    pub repo: Option<String>,
    pub local_path: Option<String>,
    pub note: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let code_dir = dirs_home().join("Desktop").join("code");
        let local = |dir: &str| Some(code_dir.join(dir).to_string_lossy().replace('\\', "/"));
        let project = |name: &str, repo: &str, dir: &str| ProjectConfig {
            name: name.into(),
            repo: Some(repo.into()),
            local_path: local(dir),
            note: None,
        };

        let portfolio = project("Portfolio", "trixxyy-cloner/Portfolio", "portfolio");

        Self {
            refresh_seconds: 120,
            start_with_windows: true,
            lock_position: false,
            github_token: None,
            digitalocean_token: None,
            projects: vec![
                project("Krogkollen", "trixxyy-cloner/krogkollen", "krogkollen"),
                portfolio,
                project("Dev Dashboard", "trixxyy-cloner/dev-dashboard-widget", "dev-dashboard-widget"),
            ],
        }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default()
}

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_home().join(".config"));
    base.join("dev-dashboard-widget").join("config.json")
}

/// Loads the config, writing the defaults on first run so the user has a file to edit.
pub fn load() -> Result<Config, String> {
    let path = config_path();
    if !path.exists() {
        let config = Config::default();
        save(&config)?;
        return Ok(config);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("Fel i {}: {e}", path.display()))
}

pub fn save(config: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

/// Treats empty strings in the config the same as missing values.
pub fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}
