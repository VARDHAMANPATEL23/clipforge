//! End to end: source -> analysis (cached) -> moments -> rendered vertical clips.
use crate::ingest::DlProgress;
use crate::llm::Llm;
use crate::media::{pick_encoder, probe};
use crate::settings::{DEFAULT_NICHE, Settings};
use crate::picker::{self, Moment, PickOpts};
use crate::render::{self, Layout, RenderOpts};
use crate::transcribe::Segment;
use crate::{ingest, paths, signals, transcribe};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    pub out: PathBuf,
    pub max_clips: usize,
    pub min_len: f64,
    pub max_len: f64,
    pub target_len: f64,
    pub layout: Layout,
    pub asr: String, // whisper | none
    pub asr_model: String,
    pub language: Option<String>,
    pub llm: String, // none | gemini | lmstudio | ollama | openrouter
    pub llm_model: Option<String>,
    pub vision: bool,
    pub niche: String,
    pub subs: bool,
    pub hook: bool,
    pub normalize_audio: bool,
    pub embed_tag: bool,
    pub codec: String,
    pub quality: String,
    pub software: bool,
    pub workers: usize,
    pub fresh: bool,
}

impl Default for Options {
    fn default() -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        Self {
            out: home.join("Movies").join("clipforge"),
            max_clips: 8,
            min_len: 15.0,
            max_len: 60.0,
            target_len: 30.0,
            layout: Layout::default(),
            asr: "whisper".into(),
            asr_model: "turbo".into(),
            language: None,
            llm: "none".into(),
            llm_model: None,
            vision: false,
            niche: DEFAULT_NICHE.into(),
            subs: true,
            hook: true,
            normalize_audio: true,
            embed_tag: true,
            codec: "h264".into(),
            quality: "high".into(),
            software: false,
            workers: 3,
            fresh: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Stage(&'static str),
    Log(String),
    Progress(DlProgress),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipMeta {
    pub file: Option<String>,
    #[serde(flatten)]
    pub moment: Moment,
    pub caption: String,
}

pub struct Outcome {
    pub out_dir: PathBuf,
    pub clips: Vec<ClipMeta>,
}

fn cache_key(p: &Path) -> Result<String> {
    let md = std::fs::metadata(p)?;
    let mtime = md.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let raw = format!("{}|{}|{}", p.canonicalize()?.display(), md.len(), mtime);
    let digest = Sha1::digest(raw.as_bytes());
    Ok(digest.iter().take(8).map(|b| format!("{b:02x}")).collect())
}

pub fn process(source: &str, opt: &Options, emit: &(dyn Fn(Event) + Sync)) -> Result<Outcome> {
    let t0 = Instant::now();
    let log = |m: &str| emit(Event::Log(m.to_string()));
    let cache_root = paths::cache_dir();
    std::fs::create_dir_all(&cache_root)?;

    emit(Event::Stage("fetch"));
    let video = ingest::fetch(source, &cache_root, &|p| emit(Event::Progress(p)))?;
    let info = probe(&video)?;
    let work = cache_root.join(cache_key(&video)?);
    std::fs::create_dir_all(&work)?;
    if opt.fresh {
        for e in std::fs::read_dir(&work)?.flatten() {
            if e.path().extension().is_some_and(|x| x == "json") {
                std::fs::remove_file(e.path())?;
            }
        }
    }
    let enc = pick_encoder(&opt.codec, &opt.quality, opt.software);
    log(&format!(
        "{}  {}x{} @ {:.0}fps  {:.1} min  encoder: {}",
        video.file_name().unwrap_or_default().to_string_lossy(),
        info.width, info.height, info.fps, info.duration / 60.0, enc.name
    ));

    emit(Event::Stage("audio"));
    let wav = if info.has_audio { Some(ingest::extract_audio(&video, &work.join("audio.wav"))?) } else { None };

    emit(Event::Stage("transcribe"));
    let mut segs: Vec<Segment> = vec![];
    if let (Some(w), true) = (&wav, opt.asr != "none") {
        let cache = work.join(format!("transcript_{}.json", opt.asr_model));
        match transcribe::transcribe(w, &cache, &opt.asr_model, opt.language.as_deref(), &log) {
            Ok(s) => segs = s,
            Err(e) => log(&format!("transcription failed ({e}); continuing without speech")),
        }
    }

    emit(Event::Stage("analyse"));
    let sig = signals::compute(&video, wav.as_deref(), info.duration, &enc, &work.join("signals.json"))?;

    emit(Event::Stage("pick"));
    let llm = match opt.llm.as_str() {
        "none" | "" => None,
        provider => Some(Llm::new(provider, opt.llm_model.as_deref(), opt.vision, &Settings::load())?),
    };
    let moments_cache = work.join(format!(
        "moments_{}_{}_{}_{}{}.json",
        opt.llm, opt.max_clips, opt.min_len as i64, opt.max_len as i64, if opt.vision { "_v" } else { "" }
    ));
    let pick_opts = PickOpts {
        max_clips: opt.max_clips, min_len: opt.min_len, max_len: opt.max_len, target: opt.target_len,
        llm: llm.as_ref(), vision: opt.vision, niche: &opt.niche, video: &video, enc: &enc,
    };
    let moments = picker::pick(&sig, &segs, info.duration, &pick_opts, Some(&moments_cache), &log)?;
    if moments.is_empty() {
        bail!("no moments found");
    }

    let stem = video.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let out_dir = opt.out.join(render::slug(&stem, 60));
    emit(Event::Stage("render"));
    log(&format!("rendering {} clips -> {} ({})", moments.len(), out_dir.display(), opt.layout.mode));
    for (i, m) in moments.iter().enumerate() {
        log(&format!("{:02}. {:7.1}s-{:7.1}s ({:4.1}s) {}", i + 1, m.start, m.end, m.duration(), m.title));
    }
    let ro = RenderOpts { subs: opt.subs, hook: opt.hook, normalize_audio: opt.normalize_audio, embed_tag: opt.embed_tag, workers: opt.workers };
    let paths = render::render_all(&moments, &info, &segs, &out_dir, &enc, &opt.layout, &ro, &log)?;

    let clips: Vec<ClipMeta> = moments
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            let prefix = format!("{:02}_", i + 1);
            let file = paths
                .iter()
                .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .find(|n| n.starts_with(&prefix));
            let caption = format!("{} {}", m.title, m.hashtags.join(" ")).trim().to_string();
            ClipMeta { file, moment: m, caption }
        })
        .collect();
    std::fs::write(out_dir.join("clips.json"), serde_json::to_string_pretty(&clips)?)?;
    log(&format!("{} clips in {:.0}s -> {}", paths.len(), t0.elapsed().as_secs_f64(), out_dir.display()));
    Ok(Outcome { out_dir, clips })
}
