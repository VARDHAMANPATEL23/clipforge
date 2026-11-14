//! Render vertical 1080x1920 clips with ffmpeg (VideoToolbox on Mac, NVENC on Linux/WSL with NVIDIA), in parallel.
use crate::media::{Encoder, VideoInfo, has_libass, require, run};
use crate::picker::Moment;
use crate::subtitles::{AssOpts, build_ass};
use crate::transcribe::{Segment, words_in};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const W: u32 = 1080;
const H: u32 = 1920;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    pub mode: String, // blur | crop | facecam | fit
    pub zoom: f64,
    pub crop_x: f64,
    pub facecam: Option<[f64; 4]>, // x,y,w,h as fractions of source
    pub cam_height: u32,
}

impl Default for Layout {
    fn default() -> Self {
        Self { mode: "blur".into(), zoom: 1.35, crop_x: 0.5, facecam: None, cam_height: 640 }
    }
}

#[derive(Clone, Debug)]
pub struct RenderOpts {
    pub subs: bool,
    pub hook: bool,
    pub normalize_audio: bool,
    pub embed_tag: bool,
    pub workers: usize,
}

fn even(x: f64) -> u32 {
    ((x / 2.0).round() as u32 * 2).max(2)
}

pub fn video_filter(info: &VideoInfo, lay: &Layout) -> Result<String> {
    let aspect = info.width as f64 / info.height as f64;
    let cx = lay.crop_x;
    match lay.mode.as_str() {
        "crop" => Ok(format!(
            "[0:v]crop=trunc(ih*9/16/2)*2:ih:(iw-trunc(ih*9/16/2)*2)*{cx:.3}:0,\
             scale={W}:{H}:flags=lanczos,setsar=1[v0]"
        )),
        "facecam" => {
            let Some([fx, fy, fw, fh]) = lay.facecam else {
                bail!("facecam layout needs a facecam region (x,y,w,h fractions)");
            };
            let ch = lay.cam_height;
            let gh = H - ch;
            Ok(format!(
                "[0:v]split=2[a][b];\
                 [a]crop=iw*{fw}:ih*{fh}:iw*{fx}:ih*{fy},\
                 scale={W}:{ch}:force_original_aspect_ratio=increase,crop={W}:{ch}[cam];\
                 [b]scale=-2:{gh}:flags=lanczos,crop={W}:{gh}:(iw-{W})*{cx:.3}:0[game];\
                 [cam][game]vstack,setsar=1[v0]"
            ))
        }
        mode => {
            let zoom = if mode == "fit" { 1.0 } else { lay.zoom.max(1.0) };
            let fg_h = even((H as f64).min(W as f64 / aspect * zoom));
            let fg_w = even(fg_h as f64 * aspect);
            Ok(format!(
                "[0:v]split=2[a][b];\
                 [a]scale=270:480:force_original_aspect_ratio=increase,crop=270:480,\
                 gblur=sigma=10,eq=brightness=-0.12:saturation=1.2,scale={W}:{H}[bg];\
                 [b]scale={fg_w}:{fg_h}:flags=lanczos,crop='min(iw,{W})':ih[fg];\
                 [bg][fg]overlay=(W-w)/2:(H-h)/2,setsar=1[v0]"
            ))
        }
    }
}

pub fn slug(s: &str, n: usize) -> String {
    let cleaned: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || c.is_whitespace())
        .collect();
    let mut out = String::new();
    let mut sep = false;
    for c in cleaned.trim().chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            sep = true;
        } else {
            if sep && !out.is_empty() {
                out.push('-');
            }
            sep = false;
            out.push(c);
        }
    }
    let out: String = out.chars().take(n).collect::<String>().trim_matches('-').to_string();
    if out.is_empty() { "clip".into() } else { out }
}

fn s(x: impl ToString) -> String {
    x.to_string()
}

