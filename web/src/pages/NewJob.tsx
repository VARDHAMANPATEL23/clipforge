import { useEffect, useRef, useState } from "react";
import { api, isUrl, upload } from "../api";
import { OptionsForm } from "../OptionsForm";
import type { Options } from "../types";
import { Card, Field, Icon, Seg } from "../ui";

function validate(source: string, o: Options): string | null {
  if (!source.trim()) return "Choose a video: upload a file or enter a path or URL.";
  const nums = [o.max_clips, o.min_len, o.target_len, o.max_len, o.workers, o.layout.cam_height];
  if (nums.some((n) => !Number.isFinite(n))) return "Fill in every number field.";
  if (!(o.min_len <= o.target_len && o.target_len <= o.max_len)) return "Lengths must satisfy min <= target <= max.";
  if (!o.out.trim()) return "Set an output folder.";
  if (o.layout.mode === "facecam" && !o.layout.facecam) return "The facecam layout needs a webcam region.";
  return null;
}

export function NewJob({ go }: { go: (hash: string) => void }) {
  const [options, setOptions] = useState<Options | null>(null);
  const [tab, setTab] = useState<"upload" | "path">("upload");
  const [source, setSource] = useState("");
  const [fileName, setFileName] = useState<string | null>(null);
  const [pct, setPct] = useState<number | null>(null);
  const [over, setOver] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    api.defaults().then(setOptions).catch((e: Error) => setError(`Cannot reach the backend: ${e.message}`));
  }, []);

  async function handleFile(file: File | undefined) {
    if (!file) return;
    setError(null);
    setPct(0);
    try {
      const res = await upload(file, setPct);
      setSource(res.path);
      setFileName(res.name);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPct(null);
    }
  }

  async function submit() {
    if (!options) return;
    const problem = validate(source, options);
    if (problem) return setError(problem);
    setBusy(true);
    setError(null);
    try {
      const job = await api.createJob(source.trim(), options);
      go(`#/job/${job.id}`);
    } catch (e) {
      setError((e as Error).message);
      setBusy(false);
    }
  }

  return (
    <div className="page">
      <div className="page-head">
        <h1>New job</h1>
        <p>Turn a long recording into vertical clips for Shorts, Reels and TikTok.</p>
      </div>

      <Card
        title="Source"
        right={
          <Seg
            value={tab}
            onChange={(t) => {
              setTab(t);
              setSource("");
              setFileName(null);
            }}
            options={[{ value: "upload", label: "Upload" }, { value: "path", label: "Path or URL" }]}
          />
        }
      >
        {tab === "upload" ? (
          <>
            <input ref={input} type="file" accept="video/*,.mkv,.ts" hidden onChange={(e) => handleFile(e.target.files?.[0])} />
            <div
              className={`drop ${over ? "over" : ""} ${source ? "has" : ""}`}
              onClick={() => pct === null && input.current?.click()}
              onDragOver={(e) => {
                e.preventDefault();
                setOver(true);
              }}
              onDragLeave={() => setOver(false)}
              onDrop={(e) => {
                e.preventDefault();
                setOver(false);
                handleFile(e.dataTransfer.files[0]);
              }}
            >
              {pct !== null ? (
                <div style={{ width: "100%", maxWidth: 380 }} className="stack">
                  <span>Uploading... {Math.round(pct)}%</span>
                  <div className="progress">
                    <i style={{ width: `${pct}%` }} />
                  </div>
                </div>
              ) : source ? (
                <>
                  <div className="row grow" style={{ minWidth: 0 }}>
                    <Icon name="film" />
                    <div className="grow">
                      <div className="trunc">{fileName}</div>
                      <div className="hint mono trunc">{source}</div>
                    </div>
                  </div>
                  <span className="btn sm">Replace</span>
                </>
              ) : (
                <>
                  <Icon name="upload" size={22} />
                  <b>Drop a video here, or click to browse</b>
                  <span className="hint">Copied into ~/.clipforge/uploads. For big files, use Path or URL instead.</span>
                </>
              )}
            </div>
          </>
        ) : (
          <Field label="Local file path, or a YouTube / Twitch URL" hint={isUrl(source) ? "URLs are downloaded with yt-dlp." : "Used in place, nothing is copied."}>
            <input type="text" className="mono-input" placeholder="/Users/you/Movies/session.mp4" value={source} onChange={(e) => setSource(e.target.value)} />
          </Field>
        )}
      </Card>

      {options && <OptionsForm options={options} onChange={setOptions} source={source || null} />}

      {error && <div className="alert section">{error}</div>}
      <div className="row" style={{ justifyContent: "flex-end" }}>
        <button className="btn primary" disabled={busy || !options || pct !== null} onClick={submit}>
          {busy ? "Starting..." : "Generate clips"}
        </button>
      </div>
    </div>
  );
}
