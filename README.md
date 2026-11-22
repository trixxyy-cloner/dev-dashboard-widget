# Dev Dashboard Widget

A small always-on-desktop widget for Windows that shows the status of my projects at a glance:

- **Local git** – current branch, uncommitted files, commits ahead/behind
- **GitHub** – latest commit, open pull requests, latest Actions run
- **Vercel** – latest production deploy
- **DigitalOcean** – every App Platform app and its deploy phase (each running app costs money)

Click a project name to open its repo, or any row to open that page. Refreshes every 2 minutes, or on demand. Lives in the system tray (left-click toggles the widget, right-click for refresh/config/quit) and starts with Windows.

Built with **Tauri v2** (Rust) + **React** + **TypeScript** + **Tailwind CSS v4**. All API calls happen on the Rust side, so tokens never reach the web view or the repo.

## Configuration

On first start the app writes `%APPDATA%\dev-dashboard-widget\config.json`. Open it with the ☰ button in the widget.

```json
{
  "refreshSeconds": 120,
  "startWithWindows": true,
  "githubToken": null,
  "vercelToken": "…",
  "vercelTeamId": null,
  "digitaloceanToken": "…",
  "projects": [
    {
      "name": "Portfolio",
      "repo": "owner/repo",
      "localPath": "C:/path/to/repo",
      "vercelProject": "portfolio",
      "note": null
    }
  ]
}
```

- `githubToken` – optional; falls back to the GitHub CLI (`gh auth token`).
- `vercelToken` – create at vercel.com → Account Settings → Tokens.
- `digitaloceanToken` – create at cloud.digitalocean.com → API → Tokens (read scope is enough).
- Any field left `null` simply hides that source.

## Development

Requires Node, Rust and the Tauri prerequisites for Windows.

```bash
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # installer in src-tauri/target/release/bundle/nsis
cd src-tauri && cargo test
```