#[allow(clippy::too_many_arguments)]
pub fn render_one(
    idx: usize, m: &Moment, info: &VideoInfo, segments: &[Segment], out_dir: &Path,
    enc: &Encoder, lay: &Layout, o: &RenderOpts,
) -> Result<PathBuf> {
    let name = format!("{idx:02}_{}", slug(&m.title, 40));
    let out = out_dir.join(format!("{name}.mp4"));
    let tmp = tempfile::Builder::new().prefix("clipforge_").tempdir()?;
    let mut graph = video_filter(info, lay)?;
    let mut last = "v0";
    if (o.subs || (o.hook && !m.hook.is_empty())) && has_libass() {
        let words = if o.subs { words_in(segments, m.start, m.end) } else { vec![] };
        let hook = if o.hook { m.hook.as_str() } else { "" };
        let margin_v = if lay.mode == "facecam" { 760 } else { 560 };
        build_ass(&words, m.start, m.end, &tmp.path().join("subs.ass"), &AssOpts { hook, margin_v })?;
        graph += ";[v0]subtitles=subs.ass[v1]";
        last = "v1";
    }
    let mut cmd: Vec<String> = vec![s("-y"), s("-v"), s("error")];
    cmd.extend(enc.decode_args.iter().cloned());
    cmd.extend([
        s("-ss"), format!("{:.3}", m.start), s("-t"), format!("{:.3}", m.duration()),
        s("-i"), info.path.to_string_lossy().into_owned(),
        s("-filter_complex"), graph, s("-map"), format!("[{last}]"),
    ]);
    if info.has_audio {
        cmd.extend([s("-map"), s("0:a:0")]);
        if o.normalize_audio {
            cmd.extend([s("-af"), s("loudnorm=I=-14:TP=-1.5:LRA=11")]);
        }
        cmd.extend([s("-c:a"), s("aac"), s("-b:a"), s("160k"), s("-ar"), s("48000")]);
    }
    cmd.extend(enc.video_args.iter().cloned());
    if o.embed_tag {
        // bitexact stops ffmpeg stamping its own version; the MP4 muxer then only keeps a stream-level encoder tag
        cmd.extend([s("-fflags"), s("+bitexact"), s("-metadata:s:v:0"), s("encoder=shardreel")]);
        let caption = format!("{} {}", m.title, m.hashtags.join(" ")).trim().to_string();
        for (key, value) in [
            ("comment", "Built with shardreel"),
            ("title", m.title.as_str()),
            ("description", caption.as_str()),
        ] {
            if !value.is_empty() {
                cmd.extend([s("-metadata"), format!("{key}={value}")]);
            }
        }
    }
    cmd.extend([
        s("-pix_fmt"), s("yuv420p"), s("-r"), s(info.fps.round().min(60.0) as u32),
        s("-movflags"), s("+faststart"), s("out.mp4"),
    ]);
    run(&cmd, Some(tmp.path()))?;
    std::fs::copy(tmp.path().join("out.mp4"), &out)?;
    let thumb_t = (m.duration() - 0.1).min(m.peak - m.start).max(0.0);
    run(
        &[
            s("-y"), s("-v"), s("error"), s("-ss"), format!("{thumb_t:.2}"), s("-i"),
            out.to_string_lossy().into_owned(), s("-frames:v"), s("1"), s("-q:v"), s("3"),
            out_dir.join(format!("{name}.jpg")).to_string_lossy().into_owned(),
        ],
        None,
    )?;
    Ok(out)
}

pub fn render_all(
    moments: &[Moment], info: &VideoInfo, segments: &[Segment], out_dir: &Path,
    enc: &Encoder, lay: &Layout, o: &RenderOpts, log: &(dyn Fn(&str) + Sync),
) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(out_dir)?;
    require("ffmpeg")?;
    if (o.subs || o.hook) && !has_libass() {
        log("this ffmpeg has no libass: captions skipped");
    }
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, PathBuf)>> = Mutex::new(vec![]);
    std::thread::scope(|sc| {
        for _ in 0..o.workers.max(1).min(moments.len().max(1)) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                let Some(m) = moments.get(i) else { break };
                match render_one(i + 1, m, info, segments, out_dir, enc, lay, o) {
                    Ok(p) => {
                        log(&format!("rendered {}", p.file_name().unwrap_or_default().to_string_lossy()));
                        results.lock().unwrap().push((i + 1, p));
                    }
                    Err(e) => log(&format!("clip {} failed: {e}", i + 1)),
                }
            });
        }
    });
    let mut r = results.into_inner().unwrap();
    r.sort_by_key(|(i, _)| *i);
    Ok(r.into_iter().map(|(_, p)| p).collect())
}
