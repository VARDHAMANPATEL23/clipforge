import { useEffect, useState } from "react";
import { api } from "./api";
import { Logo } from "./ui";
import { JobPage } from "./pages/JobPage";
import { JobsPage } from "./pages/JobsPage";
import { NewJob } from "./pages/NewJob";
import { SettingsPage } from "./pages/SettingsPage";

const parse = () => (window.location.hash.replace(/^#/, "") || "/").split("/").filter(Boolean);

function useRoute() {
  const [route, setRoute] = useState(parse);
  useEffect(() => {
    const on = () => setRoute(parse());
    window.addEventListener("hashchange", on);
    return () => window.removeEventListener("hashchange", on);
  }, []);
  return route;
}

const go = (hash: string) => {
  window.location.hash = hash;
};

export function App() {
  const route = useRoute();
  const [ready, setReady] = useState<boolean | null>(null);

  useEffect(() => {
    api.health().then((h) => setReady(h.ffmpeg && h.ffprobe)).catch(() => setReady(null));
  }, []);

  const [section, arg] = route;
  const link = (hash: string, label: string, active: boolean) => (
    <a href={hash} className={active ? "active" : ""}>{label}</a>
  );

  return (
    <>
      <header className="topbar">
        <a href="#/" className="brand">
          <Logo size={22} />
          <span>shard<span className="dim">reel</span></span>
        </a>
        <nav className="nav">
          {link("#/", "New job", section === undefined)}
          {link("#/jobs", "Jobs", section === "jobs" || section === "job")}
          {link("#/settings", "Settings", section === "settings")}
        </nav>
        <div className="grow" />
        <span className="row muted" style={{ fontSize: 12.5, gap: 8 }} title={ready === null ? "Backend unreachable" : ready ? "ffmpeg ready" : "ffmpeg missing"}>
          <span className={`dot ${ready ? "on" : ""}`} />
          {ready === null ? "offline" : ready ? "ready" : "setup needed"}
        </span>
      </header>
      {section === "job" && arg ? <JobPage id={arg} go={go} />
        : section === "jobs" ? <JobsPage go={go} />
        : section === "settings" ? <SettingsPage />
        : <NewJob go={go} />}
    </>
  );
}
