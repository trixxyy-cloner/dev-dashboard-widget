import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import type { Dashboard } from "../types";

const DEFAULT_REFRESH_SECONDS = 120;

export function useDashboard() {
  const [data, setData] = useState<Dashboard | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setData(await invoke<Dashboard>("get_dashboard"));
      setError(null);
    } catch (e) {
      // Rust commands reject with a plain string message.
      setError(typeof e === "string" ? e : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // "Uppdatera" in the tray menu.
  useEffect(() => {
    const unlisten = listen("refresh", () => void refresh());
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [refresh]);

  // "Lås position" in the tray menu.
  useEffect(() => {
    const unlisten = listen<boolean>("lock-changed", (event) => {
      setData((current) => current && { ...current, lockPosition: event.payload });
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

  const refreshSeconds = data?.refreshSeconds ?? DEFAULT_REFRESH_SECONDS;
  useEffect(() => {
    const id = window.setInterval(() => void refresh(), refreshSeconds * 1000);
    return () => window.clearInterval(id);
  }, [refresh, refreshSeconds]);

  return { data, error, loading, refresh };
}

/** Re-renders periodically so "5 min sedan" labels stay current between fetches. */
export function useNow(intervalMs = 30_000) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), intervalMs);
    return () => window.clearInterval(id);
  }, [intervalMs]);
  return now;
}
