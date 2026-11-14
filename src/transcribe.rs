//! Word-level transcription types and helpers.
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Word {
    pub w: String,
    pub s: f64,
    pub e: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub s: f64,
    pub e: f64,
    pub text: String,
    pub words: Vec<Word>,
}

pub fn words_in(segments: &[Segment], start: f64, end: f64) -> Vec<Word> {
    segments
        .iter()
        .flat_map(|s| s.words.iter())
        .filter(|w| w.s >= start - 0.05 && w.e <= end + 0.05)
        .cloned()
        .collect()
}

pub fn text_in(segments: &[Segment], start: f64, end: f64) -> String {
    words_in(segments, start, end).iter().map(|w| w.w.as_str()).collect::<Vec<_>>().join(" ")
}

fn model_file(name: &str) -> &'static str {
    match name {
        "large" => "ggml-large-v3.bin",
        "small" => "ggml-small.bin",
        _ => "ggml-large-v3-turbo.bin",
    }
}

pub fn model_path(name: &str) -> PathBuf {
    crate::paths::data_dir().join("models").join(model_file(name))
}

/// Download the ggml model once into ~/.clipforge/models.
pub fn ensure_model(name: &str, log: &dyn Fn(&str)) -> Result<PathBuf> {
    let path = model_path(name);
    if path.exists() {
        return Ok(path);
    }
    std::fs::create_dir_all(path.parent().unwrap())?;
    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", model_file(name));
    log(&format!("downloading whisper model {}", model_file(name)));
    let mut resp = reqwest::blocking::Client::builder().timeout(None).build()?.get(&url).send()?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let part = path.with_extension("part");
    let mut file = std::fs::File::create(&part)?;
    let (mut done, mut next_pct, mut buf) = (0u64, 10u64, vec![0u8; 1 << 20]);
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        done += n as u64;
        if total > 0 && done * 100 / total >= next_pct {
            log(&format!("model download {next_pct}%"));
            next_pct += 10;
        }
    }
    drop(file);
    std::fs::rename(&part, &path)?;
    Ok(path)
}

fn read_wav(wav: &Path) -> Result<Vec<f32>> {
    let mut r = hound::WavReader::open(wav)?;
    Ok(r.samples::<i16>().filter_map(|s| s.ok()).map(|s| s as f32 / 32768.0).collect())
}

pub fn transcribe(
    wav: &Path, cache: &Path, model: &str, language: Option<&str>, log: &dyn Fn(&str),
) -> Result<Vec<Segment>> {
    if let Some(segs) = std::fs::read_to_string(cache).ok().and_then(|t| serde_json::from_str(&t).ok()) {
        return Ok(segs);
    }
    let model_path = ensure_model(model, log)?;
    log(&format!("transcribing with whisper.cpp ({}), model {model}", crate::platform::whisper_backend()));
    let ctx = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
        .map_err(|e| anyhow!("whisper init failed: {e}"))?;
    let eot = ctx.token_eot();
    let mut state = ctx.create_state().map_err(|e| anyhow!("whisper state failed: {e}"))?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(language.or(Some("auto")));
    params.set_token_timestamps(true);
    params.set_no_context(true);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    params.set_n_threads(std::thread::available_parallelism().map_or(4, |n| n.get() as i32));
    let audio = read_wav(wav)?;
    state.full(params, &audio).map_err(|e| anyhow!("whisper failed: {e}"))?;

    let mut segments = vec![];
    for i in 0..state.full_n_segments() {
        let Some(seg) = state.get_segment(i) else { continue };
        let (seg_s, seg_e) = (seg.start_timestamp() as f64 / 100.0, seg.end_timestamp() as f64 / 100.0);
        let mut words: Vec<Word> = vec![];
        let mut cur: Option<Word> = None;
        for t in 0..seg.n_tokens() {
            let Some(tok) = seg.get_token(t) else { continue };
            let d = tok.token_data();
            if d.id >= eot {
                continue;
            }
            let text = tok.to_str_lossy().map(|c| c.into_owned()).unwrap_or_default();
            if text.trim().is_empty() {
                continue;
            }
            let ts = if d.t0 >= 0 { (d.t0 as f64 / 100.0).clamp(seg_s, seg_e) } else { seg_s };
            let te = if d.t1 >= 0 { (d.t1 as f64 / 100.0).clamp(ts, seg_e.max(ts)) } else { ts };
            match cur.as_mut() {
                Some(w) if !text.starts_with(' ') => {
                    w.w.push_str(&text);
                    w.e = te.max(w.e);
                }
                _ => {
                    words.extend(cur.take());
                    cur = Some(Word { w: text.trim().to_string(), s: ts, e: te });
                }
            }
        }
        words.extend(cur);
        for w in &mut words {
            w.s = (w.s * 1000.0).round() / 1000.0;
            w.e = (w.e * 1000.0).round() / 1000.0;
        }
        let text = seg.to_str_lossy().map(|c| c.trim().to_string()).unwrap_or_default();
        // drop whisper hallucinations on silence ("Thank you.", "you") with ~no words
        if !words.is_empty() && text.chars().count() > 1 {
            segments.push(Segment { s: seg_s, e: seg_e, text, words });
        }
    }
    std::fs::write(cache, serde_json::to_string(&segments)?)?;
    Ok(segments)
}
