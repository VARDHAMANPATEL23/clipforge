//! User settings (LLM provider and keys) stored in ~/.clipforge/settings.json.
//! Lookup order for secrets: settings.json, process env, ~/.clipforge/.env.
use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_NICHE: &str = "Game: GTA 6 (Grand Theft Auto VI). Audience: gaming TikTok/Shorts.";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub llm: String, // none | gemini | lmstudio | ollama | openrouter
    pub llm_model: Option<String>,
    pub gemini_api_key: String,
    pub openrouter_api_key: String,
    pub base_url: Option<String>,
    pub niche: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            llm: "none".into(),
            llm_model: None,
            gemini_api_key: String::new(),
            openrouter_api_key: String::new(),
            base_url: None,
            niche: DEFAULT_NICHE.into(),
        }
    }
}

fn file() -> std::path::PathBuf {
    paths::data_dir().join("settings.json")
}

fn dotenv() -> HashMap<String, String> {
    let text = std::fs::read_to_string(paths::data_dir().join(".env")).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().trim_matches(['"', '\'']).to_string()))
        .collect()
}

pub fn env_or_dotenv(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty()).or_else(|| dotenv().remove(key).filter(|v| !v.is_empty()))
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(paths::data_dir())?;
        std::fs::write(file(), serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn gemini_key(&self) -> Option<String> {
        Some(self.gemini_api_key.clone()).filter(|k| !k.is_empty()).or_else(|| env_or_dotenv("GEMINI_API_KEY"))
    }

    pub fn openrouter_key(&self) -> Option<String> {
        Some(self.openrouter_api_key.clone()).filter(|k| !k.is_empty()).or_else(|| env_or_dotenv("OPENROUTER_API_KEY"))
    }
}
