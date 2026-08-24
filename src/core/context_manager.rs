use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            name: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            name: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            name: None,
        }
    }

    pub fn tool(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into(),
            name: Some(name.into()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContextManager {
    messages: Vec<ChatMessage>,
    pub max_token_budget: usize,
}

impl ContextManager {
    pub fn new(max_token_budget: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_token_budget,
        }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    pub fn set_system_prompt(&mut self, prompt: &str) {
        if let Some(first) = self.messages.first_mut() {
            if first.role == "system" {
                first.content = prompt.to_string();
                return;
            }
        }
        self.messages.insert(0, ChatMessage::system(prompt));
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn estimate_tokens(&self) -> usize {
        self.messages.iter().map(|m| m.content.len() / 4 + 4).sum()
    }
}
