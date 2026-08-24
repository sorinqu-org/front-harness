use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub owned_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelsResponse {
    pub data: Vec<ModelInfo>,
}

pub async fn discover_models(base_url: &str, api_key: &str) -> Result<Vec<ModelInfo>> {
    let client = Client::new();
    let url = format!("{}/models", base_url.trim_end_matches('/'));

    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    let res = req.send().await?;
    if !res.status().is_success() {
        return Ok(Vec::new());
    }

    let body: ModelsResponse = res.json().await?;
    Ok(body.data)
}

pub fn save_models_cache(path: &Path, models: &[ModelInfo]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json_str = serde_json::to_string_pretty(models)?;
    std::fs::write(path, json_str)?;
    Ok(())
}

pub fn load_models_cache(path: &Path) -> Option<Vec<ModelInfo>> {
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(models) = serde_json::from_str::<Vec<ModelInfo>>(&content) {
                return Some(models);
            }
        }
    }
    None
}
