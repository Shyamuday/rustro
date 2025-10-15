/// Position tracking with stop loss and trailing stop
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{Result, TradingError};
use crate::events::{Event, EventBus, EventPayload, EventType};
use crate::types::{Config, Position, PositionStatus, Trade};

pub struct PositionManager {
    event_bus: Arc<EventBus>,
    config: Arc<Config>,
    
    /// Active positions
    positions: Arc<RwLock<HashMap<String, Position>>>,
    
    /// Completed trades
    trades: Arc<RwLock<Vec<Trade>>>,
    
    /// Daily PNL tracker
    daily_pnl: Arc<RwLock<f64>>,
}

impl PositionManager {
    pub fn new(event_bus: Arc<EventBus>, config: Arc<Config>) -> Self {
        PositionManager {
            event_bus,
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            trades: Arc::new(RwLock::new(Vec::new())),
            daily_pnl: Arc::new(RwLock::new(0.0)),
        }
    }
    
    /// Open a new position
    pub async fn open_position(&self, position: Position) -> Result<()> {
        let position_id = position.position_id.clone();
        
        // Store position
        {
            let mut positions = self.positions.write().await;
            if positions.contains_key(&position_id) {
                return Err(TradingError::DuplicatePosition(position_id));
            }
            positions.insert(position_id.clone(), position.clone());
        }
        
        // Emit event
        self.event_bus.publish(Event::new(
            EventType::PositionOpened,
            EventPayload::PositionOpened {
                position_id: position_id.clone(),
                symbol: position.symbol.clone(),
                quantity: position.quantity,
                entry_price: position.entry_price,
                entry_reason: position.entry_reason.clone(),
            },
        )).await?;
        
        info!(
            "Position opened: {} x {} @ {:.2}",
            position.symbol,
            position.quantity,
            position.entry_price
        );
        
        Ok(())
    }
    
    /// Update position with current price and check stop loss/target
    pub async fn update_position(
        &self,
        position_id: &str,
        current_price: f64,
    ) -> Result<Option<String>> {
        let mut positions = self.positions.write().await;
        
        let position = positions.get_mut(position_id)
            .ok_or_else(|| TradingError::PositionNotFound(position_id.to_string()))?;
        
        // Update current price
        position.current_price = current_price;
        
        // Calculate PNL
        let price_diff = current_price - position.entry_price;
        position.pnl = price_diff * position.quantity as f64;
        position.pnl_pct = (price_diff / position.entry_price) * 100.0;
        
        // Update trailing stop if active
        if self.config.use_trailing_stop && position.trailing_active {
            let new_trail = current_price * (1.0 - self.config.trail_gap_pct);
            if let Some(current_trail) = position.trailing_stop {
                if new_trail > current_trail {
                    position.trailing_stop = Some(new_trail);
                    
                    // Emit trailing stop update event
                    self.event_bus.publish(Event::new(
                        EventType::TrailingStopUpdated,
                        EventPayload::TrailingStopUpdated {
                            position_id: position_id.to_string(),
                            new_trail_stop: new_trail,
                            high_price: current_price,
                        },
                    )).await?;
                    
                    debug!(
                        "Trailing stop updated for {}: {:.2}",
                        position_id,
                        new_trail
                    );
                }
            }
        }
        
        // Activate trailing stop if PNL threshold reached
        if self.config.use_trailing_stop 
            && !position.trailing_active 
            && position.pnl_pct >= self.config.trail_activate_pnl_pct * 100.0
        {
            position.trailing_active = true;
            position.trailing_stop = Some(current_price * (1.0 - self.config.trail_gap_pct));
            
            self.event_bus.publish(Event::new(
                EventType::TrailingStopActivated,
                EventPayload::TrailingStopActivated {
                    position_id: position_id.to_string(),
                    activation_pnl_pct: position.pnl_pct,
                    trail_stop: position.trailing_stop.unwrap(),
                },
            )).await?;
            
            info!(
                "Trailing stop activated for {} @ {:.2} (PNL: {:.2}%)",
                position_id,
                position.trailing_stop.unwrap(),
                position.pnl_pct
            );
        }
        
        // Check stop loss
        if current_price <= position.stop_loss {
            self.event_bus.publish(Event::new(
                EventType::StopLossTriggered,
                EventPayload::StopLossTriggered {
                    position_id: position_id.to_string(),
                    stop_loss: position.stop_loss,
                    current_price,
                },
            )).await?;
            
            warn!(
                "Stop loss triggered for {}: {:.2} <= {:.2}",
                position_id,
                current_price,
                position.stop_loss
            );
            
            return Ok(Some("STOP_LOSS".to_string()));
        }
        
        // Check trailing stop
        if let Some(trail_stop) = position.trailing_stop {
            if position.trailing_active && current_price <= trail_stop {
                info!(
                    "Trailing stop triggered for {}: {:.2} <= {:.2}",
                    position_id,
                    current_price,
                    trail_stop
                );
                return Ok(Some("TRAILING_STOP".to_string()));
            }
        }
        
        // Check target
        if let Some(target) = position.target {
            if current_price >= target {
                self.event_bus.publish(Event::new(
                    EventType::TargetReached,
                    EventPayload::TargetReached {
                        position_id: position_id.to_string(),
                        target,
                        current_price,
                    },
                )).await?;
                
                info!(
                    "Target reached for {}: {:.2} >= {:.2}",
                    position_id,
                    current_price,
                    target
                );
                return Ok(Some("TARGET".to_string()));
            }
        }
        
        // Emit position update event
        self.event_bus.publish(Event::new(
            EventType::PositionUpdated,
            EventPayload::PositionUpdated {
                position_id: position_id.to_string(),
                current_price,
                pnl: position.pnl,
                pnl_pct: position.pnl_pct,
            },
        )).await?;
        
        Ok(None)
    }
    
