use crate::config::constants::*;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub reasoning_effort: String,
    pub timeout_seconds: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_LLM_BASE_URL.to_string(),
            api_key: String::new(),
            model: DEFAULT_LLM_MODEL.to_string(),
            reasoning_effort: DEFAULT_REASONING_EFFORT.to_string(),
            timeout_seconds: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub provider: String, // "tavily" or "duckduckgo"
    pub tavily_api_key: Option<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            provider: "duckduckgo".to_string(),
            tavily_api_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub dev_server_port: u16,
    pub desktop_width: u32,
    pub desktop_height: u32,
    pub mobile_width: u32,
    pub mobile_height: u32,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            dev_server_port: DEFAULT_DEV_SERVER_PORT,
            desktop_width: 1920,
            desktop_height: 1080,
            mobile_width: 375,
            mobile_height: 812,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignDirectives {
    pub style_prompt: String,
    pub references: Vec<String>,
    pub selected_skills: Vec<String>,
}

impl Default for DesignDirectives {
    fn default() -> Self {
        Self {
            style_prompt: "Modern Dark Industrial: Dark Slate (#09090b), Electric Amber (#f59e0b) accent, Space Grotesk headings, Inter body, GSAP ScrollTrigger, Lucide vector icons".to_string(),
            references: Vec::new(),
            selected_skills: vec![
                "hallmark".into(),
                "taste".into(),
                "stop_slop".into(),
                "motion".into(),
                "icons".into(),
                "modern_web".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub llm: LlmConfig,
    pub search: SearchConfig,
    pub browser: BrowserConfig,
    pub design: DesignDirectives,
    pub workspace_dir: PathBuf,
    pub local_site_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            search: SearchConfig::default(),
            browser: BrowserConfig::default(),
            design: DesignDirectives::default(),
            workspace_dir: PathBuf::from("workspace"),
            local_site_dir: None,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let mut settings = Self::default();
        if let Ok(current) = std::env::current_dir() {
            settings.workspace_dir = current.join("workspace");
        }

        if let Ok(val) = std::env::var("LLM_BASE_URL") {
            settings.llm.base_url = val;
        }
        if let Ok(val) = std::env::var("LLM_API_KEY") {
            settings.llm.api_key = val;
        }
        if let Ok(val) = std::env::var("LLM_MODEL") {
            settings.llm.model = val;
        }
        if let Ok(val) = std::env::var("LLM_REASONING_EFFORT") {
            settings.llm.reasoning_effort = val;
        }

        if let Ok(val) = std::env::var("SEARCH_PROVIDER").or_else(|_| std::env::var("SEARCH_ENGINE")) {
            settings.search.provider = val.to_lowercase();
        }
        if let Ok(val) = std::env::var("TAVILY_API_KEY") {
            settings.search.tavily_api_key = Some(val);
        }

        if let Ok(val) = std::env::var("DEV_SERVER_PORT") {
            if let Ok(port) = val.parse::<u16>() {
                settings.browser.dev_server_port = port;
            }
        }
        if let Ok(val) = std::env::var("BROWSER_HEADLESS") {
            settings.browser.headless = val.to_lowercase() == "true" || val == "1";
        }

        if let Ok(val) = std::env::var("WORKSPACE_DIR") {
            settings.workspace_dir = PathBuf::from(val);
        }
        if let Ok(val) = std::env::var("LOCAL_SITE_DIR") {
            settings.local_site_dir = Some(PathBuf::from(val));
        }
        if let Ok(val) = std::env::var("DESIGN_STYLE") {
            settings.design.style_prompt = val;
        }

        if let Some(config_path) = Self::global_config_path() {
            if config_path.exists() {
                let _ = settings.merge_from_file(&config_path);
            }
        }

        Ok(settings)
    }

    pub fn global_config_path() -> Option<PathBuf> {
        dirs_or_home().map(|h| h.join(DEFAULT_CONFIG_DIR).join("config.yaml"))
    }

    pub fn merge_from_file(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path).with_context(|| format!("Failed to open config file at {:?}", path))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let yaml_val: serde_yaml::Value = serde_yaml::from_str(&content)?;

        if let Some(llm) = yaml_val.get("llm") {
            if let Some(base_url) = llm.get("base_url").and_then(|v| v.as_str()) {
                self.llm.base_url = base_url.to_string();
            }
            if let Some(api_key) = llm.get("api_key").and_then(|v| v.as_str()) {
                self.llm.api_key = api_key.to_string();
            }
            if let Some(model) = llm.get("model").and_then(|v| v.as_str()) {
                self.llm.model = model.to_string();
            }
            if let Some(reasoning) = llm.get("reasoning_effort").and_then(|v| v.as_str()) {
                self.llm.reasoning_effort = reasoning.to_string();
            }
        }

        if let Some(search) = yaml_val.get("search") {
            if let Some(provider) = search.get("provider").and_then(|v| v.as_str()) {
                self.search.provider = provider.to_string();
            }
            if let Some(key) = search.get("tavily_api_key").and_then(|v| v.as_str()) {
                self.search.tavily_api_key = Some(key.to_string());
            }
        }

        if let Some(design) = yaml_val.get("design") {
            if let Some(style) = design.get("style_prompt").and_then(|v| v.as_str()) {
                self.design.style_prompt = style.to_string();
            }
        }

        Ok(())
    }

    pub fn save_global_config(&self) -> Result<()> {
        if let Some(path) = Self::global_config_path() {
            if let Some(parent) = path.parent() {
                create_dir_all(parent)?;
            }
            let yaml_str = serde_yaml::to_string(self)?;
            std::fs::write(&path, yaml_str)?;
        }
        Ok(())
    }
}

pub fn dirs_or_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
