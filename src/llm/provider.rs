use crate::config::settings::LlmConfig;
use crate::core::context_manager::ChatMessage;
use crate::core::event_bus::EventBus;
use crate::llm::reasoning::ReasoningLevel;
use crate::llm::streaming::{parse_sse_line, StreamChunk};
use anyhow::{bail, Result};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone)]
pub struct LlmProvider {
    config: LlmConfig,
    client: Client,
}

impl LlmProvider {
    pub fn new(config: LlmConfig) -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert("User-Agent", HeaderValue::from_static("claude-cli/1.0.108 (external, cli)"));
        default_headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        default_headers.insert("anthropic-beta", HeaderValue::from_static("claude-code-20250219,oauth-2025-04-20"));
        default_headers.insert("anthropic-dangerous-direct-browser-access", HeaderValue::from_static("true"));
        default_headers.insert("x-app", HeaderValue::from_static("cli"));

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(default_headers)
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    pub async fn stream_chat(
        &self,
        agent_name: &str,
        messages: &[ChatMessage],
        event_bus: Option<&EventBus>,
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        let reasoning = ReasoningLevel::parse(&self.config.reasoning_effort);

        let mut body = json!({
            "model": self.config.model,
            "messages": messages,
            "stream": true,
            "temperature": 0.7
        });

        if let Some(effort) = reasoning.to_openai_param() {
            body["reasoning_effort"] = json!(effort);
        }

        let mut request = self.client.post(&url).json(&body);
        if !self.config.api_key.is_empty() {
            request = request.bearer_auth(&self.config.api_key);
        }

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                // If stream connect fails, attempt non-streaming fallback
                if let Some(bus) = event_bus {
                    bus.emit_log("warn", agent_name, &format!("Streaming request failed ({}), falling back to non-streaming", e));
                }
                return self.chat_complete(messages).await;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            bail!("LLM API returned HTTP {}: {}", status, err_text);
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk = match chunk_res {
                Ok(c) => c,
                Err(e) => {
                    if !full_response.is_empty() {
                        if let Some(bus) = event_bus {
                            bus.emit_log("warn", agent_name, &format!("Stream truncated ({}), using partial response", e));
                        }
                        return Ok(full_response);
                    } else {
                        return self.chat_complete(messages).await;
                    }
                }
            };

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].to_string();
                buffer.drain(..=pos);

                if let Some(stream_chunk) = parse_sse_line(&line) {
                    match stream_chunk {
                        StreamChunk::Content(text) => {
                            full_response.push_str(&text);
                            if let Some(bus) = event_bus {
                                bus.emit_token(agent_name, &text);
                            }
                        }
                        StreamChunk::Reasoning(reasoning_text) => {
                            if let Some(bus) = event_bus {
                                bus.emit_log("debug", agent_name, &format!("Thought: {}", reasoning_text));
                            }
                        }
                        StreamChunk::Done => break,
                    }
                }
            }
        }

        if full_response.trim().is_empty() {
            return self.chat_complete(messages).await;
        }

        Ok(full_response)
    }

    pub async fn chat_complete(&self, messages: &[ChatMessage]) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));
        let reasoning = ReasoningLevel::parse(&self.config.reasoning_effort);

        let mut body = json!({
            "model": self.config.model,
            "messages": messages,
            "stream": false,
            "temperature": 0.7
        });

        if let Some(effort) = reasoning.to_openai_param() {
            body["reasoning_effort"] = json!(effort);
        }

        let mut request = self.client.post(&url).json(&body);
        if !self.config.api_key.is_empty() {
            request = request.bearer_auth(&self.config.api_key);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            bail!("LLM API error ({}): {}", status, err_text);
        }

        let val: Value = response.json().await?;
        if let Some(text) = val.pointer("/choices/0/message/content").and_then(|c| c.as_str()) {
            Ok(text.to_string())
        } else {
            bail!("Unexpected response structure from LLM API");
        }
    }
}
