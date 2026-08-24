use crate::tools::base::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub content: String,
}

pub struct WebSearchTool {
    tavily_api_key: Option<String>,
    client: Client,
}

impl WebSearchTool {
    pub fn new(tavily_api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            tavily_api_key,
            client,
        }
    }

    async fn search_tavily(&self, query: &str, api_key: &str) -> Result<Vec<SearchResultItem>> {
        let res = self
            .client
            .post("https://api.tavily.com/search")
            .json(&json!({
                "api_key": api_key,
                "query": query,
                "search_depth": "advanced",
                "max_results": 5
            }))
            .send()
            .await?;

        if !res.status().is_success() {
            return Ok(Vec::new());
        }

        let val: Value = res.json().await?;
        let mut results = Vec::new();

        if let Some(arr) = val.get("results").and_then(|r| r.as_array()) {
            for item in arr {
                let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                results.push(SearchResultItem { title, url, content });
            }
        }

        Ok(results)
    }

    async fn search_fallback(&self, query: &str) -> Result<Vec<SearchResultItem>> {
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );
        let res = self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 FrontHarness/1.0")
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            Ok(vec![SearchResultItem {
                title: format!("Search results for '{}'", query),
                url: "https://duckduckgo.com".to_string(),
                content: body.chars().take(1500).collect(),
            }])
        } else {
            Ok(Vec::new())
        }
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.replace(' ', "+")
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Searches the web for modern design references, typography, and technical patterns."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return Ok(ToolResult::failure("Missing 'query' parameter")),
        };

        let items = if let Some(key) = &self.tavily_api_key {
            match self.search_tavily(query, key).await {
                Ok(res) if !res.is_empty() => res,
                _ => self.search_fallback(query).await.unwrap_or_default(),
            }
        } else {
            self.search_fallback(query).await.unwrap_or_default()
        };

        let json_out = serde_json::to_string_pretty(&items)?;
        Ok(ToolResult::success(json_out))
    }
}
