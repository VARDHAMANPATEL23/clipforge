import { useEffect, useRef, useState } from "react";
import { api, frameUrl } from "./api";
import { Icon, fmtTime } from "./ui";

type Rect = [number, number, number, number];

const clamp = (v: number) => Math.min(1, Math.max(0, v));
const r3 = (v: number) => Math.round(v * 1000) / 1000;

export function FacecamPicker({
  path,
  value,
  onApply,
  onClose,
}: {
  path: string;
  value: Rect | null;
  onApply: (r: Rect) => void;
  onClose: () => void;
}) {
  const [duration, setDuration] = useState(0);
  const [t, setT] = useState(1);
  const [shownT, setShownT] = useState(1);
  const [rect, setRect] = useState<Rect | null>(value);
  const [error, setError] = useState<string | null>(null);
  const frame = useRef<HTMLDivElement>(null);
  const start = useRef<[number, number] | null>(null);

  useEffect(() => {
    api.probe(path).then((i) => setDuration(i.duration)).catch((e: Error) => setError(e.message));
  }, [path]);

  useEffect(() => {
    const id = setTimeout(() => setShownT(t), 250);
    return () => clearTimeout(id);
  }, [t]);

  const point = (e: React.PointerEvent): [number, number] => {
    const b = frame.current!.getBoundingClientRect();
    return [clamp((e.clientX - b.left) / b.width), clamp((e.clientY - b.top) / b.height)];
  };

  const update = (a: [number, number], b: [number, number]) =>
    setRect([r3(Math.min(a[0], b[0])), r3(Math.min(a[1], b[1])), r3(Math.abs(a[0] - b[0])), r3(Math.abs(a[1] - b[1]))]);

  const ok = rect !== null && rect[2] > 0.02 && rect[3] > 0.02;

  return (
    <div className="modal-bg" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <div>
            <h2>Facecam region</h2>
            <p className="muted">Drag on the frame to draw a box around your webcam.</p>
          </div>
          <button className="btn ghost sm" onClick={onClose} aria-label="Close">
            <Icon name="x" />
          </button>
        </div>
        <div
          className="frame"
          ref={frame}
          onPointerDown={(e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
            start.current = point(e);
            update(start.current, start.current);
          }}
          onPointerMove={(e) => start.current && update(start.current, point(e))}
          onPointerUp={() => (start.current = null)}
        >
          <img src={frameUrl(path, shownT)} alt="Video frame" />
          {rect && <div className="box" style={{ left: `${rect[0] * 100}%`, top: `${rect[1] * 100}%`, width: `${rect[2] * 100}%`, height: `${rect[3] * 100}%` }} />}
        </div>
        <div className="modal-foot">
          <div className="row grow">
            <span className="mono muted">{fmtTime(t)}</span>
            <input
              type="range"
              min={0}
              max={Math.max(1, duration)}
              step={0.5}
              value={t}
              onChange={(e) => setT(Number(e.target.value))}
              disabled={!duration}
              aria-label="Frame time"
            />
          </div>
          <span className="mono muted">{rect ? rect.join(", ") : "x, y, w, h"}</span>
          <button className="btn" onClick={() => setRect(null)} disabled={!rect}>
            Reset
          </button>
          <button className="btn primary" disabled={!ok} onClick={() => rect && onApply(rect)}>
            Apply
          </button>
        </div>
        {error && <div className="alert" style={{ margin: "0 18px 18px" }}>{error}</div>}
      </div>
    </div>
  );
}
