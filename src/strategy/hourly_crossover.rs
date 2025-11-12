/// Hourly ADX/DMI crossover detector
/// Monitors hourly bars for crossover signals aligned with daily bias

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::data::ConcurrentBarStore;
use crate::error::Result;
use crate::strategy::{calculate_adx, BiasDirection};
use crate::types::Bar;

/// Crossover signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossoverSignal {
    pub underlying: String,
    pub spot_token: String,
    pub timestamp: DateTime<Utc>,
    pub direction: BiasDirection,
    pub adx: f64,
    pub plus_di: f64,
    pub minus_di: f64,
    pub close_price: f64,
    pub aligned_with_daily: bool,
}

/// Hourly crossover state for tracking
#[derive(Debug, Clone)]
struct CrossoverState {
    last_plus_di: f64,
    last_minus_di: f64,
    last_adx: f64,
}

/// Hourly crossover monitor
pub struct HourlyCrossoverMonitor {
    adx_period: usize,
    adx_threshold: f64,
    hourly_stores: Arc<RwLock<HashMap<String, Arc<ConcurrentBarStore>>>>,
    crossover_states: Arc<RwLock<HashMap<String, CrossoverState>>>,
}

impl HourlyCrossoverMonitor {
    pub fn new(adx_period: usize, adx_threshold: f64) -> Self {
        Self {
            adx_period,
            adx_threshold,
            hourly_stores: Arc::new(RwLock::new(HashMap::new())),
            crossover_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register hourly bar store for an underlying
    pub async fn register_underlying(
        &self,
        underlying: String,
        spot_token: String,
        hourly_store: Arc<ConcurrentBarStore>,
    ) {
        let mut stores = self.hourly_stores.write().await;
        stores.insert(spot_token, hourly_store);
        info!("📊 Registered {} for hourly monitoring", underlying);
    }

    /// Check for crossover on latest hourly bar
    pub async fn check_crossover(
        &self,
        underlying: &str,
        spot_token: &str,
        daily_bias: BiasDirection,
    ) -> Result<Option<CrossoverSignal>> {
        // Get hourly bars
        let stores = self.hourly_stores.read().await;
        let store = stores.get(spot_token)
            .ok_or_else(|| crate::error::TradingError::MissingData(
                format!("No hourly store for {}", underlying)
            ))?;

        let hourly_bars = store.get_recent(self.adx_period + 10).await?;
        
        if hourly_bars.len() < self.adx_period + 2 {
            warn!("{}: Not enough hourly bars ({} < {})", 
                  underlying, hourly_bars.len(), self.adx_period + 2);
            return Ok(None);
        }

        // Calculate current ADX/DMI
        let (current_adx, current_plus_di, current_minus_di) = 
            calculate_adx(&hourly_bars, self.adx_period)
                .ok_or_else(|| crate::error::TradingError::InvalidBarData(
                    "Failed to calculate ADX".to_string()
                ))?;

        // Check ADX threshold
        if current_adx < self.adx_threshold {
            return Ok(None);
        }

        // Get previous state
        let mut states = self.crossover_states.write().await;
        let prev_state = states.get(spot_token);

        // Detect crossover
        let crossover_direction = if let Some(prev) = prev_state {
            self.detect_crossover(
                prev.last_plus_di,
                prev.last_minus_di,
                current_plus_di,
                current_minus_di,
            )
        } else {
            // First check, no crossover yet
            None
        };

        // Update state
        states.insert(spot_token.to_string(), CrossoverState {
            last_plus_di: current_plus_di,
            last_minus_di: current_minus_di,
            last_adx: current_adx,
        });

        // If crossover detected, check alignment
        if let Some(direction) = crossover_direction {
            let aligned = self.is_aligned_with_daily(direction, daily_bias);
            
            if aligned {
                let latest_bar = hourly_bars.last().unwrap();
                
                info!("🎯 CROSSOVER DETECTED: {} {} @ {}", 
                      underlying, direction.as_str(), latest_bar.timestamp);
                info!("   ADX: {:.2}, +DI: {:.2}, -DI: {:.2}, Close: {:.2}",
                      current_adx, current_plus_di, current_minus_di, latest_bar.close);
                info!("   ✅ ALIGNED with daily bias: {}", daily_bias.as_str());

                return Ok(Some(CrossoverSignal {
                    underlying: underlying.to_string(),
                    spot_token: spot_token.to_string(),
                    timestamp: latest_bar.timestamp,
                    direction,
                    adx: current_adx,
                    plus_di: current_plus_di,
                    minus_di: current_minus_di,
                    close_price: latest_bar.close,
                    aligned_with_daily: true,
                }));
            } else {
                info!("⚠️  Crossover detected for {} but NOT aligned with daily bias", underlying);
                info!("   Hourly: {}, Daily: {}", direction.as_str(), daily_bias.as_str());
            }
        }

        Ok(None)
    }

    /// Detect crossover between previous and current DI values
    fn detect_crossover(
        &self,
        prev_plus_di: f64,
        prev_minus_di: f64,
        curr_plus_di: f64,
        curr_minus_di: f64,
    ) -> Option<BiasDirection> {
        // Bullish crossover: +DI crosses above -DI
        if prev_plus_di <= prev_minus_di && curr_plus_di > curr_minus_di {
            return Some(BiasDirection::CE);
        }

        // Bearish crossover: -DI crosses above +DI
        if prev_minus_di <= prev_plus_di && curr_minus_di > curr_plus_di {
            return Some(BiasDirection::PE);
        }

        None
    }

    /// Check if hourly crossover aligns with daily bias
    fn is_aligned_with_daily(
        &self,
        hourly_direction: BiasDirection,
        daily_bias: BiasDirection,
    ) -> bool {
        hourly_direction == daily_bias
    }

    /// Check all monitored underlyings for crossover
    pub async fn check_all_crossovers(
        &self,
        daily_biases: &HashMap<String, BiasDirection>,
    ) -> Result<Vec<CrossoverSignal>> {
        let mut signals = Vec::new();

        let stores = self.hourly_stores.read().await;
        
        for (spot_token, _store) in stores.iter() {
            // Find underlying name from daily biases
            if let Some((underlying, daily_bias)) = daily_biases.iter()
                .find(|(_, _)| true) // Would match by token
            {
                if let Some(signal) = self.check_crossover(
                    underlying,
                    spot_token,
                    *daily_bias,
                ).await? {
                    signals.push(signal);
                }
            }
        }

        Ok(signals)
    }

    /// Get current ADX/DMI values for an underlying
    pub async fn get_current_indicators(
        &self,
        spot_token: &str,
    ) -> Result<Option<(f64, f64, f64)>> {
        let stores = self.hourly_stores.read().await;
        let store = stores.get(spot_token)
            .ok_or_else(|| crate::error::TradingError::MissingData(
                format!("No hourly store for token {}", spot_token)
            ))?;

        let hourly_bars = store.get_recent(self.adx_period + 10).await?;
        
        if hourly_bars.len() < self.adx_period + 1 {
            return Ok(None);
        }

        Ok(calculate_adx(&hourly_bars, self.adx_period))
    }

    /// Clear crossover state (e.g., at EOD)
    pub async fn clear_states(&self) {
        let mut states = self.crossover_states.write().await;
        states.clear();
        info!("🧹 Cleared crossover states");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossover_detection() {
        let monitor = HourlyCrossoverMonitor::new(14, 25.0);

        // Test bullish crossover
        let crossover = monitor.detect_crossover(
            20.0, 25.0,  // prev: +DI below -DI
            26.0, 24.0,  // curr: +DI above -DI
        );
        assert_eq!(crossover, Some(BiasDirection::CE));

        // Test bearish crossover
        let crossover = monitor.detect_crossover(
            25.0, 20.0,  // prev: +DI above -DI
            24.0, 26.0,  // curr: -DI above +DI
        );
        assert_eq!(crossover, Some(BiasDirection::PE));

        // Test no crossover
        let crossover = monitor.detect_crossover(
            25.0, 20.0,
            26.0, 21.0,  // Still +DI above -DI
        );
        assert_eq!(crossover, None);
    }

    #[test]
    fn test_alignment_check() {
        let monitor = HourlyCrossoverMonitor::new(14, 25.0);

        // Aligned: Both CE
        assert!(monitor.is_aligned_with_daily(BiasDirection::CE, BiasDirection::CE));

        // Aligned: Both PE
        assert!(monitor.is_aligned_with_daily(BiasDirection::PE, BiasDirection::PE));

        // Not aligned: CE vs PE
        assert!(!monitor.is_aligned_with_daily(BiasDirection::CE, BiasDirection::PE));

        // Not aligned: PE vs CE
        assert!(!monitor.is_aligned_with_daily(BiasDirection::PE, BiasDirection::CE));
    }
}

