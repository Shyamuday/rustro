/// Risk management: VIX monitoring, loss limits, circuit breakers
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::{Result, TradingError};
use crate::events::{Event, EventBus, EventPayload, EventType};
use crate::positions::PositionManager;
use crate::types::Config;

pub struct RiskManager {
    event_bus: Arc<EventBus>,
    config: Arc<Config>,
    position_manager: Arc<PositionManager>,
    
    /// Current VIX level
    current_vix: Arc<RwLock<Option<f64>>>,
    
    /// VIX circuit breaker active
    circuit_breaker_active: Arc<RwLock<bool>>,
    
    /// Daily loss tracker
    daily_start_capital: Arc<RwLock<f64>>,
    consecutive_losses: Arc<RwLock<usize>>,
}

impl RiskManager {
    pub fn new(
        event_bus: Arc<EventBus>,
        config: Arc<Config>,
        position_manager: Arc<PositionManager>,
    ) -> Self {
        RiskManager {
            event_bus,
            config,
            position_manager,
            current_vix: Arc::new(RwLock::new(None)),
            circuit_breaker_active: Arc::new(RwLock::new(false)),
            daily_start_capital: Arc::new(RwLock::new(1_000_000.0)), // Default 10L
            consecutive_losses: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Update VIX and check circuit breaker
    pub async fn update_vix(&self, vix: f64) -> Result<()> {
        {
            let mut current = self.current_vix.write().await;
            *current = Some(vix);
        }
        
        // Emit VIX data event
        self.event_bus.publish(Event::new(
            EventType::VixDataReceived,
            EventPayload::VixDataReceived {
                vix,
                timestamp: chrono::Utc::now(),
            },
        )).await?;
        
        // Check for VIX spike
        if vix >= self.config.vix_spike_threshold {
            let was_active = {
                let breaker = self.circuit_breaker_active.read().await;
                *breaker
            };
            
            if !was_active {
                // Activate circuit breaker
                {
                    let mut breaker = self.circuit_breaker_active.write().await;
                    *breaker = true;
                }
                
                // Get all open positions
                let positions = self.position_manager.get_open_positions().await;
                let position_ids: Vec<String> = positions.iter()
                    .map(|p| p.position_id.clone())
                    .collect();
                
                // Emit VIX spike event
                self.event_bus.publish(Event::new(
                    EventType::VixSpike,
                    EventPayload::VixSpike {
                        vix,
                        threshold: self.config.vix_spike_threshold,
                        positions_to_exit: position_ids.clone(),
                    },
                )).await?;
                
                warn!(
                    "VIX SPIKE: {:.2} >= {:.2} - Circuit breaker ACTIVE - {} positions to exit",
                    vix,
                    self.config.vix_spike_threshold,
                    position_ids.len()
                );
                
                // Request position closures
                for position_id in position_ids {
                    self.event_bus.publish(Event::new(
                        EventType::ExitSignalGenerated,
                        EventPayload::ExitSignalGenerated {
                            position_id,
                            primary_reason: "VIX_SPIKE".to_string(),
                            secondary_reasons: vec![format!("VIX: {:.2}", vix)],
                            priority: 1, // Mandatory priority
                        },
                    )).await?;
                }
            }
        } else if vix < self.config.vix_resume_threshold {
            // Check if we can resume
            let was_active = {
                let mut breaker = self.circuit_breaker_active.write().await;
                let active = *breaker;
                if active {
                    *breaker = false;
                }
                active
            };
            
            if was_active {
                self.event_bus.publish(Event::new(
                    EventType::VixNormalResumed,
                    EventPayload::VixNormalResumed {
                        vix,
                        threshold: self.config.vix_resume_threshold,
                    },
                )).await?;
                
                info!(
                    "VIX normalized: {:.2} < {:.2} - Circuit breaker DEACTIVATED",
                    vix,
                    self.config.vix_resume_threshold
                );
            }
        }
        
        Ok(())
    }
    
    /// Check if circuit breaker is active
    pub async fn is_circuit_breaker_active(&self) -> bool {
        let breaker = self.circuit_breaker_active.read().await;
        *breaker
    }
    
    /// Check daily loss limit
    pub async fn check_daily_loss_limit(&self) -> Result<bool> {
        let daily_pnl = self.position_manager.get_daily_pnl().await;
        let start_capital = {
            let cap = self.daily_start_capital.read().await;
            *cap
        };
        
        let loss_pct = (daily_pnl / start_capital) * 100.0;
        let limit_pct = -self.config.daily_loss_limit_pct;
        
        if loss_pct <= limit_pct {
            // Daily loss limit breached
            let positions = self.position_manager.get_open_positions().await;
            let position_ids: Vec<String> = positions.iter()
                .map(|p| p.position_id.clone())
                .collect();
            
            self.event_bus.publish(Event::new(
                EventType::DailyLossLimitBreached,
                EventPayload::DailyLossLimitBreached {
                    daily_pnl,
                    limit: start_capital * (limit_pct / 100.0),
                    positions_to_close: position_ids.clone(),
                },
            )).await?;
            
            warn!(
                "DAILY LOSS LIMIT BREACHED: {:.2}% (limit: {:.2}%) - Closing all positions",
                loss_pct,
                limit_pct
            );
            
            // Request position closures
            for position_id in position_ids {
                self.event_bus.publish(Event::new(
                    EventType::ExitSignalGenerated,
                    EventPayload::ExitSignalGenerated {
                        position_id,
                        primary_reason: "DAILY_LOSS_LIMIT".to_string(),
                        secondary_reasons: vec![format!("Loss: {:.2}%", loss_pct)],
                        priority: 1, // Mandatory
                    },
                )).await?;
            }
            
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Check consecutive loss limit
    pub async fn check_consecutive_losses(&self, trade_result: bool) -> bool {
        let mut losses = self.consecutive_losses.write().await;
        
        if trade_result {
            // Win - reset counter
            *losses = 0;
        } else {
            // Loss - increment
            *losses += 1;
            
            if *losses >= self.config.consecutive_loss_limit {
                warn!(
                    "CONSECUTIVE LOSS LIMIT reached: {} losses",
                    *losses
                );
                return true;
            }
        }
        
        false
    }
    
    /// Calculate position size based on VIX and DTE
    pub fn calculate_position_size(
        &self,
        base_capital: f64,
        vix: f64,
        days_to_expiry: i32,
    ) -> i32 {
        // VIX multiplier
        let vix_mult = if vix <= 12.0 {
            self.config.vix_mult_anchors.vix_12_or_below
        } else if vix <= 20.0 {
            // Linear interpolation between 12 and 20
            let t = (vix - 12.0) / (20.0 - 12.0);
            self.config.vix_mult_anchors.vix_12_or_below * (1.0 - t) + self.config.vix_mult_anchors.vix_20 * t
        } else if vix <= 30.0 {
            // Linear interpolation between 20 and 30
            let t = (vix - 20.0) / (30.0 - 20.0);
            self.config.vix_mult_anchors.vix_20 * (1.0 - t) + self.config.vix_mult_anchors.vix_30 * t
        } else {
            self.config.vix_mult_anchors.vix_30_or_above
        };
        
        // DTE multiplier
        let dte_mult = if days_to_expiry >= 5 {
            self.config.dte_mult.gte_5_days
        } else if days_to_expiry >= 2 {
            self.config.dte_mult.days_2_to_4
        } else {
            self.config.dte_mult.day_1
        };
        
        // Calculate quantity
        let base_size = base_capital * (self.config.base_position_size_pct / 100.0);
        let adjusted_size = base_size * vix_mult * dte_mult;
        
        // Round down to nearest lot
        let quantity = (adjusted_size / 50.0).floor() as i32 * 50;
        
        info!(
            "Position size: VIX={:.1} (mult={:.2}), DTE={} (mult={:.2}) → {} qty",
            vix, vix_mult, days_to_expiry, dte_mult, quantity
        );
        
        quantity.max(50) // At least 1 lot (NIFTY)
    }
    
    /// Pre-entry risk check
    pub async fn pre_entry_risk_check(&self) -> Result<()> {
        // Check circuit breaker
        if self.is_circuit_breaker_active().await {
            return Err(TradingError::RiskCheckFailed(
                "VIX circuit breaker active".to_string()
            ));
        }
        
        // Check daily loss limit
        if self.check_daily_loss_limit().await? {
            return Err(TradingError::DailyLossLimit(
                "Daily loss limit breached".to_string()
            ));
        }
        
        // Check max positions
        let open_positions = self.position_manager.get_open_positions().await;
        if open_positions.len() >= self.config.max_positions {
            return Err(TradingError::PositionLimitExceeded(
                format!("Max positions: {}", self.config.max_positions)
            ));
        }
        
        // Emit risk check passed event
        self.event_bus.publish(Event::new(
            EventType::RiskCheckPassed,
            EventPayload::RiskCheckPassed {
                check_type: "PRE_ENTRY".to_string(),
            },
        )).await?;
        
        Ok(())
    }
    
    /// Get current VIX
    pub async fn get_current_vix(&self) -> Option<f64> {
        let vix = self.current_vix.read().await;
        *vix
    }
    
    /// Set daily start capital
    pub async fn set_daily_start_capital(&self, capital: f64) {
        let mut cap = self.daily_start_capital.write().await;
        *cap = capital;
        info!("Daily start capital set to: {:.2}", capital);
    }
    
    /// Reset daily counters
    pub async fn reset_daily(&self) {
        {
            let mut losses = self.consecutive_losses.write().await;
            *losses = 0;
        }
        {
            let mut breaker = self.circuit_breaker_active.write().await;
            *breaker = false;
        }
        info!("Risk manager daily reset complete");
    }
}

