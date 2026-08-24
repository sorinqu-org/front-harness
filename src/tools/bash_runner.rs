use crate::tools::base::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::process::Command;

pub struct BashRunnerTool {
    working_dir: PathBuf,
}

impl BashRunnerTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl Tool for BashRunnerTool {
    fn name(&self) -> &str {
        "bash_runner"
    }

    fn description(&self) -> &str {
        "Executes a bash command in the project environment and returns stdout and stderr."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let cmd_str = match args.get("command").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return Ok(ToolResult::failure("Missing 'command' argument")),
        };

        let output = Command::new("bash")
            .arg("-c")
            .arg(cmd_str)
            .current_dir(&self.working_dir)
            .output()
            .await;

        match output {
            Ok(res) => {
                let stdout = String::from_utf8_lossy(&res.stdout).to_string();
                let stderr = String::from_utf8_lossy(&res.stderr).to_string();
                let combined = if stderr.is_empty() {
                    stdout
                } else if stdout.is_empty() {
                    stderr.clone()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };

                if res.status.success() {
                    Ok(ToolResult::success(combined))
                } else {
                    Ok(ToolResult::failure(format!(
                        "Exited with code {}: {}",
                        res.status.code().unwrap_or(-1),
                        combined
                    )))
                }
            }
            Err(e) => Ok(ToolResult::failure(format!("Execution failed: {}", e))),
        }
    }
}
