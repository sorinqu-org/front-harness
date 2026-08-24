use crate::core::events::Event;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Clone)]
pub struct EventBus {
    sender: Sender<Event>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub fn emit_token(&self, agent: &str, chunk: &str) {
        self.publish(Event::TokenStream {
            agent: agent.to_string(),
            chunk: chunk.to_string(),
        });
    }

    pub fn emit_log(&self, level: &str, source: &str, message: &str) {
        self.publish(Event::LogMessage {
            level: level.to_string(),
            source: source.to_string(),
            message: message.to_string(),
        });
    }

    pub fn emit_error(&self, source: &str, message: &str) {
        self.publish(Event::Error {
            source: source.to_string(),
            message: message.to_string(),
        });
    }

    pub fn emit_phase(&self, from: &str, to: &str, description: &str) {
        self.publish(Event::PhaseChange {
            from: from.to_string(),
            to: to.to_string(),
            description: description.to_string(),
        });
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(2048)
    }
}
