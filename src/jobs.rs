//! Job store: queue (one at a time), live event broadcast for SSE, JSON persistence.
use crate::ingest::DlProgress;
use crate::paths;
use crate::pipeline::{self, ClipMeta, Event, Options};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{Semaphore, broadcast};

const MAX_LOG_LINES: usize = 2000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub source: String,
    pub options: Options,
    pub status: String, // queued | running | done | failed
    pub stage: String,
    #[serde(default)]
    pub progress: Option<DlProgress>,
    pub log: Vec<String>,
    pub clips: Vec<ClipMeta>,
    pub out_dir: Option<PathBuf>,
    pub error: Option<String>,
    pub created_at: u64,
}

#[derive(Default)]
struct State {
    jobs: HashMap<String, Job>,
    tx: HashMap<String, broadcast::Sender<String>>,
}

pub struct Jobs {
    state: Mutex<State>,
    gate: Semaphore,
}

fn jobs_dir() -> PathBuf {
    paths::data_dir().join("jobs")
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

impl Jobs {
    pub fn load() -> Arc<Self> {
        let mut state = State::default();
        for e in std::fs::read_dir(jobs_dir()).into_iter().flatten().flatten() {
            let Some(mut job) = std::fs::read_to_string(e.path()).ok().and_then(|t| serde_json::from_str::<Job>(&t).ok())
            else {
                continue;
            };
            if job.status == "queued" || job.status == "running" {
                job.status = "failed".into();
                job.error = Some("interrupted by restart".into());
            }
            state.jobs.insert(job.id.clone(), job);
        }
        Arc::new(Self { state: Mutex::new(state), gate: Semaphore::new(1) })
    }

    pub fn list(&self) -> Vec<Job> {
        let mut v: Vec<Job> = self.state.lock().unwrap().jobs.values().cloned().collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v
    }

    pub fn busy(&self) -> bool {
        self.state.lock().unwrap().jobs.values().any(|j| j.status == "queued" || j.status == "running")
    }

    pub fn get(&self, id: &str) -> Option<Job> {
        self.state.lock().unwrap().jobs.get(id).cloned()
    }

    /// Snapshot plus a receiver for later events, taken under one lock so none are missed.
    pub fn subscribe(&self, id: &str) -> Option<(Job, broadcast::Receiver<String>)> {
        let mut st = self.state.lock().unwrap();
        let job = st.jobs.get(id)?.clone();
        let rx = st.tx.entry(id.to_string()).or_insert_with(|| broadcast::channel(512).0).subscribe();
        Some((job, rx))
    }

    fn update(&self, id: &str, f: impl FnOnce(&mut Job), ev: Value) {
        let mut st = self.state.lock().unwrap();
        let Some(job) = st.jobs.get_mut(id) else { return };
        f(job);
        let snapshot = job.clone();
        if let Some(tx) = st.tx.get(id) {
            let _ = tx.send(ev.to_string());
        }
        drop(st);
        if snapshot.status == "done" || snapshot.status == "failed" {
            let _ = std::fs::create_dir_all(jobs_dir());
            let _ = serde_json::to_string_pretty(&snapshot)
                .map(|t| std::fs::write(jobs_dir().join(format!("{id}.json")), t));
        }
    }

    pub fn submit(self: &Arc<Self>, source: String, options: Options) -> Job {
        let id = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
        let job = Job {
            id: id.clone(), source: source.clone(), options: options.clone(), status: "queued".into(),
            stage: "queued".into(), progress: None, log: vec![], clips: vec![], out_dir: None, error: None, created_at: now(),
        };
        self.state.lock().unwrap().jobs.insert(id.clone(), job.clone());
        let this = Arc::clone(self);
        tokio::spawn(async move {
            let Ok(_permit) = this.gate.acquire().await else { return };
            this.update(&id, |j| j.status = "running".into(), json!({"type": "status", "status": "running"}));
            let runner = Arc::clone(&this);
            let rid = id.clone();
            let result = tokio::task::spawn_blocking(move || {
                let emit = |ev: Event| match ev {
                    Event::Stage(s) => runner.update(
                        &rid,
                        |j| {
                            j.stage = s.to_string();
                            j.progress = None;
                        },
                        json!({"type": "stage", "stage": s}),
                    ),
                    Event::Progress(p) => runner.update(
                        &rid,
                        |j| j.progress = Some(p.clone()),
                        json!({"type": "progress", "progress": p}),
                    ),
                    Event::Log(line) => runner.update(
                        &rid,
                        |j| {
                            j.log.push(line.clone());
                            if j.log.len() > MAX_LOG_LINES {
                                j.log.remove(0);
                            }
                        },
                        json!({"type": "log", "line": line}),
                    ),
                };
                pipeline::process(&source, &options, &emit)
            })
            .await;
            match result {
                Ok(Ok(out)) => this.update(
                    &id,
                    |j| {
                        j.status = "done".into();
                        j.stage = "done".into();
                        j.out_dir = Some(out.out_dir);
                        j.clips = out.clips;
                    },
                    json!({"type": "status", "status": "done"}),
                ),
                other => {
                    let msg = match other {
                        Ok(Err(e)) => format!("{e:#}"),
                        Err(e) => format!("job crashed: {e}"),
                        Ok(Ok(_)) => unreachable!(),
                    };
                    this.update(
                        &id,
                        |j| {
                            j.status = "failed".into();
                            j.error = Some(msg.clone());
                        },
                        json!({"type": "status", "status": "failed", "error": msg}),
                    )
                }
            }
        });
        job
    }
}
