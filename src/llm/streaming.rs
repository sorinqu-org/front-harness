use serde_json::Value;

#[derive(Debug, Clone)]
pub enum StreamChunk {
    Content(String),
    Reasoning(String),
    Done,
}

pub fn parse_sse_line(line: &str) -> Option<StreamChunk> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return None;
    }

    let payload = if let Some(stripped) = trimmed.strip_prefix("data:") {
        stripped.trim()
    } else {
        trimmed
    };

    if payload == "[DONE]" {
        return Some(StreamChunk::Done);
    }

    if let Ok(json) = serde_json::from_str::<Value>(payload) {
        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
            if let Some(first) = choices.first() {
                if let Some(delta) = first.get("delta") {
                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                        if !content.is_empty() {
                            return Some(StreamChunk::Content(content.to_string()));
                        }
                    }
                    if let Some(reasoning) = delta.get("reasoning_content").and_then(|c| c.as_str()) {
                        if !reasoning.is_empty() {
                            return Some(StreamChunk::Reasoning(reasoning.to_string()));
                        }
                    }
                    if let Some(thought) = delta.get("thought").and_then(|c| c.as_str()) {
                        if !thought.is_empty() {
                            return Some(StreamChunk::Reasoning(thought.to_string()));
                        }
                    }
                }
            }
        }
    }

    None
}
