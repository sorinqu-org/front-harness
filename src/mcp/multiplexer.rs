use crate::mcp::client::{McpClient, McpToolInfo};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct McpMultiplexer {
    clients: HashMap<String, McpClient>,
}

impl McpMultiplexer {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn register_client(&mut self, client: McpClient) {
        self.clients.insert(client.server_name().to_string(), client);
    }

    pub async fn discover_all_tools(&mut self) -> Result<Vec<(String, McpToolInfo)>> {
        let mut list = Vec::new();
        for (server_name, client) in self.clients.iter_mut() {
            if let Ok(tools) = client.list_tools().await {
                for tool in tools {
                    list.push((server_name.clone(), tool));
                }
            }
        }
        Ok(list)
    }

    pub async fn execute_tool(
        &mut self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value> {
        if let Some(client) = self.clients.get_mut(server_name) {
            client.call_tool(tool_name, arguments).await
        } else {
            anyhow::bail!("MCP server '{}' not found", server_name);
        }
    }
}
