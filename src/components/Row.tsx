import { openUrl } from "@tauri-apps/plugin-opener";
import type { ReactNode } from "react";
import type { Tone } from "../types";

const dotColor: Record<Tone, string> = {
  ok: "bg-emerald-400",
  busy: "bg-amber-400 animate-pulse",
  bad: "bg-rose-500",
  idle: "bg-zinc-500",
};

export function Dot({ tone }: { tone: Tone }) {
  return <span className={`size-2 shrink-0 rounded-full ${dotColor[tone]}`} />;
}

type RowProps = {
  label: string;
  url?: string | null;
  children: ReactNode;
  aside?: ReactNode;
};

/** One line in a project card; clickable when it has a URL. */
export function Row({ label, url, children, aside }: RowProps) {
  const content = (
    <>
      <span className="w-12 shrink-0 font-mono text-[10px] tracking-wide text-zinc-500 uppercase">{label}</span>
      <span className="flex min-w-0 flex-1 items-center gap-2 truncate">{children}</span>
      {aside && <span className="shrink-0 text-[11px] text-zinc-500">{aside}</span>}
    </>
  );
  const base = "flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-xs text-zinc-300";

  if (!url) return <div className={base}>{content}</div>;
  return (
    <button type="button" onClick={() => void openUrl(url)} className={`${base} cursor-pointer hover:bg-white/5`} title={url}>
      {content}
    </button>
  );
}

export function ErrorText({ children }: { children: ReactNode }) {
  return <span className="truncate text-rose-400/90">{children}</span>;
}
