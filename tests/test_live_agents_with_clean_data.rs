use frontharness::agents::art_director_agent::ArtDirectorAgent;
use frontharness::agents::coder_agent::CoderAgent;
use frontharness::config::settings::LlmConfig;
use frontharness::llm::provider::LlmProvider;
use frontharness::tools::base::ToolRegistry;
use std::sync::Arc;

#[tokio::test]
#[ignore]
async fn test_live_art_director_and_coder() {
    let config = LlmConfig {
        base_url: "https://agentrouter.org/v1".to_string(),
        api_key: "sk-ZTteZtXRIjm2HTA8TOb40W17aNsDraXN5FB5cA0einnpuZ1y".to_string(),
        model: "gpt-5.6-sol".to_string(),
        reasoning_effort: "high".to_string(),
        timeout_seconds: 120,
    };
    let llm = Arc::new(LlmProvider::new(config));
    let tools = ToolRegistry::new();

    let art_director = ArtDirectorAgent::new(llm.clone(), tools.clone(), None);
    let audit_data = r#"{
        "title": "Audi Service Челябинск",
        "headings": ["Официальный сервис Audi", "Наши услуги", "Команда"],
        "bodyText": "Сервисный центр автомобилей Audi и группы VAG в Челябинске."
    }"#;
    let research = "Modern dark industrial automotive service with Bento grid and high contrast.";
    
    println!("[Test] Running ArtDirectorAgent turn...");
    let spec = art_director.create_design_system(research, audit_data, "Redesign website").await;
    assert!(spec.is_ok(), "ArtDirectorAgent failed: {:?}", spec.err());
    let spec_str = spec.unwrap();
    println!("[Test] ArtDirectorAgent spec length: {}", spec_str.len());

    let coder = CoderAgent::new(llm.clone(), tools.clone(), None);
    println!("[Test] Running CoderAgent turn...");
    let code = coder.generate_frontend(&spec_str, audit_data).await;
    assert!(code.is_ok(), "CoderAgent failed: {:?}", code.err());
    let html = code.unwrap();
    println!("[Test] CoderAgent HTML generated length: {}", html.len());
    assert!(html.contains("<!DOCTYPE html>") || html.contains("<html"));
}
