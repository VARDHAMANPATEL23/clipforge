import { useState } from "react";
import { isUrl } from "./api";
import { FacecamPicker } from "./FacecamPicker";
import { Icon } from "./ui";

type Rect = [number, number, number, number];

const pct = (v: number) => `${Math.round(v * 100)}%`;

/** Plain-English description of where the box sits, e.g. "bottom right, 26% wide, 32% tall". */
function describe([x, y, w, h]: Rect): string {
  const cx = x + w / 2;
  const cy = y + h / 2;
  const v = cy < 0.34 ? "top" : cy > 0.66 ? "bottom" : "middle";
  const hz = cx < 0.34 ? "left" : cx > 0.66 ? "right" : "centre";
  const where = v === "middle" && hz === "centre" ? "centre" : `${v} ${hz}`;
  return `${where}, ${pct(w)} wide, ${pct(h)} tall`;
}

function parseRect(draft: string[]): Rect | null {
  if (draft.some((d) => d.trim() === "")) return null;
  const n = draft.map(Number);
  if (!n.every((v) => Number.isFinite(v) && v >= 0 && v <= 1)) return null;
  const [x, y, w, h] = n;
  if (w < 0.02 || h < 0.02 || x + w > 1.001 || y + h > 1.001) return null;
  return [x, y, w, h];
}

function disabledReason(source: string | null, local: string | null): string | null {
  if (local) return null;
  if (!source) return "Choose a video in Source above to turn this on.";
  if (isUrl(source)) return "Not available for URLs yet. Use a local file or upload, or enter the numbers below.";
  return null;
}

export function FacecamRegion({
  value,
  onChange,
  local,
  source,
}: {
  value: Rect | null;
  onChange: (r: Rect | null) => void;
  local: string | null;
  source: string | null;
}) {
  const [picker, setPicker] = useState(false);
  const [rev, setRev] = useState(0);
  const reason = disabledReason(source, local);

  return (
    <div style={{ marginTop: 20 }}>
      <div className="row between wrap" style={{ marginBottom: 10 }}>
        <div className="row">
          <span className="label">Webcam region</span>
          <span className="beta">Beta</span>
        </div>
        <button type="button" className={`btn sm ${value ? "" : "primary"}`} disabled={!local} onClick={() => setPicker(true)}>
          <Icon name="crop" size={13} /> {value ? "Change on frame" : "Pick on frame"}
        </button>
      </div>

      {value ? (
        <div className="region-set">
          <div>
            <div className="label">Webcam: {describe(value)}</div>
            <div className="hint mono">{value.join(", ")}</div>
          </div>
          <button
            type="button"
            className="btn sm ghost"
            onClick={() => {
              onChange(null);
              setRev((r) => r + 1);
            }}
          >
            Clear
          </button>
        </div>
      ) : (
        <ol className="guide">
          <li>Choose your video in <b>Source</b> at the top of this page (upload it, or enter a file path).</li>
          <li>Press <b>Pick on frame</b>. A frame from your video opens.</li>
          <li>Drag a box around your webcam, then press <b>Apply</b>.</li>
        </ol>
      )}
      {reason && <p className="hint" style={{ marginTop: 8 }}>{reason}</p>}
      <p className="hint" style={{ marginTop: 8 }}>
        Beta: check the preview tile and your first clip. The facecam layout needs a region before you can start.
      </p>

      <details className="manual" key={rev}>
        <summary>Enter numbers manually (advanced)</summary>
        <ManualRegion value={value} onChange={onChange} />
      </details>

      {picker && local && (
        <FacecamPicker
          path={local}
          value={value}
          onClose={() => setPicker(false)}
          onApply={(r) => {
            onChange(r);
            setRev((n) => n + 1);
            setPicker(false);
          }}
        />
      )}
    </div>
  );
}

function ManualRegion({ value, onChange }: { value: Rect | null; onChange: (r: Rect | null) => void }) {
  const [draft, setDraft] = useState<string[]>(value ? value.map(String) : ["", "", "", ""]);
  const filled = draft.some((d) => d.trim() !== "");
  const valid = parseRect(draft) !== null;

  const edit = (i: number, text: string) => {
    const next = draft.map((d, j) => (j === i ? text : d));
    setDraft(next);
    onChange(parseRect(next));
  };

  return (
    <div style={{ marginTop: 12 }}>
      <div className="grid g4">
        {(["x", "y", "width", "height"] as const).map((name, i) => (
          <div className="field" key={name}>
            <label>{name}</label>
            <input type="text" inputMode="decimal" placeholder="0 to 1" value={draft[i]} onChange={(e) => edit(i, e.target.value)} />
          </div>
        ))}
      </div>
      <p className="hint" style={{ marginTop: 8 }}>
        Fractions of the full frame, from 0 to 1. x and y are the box's top-left corner. Example for a webcam in the bottom right
        corner: 0.74, 0.685, 0.26, 0.315.
      </p>
      {filled && !valid && (
        <p className="hint" style={{ marginTop: 4 }}>Enter all four values between 0 and 1, with the box inside the frame.</p>
      )}
    </div>
  );
}
