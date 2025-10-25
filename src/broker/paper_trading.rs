/// Paper trading (simulation mode) wrapper
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::Result;
use crate::types::{OrderType, Side};

/// Paper trading broker that simulates orders
pub struct PaperTradingBroker {
    /// Simulated orders
    orders: Arc<RwLock<HashMap<String, SimulatedOrder>>>,
    
    /// Simulated fills (instant for paper trading)
    auto_fill: bool,
    
    /// Simulated slippage (basis points)
    slippage_bps: f64,
}

#[derive(Debug, Clone)]
struct SimulatedOrder {
    _order_id: String,
    _symbol: String,
    side: Side,
    _quantity: i32,
    _order_type: OrderType,
    limit_price: Option<f64>,
    fill_price: Option<f64>,
    filled: bool,
}

impl PaperTradingBroker {
    pub fn new(auto_fill: bool, slippage_bps: f64) -> Self {
        PaperTradingBroker {
            orders: Arc::new(RwLock::new(HashMap::new())),
            auto_fill,
            slippage_bps,
        }
    }
    
    /// Place a simulated order
    pub async fn place_order(
        &self,
        symbol: String,
        side: Side,
        quantity: i32,
        order_type: OrderType,
        limit_price: Option<f64>,
    ) -> Result<String> {
        let order_id = format!("PAPER_{}", uuid::Uuid::new_v4());
        
        let mut order = SimulatedOrder {
            _order_id: order_id.clone(),
            _symbol: symbol.clone(),
            side,
            _quantity: quantity,
            _order_type: order_type,
            limit_price,
            fill_price: None,
            filled: false,
        };
        
        // Auto-fill if enabled
        if self.auto_fill {
            let fill_price = self.calculate_fill_price(&order);
            order.fill_price = Some(fill_price);
            order.filled = true;
            
            warn!(
                "📝 [PAPER] Order filled: {} {} {} @ {:.2} (simulated)",
                side.as_str(),
                quantity,
                symbol,
                fill_price
            );
        }
        
        let mut orders = self.orders.write().await;
        orders.insert(order_id.clone(), order);
        
        info!("📝 [PAPER] Order placed: {} (simulated)", order_id);
        
        Ok(order_id)
    }
    
    /// Calculate simulated fill price with slippage
    fn calculate_fill_price(&self, order: &SimulatedOrder) -> f64 {
        let base_price = order.limit_price.unwrap_or(100.0);
        let slippage = base_price * (self.slippage_bps / 10000.0);
        
        match order.side {
            Side::Buy => base_price + slippage,  // Buy higher
            Side::Sell => base_price - slippage, // Sell lower
        }
    }
    
    /// Get order status
    pub async fn get_order_status(&self, order_id: &str) -> Option<bool> {
        let orders = self.orders.read().await;
        orders.get(order_id).map(|o| o.filled)
    }
    
    /// Get simulated fill price
    pub async fn get_fill_price(&self, order_id: &str) -> Option<f64> {
        let orders = self.orders.read().await;
        orders.get(order_id).and_then(|o| o.fill_price)
    }
    
    /// Get total simulated orders
    pub async fn total_orders(&self) -> usize {
        let orders = self.orders.read().await;
        orders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_paper_trading() {
        let broker = PaperTradingBroker::new(true, 5.0); // 5bps slippage
        
        let order_id = broker.place_order(
            "NIFTY19500CE".to_string(),
            Side::Buy,
            50,
            OrderType::Limit,
            Some(125.0),
        ).await.unwrap();
        
        assert!(broker.get_order_status(&order_id).await.unwrap());
        
        let fill_price = broker.get_fill_price(&order_id).await.unwrap();
        assert!(fill_price > 125.0); // Should have slippage
    }
}
