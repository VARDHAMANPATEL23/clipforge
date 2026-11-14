//! Disk usage report and clean-up for ~/.clipforge (never touches rendered clips).
use crate::paths;
use anyhow::{Result, bail};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize, Default)]
pub struct Item {
    pub bytes: u64,
    pub files: u64,
}

#[derive(Serialize)]
pub struct Model {
    pub name: String,
    pub bytes: u64,
}

#[derive(Serialize)]
pub struct Report {
    pub downloads: Item,
    pub analysis: Item,
    pub uploads: Item,
    pub models: Vec<Model>,
    pub total: u64,
}

fn downloads_dir() -> PathBuf {
    paths::cache_dir().join("downloads")
}

fn uploads_dir() -> PathBuf {
    paths::data_dir().join("uploads")
}

fn models_dir() -> PathBuf {
    paths::data_dir().join("models")
}

fn measure(path: &Path) -> Item {
    let Ok(md) = std::fs::symlink_metadata(path) else { return Item::default() };
    if md.is_file() {
        return Item { bytes: md.len(), files: 1 };
    }
    if !md.is_dir() {
        return Item::default();
    }
    std::fs::read_dir(path).into_iter().flatten().flatten().fold(Item::default(), |mut acc, e| {
        let i = measure(&e.path());
        acc.bytes += i.bytes;
        acc.files += i.files;
        acc
    })
}

/// Direct children of the analysis cache: everything in cache/ except downloads/.
fn analysis_entries() -> Vec<PathBuf> {
    std::fs::read_dir(paths::cache_dir())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p != &downloads_dir())
        .collect()
}

fn children(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).collect()
}

fn sum(items: impl Iterator<Item = Item>) -> Item {
    items.fold(Item::default(), |mut a, i| {
        a.bytes += i.bytes;
        a.files += i.files;
        a
    })
}

pub fn report() -> Report {
    let downloads = measure(&downloads_dir());
    let analysis = sum(analysis_entries().iter().map(|p| measure(p)));
    let uploads = measure(&uploads_dir());
    let mut models: Vec<Model> = children(&models_dir())
        .into_iter()
        .filter(|p| p.is_file())
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().into_owned();
            Some(Model { name, bytes: measure(&p).bytes })
        })
        .collect();
    models.sort_by(|a, b| a.name.cmp(&b.name));
    let total = downloads.bytes + analysis.bytes + uploads.bytes + models.iter().map(|m| m.bytes).sum::<u64>();
    Report { downloads, analysis, uploads, models, total }
}

fn remove(path: &Path) -> Result<()> {
    if std::fs::symlink_metadata(path)?.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Clears one category and returns the bytes freed. `kind` is downloads, analysis, uploads or a model file name.
pub fn clear(kind: &str) -> Result<u64> {
    let targets: Vec<PathBuf> = match kind {
        "downloads" => children(&downloads_dir()),
        "analysis" => analysis_entries(),
        "uploads" => children(&uploads_dir()),
        name => {
            if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
                bail!("bad storage kind");
            }
            let path = models_dir().join(name);
            if !path.is_file() {
                bail!("unknown storage kind '{name}'");
            }
            vec![path]
        }
    };
    let freed = sum(targets.iter().map(|p| measure(p))).bytes;
    for t in &targets {
        remove(t)?;
    }
    Ok(freed)
}
