import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ReactNode } from "react";

type Props = {
  loading: boolean;
  updatedLabel: string | null;
  onRefresh: () => void;
  onOpenConfig: () => void;
};

export function Header({ loading, updatedLabel, onRefresh, onOpenConfig }: Props) {
  return (
    // The whole header is the drag handle since the window has no title bar.
    <header data-tauri-drag-region className="flex cursor-grab items-center gap-2 px-3 pt-3 pb-2">
      <span data-tauri-drag-region className="font-mono text-xs text-zinc-400">
        <span className="text-emerald-400">~/</span>dev-dashboard
      </span>
      <span data-tauri-drag-region className="flex-1 truncate text-right text-[10px] text-zinc-500">
        {loading ? "hämtar…" : updatedLabel}
      </span>
      <IconButton title="Uppdatera" onClick={onRefresh}>
        <svg viewBox="0 0 24 24" className={`size-3.5 ${loading ? "animate-spin" : ""}`} fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M21 12a9 9 0 1 1-2.64-6.36M21 3v6h-6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </IconButton>
      <IconButton title="Öppna config" onClick={onOpenConfig}>
        <svg viewBox="0 0 24 24" className="size-3.5" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M4 6h16M4 12h16M4 18h10" strokeLinecap="round" />
        </svg>
      </IconButton>
      {/* Hides to the tray instead of quitting; "Avsluta" lives in the tray menu. */}
      <IconButton title="Göm (finns kvar i systemfältet)" onClick={() => void getCurrentWindow().hide()}>
        <svg viewBox="0 0 24 24" className="size-3.5" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M6 6l12 12M18 6L6 18" strokeLinecap="round" />
        </svg>
      </IconButton>
    </header>
  );
}

function IconButton({ title, onClick, children }: { title: string; onClick: () => void; children: ReactNode }) {
  return (
    <button
      type="button"
      title={title}
      aria-label={title}
      onClick={onClick}
      className="grid size-6 cursor-pointer place-items-center rounded-md text-zinc-400 hover:bg-white/10 hover:text-zinc-100"
    >
      {children}
    </button>
  );
}
