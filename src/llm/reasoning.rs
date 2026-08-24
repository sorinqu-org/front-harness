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
