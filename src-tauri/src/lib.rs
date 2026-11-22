mod config;
mod digitalocean;
mod git;
mod github;
mod snap;
mod source;
mod vercel;

use config::{non_empty, Config, ProjectConfig};
use futures::future::join_all;
use serde::Serialize;
use snap::SnapState;
use source::Source;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Dashboard {
    refresh_seconds: u64,
    lock_position: bool,
    fetched_ms: u128,
    projects: Vec<ProjectStatus>,
    digitalocean: Source<Vec<digitalocean::DoApp>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectStatus {
    name: String,
    note: Option<String>,
    /// From config, so the name stays clickable even when the GitHub API fails.
    repo_url: Option<String>,
    local: Source<git::LocalStatus>,
    github: Source<github::GithubStatus>,
    vercel: Source<Option<vercel::VercelDeploy>>,
}

struct Http(reqwest::Client);

async fn project_status(client: &reqwest::Client, config: &Config, gh_token: Option<&str>, project: &ProjectConfig) -> ProjectStatus {
    let local = async {
        match non_empty(&project.local_path) {
            Some(path) => git::status(path).await.into(),
            None => Source::Off,
        }
    };
    let github = async {
        match (non_empty(&project.repo), gh_token) {
            (Some(repo), Some(token)) => github::status(client, token, repo).await.into(),
            (Some(_), None) => Source::Error { message: "Ingen GitHub-token (kör `gh auth login`)".into() },
            (None, _) => Source::Off,
        }
    };
    let vercel = async {
        match (non_empty(&project.vercel_project), non_empty(&config.vercel_token)) {
            (Some(name), Some(token)) => {
                vercel::latest_production(client, token, non_empty(&config.vercel_team_id), name).await.into()
            }
            (Some(_), None) => Source::Error { message: "vercelToken saknas i config".into() },
            (None, _) => Source::Off,
        }
    };
    let (local, github, vercel) = tokio::join!(local, github, vercel);

    ProjectStatus {
        name: project.name.clone(),
        note: project.note.clone(),
        repo_url: non_empty(&project.repo).map(|repo| format!("https://github.com/{repo}")),
        local,
        github,
        vercel,
    }
}

#[tauri::command]
async fn get_dashboard(http: tauri::State<'_, Http>, snap: tauri::State<'_, SnapState>) -> Result<Dashboard, String> {
    let config = config::load()?;
    let client = &http.0;
    let gh_token = github::resolve_token(non_empty(&config.github_token)).await;

    let projects = join_all(
        config.projects.iter().map(|p| project_status(client, &config, gh_token.as_deref(), p)),
    );
    let digitalocean = async {
        match non_empty(&config.digitalocean_token) {
            Some(token) => digitalocean::apps(client, token).await.into(),
            None => Source::Off,
        }
    };
    let (projects, digitalocean) = tokio::join!(projects, digitalocean);

    Ok(Dashboard {
        refresh_seconds: config.refresh_seconds.max(30),
        lock_position: snap.locked.load(Ordering::Relaxed),
        fetched_ms: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0),
        projects,
        digitalocean,
    })
}

#[tauri::command]
fn open_config(app: AppHandle) -> Result<(), String> {
    config::load()?; // make sure the file exists before opening it
    let path = config::config_path();
    app.opener().open_path(path.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

/// Keeps the Windows autostart entry in line with `startWithWindows` in the config.
fn sync_autostart(app: &AppHandle) {
    // A debug build would register the dev exe in the registry, so only release builds touch it.
    if cfg!(debug_assertions) {
        return;
    }
    let enabled = config::load().map(|c| c.start_with_windows).unwrap_or(false);
    let launcher = app.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    if let Err(e) = result {
        eprintln!("autostart: {e}");
    }
}

/// Flips the lock, persists it and tells the web view so it can turn dragging on/off.
fn toggle_lock(app: &AppHandle) -> bool {
    let state = app.state::<SnapState>();
    let locked = !state.locked.load(Ordering::Relaxed);
    state.locked.store(locked, Ordering::Relaxed);
    match config::load() {
        Ok(mut config) => {
            config.lock_position = locked;
            if let Err(e) = config::save(&config) {
                eprintln!("save config: {e}");
            }
        }
        Err(e) => eprintln!("load config: {e}"),
    }
    let _ = app.emit("lock-changed", locked);
    locked
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let locked = app.state::<SnapState>().locked.load(Ordering::Relaxed);
    let toggle = MenuItem::with_id(app, "toggle", "Visa / göm", true, None::<&str>)?;
    let lock = CheckMenuItem::with_id(app, "lock", "Lås position", true, locked, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Uppdatera", true, None::<&str>)?;
    let config = MenuItem::with_id(app, "config", "Öppna config", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Avsluta", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &lock, &refresh, &config, &quit])?;
    let lock_item = lock.clone();

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Dev Dashboard")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => toggle_window(app),
            "lock" => {
                let locked = toggle_lock(app);
                let _ = lock_item.set_checked(locked);
            }
            "refresh" => {
                let _ = app.emit("refresh", ());
            }
            "config" => {
                let _ = open_config(app.clone());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                toggle_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = reqwest::Client::builder()
        .user_agent("dev-dashboard-widget")
        .timeout(Duration::from_secs(15))
        .build()
        .expect("failed to build HTTP client");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .manage(Http(client))
        .manage(SnapState::default())
        .setup(|app| {
            let locked = config::load().map(|c| c.lock_position).unwrap_or(false);
            app.state::<SnapState>().locked.store(locked, Ordering::Relaxed);
            build_tray(app)?;
            sync_autostart(app.handle());
            if let Some(window) = app.get_webview_window("main") {
                snap::ensure_visible(&window);
                let _ = window.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Moved(_) = event {
                if let Some(webview) = window.app_handle().get_webview_window(window.label()) {
                    snap::on_moved(&webview);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![get_dashboard, open_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
