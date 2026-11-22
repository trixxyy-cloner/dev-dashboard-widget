// Mirrors the Serialize structs in src-tauri/src.

export type Source<T> =
  | { state: "ok"; data: T }
  | { state: "error"; message: string }
  | { state: "off" };

export type LocalStatus = {
  branch: string;
  changedFiles: number;
  ahead: number;
  behind: number;
};

export type Commit = { sha: string; message: string; date: string; url: string };

export type CiRun = {
  name: string;
  status: string;
  conclusion: string | null;
  url: string;
};

export type GithubStatus = {
  url: string;
  lastCommit: Commit | null;
  openPrs: number;
  prsUrl: string;
  ci: CiRun | null;
};

export type VercelDeploy = {
  state: string;
  url: string;
  inspectorUrl: string | null;
  createdMs: number;
  commitMessage: string | null;
};

export type DoApp = {
  name: string;
  phase: string;
  liveUrl: string | null;
  consoleUrl: string;
  updatedAt: string | null;
};

export type ProjectStatus = {
  name: string;
  note: string | null;
  repoUrl: string | null;
  local: Source<LocalStatus>;
  github: Source<GithubStatus>;
  vercel: Source<VercelDeploy | null>;
};

export type Dashboard = {
  refreshSeconds: number;
  lockPosition: boolean;
  fetchedMs: number;
  projects: ProjectStatus[];
  digitalocean: Source<DoApp[]>;
};

export type Tone = "ok" | "busy" | "bad" | "idle";
