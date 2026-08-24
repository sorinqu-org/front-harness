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
