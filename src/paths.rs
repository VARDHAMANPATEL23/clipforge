use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".clipforge")
}

pub fn cache_dir() -> PathBuf {
    data_dir().join("cache")
}
