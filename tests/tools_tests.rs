use frontharness::tools::base::Tool;
use frontharness::tools::file_system::FileSystemTool;
use serde_json::json;
use std::env::temp_dir;

#[tokio::test]
async fn test_file_system_tool() {
    let dir = temp_dir().join("fh_test_fs");
    let tool = FileSystemTool::new(dir.clone());

    let write_res = tool
        .execute(json!({
            "action": "write",
            "path": "test.txt",
            "content": "FrontHarness Test Content"
        }))
        .await
        .unwrap();
    assert!(write_res.success);

    let read_res = tool
        .execute(json!({
            "action": "read",
            "path": "test.txt"
        }))
        .await
        .unwrap();
    assert!(read_res.success);
    assert_eq!(read_res.output, "FrontHarness Test Content");

    let _ = std::fs::remove_dir_all(&dir);
}

use frontharness::tools::web_search::WebSearchTool;
use frontharness::llm::reasoning::{get_available_efforts_for_model, cycle_next_effort};

#[tokio::test]
async fn test_duckduckgo_search_tool() {
    let tool = WebSearchTool::new("duckduckgo", None);
    let res = tool
        .execute(json!({
            "query": "audi service chelyabinsk"
        }))
        .await
        .unwrap();
    assert!(res.success);
    assert!(!res.output.is_empty());
}

#[test]
fn test_reasoning_effort_parsing_and_cycling() {
    let o1_efforts = get_available_efforts_for_model("o3-mini");
    assert!(o1_efforts.contains(&"high"));
    assert!(o1_efforts.contains(&"low"));

    let claude_efforts = get_available_efforts_for_model("claude-3-7-sonnet");
    assert!(claude_efforts.contains(&"2048"));
    assert!(claude_efforts.contains(&"8192"));

    let next = cycle_next_effort("low", "gpt-5.6-sol");
    assert_eq!(next, "medium");
}
