import { doTone, timeAgo } from "../lib/status";
import type { DoApp, Source } from "../types";
import { Dot, ErrorText, Row } from "./Row";

type Props = { source: Source<DoApp[]>; now: number; onOpenConfig: () => void };

export function DigitalOceanCard({ source, now, onOpenConfig }: Props) {
  return (
    <section className="rounded-xl border border-white/5 bg-white/[0.03] p-2">
      <header className="flex items-center justify-between gap-2 px-2 pb-1">
        <h2 className="text-sm font-semibold text-zinc-100">DigitalOcean</h2>
        {source.state === "ok" && source.data.length > 0 && (
          // Every running app is billed, so make the count hard to miss.
          <span className="rounded-full bg-amber-400/10 px-2 py-0.5 text-[10px] text-amber-300">
            {source.data.length} app{source.data.length > 1 ? "ar" : ""} kostar
          </span>
        )}
      </header>
      {source.state === "off" && (
        <button
          type="button"
          onClick={onOpenConfig}
          className="w-full cursor-pointer rounded-md px-2 py-1 text-left text-xs text-zinc-500 hover:bg-white/5"
        >
          Lägg till <span className="font-mono text-zinc-300">digitaloceanToken</span> i config →
        </button>
      )}
      {source.state === "error" && (
        <Row label="api">
          <ErrorText>{source.message}</ErrorText>
        </Row>
      )}
      {source.state === "ok" && source.data.length === 0 && (
        <Row label="appar">
          <span className="text-zinc-500">inga appar körs</span>
        </Row>
      )}
      {source.state === "ok" &&
        source.data.map((app) => (
          <Row key={app.consoleUrl} label="app" url={app.consoleUrl} aside={app.updatedAt && timeAgo(app.updatedAt, now)}>
            <Dot tone={doTone(app.phase)} />
            <span className="truncate text-zinc-200">{app.name}</span>
            <span className="text-zinc-500">{app.phase.toLowerCase()}</span>
          </Row>
        ))}
    </section>
  );
}
