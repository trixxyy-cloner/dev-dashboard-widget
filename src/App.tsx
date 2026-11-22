import { invoke } from "@tauri-apps/api/core";
import { DigitalOceanCard } from "./components/DigitalOceanCard";
import { Header } from "./components/Header";
import { ProjectCard } from "./components/ProjectCard";
import { useDashboard, useNow } from "./hooks/useDashboard";
import { timeAgo } from "./lib/status";

export default function App() {
  const { data, error, loading, refresh } = useDashboard();
  const now = useNow();

  const openConfig = () => {
    invoke("open_config").catch((e: unknown) => console.error(e));
  };

  return (
    <main className="flex h-full flex-col overflow-hidden rounded-2xl border border-white/10 bg-zinc-950/85 text-zinc-200 shadow-2xl">
      <Header
        loading={loading}
        locked={data?.lockPosition ?? false}
        updatedLabel={data ? `uppdaterad ${timeAgo(data.fetchedMs, now)}` : null}
        onRefresh={() => void refresh()}
        onOpenConfig={openConfig}
      />
      <div className="flex-1 space-y-2 overflow-y-auto px-3 pb-3 [scrollbar-width:thin]">
        {error && (
          <p className="rounded-xl border border-rose-500/20 bg-rose-500/10 p-3 text-xs text-rose-300">{error}</p>
        )}
        {data?.projects.map((project) => <ProjectCard key={project.name} project={project} now={now} />)}
        {data && <DigitalOceanCard source={data.digitalocean} now={now} onOpenConfig={openConfig} />}
      </div>
    </main>
  );
}
