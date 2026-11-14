//! Non-speech signals: loudness spikes and on-screen action, reduced to per-second arrays.
use crate::media::{Encoder, require};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Signals {
    pub loud: Vec<f64>,
    pub spike: Vec<f64>,
    pub activity: Vec<f64>,
    pub cuts: Vec<f64>,
}

/// RMS dBFS per second (max over 250 ms sub-windows, so short bangs count).
pub fn loudness_per_second(wav: &Path) -> Result<Vec<f64>> {
    let mut reader = hound::WavReader::open(wav)?;
    let hop = (reader.spec().sample_rate / 4) as usize;
    let pcm: Vec<f32> = reader
        .samples::<i16>()
        .filter_map(|s| s.ok())
        .map(|s| s as f32 / 32768.0)
        .collect();
    let n = pcm.len() / hop.max(1);
    if n == 0 {
        return Ok(vec![0.0]);
    }
    let db: Vec<f64> = pcm
        .chunks_exact(hop)
        .take(n)
        .map(|f| {
            let ms = f.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / hop as f64;
            20.0 * (ms + 1e-10).sqrt().log10()
        })
        .collect();
    let secs = db.len() / 4;
    if secs == 0 {
        return Ok(vec![db.iter().cloned().fold(f64::MIN, f64::max)]);
    }
    Ok(db
        .chunks_exact(4)
        .take(secs)
        .map(|c| c.iter().cloned().fold(f64::MIN, f64::max))
        .collect())
}

pub fn rolling_median(x: &[f64], window: usize) -> Vec<f64> {
    let half = window / 2;
    let len = x.len();
    (0..len)
        .map(|i| {
            let mut w: Vec<f64> = (i as isize - half as isize..=(i + half) as isize)
                .map(|j| x[j.clamp(0, len as isize - 1) as usize])
                .collect();
            w.sort_by(|a, b| a.partial_cmp(b).unwrap());
            w[w.len() / 2]
        })
        .collect()
}

/// Per-second mean scene-change score (about on-screen action) and hard-cut timestamps.
pub fn visual_activity(video: &Path, duration: f64, enc: &Encoder) -> Result<(Vec<f64>, Vec<f64>)> {
    let out = Command::new(require("ffmpeg")?)
        .args(["-v", "error"])
        .args(&enc.decode_args)
        .arg("-i")
        .arg(video)
        .args([
            "-an", "-vf",
            "fps=4,scale=256:-2,select='gte(scene,0)',metadata=print:key=lavfi.scene_score:file=-",
            "-f", "null", "-",
        ])
        .output()?;
    let secs = (duration.ceil() as usize).max(1);
    let (mut sums, mut counts) = (vec![0.0f64; secs], vec![0.0f64; secs]);
    let mut cuts = vec![];
    let mut t = 0.0;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(v) = value_after(line, "pts_time:") {
            t = v;
        } else if let Some(score) = value_after(line, "scene_score=") {
            let i = (t as usize).min(secs - 1);
            sums[i] += score;
            counts[i] += 1.0;
            if score > 0.35 {
                cuts.push((t * 100.0).round() / 100.0);
            }
        }
    }
    let act = sums.iter().zip(&counts).map(|(s, c)| s / c.max(1.0)).collect();
    Ok((act, cuts))
}

fn value_after(line: &str, key: &str) -> Option<f64> {
    let rest = &line[line.find(key)? + key.len()..];
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

pub fn zscore(x: &[f64]) -> Vec<f64> {
    let n = x.len().max(1) as f64;
    let mean = x.iter().sum::<f64>() / n;
    let sd = (x.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
    if sd > 1e-6 { x.iter().map(|v| (v - mean) / sd).collect() } else { vec![0.0; x.len()] }
}

fn fit(mut v: Vec<f64>, len: usize, fill: f64) -> Vec<f64> {
    v.resize(len, fill);
    v
}

pub fn compute(video: &Path, wav: Option<&Path>, duration: f64, enc: &Encoder, cache: &Path) -> Result<Signals> {
    if let Ok(text) = std::fs::read_to_string(cache) {
        if let Ok(s) = serde_json::from_str(&text) {
            return Ok(s);
        }
    }
    let secs = (duration.ceil() as usize).max(1);
    let loud = match wav {
        Some(w) => fit(loudness_per_second(w)?, secs, -60.0),
        None => vec![-60.0; secs],
    };
    let med = rolling_median(&loud, 61);
    let spike = loud.iter().zip(&med).map(|(l, m)| (l - m).max(0.0)).collect();
    let (act, cuts) = visual_activity(video, duration, enc)?;
    let out = Signals { loud, spike, activity: fit(act, secs, 0.0), cuts };
    std::fs::write(cache, serde_json::to_string(&out)?)?;
    Ok(out)
}
