use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    TokenStream {
        agent: String,
        chunk: String,
    },
    ToolCallStart {
        agent: String,
        tool_name: String,
        input: String,
    },
    ToolCallEnd {
        agent: String,
        tool_name: String,
        output: String,
        success: bool,
    },
    PhaseChange {
        from: String,
        to: String,
        description: String,
    },
    ScreenshotCaptured {
        path: String,
        viewport: String,
        label: String,
    },
    AssetDownloaded {
        url: String,
        local_path: String,
        asset_type: String,
    },
    DagStepUpdate {
        step_id: String,
        name: String,
        status: String,
        detail: String,
    },
    LogMessage {
        level: String,
        source: String,
        message: String,
    },
    Error {
        source: String,
        message: String,
    },
    Completed {
        summary: String,
    },
}
