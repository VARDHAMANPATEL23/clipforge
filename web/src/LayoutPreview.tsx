import type { CSSProperties, ReactNode } from "react";
import type { Layout } from "./types";

const W = 1080;
const H = 1920;

interface Props {
  layout: Layout;
  src?: string;
  aspect?: number;
  width?: number;
}

/** Mirrors the ffmpeg filter graphs in render.rs, scaled down to `width` px. */
export function LayoutPreview({ layout, src, aspect = 16 / 9, width = 120 }: Props) {
  const k = width / W;
  const wp = width;
  const hp = H * k;

  const pic = (style: CSSProperties) =>
    src ? <img src={src} style={style} alt="" draggable={false} /> : <div className="ph" style={style} />;
  const panel = (top: number, height: number, child: ReactNode) => (
    <div style={{ position: "absolute", left: 0, top, width: wp, height, overflow: "hidden" }}>{child}</div>
  );

  let body: ReactNode;
  if (layout.mode === "crop") {
    const w = hp * aspect;
    body = pic({ left: -(w - wp) * layout.crop_x, top: 0, width: w, height: hp });
  } else if (layout.mode === "facecam") {
    const ch = layout.cam_height * k;
    const gh = hp - ch;
    const gw = gh * aspect;
    const game = panel(ch, gh, pic({ left: -(gw - wp) * layout.crop_x, top: 0, width: gw, height: gh }));
    let cam: ReactNode;
    if (layout.facecam) {
      const [fx, fy, fw, fh] = layout.facecam;
      const sw = aspect * 1000;
      const sh = 1000;
      const scale = Math.max(wp / (fw * sw), ch / (fh * sh));
      cam = pic({
        left: -fx * sw * scale - (fw * sw * scale - wp) / 2,
        top: -fy * sh * scale - (fh * sh * scale - ch) / 2,
        width: sw * scale,
        height: sh * scale,
      });
    } else {
      cam = <div className="ph" style={{ left: 0, top: 0, width: wp, height: ch }} />;
    }
    body = (
      <>
        {panel(0, ch, cam)}
        {game}
        <div className="sep" style={{ top: ch }} />
      </>
    );
  } else {
    const zoom = layout.mode === "fit" ? 1 : Math.max(1, layout.zoom);
    const fgH = Math.min(H, (W / aspect) * zoom) * k;
    const fgW = fgH * aspect;
    body = (
      <>
        {pic({ left: 0, top: 0, width: wp, height: hp, objectFit: "cover", filter: "blur(6px) brightness(.55)" })}
        {pic({ left: (wp - fgW) / 2, top: (hp - fgH) / 2, width: fgW, height: fgH })}
      </>
    );
  }

  return (
    <div className="lp" style={{ width: wp, height: hp }}>
      {body}
    </div>
  );
}
