/// ADX-based trading strategy implementation
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{Result, TradingError};
use crate::strategy::indicators::*;
use crate::types::{Bar, Config, Direction, OptionType, Side};

/// Entry signal details
#[derive(Debug, Clone)]
pub struct EntrySignal {
    pub direction: Direction,
    pub underlying_ltp: f64,
    pub strike: i32,
    pub option_type: OptionType,
    pub side: Side,
    pub reason: String,
    pub confidence: f64,
}

/// ADX Strategy state
pub struct AdxStrategy {
    config: Arc<Config>,
    daily_direction: Arc<RwLock<Option<Direction>>>,
    last_daily_analysis: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
    last_hourly_analysis: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl AdxStrategy {
    pub fn new(config: Arc<Config>) -> Self {
        AdxStrategy {
            config,
            daily_direction: Arc::new(RwLock::new(None)),
            last_daily_analysis: Arc::new(RwLock::new(None)),
            last_hourly_analysis: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Analyze daily bars and determine direction
    pub async fn analyze_daily(&self, daily_bars: &[Bar]) -> Result<Direction> {
        info!("Running daily direction analysis");
        
        // Calculate daily ADX
        let (daily_adx, daily_plus_di, daily_minus_di) = calculate_adx(
            daily_bars,
            self.config.daily_adx_period,
        ).ok_or_else(|| TradingError::MissingData("Insufficient bars for daily ADX".to_string()))?;
        
        debug!(
            "Daily ADX: {:.2}, +DI: {:.2}, -DI: {:.2}",
            daily_adx, daily_plus_di, daily_minus_di
        );
        
        // Determine direction based on ADX and DI crossover
        let direction = if daily_adx < self.config.daily_adx_threshold {
            // Weak trend - no trade
            info!("Daily ADX ({:.2}) below threshold ({:.2}) - NO TRADE", 
                  daily_adx, self.config.daily_adx_threshold);
            Direction::NoTrade
        } else if daily_plus_di > daily_minus_di {
            // Strong uptrend - trade Call options
            info!("Daily uptrend confirmed: +DI ({:.2}) > -DI ({:.2}), ADX: {:.2}", 
                  daily_plus_di, daily_minus_di, daily_adx);
            Direction::CE
        } else if daily_minus_di > daily_plus_di {
            // Strong downtrend - trade Put options
            info!("Daily downtrend confirmed: -DI ({:.2}) > +DI ({:.2}), ADX: {:.2}", 
                  daily_minus_di, daily_plus_di, daily_adx);
            Direction::PE
        } else {
            // Neutral
            warn!("Daily direction unclear - NO TRADE");
            Direction::NoTrade
        };
        
        // Update state
        {
            let mut dir = self.daily_direction.write().await;
            *dir = Some(direction);
        }
        {
            let mut last = self.last_daily_analysis.write().await;
            *last = Some(chrono::Utc::now());
        }
        
        Ok(direction)
    }
    
    /// Analyze hourly bars and check alignment
    pub async fn analyze_hourly(&self, hourly_bars: &[Bar]) -> Result<bool> {
        debug!("Running hourly alignment check");
        
        // Get daily direction
        let daily_direction = {
            let dir = self.daily_direction.read().await;
            dir.clone()
        };
        
        if daily_direction.is_none() {
            return Err(TradingError::InvalidStrategyState(
                "Daily direction not set".to_string()
            ));
        }
        
        let daily_direction = daily_direction.unwrap();
        
        if daily_direction == Direction::NoTrade {
            return Ok(false);
        }
        
        // Calculate hourly ADX
        let (hourly_adx, hourly_plus_di, hourly_minus_di) = calculate_adx(
            hourly_bars,
            self.config.hourly_adx_period,
        ).ok_or_else(|| TradingError::MissingData("Insufficient bars for hourly ADX".to_string()))?;
        
        debug!(
            "Hourly ADX: {:.2}, +DI: {:.2}, -DI: {:.2}",
            hourly_adx, hourly_plus_di, hourly_minus_di
        );
        
        // Check alignment
        let aligned = match daily_direction {
            Direction::CE => {
                // For CE direction, hourly must show uptrend
                hourly_adx >= self.config.hourly_adx_threshold
                    && hourly_plus_di > hourly_minus_di
            }
            Direction::PE => {
                // For PE direction, hourly must show downtrend
                hourly_adx >= self.config.hourly_adx_threshold
                    && hourly_minus_di > hourly_plus_di
            }
            Direction::NoTrade => false,
        };
        
        if aligned {
            info!(
                "Hourly alignment confirmed for {} direction (ADX: {:.2})",
                daily_direction.as_str(),
                hourly_adx
            );
        } else {
            debug!(
                "Hourly alignment NOT confirmed for {} direction",
                daily_direction.as_str()
            );
        }
        
        // Update state
        {
            let mut last = self.last_hourly_analysis.write().await;
            *last = Some(chrono::Utc::now());
        }
        
        Ok(aligned)
    }
    
    /// Evaluate entry filters and generate signal
    pub async fn evaluate_entry(
        &self,
        hourly_bars: &[Bar],
        underlying_ltp: f64,
        vix: f64,
    ) -> Result<Option<EntrySignal>> {
        // Get daily direction
        let daily_direction = {
            let dir = self.daily_direction.read().await;
            dir.clone()
        };
        
        if daily_direction.is_none() || daily_direction == Some(Direction::NoTrade) {
            return Ok(None);
        }
        
        let daily_direction = daily_direction.unwrap();
        
        // Filter 1: RSI check
        let rsi = calculate_rsi(hourly_bars, self.config.rsi_period)
            .ok_or_else(|| TradingError::MissingData("Insufficient bars for RSI".to_string()))?;
        
        let rsi_ok = match daily_direction {
            Direction::CE => rsi < self.config.rsi_overbought,
            Direction::PE => rsi > self.config.rsi_oversold,
            Direction::NoTrade => false,
        };
        
        if !rsi_ok {
            debug!("RSI filter failed: RSI = {:.2}", rsi);
            return Ok(None);
        }
        
        // Filter 2: EMA check
        let ema = calculate_ema(hourly_bars, self.config.ema_period)
            .ok_or_else(|| TradingError::MissingData("Insufficient bars for EMA".to_string()))?;
        
        let last_close = hourly_bars.last()
            .ok_or_else(|| TradingError::MissingData("No bars available".to_string()))?
            .close;
        
        let ema_ok = match daily_direction {
            Direction::CE => last_close > ema,
            Direction::PE => last_close < ema,
            Direction::NoTrade => false,
        };
        
        if !ema_ok {
            debug!("EMA filter failed: Close = {:.2}, EMA = {:.2}", last_close, ema);
            return Ok(None);
        }
        
        // Filter 3: VIX check
        if vix > self.config.vix_threshold {
            warn!("VIX too high: {:.2} > {:.2}", vix, self.config.vix_threshold);
            return Ok(None);
        }
        
        // All filters passed - generate signal
        let strike = round_to_strike(underlying_ltp, self.config.strike_increment);
        
        let (option_type, side) = match daily_direction {
            Direction::CE => (OptionType::CE, Side::Buy),
            Direction::PE => (OptionType::PE, Side::Buy),
            Direction::NoTrade => return Ok(None),
        };
        
        let reason = format!(
            "Daily: {}, Hourly aligned, RSI: {:.1}, EMA: {:.1}, VIX: {:.1}",
            daily_direction.as_str(),
            rsi,
            ema,
            vix
        );
        
        let signal = EntrySignal {
            direction: daily_direction,
            underlying_ltp,
            strike,
            option_type,
            side,
            reason,
            confidence: 0.8, // Can be refined based on signal strength
        };
        
        info!("Entry signal generated: {:?} @ strike {}", option_type, strike);
        
        Ok(Some(signal))
    }
    
    /// Check if exit conditions are met (technical)
    pub async fn check_technical_exit(
        &self,
        entry_direction: Direction,
        current_bars: &[Bar],
    ) -> bool {
        // Check if alignment is lost
        if let Ok((_hourly_adx, hourly_plus_di, hourly_minus_di)) = 
            calculate_adx(current_bars, self.config.hourly_adx_period)
                .ok_or_else(|| TradingError::MissingData("Insufficient bars".to_string()))
        {
            let aligned = match entry_direction {
                Direction::CE => hourly_plus_di > hourly_minus_di,
                Direction::PE => hourly_minus_di > hourly_plus_di,
                Direction::NoTrade => false,
            };
            
            if !aligned {
                info!("Technical exit: Alignment lost");
                return true;
            }
        }
        
        false
    }
    
    /// Get current daily direction
    pub async fn get_daily_direction(&self) -> Option<Direction> {
        let dir = self.daily_direction.read().await;
        dir.clone()
    }
    
    /// Reset strategy state (e.g., at EOD)
    pub async fn reset(&self) {
        let mut dir = self.daily_direction.write().await;
        *dir = None;
        
        let mut last_daily = self.last_daily_analysis.write().await;
        *last_daily = None;
        
        let mut last_hourly = self.last_hourly_analysis.write().await;
        *last_hourly = None;
        
        info!("Strategy state reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    fn create_test_config() -> Config {
        // Would load from TOML in real code
        unimplemented!("Create test config")
    }
    
    fn create_trending_bars(count: usize, uptrend: bool) -> Vec<Bar> {
        (0..count)
            .map(|i| {
                let base = 19000.0;
                let trend = if uptrend { i as f64 * 20.0 } else { -(i as f64 * 20.0) };
                Bar {
                    timestamp: Utc::now(),
                    timestamp_ms: Utc::now().timestamp_millis(),
                    open: base + trend,
                    high: base + trend + 50.0,
                    low: base + trend - 50.0,
                    close: base + trend + 25.0,
                    volume: 1000000,
                    bar_complete: true,
                }
            })
            .collect()
    }
}
