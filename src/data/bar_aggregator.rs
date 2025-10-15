/// Bar aggregation from live ticks
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use tracing::{debug, info, warn};

use crate::data::ConcurrentBarStore;
use crate::error::Result;
use crate::events::{Event, EventBus, EventPayload, EventType};
use crate::types::{Bar, Tick};

/// Timeframe for bar aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    OneMinute,
    FiveMinute,
    FifteenMinute,
    OneHour,
    OneDay,
}

impl Timeframe {
    pub fn as_str(&self) -> &str {
        match self {
            Timeframe::OneMinute => "1m",
            Timeframe::FiveMinute => "5m",
            Timeframe::FifteenMinute => "15m",
            Timeframe::OneHour => "1h",
            Timeframe::OneDay => "1d",
        }
    }
    
    pub fn duration_minutes(&self) -> i64 {
        match self {
            Timeframe::OneMinute => 1,
            Timeframe::FiveMinute => 5,
            Timeframe::FifteenMinute => 15,
            Timeframe::OneHour => 60,
            Timeframe::OneDay => 1440, // 24 * 60
        }
    }
    
    /// Get bar boundary timestamp
    pub fn get_bar_boundary(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        let ist = timestamp.with_timezone(&chrono_tz::Asia::Kolkata);
        
        match self {
            Timeframe::OneMinute => {
                chrono_tz::Asia::Kolkata
                    .with_ymd_and_hms(
                        ist.year(),
                        ist.month(),
                        ist.day(),
                        ist.hour(),
                        ist.minute(),
                        0,
                    )
                    .unwrap()
                    .with_timezone(&Utc)
            }
            Timeframe::FiveMinute => {
                let minute = (ist.minute() / 5) * 5;
                chrono_tz::Asia::Kolkata
                    .with_ymd_and_hms(
                        ist.year(),
                        ist.month(),
                        ist.day(),
                        ist.hour(),
                        minute,
                        0,
                    )
                    .unwrap()
                    .with_timezone(&Utc)
            }
            Timeframe::FifteenMinute => {
                let minute = (ist.minute() / 15) * 15;
                chrono_tz::Asia::Kolkata
                    .with_ymd_and_hms(
                        ist.year(),
                        ist.month(),
                        ist.day(),
                        ist.hour(),
                        minute,
                        0,
                    )
                    .unwrap()
                    .with_timezone(&Utc)
            }
            Timeframe::OneHour => {
                chrono_tz::Asia::Kolkata
                    .with_ymd_and_hms(
                        ist.year(),
                        ist.month(),
                        ist.day(),
                        ist.hour(),
                        0,
                        0,
                    )
                    .unwrap()
                    .with_timezone(&Utc)
            }
            Timeframe::OneDay => {
                chrono_tz::Asia::Kolkata
                    .with_ymd_and_hms(
                        ist.year(),
                        ist.month(),
                        ist.day(),
                        0,
                        0,
                        0,
                    )
                    .unwrap()
                    .with_timezone(&Utc)
            }
        }
    }
}

/// Bar in progress (not yet complete)
#[derive(Debug, Clone)]
struct PartialBar {
    timestamp: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
    tick_count: usize,
}

impl PartialBar {
    fn new(timestamp: DateTime<Utc>, price: f64, volume: i64) -> Self {
        PartialBar {
            timestamp,
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            tick_count: 1,
        }
    }
    
    fn update(&mut self, price: f64, volume: i64) {
        self.close = price;
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.volume += volume;
        self.tick_count += 1;
    }
    
    fn to_bar(&self, complete: bool) -> Bar {
        Bar {
            timestamp: self.timestamp,
            timestamp_ms: self.timestamp.timestamp_millis(),
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            bar_complete: complete,
        }
    }
}

