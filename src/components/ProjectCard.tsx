import { openUrl } from "@tauri-apps/plugin-opener";
import { ciLabel, ciTone, deployLabel, deployTone, timeAgo } from "../lib/status";
import type { GithubStatus, LocalStatus, ProjectStatus, Source } from "../types";
import { Dot, ErrorText, Row } from "./Row";

type Props = { project: ProjectStatus; now: number };

export function ProjectCard({ project, now }: Props) {
  const { repoUrl } = project;
  return (
    <section className="rounded-xl border border-white/5 bg-white/[0.03] p-2">
      <header className="flex items-center justify-between gap-2 px-2 pb-1">
        <h2 className="truncate text-sm font-semibold text-zinc-100">
          {repoUrl ? (
            <button
              type="button"
              onClick={() => void openUrl(repoUrl)}
              title={repoUrl}
              className="cursor-pointer hover:text-emerald-300 hover:underline"
            >
              {project.name}
            </button>
          ) : (
            project.name
          )}
        </h2>
        {project.note && (
          <span className="truncate rounded-full bg-amber-400/10 px-2 py-0.5 text-[10px] text-amber-300">{project.note}</span>
        )}
      </header>
      <LocalRow source={project.local} />
      <GithubRows source={project.github} now={now} />
    </section>
  );
}

function LocalRow({ source }: { source: Source<LocalStatus> }) {
  if (source.state === "off") return null;
  if (source.state === "error") {
    return (
      <Row label="lokal">
        <ErrorText>{source.message}</ErrorText>
      </Row>
    );
  }
  const { branch, changedFiles, ahead, behind } = source.data;
  const clean = changedFiles === 0 && ahead === 0;
  return (
    <Row
      label="lokal"
      aside={
        <span className="font-mono">
          {ahead > 0 && <span className="text-amber-300">↑{ahead} </span>}
          {behind > 0 && <span className="text-sky-300">↓{behind}</span>}
        </span>
      }
    >
      <Dot tone={clean ? "ok" : "busy"} />
      <span className="truncate font-mono text-zinc-200">{branch}</span>
      {changedFiles > 0 && <span className="text-amber-300">{changedFiles} ändrade</span>}
    </Row>
  );
}

function GithubRows({ source, now }: { source: Source<GithubStatus>; now: number }) {
  if (source.state === "off") return null;
  if (source.state === "error") {
    return (
      <Row label="github">
        <ErrorText>{source.message}</ErrorText>
      </Row>
    );
  }
  const { lastCommit, ci, openPrs, prsUrl, url, deploy } = source.data;
  return (
    <>
      <Row label="commit" url={lastCommit?.url ?? url} aside={lastCommit && timeAgo(lastCommit.date, now)}>
        {lastCommit ? (
          <>
            <span className="font-mono text-violet-300">{lastCommit.sha}</span>
            <span className="truncate">{lastCommit.message}</span>
          </>
        ) : (
          <span className="text-zinc-500">inga commits</span>
        )}
      </Row>
      {ci && (
        <Row label="ci" url={ci.url}>
          <Dot tone={ciTone(ci)} />
          <span className="truncate">{ci.name}</span>
          <span className="text-zinc-500">{ciLabel(ci)}</span>
        </Row>
      )}
      <Row label="pr" url={prsUrl}>
        <span className={openPrs > 0 ? "text-sky-300" : "text-zinc-500"}>
          {openPrs === 0 ? "inga öppna" : `${openPrs} öppen${openPrs > 1 ? "a" : ""}`}
        </span>
      </Row>
      {deploy && (
        <Row label="deploy" url={deploy.url} aside={timeAgo(deploy.updatedAt, now)}>
          <Dot tone={deployTone(deploy.state)} />
          <span>{deploy.provider}</span>
          <span className="text-zinc-500">{deployLabel(deploy.state)}</span>
        </Row>
      )}
    </>
  );
}
