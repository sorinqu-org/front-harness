use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct McpClient {
    server_name: String,
    child: Option<Child>,
}

impl McpClient {
    pub fn new(server_name: &str) -> Self {
        Self {
            server_name: server_name.to_string(),
            child: None,
        }
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub async fn spawn_stdio(&mut self, command: &str, args: &[&str]) -> Result<()> {
        let child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        self.child = Some(child);
        Ok(())
    }

    pub async fn list_tools(&mut self) -> Result<Vec<McpToolInfo>> {
        if let Some(child) = &mut self.child {
            if let Some(stdin) = child.stdin.as_mut() {
                let req = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/list",
                    "params": {}
                });
                let req_str = format!("{}\n", req);
                stdin.write_all(req_str.as_bytes()).await?;
                stdin.flush().await?;

                if let Some(stdout) = child.stdout.as_mut() {
                    let mut reader = BufReader::new(stdout);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await? > 0 {
                        if let Ok(res) = serde_json::from_str::<Value>(&line) {
                            if let Some(tools) = res.pointer("/result/tools").and_then(|t| t.as_array()) {
                                let mut out = Vec::new();
                                for t in tools {
                                    let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let description = t.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let input_schema = t.get("inputSchema").cloned().unwrap_or(json!({}));
                                    out.push(McpToolInfo { name, description, input_schema });
                                }
                                return Ok(out);
                            }
                        }
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        if let Some(child) = &mut self.child {
            if let Some(stdin) = child.stdin.as_mut() {
                let req = json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": name,
                        "arguments": arguments
                    }
                });
                let req_str = format!("{}\n", req);
                stdin.write_all(req_str.as_bytes()).await?;
                stdin.flush().await?;

                if let Some(stdout) = child.stdout.as_mut() {
                    let mut reader = BufReader::new(stdout);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await? > 0 {
                        if let Ok(res) = serde_json::from_str::<Value>(&line) {
                            if let Some(result) = res.get("result") {
                                return Ok(result.clone());
                            }
                        }
                    }
                }
            }
        }
        Ok(json!({ "error": "MCP server unresponsive" }))
    }
}
