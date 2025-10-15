/// Event Bus - Pub/Sub system for event-driven architecture
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, warn};

use super::types::{Event, EventType};
use crate::error::{Result, TradingError};

pub type EventHandler = Arc<dyn Fn(Event) -> futures_util::future::BoxFuture<'static, Result<()>> + Send + Sync>;

/// Event bus for publish-subscribe pattern
pub struct EventBus {
    /// Subscribers for each event type
    subscribers: Arc<RwLock<HashMap<EventType, Vec<EventHandler>>>>,
    
    /// Channel for publishing events
    tx: mpsc::UnboundedSender<Event>,
    rx: Arc<RwLock<mpsc::UnboundedReceiver<Event>>>,
    
    /// Idempotency tracker (prevents duplicate event processing)
    processed_events: Arc<RwLock<HashSet<String>>>,
    
    /// Event log file path
    event_log_path: String,
}

impl EventBus {
    pub fn new(event_log_path: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        EventBus {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            tx,
            rx: Arc::new(RwLock::new(rx)),
            processed_events: Arc::new(RwLock::new(HashSet::new())),
            event_log_path,
        }
    }
    
    /// Subscribe to an event type
    pub async fn subscribe(
        &self,
        event_type: EventType,
        handler: EventHandler,
    ) {
        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
        debug!("Subscribed handler to event: {:?}", event_type);
    }
    
    /// Publish an event to all subscribers
    pub async fn publish(&self, event: Event) -> Result<()> {
        // Check idempotency
        {
            let mut processed = self.processed_events.write().await;
            if processed.contains(&event.idempotency_key) {
                warn!(
                    "Duplicate event detected: {} ({})",
                    event.event_type.as_str(),
                    event.idempotency_key
                );
                return Err(TradingError::DuplicateEvent(
                    event.idempotency_key.clone()
                ));
            }
            processed.insert(event.idempotency_key.clone());
        }
        
        // Log event to file
        self.log_event(&event).await?;
        
        // Send to event processing queue
        self.tx.send(event).map_err(|e| {
            TradingError::EventDispatchFailed(format!("Failed to send event: {}", e))
        })?;
        
        Ok(())
    }
    
    /// Start processing events from the queue
    pub async fn start_processing(&self) {
        let subscribers = Arc::clone(&self.subscribers);
        let rx = Arc::clone(&self.rx);
        
        tokio::spawn(async move {
            let mut rx = rx.write().await;
            
            while let Some(event) = rx.recv().await {
                let event_type = event.event_type.clone();
                
                debug!(
                    "Processing event: {} at {}",
                    event_type.as_str(),
                    event.timestamp
                );
                
                // Get all handlers for this event type
                let handlers = {
                    let subs = subscribers.read().await;
                    subs.get(&event_type).cloned()
                };
                
                if let Some(handlers) = handlers {
                    // Execute all handlers
                    for handler in handlers {
                        let event_clone = event.clone();
                        match handler(event_clone).await {
                            Ok(_) => {
                                debug!("Handler executed successfully for: {:?}", event_type);
                            }
                            Err(e) => {
                                error!(
                                    "Handler failed for event {:?}: {} ({})",
                                    event_type,
                                    e,
                                    e.error_code()
                                );
                            }
                        }
                    }
                } else {
                    debug!("No handlers registered for event: {:?}", event_type);
                }
            }
        });
    }
    
    /// Log event to JSON file (append-only)
    async fn log_event(&self, event: &Event) -> Result<()> {
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncWriteExt;
        
        let json_line = serde_json::to_string(event)
            .map_err(|e| TradingError::InternalError(format!("Event serialization failed: {}", e)))?;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.event_log_path)
            .await?;
        
        file.write_all(format!("{}\n", json_line).as_bytes()).await?;
        file.sync_all().await?;
        
        Ok(())
    }
    
    /// Replay events from log (for recovery)
    pub async fn replay_events(&self, from_timestamp: chrono::DateTime<chrono::Utc>) -> Result<Vec<Event>> {
        use tokio::fs::File;
        use tokio::io::{AsyncBufReadExt, BufReader};
        
        let file = File::open(&self.event_log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let mut replayed_events = Vec::new();
        
        while let Some(line) = lines.next_line().await? {
            if let Ok(event) = serde_json::from_str::<Event>(&line) {
                if event.timestamp >= from_timestamp {
                    replayed_events.push(event);
                }
            }
        }
        
        Ok(replayed_events)
    }
    
    /// Clear processed events (for testing or daily reset)
    pub async fn clear_processed_events(&self) {
        let mut processed = self.processed_events.write().await;
        processed.clear();
        debug!("Cleared processed events tracker");
    }
    
    /// Get count of processed events
    pub async fn processed_count(&self) -> usize {
        let processed = self.processed_events.read().await;
        processed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::EventPayload;
    
    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new("test_events.jsonl".to_string());
        
        let called = Arc::new(RwLock::new(false));
        let called_clone = Arc::clone(&called);
        
        let handler: EventHandler = Arc::new(move |_event| {
            let called = Arc::clone(&called_clone);
            Box::pin(async move {
                let mut c = called.write().await;
                *c = true;
                Ok(())
            })
        });
        
        bus.subscribe(EventType::ConfigLoaded, handler).await;
        bus.start_processing().await;
        
        let event = Event::new(
            EventType::ConfigLoaded,
            EventPayload::ConfigLoaded {
                config_hash: "test".to_string(),
                data_paths: vec![],
            },
        );
        
        bus.publish(event).await.unwrap();
        
        // Give time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let was_called = *called.read().await;
        assert!(was_called);
        
        // Cleanup
        let _ = std::fs::remove_file("test_events.jsonl");
    }
    
    #[tokio::test]
    async fn test_idempotency() {
        let bus = EventBus::new("test_idempotency.jsonl".to_string());
        bus.start_processing().await;
        
        let event = Event::new(
            EventType::ConfigLoaded,
            EventPayload::ConfigLoaded {
                config_hash: "test".to_string(),
                data_paths: vec![],
            },
        );
        
        // First publish should succeed
        let result1 = bus.publish(event.clone()).await;
        assert!(result1.is_ok());
        
        // Second publish with same key should fail
        let result2 = bus.publish(event).await;
        assert!(result2.is_err());
        
        // Cleanup
        let _ = std::fs::remove_file("test_idempotency.jsonl");
    }
}

