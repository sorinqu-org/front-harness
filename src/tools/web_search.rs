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
    provider: String, // "tavily" or "duckduckgo"
    tavily_api_key: Option<String>,
    client: Client,
}

impl WebSearchTool {
    pub fn new(provider: &str, tavily_api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            provider: provider.to_lowercase(),
            tavily_api_key,
            client,
        }
    }

    pub fn with_default_keyless() -> Self {
        Self::new("duckduckgo", None)
    }

    pub async fn search_tavily(&self, query: &str, api_key: &str) -> Result<Vec<SearchResultItem>> {
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
            return self.search_duckduckgo(query).await;
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

        if results.is_empty() {
            return self.search_duckduckgo(query).await;
        }

        Ok(results)
    }

    pub async fn search_duckduckgo(&self, query: &str) -> Result<Vec<SearchResultItem>> {
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );
        let res = self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        if !res.status().is_success() {
            return Ok(Vec::new());
        }

        let html = res.text().await.unwrap_or_default();
        let mut items = Vec::new();

        // Extract snippets from DuckDuckGo HTML
        for chunk in html.split("<div class=\"result__body\">").skip(1).take(5) {
            let title = extract_tag_content(chunk, "result__snippet").unwrap_or_else(|| format!("Result for {}", query));
            let url = extract_href(chunk).unwrap_or_else(|| "https://duckduckgo.com".to_string());
            let snippet = clean_html(&title);
            
            items.push(SearchResultItem {
                title: format!("Reference: {}", query),
                url,
                content: snippet,
            });
        }

        if items.is_empty() {
            items.push(SearchResultItem {
                title: format!("Search overview for '{}'", query),
                url: "https://duckduckgo.com".to_string(),
                content: clean_html(&html).chars().take(1500).collect(),
            });
        }

        Ok(items)
    }
}

fn extract_tag_content(html: &str, class_name: &str) -> Option<String> {
    let pattern = format!("class=\"{}\"", class_name);
    if let Some(pos) = html.find(&pattern) {
        let rest = &html[pos..];
        if let Some(start) = rest.find('>') {
            let content_start = &rest[start + 1..];
            if let Some(end) = content_start.find('<') {
                return Some(content_start[..end].trim().to_string());
            }
        }
    }
    None
}

fn extract_href(html: &str) -> Option<String> {
    if let Some(pos) = html.find("href=\"") {
        let rest = &html[pos + 6..];
        if let Some(end) = rest.find('\"') {
            let u = &rest[..end];
            if u.starts_with("//duckduckgo.com/l/?uddg=") {
                let encoded = &u["//duckduckgo.com/l/?uddg=".len()..];
                return Some(encoded.split('&').next().unwrap_or(encoded).to_string());
            }
            return Some(u.to_string());
        }
    }
    None
}

fn clean_html(input: &str) -> String {
    let mut out = String::new();
    let mut inside_tag = false;
    for ch in input.chars() {
        if ch == '<' {
            inside_tag = true;
        } else if ch == '>' {
            inside_tag = false;
        } else if !inside_tag {
            out.push(ch);
        }
    }
    out.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
        "Searches the web via Tavily or DuckDuckGo for modern design references, typography, and technical patterns."
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

        let items = if self.provider == "tavily" {
            if let Some(key) = &self.tavily_api_key {
                self.search_tavily(query, key).await?
            } else {
                self.search_duckduckgo(query).await?
            }
        } else {
            self.search_duckduckgo(query).await?
        };

        let json_out = serde_json::to_string_pretty(&items)?;
        Ok(ToolResult::success(json_out))
    }
}
