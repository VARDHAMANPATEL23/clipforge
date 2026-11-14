use crate::media::{require, run};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DlProgress {
    pub percent: f64,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: Option<f64>,
    pub eta: Option<u64>,
    pub kind: String, // video | audio
}

const MARK: &str = "CFP|";
const TEMPLATE: &str = "CFP|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress.speed)s|%(progress.eta)s|%(info.vcodec)s";

fn parse_progress(line: &str) -> Option<DlProgress> {
    let f: Vec<&str> = line.strip_prefix(MARK)?.split('|').collect();
    let num = |i: usize| f.get(i).and_then(|v| v.trim().parse::<f64>().ok());
    let downloaded = num(0)? as u64;
    let total = num(1).or_else(|| num(2)).filter(|t| *t > 0.0).map(|t| t as u64);
    let percent = total.map_or(0.0, |t| (downloaded as f64 / t as f64 * 100.0).min(100.0));
    let kind = if f.get(5).is_some_and(|v| v.trim() == "none") { "audio" } else { "video" };
    Some(DlProgress { percent, downloaded, total, speed: num(3), eta: num(4).map(|e| e as u64), kind: kind.into() })
}

pub fn fetch(source: &str, workdir: &Path, on_progress: &dyn Fn(DlProgress)) -> Result<PathBuf> {
    if !is_url(source) {
        let expanded = match source.strip_prefix("~/") {
            Some(rest) => PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(rest),
            None => PathBuf::from(source),
        };
        return expanded.canonicalize().with_context(|| format!("file not found: {source}"));
    }
    let dl = workdir.join("downloads");
    std::fs::create_dir_all(&dl)?;
    let mut child = Command::new(require("yt-dlp")?)
        .args([
            "-f",
            "bv*[height<=1080][ext=mp4]+ba[ext=m4a]/bv*[height<=1080]+ba/b[height<=1080]/b",
            "--merge-output-format", "mp4",
            "--newline", "--progress", "--progress-template", TEMPLATE,
            "-o",
        ])
        .arg(dl.join("%(id)s.%(ext)s"))
        .args(["--print", "after_move:filepath", "--no-playlist", source])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stderr = child.stderr.take().context("no stderr")?;
    let err_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stderr.read_to_string(&mut buf);
        buf
    });
    let mut path: Option<String> = None;
    let mut last_emit = Instant::now() - Duration::from_secs(1);
    for line in BufReader::new(child.stdout.take().context("no stdout")?).lines().map_while(|l| l.ok()) {
        if let Some(p) = parse_progress(&line) {
            if last_emit.elapsed() >= Duration::from_millis(250) || p.percent >= 100.0 {
                on_progress(p);
                last_emit = Instant::now();
            }
        } else if !line.trim().is_empty() {
            path = Some(line.trim().to_string());
        }
    }
    let status = child.wait()?;
    let err = err_thread.join().unwrap_or_default();
    if !status.success() {
        bail!("yt-dlp failed: {}", err.chars().take(1500).collect::<String>());
    }
    Ok(PathBuf::from(path.context("yt-dlp printed no path")?))
}

/// 16 kHz mono WAV: what whisper wants and cheap to analyse.
pub fn extract_audio(video: &Path, out_wav: &Path) -> Result<PathBuf> {
    let mtime = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    if let (Some(a), Some(v)) = (mtime(out_wav), mtime(video)) {
        if a >= v {
            return Ok(out_wav.to_path_buf());
        }
    }
    let args: Vec<String> = [
        "-y", "-v", "error", "-i", &video.to_string_lossy(), "-vn", "-ac", "1", "-ar", "16000",
        "-c:a", "pcm_s16le", &out_wav.to_string_lossy(),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    run(&args, None)?;
    Ok(out_wav.to_path_buf())
}
