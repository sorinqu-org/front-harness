use frontharness::memory::store::{MemoryStore, ProjectSummary};

#[test]
fn test_memory_store_in_memory() {
    let store = MemoryStore::open_in_memory().unwrap();
    let summary = ProjectSummary {
        id: "proj-1".to_string(),
        title: "Test Redesign".to_string(),
        target_url: Some("https://example.com".to_string()),
        macrostructure: "Bento Grid".to_string(),
        color_palette: "Zinc + Electric Amber".to_string(),
        typography: "Space Grotesk".to_string(),
        user_rating: Some(5),
        lessons_learned: "High conversion layout".to_string(),
        created_at: "2026-08-24T12:00:00Z".to_string(),
    };

    store.save_summary(&summary).unwrap();
    let list = store.list_summaries().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].title, "Test Redesign");
}
