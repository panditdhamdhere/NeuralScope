use tokio::sync::broadcast;
use tracing::warn;

use crate::events::domain::EventEnvelope;

const DEFAULT_CAPACITY: usize = 4096;

/// In-process event bus for real-time fan-out to WebSocket clients.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
}

impl EventBus {
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publishes an event to all active subscribers.
    pub fn publish(&self, envelope: EventEnvelope) {
        if self.sender.receiver_count() == 0 {
            return;
        }

        if let Err(error) = self.sender.send(envelope) {
            warn!(%error, "Failed to publish event — no active receivers");
        }
    }

    /// Subscribes to the event stream.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::domain::Event;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn publish_delivers_to_subscriber() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let envelope = EventEnvelope {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            event: Event::LogNew {
                entry_id: Uuid::new_v4(),
                level: "info".into(),
                message: "hello".into(),
            },
            timestamp: Utc::now(),
        };

        bus.publish(envelope.clone());
        let received = rx.recv().await.expect("event");
        assert_eq!(received.id, envelope.id);
    }
}
