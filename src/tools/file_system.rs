use crate::tools::base::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};

pub struct FileSystemTool {
    base_dir: PathBuf,
}

impl FileSystemTool {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn resolve_path(&self, relative: &str) -> PathBuf {
        let p = Path::new(relative);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.base_dir.join(relative)
        }
    }
}

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        "file_system"
    }

    fn description(&self) -> &str {
        "Reads, writes, and inspects files in the workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["read", "write", "list", "exists"],
                    "description": "File system action"
                },
                "path": {
                    "type": "string",
                    "description": "Relative or absolute file path"
                },
                "content": {
                    "type": "string",
                    "description": "Content for write action"
                }
            },
            "required": ["action", "path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let action = match args.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return Ok(ToolResult::failure("Missing 'action'")),
        };
        let target_path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => self.resolve_path(p),
            None => return Ok(ToolResult::failure("Missing 'path'")),
        };

        match action {
            "read" => {
                if !target_path.exists() {
                    return Ok(ToolResult::failure(format!("File {:?} does not exist", target_path)));
                }
                match read_to_string(&target_path) {
                    Ok(content) => Ok(ToolResult::success(content)),
                    Err(e) => Ok(ToolResult::failure(format!("Read error: {}", e))),
                }
            }
            "write" => {
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                if let Some(parent) = target_path.parent() {
                    create_dir_all(parent)?;
                }
                match write(&target_path, content) {
                    Ok(_) => Ok(ToolResult::success(format!("Wrote {} bytes to {:?}", content.len(), target_path))),
                    Err(e) => Ok(ToolResult::failure(format!("Write error: {}", e))),
                }
            }
            "exists" => {
                let exists = target_path.exists();
                Ok(ToolResult::success(json!({ "exists": exists }).to_string()))
            }
            "list" => {
                let mut entries = Vec::new();
                if target_path.is_dir() {
                    for entry in std::fs::read_dir(&target_path)? {
                        let entry = entry?;
                        entries.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
                Ok(ToolResult::success(json!(entries).to_string()))
            }
            _ => Ok(ToolResult::failure(format!("Unknown action: {}", action))),
        }
    }
}