/// Bar aggregator for a single symbol and timeframe
pub struct BarAggregator {
    symbol: String,
    timeframe: Timeframe,
    current_bar: Arc<RwLock<Option<PartialBar>>>,
    bar_store: Arc<ConcurrentBarStore>,
    event_bus: Arc<EventBus>,
    last_tick_time: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl BarAggregator {
    pub fn new(
        symbol: String,
        timeframe: Timeframe,
        bar_store: Arc<ConcurrentBarStore>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        BarAggregator {
            symbol,
            timeframe,
            current_bar: Arc::new(RwLock::new(None)),
            bar_store,
            event_bus,
            last_tick_time: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Process incoming tick
    pub async fn process_tick(&self, tick: &Tick) -> Result<()> {
        let bar_boundary = self.timeframe.get_bar_boundary(tick.timestamp);
        
        let mut current = self.current_bar.write().await;
        
        match current.as_mut() {
            Some(bar) => {
                // Check if we've crossed into a new bar period
                if bar.timestamp != bar_boundary {
                    // Finalize current bar
                    let completed_bar = bar.to_bar(true);
                    
                    // Save to store
                    self.bar_store.append(completed_bar.clone()).await?;
                    
                    // Emit BAR_READY event
                    self.event_bus.publish(Event::new(
                        EventType::BarReady,
                        EventPayload::BarReady {
                            symbol: self.symbol.clone(),
                            timeframe: self.timeframe.as_str().to_string(),
                            bar_time: completed_bar.timestamp,
                            bar_complete: true,
                        },
                    )).await?;
                    
                    debug!(
                        "📊 Bar completed: {} {} @ {} - O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{}",
                        self.symbol,
                        self.timeframe.as_str(),
                        completed_bar.timestamp,
                        completed_bar.open,
                        completed_bar.high,
                        completed_bar.low,
                        completed_bar.close,
                        completed_bar.volume
                    );
                    
                    // Start new bar
                    *current = Some(PartialBar::new(bar_boundary, tick.ltp, tick.volume));
                } else {
                    // Update current bar
                    bar.update(tick.ltp, tick.volume);
                }
            }
            None => {
                // Start first bar
                *current = Some(PartialBar::new(bar_boundary, tick.ltp, tick.volume));
                info!(
                    "🆕 Started new bar: {} {} @ {}",
                    self.symbol,
                    self.timeframe.as_str(),
                    bar_boundary
                );
            }
        }
        
        // Update last tick time
        {
            let mut last_time = self.last_tick_time.write().await;
            *last_time = Some(tick.timestamp);
        }
        
        Ok(())
    }
    
    /// Get current partial bar (for monitoring)
    pub async fn get_current_bar(&self) -> Option<Bar> {
        let current = self.current_bar.read().await;
        current.as_ref().map(|b| b.to_bar(false))
    }
    
    /// Force finalize current bar (e.g., at EOD)
    pub async fn finalize_current_bar(&self) -> Result<()> {
        let mut current = self.current_bar.write().await;
        
        if let Some(bar) = current.take() {
            let completed_bar = bar.to_bar(true);
            
            self.bar_store.append(completed_bar.clone()).await?;
            
            self.event_bus.publish(Event::new(
                EventType::BarReady,
                EventPayload::BarReady {
                    symbol: self.symbol.clone(),
                    timeframe: self.timeframe.as_str().to_string(),
                    bar_time: completed_bar.timestamp,
                    bar_complete: true,
                },
            )).await?;
            
            info!(
                "✅ Finalized current bar: {} {} @ {}",
                self.symbol,
                self.timeframe.as_str(),
                completed_bar.timestamp
            );
        }
        
        Ok(())
    }
    
    /// Check for data gaps (no ticks received)
    pub async fn check_data_gap(&self, threshold_seconds: u64) -> bool {
        let last_time = self.last_tick_time.read().await;
        
        match *last_time {
            Some(last) => {
                let elapsed = (Utc::now() - last).num_seconds();
                elapsed > threshold_seconds as i64
            }
            None => true, // No data received yet
        }
    }
}

/// Multi-symbol, multi-timeframe bar aggregator
pub struct MultiBarAggregator {
    aggregators: Arc<RwLock<HashMap<(String, Timeframe), Arc<BarAggregator>>>>,
    event_bus: Arc<EventBus>,
}

impl MultiBarAggregator {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        MultiBarAggregator {
            aggregators: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        }
    }
    
    /// Add aggregator for symbol and timeframe
    pub async fn add_aggregator(
        &self,
        symbol: String,
        timeframe: Timeframe,
        bar_store: Arc<ConcurrentBarStore>,
    ) {
        let aggregator = Arc::new(BarAggregator::new(
            symbol.clone(),
            timeframe,
            bar_store,
            Arc::clone(&self.event_bus),
        ));
        
        let mut aggregators = self.aggregators.write().await;
        aggregators.insert((symbol.clone(), timeframe), aggregator);
        
        info!("➕ Added aggregator: {} {}", symbol, timeframe.as_str());
    }
    
    /// Process tick for all relevant aggregators
    pub async fn process_tick(&self, tick: Tick) -> Result<()> {
        let aggregators = self.aggregators.read().await;
        
        for ((symbol, _timeframe), aggregator) in aggregators.iter() {
            if symbol == &tick.symbol || symbol == &tick.token {
                aggregator.process_tick(&tick).await?;
            }
        }
        
        Ok(())
    }
    
    /// Finalize all current bars (e.g., at EOD)
    pub async fn finalize_all(&self) -> Result<()> {
        let aggregators = self.aggregators.read().await;
        
        for aggregator in aggregators.values() {
            aggregator.finalize_current_bar().await?;
        }
        
        info!("✅ Finalized all bars");
        Ok(())
    }
    
    /// Check for data gaps across all aggregators
    pub async fn check_all_gaps(&self, threshold_seconds: u64) -> Vec<(String, Timeframe)> {
        let aggregators = self.aggregators.read().await;
        let mut gaps = Vec::new();
        
        for ((symbol, timeframe), aggregator) in aggregators.iter() {
            if aggregator.check_data_gap(threshold_seconds).await {
                gaps.push((symbol.clone(), *timeframe));
            }
        }
        
        gaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timeframe_boundary() {
        let time = Utc::now();
        let boundary = Timeframe::FiveMinute.get_bar_boundary(time);
        
        // Should be rounded to 5-minute boundary
        assert_eq!(boundary.minute() % 5, 0);
        assert_eq!(boundary.second(), 0);
    }
    
    #[test]
    fn test_partial_bar_update() {
        let mut bar = PartialBar::new(Utc::now(), 100.0, 1000);
        
        bar.update(102.0, 500);
        assert_eq!(bar.high, 102.0);
        assert_eq!(bar.close, 102.0);
        
        bar.update(98.0, 300);
        assert_eq!(bar.low, 98.0);
        assert_eq!(bar.close, 98.0);
        assert_eq!(bar.volume, 1800);
    }
}

