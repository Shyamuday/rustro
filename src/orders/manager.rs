/// Order management with retry logic and idempotency
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::broker::AngelOneClient;
use crate::error::{Result, TradingError};
use crate::events::{Event, EventBus, EventPayload, EventType};
use crate::types::{Config, Order, OrderStatus, OrderType, Side};

pub struct OrderManager {
    broker: Arc<AngelOneClient>,
    event_bus: Arc<EventBus>,
    config: Arc<Config>,
    
    /// Active orders being tracked
    orders: Arc<RwLock<HashMap<String, Order>>>,
    
    /// Idempotency tracker
    processed_intents: Arc<RwLock<HashMap<String, String>>>,
}

impl OrderManager {
    pub fn new(
        broker: Arc<AngelOneClient>,
        event_bus: Arc<EventBus>,
        config: Arc<Config>,
    ) -> Self {
        OrderManager {
            broker,
            event_bus,
            config,
            orders: Arc::new(RwLock::new(HashMap::new())),
            processed_intents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Place an order with retry logic
    pub async fn place_order(
        &self,
        symbol: String,
        token: String,
        side: Side,
        quantity: i32,
        initial_price: f64,
        idempotency_key: String,
    ) -> Result<String> {
        // Check idempotency
        {
            let processed = self.processed_intents.read().await;
            if let Some(existing_order_id) = processed.get(&idempotency_key) {
                info!("Order already processed: {}", existing_order_id);
                return Ok(existing_order_id.clone());
            }
        }
        
        // Create order intent
        let order_id = uuid::Uuid::new_v4().to_string();
        
        let mut order = Order {
            order_id: order_id.clone(),
            broker_order_id: None,
            position_id: String::new(), // Will be set by position manager
            symbol: symbol.clone(),
            side,
            order_type: OrderType::Limit,
            quantity,
            limit_price: Some(initial_price),
            fill_price: None,
            fill_quantity: 0,
            fill_time: None,
            status: OrderStatus::Pending,
            attempts: 0,
            retry_count: 0,
            idempotency_key: idempotency_key.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Store order
        {
            let mut orders = self.orders.write().await;
            orders.insert(order_id.clone(), order.clone());
        }
        
        // Emit event
        self.event_bus.publish(Event::new(
            EventType::OrderIntentCreated,
            EventPayload::OrderIntentCreated {
                order_id: order_id.clone(),
                symbol: symbol.clone(),
                side,
                quantity,
                intent_reason: "Strategy signal".to_string(),
            },
        )).await?;
        
        // Attempt order placement with retries
        let mut current_price = initial_price;
        let max_retries = self.config.order_max_retries as usize;
        
        for attempt in 0..=max_retries {
            order.attempts = attempt as u32;
            order.retry_count = attempt as u32;
            order.limit_price = Some(current_price);
            order.updated_at = chrono::Utc::now();
            
            // Update order in store
            {
                let mut orders = self.orders.write().await;
                orders.insert(order_id.clone(), order.clone());
            }
            
            if attempt > 0 {
                // Emit retry event
                let backoff_sec = self.config.order_retry_backoffs_sec
                    .get(attempt - 1)
                    .copied()
                    .unwrap_or(8);
                
                self.event_bus.publish(Event::new(
                    EventType::OrderRetrying,
                    EventPayload::OrderRetrying {
                        order_id: order_id.clone(),
                        attempt: attempt as u32,
                        max_retries: max_retries as u32,
                        backoff_sec,
                    },
                )).await?;
                
                // Backoff
                tokio::time::sleep(tokio::time::Duration::from_secs(backoff_sec)).await;
                
                // Adjust price for retry
                if attempt <= self.config.order_retry_steps_pct.len() {
                    let adjustment_pct = self.config.order_retry_steps_pct[attempt - 1];
                    current_price = initial_price * (1.0 + adjustment_pct / 100.0);
                    info!(
                        "Retry {} for order {}: adjusted price to {:.2} (+{:.2}%)",
                        attempt,
                        order_id,
                        current_price,
                        adjustment_pct
                    );
                }
            }
            
            // Place order with broker
            match self.broker.place_order(
                &symbol,
                &token,
                side,
                quantity,
                OrderType::Limit,
                Some(current_price),
            ).await {
                Ok(broker_order_id) => {
                    // Success!
                    order.broker_order_id = Some(broker_order_id.clone());
                    order.status = OrderStatus::Submitted;
                    order.updated_at = chrono::Utc::now();
                    
                    // Update store
                    {
                        let mut orders = self.orders.write().await;
                        orders.insert(order_id.clone(), order.clone());
                    }
                    
                    // Track idempotency
                    {
                        let mut processed = self.processed_intents.write().await;
                        processed.insert(idempotency_key.clone(), order_id.clone());
                    }
                    
                    // Emit success event
                    self.event_bus.publish(Event::new(
                        EventType::OrderPlaced,
                        EventPayload::OrderPlaced {
                            order_id: order_id.clone(),
                            broker_order_id,
                            symbol: symbol.clone(),
                            quantity,
                            price: current_price,
                        },
                    )).await?;
                    
                    info!("Order placed successfully: {}", order_id);
                    return Ok(order_id);
                }
                Err(e) => {
                    error!(
                        "Order placement failed (attempt {}): {} ({})",
                        attempt + 1,
                        e,
                        e.error_code()
                    );
                    
                    if attempt == max_retries {
                        // Final failure
                        order.status = OrderStatus::Failed;
                        order.updated_at = chrono::Utc::now();
                        
                        // Update store
                        {
                            let mut orders = self.orders.write().await;
                            orders.insert(order_id.clone(), order.clone());
                        }
                        
                        // Emit failure event
                        self.event_bus.publish(Event::new(
                            EventType::OrderFailed,
                            EventPayload::OrderFailed {
                                order_id: order_id.clone(),
                                reason: e.to_string(),
                                retry_count: max_retries as u32,
                            },
                        )).await?;
                        
                        return Err(TradingError::OrderPlacementFailed(format!(
                            "Order failed after {} attempts: {}",
                            max_retries + 1,
                            e
                        )));
                    }
                    
                    // Continue to next retry
                }
            }
        }
        
        Err(TradingError::OrderPlacementFailed(
            "Max retries exceeded".to_string()
        ))
    }
    
    /// Mark order as executed
    pub async fn mark_executed(
        &self,
        order_id: &str,
        fill_price: f64,
        fill_quantity: i32,
    ) -> Result<()> {
        let mut orders = self.orders.write().await;
        
        if let Some(order) = orders.get_mut(order_id) {
            order.fill_price = Some(fill_price);
            order.fill_quantity = fill_quantity;
            order.fill_time = Some(chrono::Utc::now());
            order.status = if fill_quantity >= order.quantity {
                OrderStatus::Filled
            } else {
                OrderStatus::PartiallyFilled
            };
            order.updated_at = chrono::Utc::now();
            
            // Emit event
            self.event_bus.publish(Event::new(
                EventType::OrderExecuted,
                EventPayload::OrderExecuted {
                    order_id: order_id.to_string(),
                    broker_order_id: order.broker_order_id.clone().unwrap_or_default(),
                    fill_price,
                    fill_quantity,
                    fill_time: order.fill_time.unwrap(),
                },
            )).await?;
            
            info!("Order executed: {} @ {:.2}", order_id, fill_price);
            Ok(())
        } else {
            Err(TradingError::OrderNotFound(order_id.to_string()))
        }
    }
    
    /// Get order by ID
    pub async fn get_order(&self, order_id: &str) -> Option<Order> {
        let orders = self.orders.read().await;
        orders.get(order_id).cloned()
    }
    
    /// Get all active orders
    pub async fn get_active_orders(&self) -> Vec<Order> {
        let orders = self.orders.read().await;
        orders.values()
            .filter(|o| matches!(o.status, OrderStatus::Pending | OrderStatus::Submitted | OrderStatus::PartiallyFilled))
            .cloned()
            .collect()
    }
    
    /// Clear completed orders (for memory management)
    pub async fn clear_completed_orders(&self) {
        let mut orders = self.orders.write().await;
        orders.retain(|_, o| !matches!(o.status, OrderStatus::Filled | OrderStatus::Failed | OrderStatus::Rejected | OrderStatus::Cancelled));
        debug!("Cleared completed orders, remaining: {}", orders.len());
    }
}

