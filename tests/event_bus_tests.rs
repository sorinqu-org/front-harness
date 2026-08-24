use frontharness::core::events::Event;
use frontharness::core::EventBus;

#[tokio::test]
async fn test_event_bus_pub_sub() {
    let bus = EventBus::new(16);
    let mut rx = bus.subscribe();

    bus.emit_token("CoderAgent", "<div>Test</div>");

    if let Ok(event) = rx.recv().await {
        match event {
            Event::TokenStream { agent, chunk } => {
                assert_eq!(agent, "CoderAgent");
                assert_eq!(chunk, "<div>Test</div>");
            }
            _ => panic!("Expected TokenStream event"),
        }
    } else {
        panic!("Failed to receive event");
    }
}
