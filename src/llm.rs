//! Tiny OpenAI-compatible chat client (Gemini, LM Studio, Ollama, OpenRouter).
use crate::settings::{Settings, env_or_dotenv};
use anyhow::{Context, Result, bail};
use regex::Regex;
use serde_json::{Value, json};
use std::time::Duration;

pub struct Llm {
    base_url: String,
    api_key: String,
    pub model: String,
}

impl Llm {
    pub fn new(provider: &str, model: Option<&str>, vision: bool, s: &Settings) -> Result<Self> {
        // (base_url, key, default model, vision model)
        let (base, key, def, vis): (&str, Option<String>, &str, Option<&str>) = match provider {
            "gemini" => (
                "https://generativelanguage.googleapis.com/v1beta/openai",
                Some(s.gemini_key().context("set the Gemini API key in Settings")?),
                "gemini-flash-lite-latest",
                Some("gemini-flash-latest"),
            ),
            "lmstudio" => ("http://localhost:1234/v1", None, "local-model", None),
            "ollama" => ("http://localhost:11434/v1", None, "qwen3:14b", None),
            "openrouter" => (
                "https://openrouter.ai/api/v1",
                Some(s.openrouter_key().context("set the OpenRouter API key in Settings")?),
                "google/gemini-3.8-flash",
                None,
            ),
            other => bail!("unknown LLM provider '{other}'"),
        };
        let base_url = s.base_url.clone().filter(|u| !u.is_empty())
            .or_else(|| env_or_dotenv("CLIPFORGE_LLM_BASE_URL"))
            .unwrap_or_else(|| base.to_string());
        let api_key = env_or_dotenv("CLIPFORGE_LLM_API_KEY").or(key).unwrap_or_else(|| "none".into());
        let default_model = if vision { vis.unwrap_or(def) } else { def };
        let model = model.map(str::to_string)
            .or_else(|| s.llm_model.clone().filter(|m| !m.is_empty()))
            .or_else(|| env_or_dotenv("CLIPFORGE_LLM_MODEL"))
            .unwrap_or_else(|| default_model.to_string());
        Ok(Self { base_url: base_url.trim_end_matches('/').to_string(), api_key, model })
    }

    pub fn chat(&self, messages: Value) -> Result<String> {
        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(180)).build()?;
        let resp = client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&json!({ "model": self.model, "messages": messages, "temperature": 0.4 }))
            .send()?;
        let status = resp.status();
        let body = resp.text()?;
        if !status.is_success() {
            bail!("LLM HTTP {}: {}", status.as_u16(), body.chars().take(600).collect::<String>());
        }
        let data: Value = serde_json::from_str(&body)?;
        Ok(data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
    }
}

/// Pull the first JSON object/array out of a model reply (handles ```json fences, <think> blocks).
pub fn parse_json(text: &str) -> Result<Value> {
    let think = Regex::new(r"(?s)<think>.*?</think>").unwrap();
    let text = think.replace_all(text, "");
    let text = text.replace("```json", "").replace("```", "");
    let start = [text.find('['), text.find('{')].into_iter().flatten().min().context("no JSON in reply")?;
    let closer = if text[start..].starts_with('[') { ']' } else { '}' };
    let end = text.rfind(closer).context("unterminated JSON in reply")?;
    Ok(serde_json::from_str(&text[start..=end])?)
}
