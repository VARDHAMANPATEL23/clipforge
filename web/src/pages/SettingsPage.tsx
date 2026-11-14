import { useEffect, useState } from "react";
import { api } from "../api";
import type { Health, Options, SettingsUpdate, SettingsView, StorageReport } from "../types";
import { Card, Field, fmtBytes } from "../ui";

function Check({ label, ok, hint }: { label: string; ok: boolean; hint?: string }) {
  return (
    <div className="kv">
      <div>
        <div>{label}</div>
        {!ok && hint && <div className="hint mono">{hint}</div>}
      </div>
      <span className={`badge ${ok ? "solid" : "fail"}`}>{ok ? "OK" : "Missing"}</span>
    </div>
  );
}

interface StorageRow {
  kind: string;
  label: string;
  hint: string;
  bytes: number;
}

function StorageCard() {
  const [report, setReport] = useState<StorageReport | null>(null);
  const [confirm, setConfirm] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = () => api.storage().then(setReport).catch((e: Error) => setError(e.message));
  useEffect(() => {
    load();
  }, []);

  async function clear(kind: string) {
    if (confirm !== kind) {
      setConfirm(kind);
      return;
    }
    setBusy(true);
    setError(null);
    setMsg(null);
    try {
      const { freed } = await api.clearStorage(kind);
      setMsg(`Freed ${fmtBytes(freed)}`);
      await load();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
      setConfirm(null);
    }
  }

  const rows: StorageRow[] = report
    ? [
        { kind: "downloads", label: "Downloads", hint: "Videos fetched from URLs. They download again next time.", bytes: report.downloads.bytes },
        { kind: "analysis", label: "Analysis cache", hint: "Audio, transcripts and picks. Redone on the next run.", bytes: report.analysis.bytes },
        { kind: "uploads", label: "Uploads", hint: "Files uploaded from the browser. Jobs made from them cannot be re-rendered.", bytes: report.uploads.bytes },
        ...report.models.map((m) => ({
          kind: m.name,
          label: `Whisper model: ${m.name}`,
          hint: "Downloads again on next use.",
          bytes: m.bytes,
        })),
      ]
    : [];

  return (
    <Card title="Storage" right={report ? <span className="mono muted">{fmtBytes(report.total)} total</span> : undefined}>
      {error && <div className="alert" style={{ marginBottom: 12 }}>{error}</div>}
      {msg && <p className="muted" style={{ marginBottom: 8 }}>{msg}</p>}
      {!report ? (
        <span className="muted">Measuring...</span>
      ) : (
        <>
          {rows.map((r) => (
            <div className="kv" key={r.kind}>
              <div style={{ minWidth: 0 }}>
                <div className="trunc">{r.label}</div>
                <div className="hint">{r.hint}</div>
              </div>
              <div className="row">
                <span className="mono muted">{fmtBytes(r.bytes)}</span>
                <button className={`btn sm ${confirm === r.kind ? "primary" : ""}`} disabled={busy || r.bytes === 0} onClick={() => clear(r.kind)} onBlur={() => setConfirm(null)}>
                  {confirm === r.kind ? "Confirm clear" : "Clear"}
                </button>
              </div>
            </div>
          ))}
          <p className="hint" style={{ marginTop: 12 }}>
            Rendered clips in your output folders are never deleted. Clearing is blocked while a job runs.
          </p>
        </>
      )}
    </Card>
  );
}

