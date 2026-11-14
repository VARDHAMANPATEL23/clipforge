import { useEffect, useState } from "react";
import { api, frameUrl, isUrl } from "./api";
import { FacecamRegion } from "./FacecamRegion";
import { LayoutPreview } from "./LayoutPreview";
import type { Layout, LayoutMode, Options } from "./types";
import { Card, Field, NumInput, Seg, Switch } from "./ui";

const MODES: { value: LayoutMode; label: string; hint: string }[] = [
  { value: "blur", label: "Blur", hint: "Zoomed gameplay over blurred copy" },
  { value: "crop", label: "Crop", hint: "Full-height 9:16 slice" },
  { value: "facecam", label: "Facecam", hint: "Webcam on top, game below" },
  { value: "fit", label: "Fit", hint: "Whole frame, no zoom" },
];

interface Props {
  options: Options;
  onChange: (o: Options) => void;
  /** Local file path (or URL). Frame previews and the facecam picker need a local path. */
  source: string | null;
}

export function OptionsForm({ options: o, onChange, source }: Props) {
  const local = source && !isUrl(source) ? source : null;
  const [aspect, setAspect] = useState(16 / 9);
  const [mac, setMac] = useState(false);

  useEffect(() => {
    api.health().then((h) => setMac(h.platform === "macos")).catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!local) return;
    api.probe(local).then((i) => i.height && setAspect(i.width / i.height)).catch(() => undefined);
  }, [local]);

  const set = (patch: Partial<Options>) => onChange({ ...o, ...patch });
  const setLayout = (patch: Partial<Layout>) => set({ layout: { ...o.layout, ...patch } });
  const L = o.layout;
  const preview = local ? frameUrl(local, 3) : undefined;

  return (
    <>
      <Card title="Clips">
        <div className="grid g4">
          <Field label="Count">
            <NumInput value={o.max_clips} min={1} max={30} onChange={(v) => set({ max_clips: v })} />
          </Field>
          <Field label="Min length (s)">
            <NumInput value={o.min_len} min={5} onChange={(v) => set({ min_len: v })} />
          </Field>
          <Field label="Target (s)">
            <NumInput value={o.target_len} min={5} onChange={(v) => set({ target_len: v })} />
          </Field>
          <Field label="Max length (s)">
            <NumInput value={o.max_len} min={5} onChange={(v) => set({ max_len: v })} />
          </Field>
        </div>
        <div style={{ marginTop: 16 }}>
          <Field label="Output folder" hint="Rendered clips are saved here, in a subfolder named after the video.">
            <input type="text" className="mono-input" value={o.out} onChange={(e) => set({ out: e.target.value })} />
          </Field>
        </div>
      </Card>

      <Card title="Layout">
        <div className="tiles">
          {MODES.map((m) => (
            <button key={m.value} type="button" className={`tile ${L.mode === m.value ? "on" : ""}`} onClick={() => setLayout({ mode: m.value })}>
              <LayoutPreview layout={{ ...L, mode: m.value }} src={preview} aspect={aspect} width={84} />
              <b>
                {m.label}
                {m.value === "facecam" && <span className="beta">Beta</span>}
              </b>
              <span>{m.hint}</span>
            </button>
          ))}
        </div>
        <div className="grid g2" style={{ marginTop: 20 }}>
          {L.mode === "blur" && (
            <Field label={`Zoom  ${L.zoom.toFixed(2)}x`} hint="How far to punch into the gameplay.">
              <input type="range" min={1} max={1.8} step={0.05} value={L.zoom} onChange={(e) => setLayout({ zoom: Number(e.target.value) })} />
            </Field>
          )}
          {(L.mode === "crop" || L.mode === "facecam") && (
            <Field label={`Horizontal position  ${Math.round(L.crop_x * 100)}%`} hint="0 is left, 100 is right.">
              <input type="range" min={0} max={1} step={0.01} value={L.crop_x} onChange={(e) => setLayout({ crop_x: Number(e.target.value) })} />
            </Field>
          )}
          {L.mode === "facecam" && (
            <Field label="Webcam panel height (px of 1920)">
              <NumInput value={L.cam_height} min={200} max={1200} step={10} onChange={(v) => setLayout({ cam_height: v })} />
            </Field>
          )}
        </div>
        {L.mode === "facecam" && (
          <FacecamRegion value={L.facecam} onChange={(r) => setLayout({ facecam: r })} local={local} source={source} />
        )}
      </Card>

      <Card title="Overlays and audio">
        <Switch label="Captions" hint="Word-by-word highlighted captions" checked={o.subs} onChange={(v) => set({ subs: v })} />
        <Switch label="Hook banner" hint="Short on-screen hook in the first seconds" checked={o.hook} onChange={(v) => set({ hook: v })} />
        <Switch label="Normalize loudness" hint="-14 LUFS, suited to Shorts, Reels and TikTok" checked={o.normalize_audio} onChange={(v) => set({ normalize_audio: v })} />
        <Switch label="Embed shardreel tag" hint="Writes 'Built with shardreel' plus title and caption into the video metadata. No watermark." checked={o.embed_tag} onChange={(v) => set({ embed_tag: v })} />
      </Card>

      <Card title="Transcription">
        <div className="row wrap" style={{ gap: 20, alignItems: "flex-end" }}>
          <Field label="Speech recognition">
            <Seg value={o.asr} onChange={(v) => set({ asr: v })} options={[{ value: "whisper", label: "Whisper" }, { value: "none", label: "Off" }]} />
          </Field>
          {o.asr === "whisper" && (
            <>
              <div style={{ width: 200 }}>
                <Field label="Model">
                  <select value={o.asr_model} onChange={(e) => set({ asr_model: e.target.value as Options["asr_model"] })}>
                    <option value="turbo">large-v3-turbo</option>
                    <option value="large">large-v3</option>
                    <option value="small">small (fast)</option>
                  </select>
                </Field>
              </div>
              <div style={{ width: 140 }}>
                <Field label="Language">
                  <input type="text" placeholder="auto" value={o.language ?? ""} onChange={(e) => set({ language: e.target.value.trim() || null })} />
                </Field>
              </div>
            </>
          )}
        </div>
        {o.asr === "none" && <p className="hint" style={{ marginTop: 10 }}>Off: no captions, and moments come from audio and visual signals only.</p>}
      </Card>

      <Card title="AI ranking">
        <div className="grid g2">
          <Field label="Provider" hint="Keys live in Settings. None uses signal ranking only.">
            <select value={o.llm} onChange={(e) => set({ llm: e.target.value as Options["llm"] })}>
              <option value="none">None</option>
              <option value="gemini">Gemini</option>
              <option value="openrouter">OpenRouter</option>
              <option value="lmstudio">LM Studio (local)</option>
              <option value="ollama">Ollama (local)</option>
            </select>
          </Field>
          {o.llm !== "none" && (
            <Field label="Model" hint="Leave empty for the provider default.">
              <input type="text" className="mono-input" value={o.llm_model ?? ""} onChange={(e) => set({ llm_model: e.target.value.trim() || null })} />
            </Field>
          )}
        </div>
        {o.llm !== "none" && (
          <div style={{ marginTop: 8 }}>
            <Switch label="Vision" hint="Send frames of each candidate. Good for silent gameplay." checked={o.vision} onChange={(v) => set({ vision: v })} />
            <div style={{ marginTop: 8 }}>
              <Field label="Niche context">
                <textarea value={o.niche} onChange={(e) => set({ niche: e.target.value })} />
              </Field>
            </div>
          </div>
        )}
      </Card>

      <Card title="Encoding">
        <div className="row wrap" style={{ gap: 20, alignItems: "flex-end" }}>
          <Field label="Codec">
            <Seg value={o.codec} onChange={(v) => set({ codec: v })} options={[{ value: "h264", label: "H.264" }, { value: "hevc", label: "HEVC" }]} />
          </Field>
          <Field label="Quality">
            <Seg value={o.quality} onChange={(v) => set({ quality: v })} options={[{ value: "high", label: "High" }, { value: "medium", label: "Medium" }, { value: "low", label: "Low" }]} />
          </Field>
          <div style={{ width: 120 }}>
            <Field label="Parallel renders">
              <NumInput value={o.workers} min={1} max={8} onChange={(v) => set({ workers: v })} />
            </Field>
          </div>
        </div>
        <div style={{ marginTop: 8 }}>
          {!mac && (
            <Switch label="Software encoding" hint="Force libx264/libx265 instead of the GPU encoder (NVENC)" checked={o.software} onChange={(v) => set({ software: v })} />
          )}
          <Switch label="Fresh analysis" hint="Ignore cached transcript, signals and picks" checked={o.fresh} onChange={(v) => set({ fresh: v })} />
        </div>
      </Card>

    </>
  );
}
