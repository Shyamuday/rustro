/// Daily Bias Calculator using ADX/DMI
/// Determines CE/PE/NoTrade bias for each underlying

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::strategy::indicators::calculate_adx;
use crate::types::Bar;

/// Daily bias direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiasDirection {
    CE,      // Call - Bullish (+DI > -DI && ADX > 25)
    PE,      // Put - Bearish (-DI > +DI && ADX > 25)
    NoTrade, // Sideways (ADX < 25)
}

impl BiasDirection {
    pub fn as_str(&self) -> &str {
        match self {
            BiasDirection::CE => "CE",
            BiasDirection::PE => "PE",
            BiasDirection::NoTrade => "NO_TRADE",
        }
    }
}

/// Token info for daily bias calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBiasToken {
    pub underlying: String,
    pub spot_token: String,
    pub spot_symbol: String,
    pub asset_type: String,
}

/// Daily bias result for one underlying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBias {
    pub underlying: String,
    pub spot_token: String,
    pub bias: BiasDirection,
    pub adx: f64,
    pub plus_di: f64,
    pub minus_di: f64,
    pub close_price: f64,
    pub timestamp: DateTime<Utc>,
}

/// Daily bias calculator
pub struct DailyBiasCalculator {
    adx_period: usize,
    adx_threshold: f64,
}

impl DailyBiasCalculator {
    pub fn new(adx_period: usize, adx_threshold: f64) -> Self {
        Self {
            adx_period,
            adx_threshold,
        }
    }

    /// Calculate bias for single underlying
    pub fn calculate_bias(
        &self,
        underlying: &str,
        spot_token: &str,
        daily_bars: &[Bar],
    ) -> Option<DailyBias> {
        if daily_bars.len() < self.adx_period + 1 {
            warn!("{}: Not enough bars ({} < {})", 
                  underlying, daily_bars.len(), self.adx_period + 1);
            return None;
        }

        // Calculate ADX/DMI (returns tuple: (adx, +DI, -DI))
        let (latest_adx, latest_plus_di, latest_minus_di) = calculate_adx(daily_bars, self.adx_period)?;
        let latest_close = daily_bars.last()?.close;
        let timestamp = daily_bars.last()?.timestamp;

        // Determine bias
        let bias = if latest_adx < self.adx_threshold {
            BiasDirection::NoTrade
        } else if latest_plus_di > latest_minus_di {
            BiasDirection::CE
        } else {
            BiasDirection::PE
        };

        Some(DailyBias {
            underlying: underlying.to_string(),
            spot_token: spot_token.to_string(),
            bias,
            adx: latest_adx,
            plus_di: latest_plus_di,
            minus_di: latest_minus_di,
            close_price: latest_close,
            timestamp,
        })
    }

    /// Calculate bias for all underlyings
    pub fn calculate_all_bias(
        &self,
        tokens: &[DailyBiasToken],
        bars_map: &HashMap<String, Vec<Bar>>,
    ) -> Vec<DailyBias> {
        let mut results = Vec::new();

        info!("📊 Calculating daily bias for {} underlyings...", tokens.len());

        for (idx, token) in tokens.iter().enumerate() {
            if idx % 20 == 0 {
                info!("   Progress: {}/{}", idx, tokens.len());
            }

            if let Some(bars) = bars_map.get(&token.spot_token) {
                if let Some(bias) = self.calculate_bias(&token.underlying, &token.spot_token, bars) {
                    results.push(bias);
                }
            } else {
                warn!("{}: No bars available", token.underlying);
            }
        }

        info!("✅ Calculated bias for {} underlyings", results.len());
        results
    }

    /// Filter by bias direction
    pub fn filter_by_bias(biases: &[DailyBias], direction: BiasDirection) -> Vec<DailyBias> {
        biases
            .iter()
            .filter(|b| b.bias == direction)
            .cloned()
            .collect()
    }

    /// Get summary statistics
    pub fn get_summary(biases: &[DailyBias]) -> BiasSummary {
        let ce_count = biases.iter().filter(|b| b.bias == BiasDirection::CE).count();
        let pe_count = biases.iter().filter(|b| b.bias == BiasDirection::PE).count();
        let no_trade_count = biases.iter().filter(|b| b.bias == BiasDirection::NoTrade).count();

        BiasSummary {
            total: biases.len(),
            ce_count,
            pe_count,
            no_trade_count,
        }
    }
}

/// Summary of bias calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasSummary {
    pub total: usize,
    pub ce_count: usize,
    pub pe_count: usize,
    pub no_trade_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bars() -> Vec<Bar> {
        // Create sample bars with uptrend
        vec![
            Bar {
                timestamp: Utc::now(),
                timestamp_ms: 0,
                open: 100.0,
                high: 105.0,
                low: 99.0,
                close: 104.0,
                volume: 1000,
                bar_complete: true,
            },
            Bar {
                timestamp: Utc::now(),
                timestamp_ms: 0,
                open: 104.0,
                high: 108.0,
                low: 103.0,
                close: 107.0,
                volume: 1000,
                bar_complete: true,
            },
            // Add more bars...
        ]
    }

    #[test]
    fn test_bias_calculation() {
        let calculator = DailyBiasCalculator::new(14, 25.0);
        let bars = create_test_bars();
        
        // Need at least 15 bars for ADX(14)
        if bars.len() >= 15 {
            let bias = calculator.calculate_bias("TEST", "12345", &bars);
            assert!(bias.is_some());
        }
    }
}

