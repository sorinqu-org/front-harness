use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningLevel {
    Low,
    Medium,
    High,
    Custom(u32),
    Disabled,
}

impl ReasoningLevel {
    pub fn parse(val: &str) -> Self {
        match val.to_lowercase().as_str() {
            "low" => Self::Low,
            "medium" | "med" => Self::Medium,
            "high" => Self::High,
            "off" | "none" | "disabled" => Self::Disabled,
            num => {
                if let Ok(n) = num.parse::<u32>() {
                    Self::Custom(n)
                } else {
                    Self::High
                }
            }
        }
    }

    pub fn to_openai_param(&self) -> Option<String> {
        match self {
            Self::Low => Some("low".to_string()),
            Self::Medium => Some("medium".to_string()),
            Self::High => Some("high".to_string()),
            Self::Custom(_) => Some("high".to_string()),
            Self::Disabled => None,
        }
    }

    pub fn to_anthropic_budget(&self) -> Option<u32> {
        match self {
            Self::Low => Some(2048),
            Self::Medium => Some(8192),
            Self::High => Some(16384),
            Self::Custom(n) => Some(*n),
            Self::Disabled => None,
        }
    }
}

pub fn get_available_efforts_for_model(model_name: &str) -> Vec<&'static str> {
    let lower = model_name.to_lowercase();
    if lower.contains("claude-3-7") || lower.contains("claude-3.7") || lower.contains("claude-4") {
        vec!["none", "low", "medium", "high", "1024", "2048", "4096", "8192", "16384"]
    } else if lower.contains("o1") || lower.contains("o3") || lower.contains("gpt-5") || lower.contains("reasoning") {
        vec!["none", "low", "medium", "high", "custom"]
    } else if lower.contains("r1") || lower.contains("deepseek-r1") || lower.contains("qwq") {
        vec!["none", "low", "medium", "high"]
    } else {
        vec!["none", "low", "medium", "high"]
    }
}

pub fn cycle_next_effort(current: &str, model_name: &str) -> String {
    let options = get_available_efforts_for_model(model_name);
    let current_clean = current.trim().to_lowercase();
    
    if let Some(pos) = options.iter().position(|&opt| opt == current_clean) {
        let next_idx = (pos + 1) % options.len();
        options[next_idx].to_string()
    } else {
        options[0].to_string()
    }
}
