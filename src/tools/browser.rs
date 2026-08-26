use crate::tools::base::{Tool, ToolResult};
use crate::tools::embedded_scripts::get_or_extract_crawler_script;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditReport {
    pub target_url: String,
    pub timestamp: String,
    pub screenshots: Value,
    pub site_analysis: Value,
    pub intercepted_assets_count: usize,
    pub assets: Vec<Value>,
}

pub struct BrowserTool {
    crawler_script: PathBuf,
    output_dir: PathBuf,
}

impl BrowserTool {
    pub fn new(output_dir: PathBuf) -> Self {
        let crawler_script = get_or_extract_crawler_script();
        Self {
            crawler_script,
            output_dir,
        }
    }

    pub fn with_script_path(crawler_script: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            crawler_script,
            output_dir,
        }
    }

    pub async fn run_audit(&self, target_url: &str) -> Result<AuditReport> {
        let script_path = if self.crawler_script.exists() {
            self.crawler_script.clone()
        } else {
            get_or_extract_crawler_script()
        };

        let output = Command::new("python3")
            .arg(&script_path)
            .arg(target_url)
            .arg(&self.output_dir)
            .output()
            .await?;

        let report_file = self.output_dir.join("audit_report.json");
        if report_file.exists() {
            let content = std::fs::read_to_string(&report_file)?;
            let report: AuditReport = serde_json::from_str(&content)?;
            Ok(report)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("Playwright crawler failed: stderr='{}', stdout='{}'", stderr.trim(), stdout.trim());
        }
    }
}

#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser_audit"
    }

    fn description(&self) -> &str {
        "Inspects a target website using Playwright, extracts computed CSS, DOM structure, screenshots, and assets."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The target website URL"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return Ok(ToolResult::failure("Missing 'url' argument")),
        };

        match self.run_audit(url).await {
            Ok(report) => {
                let json_str = serde_json::to_string_pretty(&report)?;
                Ok(ToolResult::success(json_str))
            }
            Err(e) => Ok(ToolResult::failure(format!("Browser audit error: {}", e))),
        }
    }
}
