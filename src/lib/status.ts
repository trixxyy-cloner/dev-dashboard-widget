import type { CiRun, Tone } from "../types";

export function ciTone(ci: CiRun): Tone {
  if (ci.status !== "completed") return "busy";
  if (ci.conclusion === "success") return "ok";
  if (ci.conclusion === "skipped" || ci.conclusion === "cancelled") return "idle";
  return "bad";
}

export function ciLabel(ci: CiRun): string {
  if (ci.status !== "completed") return ci.status === "queued" ? "i kö" : "kör";
  return ci.conclusion ?? "klar";
}

export function vercelTone(state: string): Tone {
  if (state === "READY") return "ok";
  if (state === "ERROR") return "bad";
  if (state === "CANCELED") return "idle";
  return "busy";
}

export function doTone(phase: string): Tone {
  if (phase === "ACTIVE") return "ok";
  if (phase === "ERROR" || phase === "CANCELED") return "bad";
  if (phase === "UNKNOWN" || phase === "SUPERSEDED") return "idle";
  return "busy";
}

const rtf = new Intl.RelativeTimeFormat("sv", { numeric: "auto" });

export function timeAgo(input: string | number, now: number): string {
  const seconds = Math.round((new Date(input).getTime() - now) / 1000);
  const abs = Math.abs(seconds);
  if (abs < 60) return "nyss";
  if (abs < 3600) return rtf.format(Math.round(seconds / 60), "minute");
  if (abs < 86400) return rtf.format(Math.round(seconds / 3600), "hour");
  if (abs < 86400 * 30) return rtf.format(Math.round(seconds / 86400), "day");
  return rtf.format(Math.round(seconds / (86400 * 30)), "month");
}
