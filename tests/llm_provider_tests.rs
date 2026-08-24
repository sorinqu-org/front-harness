use frontharness::config::settings::LlmConfig;
use frontharness::core::context_manager::ChatMessage;
use frontharness::llm::provider::LlmProvider;

#[tokio::test]
#[ignore]
async fn test_real_llm_stream() {
    let config = LlmConfig {
        base_url: "https://agentrouter.org/v1".to_string(),
        api_key: "sk-ZTteZtXRIjm2HTA8TOb40W17aNsDraXN5FB5cA0einnpuZ1y".to_string(),
        model: "gpt-5.6-sol".to_string(),
        reasoning_effort: "high".to_string(),
        timeout_seconds: 60,
    };

    let provider = LlmProvider::new(config);
    let messages = vec![
        ChatMessage::system("You are a senior frontend engineer."),
        ChatMessage::user("Say 'FrontHarness LLM verified' in 4 words."),
    ];

    let result = provider.stream_chat("TestAgent", &messages, None).await;
    assert!(result.is_ok(), "LLM stream failed: {:?}", result.err());
}