    /// Close a position
    pub async fn close_position(
        &self,
        position_id: &str,
        exit_price: f64,
        exit_reason: String,
    ) -> Result<Trade> {
        let mut positions = self.positions.write().await;
        
        let mut position = positions.remove(position_id)
            .ok_or_else(|| TradingError::PositionNotFound(position_id.to_string()))?;
        
        position.status = PositionStatus::Closed;
        
        // Calculate final PNL
        let price_diff = exit_price - position.entry_price;
        let pnl_gross = price_diff * position.quantity as f64;
        let pnl_gross_pct = (price_diff / position.entry_price) * 100.0;
        
        // Estimate brokerage (simplified)
        let brokerage = (exit_price * position.quantity as f64 * 0.0003).max(20.0);
        let pnl_net = pnl_gross - brokerage;
        
        // Create trade record
        let exit_time = chrono::Utc::now();
        let duration_sec = (exit_time - position.entry_time).num_seconds();
        
        let trade = Trade {
            trade_id: uuid::Uuid::new_v4().to_string(),
            position_id: position_id.to_string(),
            symbol: position.symbol.clone(),
            underlying: position.underlying.clone(),
            strike: position.strike,
            option_type: position.option_type,
            quantity: position.quantity,
            entry_time: position.entry_time,
            entry_price: position.entry_price,
            entry_reason: position.entry_reason.clone(),
            exit_time,
            exit_price,
            exit_reason: exit_reason.clone(),
            secondary_reasons: vec![],
            pnl_gross,
            pnl_gross_pct,
            pnl_net,
            brokerage,
            duration_sec,
            high_price: position.current_price.max(position.entry_price),
            low_price: position.current_price.min(position.entry_price),
            vix_at_entry: 0.0, // Would be tracked separately
            vix_at_exit: 0.0,
        };
        
        // Update daily PNL
        {
            let mut daily_pnl = self.daily_pnl.write().await;
            *daily_pnl += pnl_net;
        }
        
        // Store trade
        {
            let mut trades = self.trades.write().await;
            trades.push(trade.clone());
        }
        
        // Emit event
        self.event_bus.publish(Event::new(
            EventType::PositionClosed,
            EventPayload::PositionClosed {
                position_id: position_id.to_string(),
                exit_price,
                exit_reason,
                pnl_gross,
                pnl_gross_pct,
            },
        )).await?;
        
        info!(
            "Position closed: {} - PNL: {:.2} ({:.2}%) - Reason: {}",
            position_id,
            pnl_net,
            pnl_gross_pct,
            trade.exit_reason
        );
        
        Ok(trade)
    }
    
    /// Get position by ID
    pub async fn get_position(&self, position_id: &str) -> Option<Position> {
        let positions = self.positions.read().await;
        positions.get(position_id).cloned()
    }
    
    /// Get all open positions
    pub async fn get_open_positions(&self) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.values()
            .filter(|p| p.status == PositionStatus::Open)
            .cloned()
            .collect()
    }
    
    /// Get daily PNL
    pub async fn get_daily_pnl(&self) -> f64 {
        let pnl = self.daily_pnl.read().await;
        *pnl
    }
    
    /// Reset daily PNL (at EOD)
    pub async fn reset_daily_pnl(&self) {
        let mut pnl = self.daily_pnl.write().await;
        *pnl = 0.0;
        info!("Daily PNL reset");
    }
    
    /// Get all trades for the day
    pub async fn get_daily_trades(&self) -> Vec<Trade> {
        let trades = self.trades.read().await;
        trades.clone()
    }
    
    /// Close all open positions (emergency)
    pub async fn close_all_positions(&self, reason: String) -> Result<Vec<Trade>> {
        let position_ids: Vec<String> = {
            let positions = self.positions.read().await;
            positions.keys().cloned().collect()
        };
        
        let mut closed_trades = Vec::new();
        
        for position_id in position_ids {
            let position = self.get_position(&position_id).await;
            if let Some(pos) = position {
                // Use current price as exit price
                match self.close_position(&position_id, pos.current_price, reason.clone()).await {
                    Ok(trade) => closed_trades.push(trade),
                    Err(e) => {
                        warn!("Failed to close position {}: {}", position_id, e);
                    }
                }
            }
        }
        
        info!("Closed {} positions - Reason: {}", closed_trades.len(), reason);
        
        Ok(closed_trades)
    }
}