export function SettingsPage() {
  const [health, setHealth] = useState<Health | null>(null);
  const [view, setView] = useState<SettingsView | null>(null);
  const [form, setForm] = useState<SettingsUpdate>({});
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.health().then(setHealth).catch((e: Error) => setError(e.message));
    api.settings().then(setView).catch((e: Error) => setError(e.message));
  }, []);

  const val = <K extends keyof SettingsUpdate>(k: K, fallback: string) => (form[k] as string | undefined) ?? fallback;
  const patch = (p: SettingsUpdate) => {
    setForm((f) => ({ ...f, ...p }));
    setSaved(false);
  };

  async function save() {
    try {
      setView(await api.saveSettings(form));
      setForm({});
      setSaved(true);
      setError(null);
    } catch (e) {
      setError((e as Error).message);
    }
  }

  const keyField = (label: string, field: "gemini_api_key" | "openrouter_api_key", has: boolean) => (
    <Field label={label} hint={has ? "A key is set. Type to replace, or clear it." : "No key set."}>
      <div className="row">
        <input
          type="password"
          autoComplete="off"
          placeholder={has ? "••••••••••••" : "Paste key"}
          value={form[field] ?? ""}
          onChange={(e) => patch({ [field]: e.target.value })}
        />
        {has && <button type="button" className="btn" onClick={() => patch({ [field]: "" })}>Clear</button>}
      </div>
    </Field>
  );

  return (
    <div className="page">
      <div className="page-head">
        <h1>Settings</h1>
        <p>AI providers and system checks.</p>
      </div>
      {error && <div className="alert section">{error}</div>}

      <Card title="System">
        {!health ? (
          <span className="muted">Checking...</span>
        ) : (
          <>
            <Check label="Apple Silicon" ok={health.apple_silicon} />
            <Check label="ffmpeg" ok={health.ffmpeg} hint="brew install ffmpeg" />
            <Check label="ffprobe" ok={health.ffprobe} hint="brew install ffmpeg" />
            <Check label={`Hardware encoder (${health.encoder})`} ok={health.hardware_encoder} />
            <Check label="Captions (libass in ffmpeg)" ok={health.libass} hint="Needs an ffmpeg build with libass: brew install homebrew-ffmpeg/ffmpeg/ffmpeg" />
            <Check label="yt-dlp (URL downloads)" ok={health.yt_dlp} hint="brew install yt-dlp" />
            <Check label="Whisper model (downloads on first run, ~1.6 GB)" ok={health.whisper_model} />
          </>
        )}
      </Card>

      <StorageCard />

      <Card
        title="AI ranking"
        right={saved ? <span className="muted">Saved</span> : undefined}
      >
        {!view ? (
          <span className="muted">Loading...</span>
        ) : (
          <div className="stack">
            <div className="grid g2">
              <Field label="Default provider">
                <select value={val("llm", view.llm)} onChange={(e) => patch({ llm: e.target.value as Options["llm"] })}>
                  <option value="none">None</option>
                  <option value="gemini">Gemini</option>
                  <option value="openrouter">OpenRouter</option>
                  <option value="lmstudio">LM Studio (local)</option>
                  <option value="ollama">Ollama (local)</option>
                </select>
              </Field>
              <Field label="Default model" hint="Empty uses the provider default.">
                <input type="text" className="mono-input" value={val("llm_model", view.llm_model ?? "")} onChange={(e) => patch({ llm_model: e.target.value })} />
              </Field>
            </div>
            <div className="grid g2">
              {keyField("Gemini API key", "gemini_api_key", view.has_gemini_key)}
              {keyField("OpenRouter API key", "openrouter_api_key", view.has_openrouter_key)}
            </div>
            <Field label="Base URL override" hint="Optional. Any OpenAI-compatible endpoint, e.g. http://localhost:1234/v1">
              <input type="text" className="mono-input" value={val("base_url", view.base_url ?? "")} onChange={(e) => patch({ base_url: e.target.value })} />
            </Field>
            <Field label="Default niche context" hint="Tells the model what game and audience the clips are for.">
              <textarea value={val("niche", view.niche)} onChange={(e) => patch({ niche: e.target.value })} />
            </Field>
            <div className="row" style={{ justifyContent: "flex-end" }}>
              <button className="btn primary" onClick={save} disabled={Object.keys(form).length === 0}>Save</button>
            </div>
          </div>
        )}
      </Card>
    </div>
  );
}
