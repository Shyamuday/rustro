/// Multi-asset historical data synchronization module
/// Downloads data for NIFTY, BANKNIFTY, FINNIFTY + their option strikes
/// Supports futures and individual stock options as well

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::broker::{AngelOneClient, InstrumentCache, TokenExtractor};
use crate::data::ConcurrentBarStore;
use crate::error::Result;
use crate::types::Instrument;
use crate::Config;

/// Supported underlying assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnderlyingAsset {
    Nifty,
    BankNifty,
    FinNifty,
}

impl UnderlyingAsset {
    pub fn as_str(&self) -> &str {
        match self {
            UnderlyingAsset::Nifty => "NIFTY",
            UnderlyingAsset::BankNifty => "BANKNIFTY",
            UnderlyingAsset::FinNifty => "FINNIFTY",
        }
    }

    pub fn strike_increment(&self) -> i32 {
        match self {
            UnderlyingAsset::Nifty => 50,
            UnderlyingAsset::BankNifty => 100,
            UnderlyingAsset::FinNifty => 50,
        }
    }

    pub fn default_price(&self) -> f64 {
        match self {
            UnderlyingAsset::Nifty => 23500.0,
            UnderlyingAsset::BankNifty => 49000.0,
            UnderlyingAsset::FinNifty => 22000.0,
        }
    }

    pub fn lot_size(&self) -> i32 {
        match self {
            UnderlyingAsset::Nifty => 50,
            UnderlyingAsset::BankNifty => 15,
            UnderlyingAsset::FinNifty => 40,
        }
    }

    pub fn all() -> Vec<UnderlyingAsset> {
        vec![
            UnderlyingAsset::Nifty,
            UnderlyingAsset::BankNifty,
            UnderlyingAsset::FinNifty,
        ]
    }
}

/// Instrument filter configuration
#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub include_spot: bool,
    pub include_futures: bool,
    pub include_options: bool,
    pub strike_range: i32,
    pub max_strikes_per_side: usize,
    pub expiry_filter: ExpiryFilter,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            include_spot: true,
            include_futures: false,
            include_options: true,
            strike_range: 200,
            max_strikes_per_side: 9,
            expiry_filter: ExpiryFilter::NearestWeekly,
        }
    }
}

/// Expiry filtering options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpiryFilter {
    NearestWeekly,
    NearestMonthly,
    AllActive,
    Specific(NaiveDate),
}

/// Comprehensive sync report for all assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAssetSyncReport {
    pub timestamp: DateTime<Utc>,
    pub duration_sec: i64,
    pub assets_synced: Vec<AssetSyncReport>,
    pub total_instruments: usize,
    pub total_bars_downloaded: usize,
    pub total_errors: usize,
    pub success_rate: f64,
}

/// Per-asset sync report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSyncReport {
    pub asset: String,
    pub underlying_token: String,
    pub underlying_bars: usize,
    pub futures_synced: usize,
    pub options_synced: usize,
    pub total_daily_bars: usize,
    pub total_hourly_bars: usize,
    pub strikes_covered: Vec<i32>,
    pub errors: Vec<String>,
}

/// Multi-asset historical data synchronizer
pub struct MultiAssetHistoricalSync {
    broker: Arc<AngelOneClient>,
    instrument_cache: Arc<InstrumentCache>,
    bar_stores: HashMap<String, Arc<ConcurrentBarStore>>,
    config: Arc<Config>,
    data_dir: String,
    filter_config: FilterConfig,
}

impl MultiAssetHistoricalSync {
    pub fn new(
        broker: Arc<AngelOneClient>,
        instrument_cache: Arc<InstrumentCache>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            broker,
            instrument_cache,
            bar_stores: HashMap::new(),
            config,
            data_dir: "data/bars".to_string(),
            filter_config: FilterConfig::default(),
        }
    }

    /// Set custom filter configuration
    pub fn with_filter_config(mut self, filter_config: FilterConfig) -> Self {
        self.filter_config = filter_config;
        self
    }

    /// Register a bar store for a specific symbol
    pub fn register_bar_store(&mut self, symbol: String, store: Arc<ConcurrentBarStore>) {
        self.bar_stores.insert(symbol, store);
    }

    /// Sync all configured assets (NIFTY, BANKNIFTY, FINNIFTY)
    pub async fn sync_all_assets(&self) -> Result<MultiAssetSyncReport> {
        let start_time = Utc::now();
        info!("🚀 Starting MULTI-ASSET historical data synchronization");
        info!("   Assets: NIFTY, BANKNIFTY, FINNIFTY");
        info!("   Filter: Spot={}, Futures={}, Options={}", 
              self.filter_config.include_spot,
              self.filter_config.include_futures,
              self.filter_config.include_options);

        // Create data directory
        tokio::fs::create_dir_all(&self.data_dir).await.ok();

        let mut asset_reports = Vec::new();
        let assets = UnderlyingAsset::all();

        for (idx, asset) in assets.iter().enumerate() {
            info!("📊 [{}/{}] Processing {}...", idx + 1, assets.len(), asset.as_str());
            
            match self.sync_single_asset(*asset).await {
                Ok(report) => {
                    info!("✅ {} sync complete: {} instruments, {} bars", 
                          asset.as_str(), 
                          report.options_synced + report.futures_synced + if report.underlying_bars > 0 { 1 } else { 0 },
                          report.total_daily_bars + report.total_hourly_bars);
                    asset_reports.push(report);
                }
                Err(e) => {
                    error!("❌ Failed to sync {}: {}", asset.as_str(), e);
                    // Create error report
                    asset_reports.push(AssetSyncReport {
                        asset: asset.as_str().to_string(),
                        underlying_token: String::new(),
                        underlying_bars: 0,
                        futures_synced: 0,
                        options_synced: 0,
                        total_daily_bars: 0,
                        total_hourly_bars: 0,
                        strikes_covered: Vec::new(),
                        errors: vec![format!("Sync failed: {}", e)],
                    });
                }
            }

            // Rate limiting between assets
            if idx < assets.len() - 1 {
                sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }

        let end_time = Utc::now();
        let duration = (end_time - start_time).num_seconds();

        // Calculate summary statistics
        let total_instruments: usize = asset_reports.iter()
            .map(|r| r.options_synced + r.futures_synced + if r.underlying_bars > 0 { 1 } else { 0 })
            .sum();
        
        let total_bars: usize = asset_reports.iter()
            .map(|r| r.total_daily_bars + r.total_hourly_bars)
            .sum();
        
        let total_errors: usize = asset_reports.iter()
            .map(|r| r.errors.len())
            .sum();

        let success_rate = if total_instruments > 0 {
            ((total_instruments - total_errors) as f64 / total_instruments as f64) * 100.0
        } else {
            0.0
        };

        let report = MultiAssetSyncReport {
            timestamp: end_time,
            duration_sec: duration,
            assets_synced: asset_reports,
            total_instruments,
            total_bars_downloaded: total_bars,
            total_errors,
            success_rate,
        };

        // Save comprehensive report
        self.save_multi_asset_report(&report).await.ok();

        info!("✅ MULTI-ASSET sync complete!");
        info!("   Duration: {}s", duration);
        info!("   Total instruments: {}", total_instruments);
        info!("   Total bars: {}", total_bars);
        info!("   Success rate: {:.1}%", success_rate);

        Ok(report)
    }

    /// Sync a single asset (underlying + futures + options)
    pub async fn sync_single_asset(&self, asset: UnderlyingAsset) -> Result<AssetSyncReport> {
        let asset_name = asset.as_str();
        info!("📥 Syncing {} and derivatives...", asset_name);

        let mut report = AssetSyncReport {
            asset: asset_name.to_string(),
            underlying_token: String::new(),
            underlying_bars: 0,
            futures_synced: 0,
            options_synced: 0,
            total_daily_bars: 0,
            total_hourly_bars: 0,
            strikes_covered: Vec::new(),
            errors: Vec::new(),
        };

        // Step 1: Find underlying token
        let underlying_token = match self.find_underlying_token(asset).await {
            Ok(token) => {
                info!("✅ Found {} token: {}", asset_name, token);
                report.underlying_token = token.clone();
                token
            }
            Err(e) => {
                let err_msg = format!("Failed to find {} token: {}", asset_name, e);
                error!("❌ {}", err_msg);
                report.errors.push(err_msg);
                return Ok(report);
            }
        };

        // Step 2: Sync underlying spot data
        if self.filter_config.include_spot {
            info!("📊 Syncing {} spot data...", asset_name);
            match self.sync_underlying_data(&underlying_token, asset_name).await {
                Ok((daily, hourly)) => {
                    report.underlying_bars = daily + hourly;
                    report.total_daily_bars += daily;
                    report.total_hourly_bars += hourly;
                    info!("✅ Downloaded {} daily + {} hourly bars for {}", daily, hourly, asset_name);
                }
                Err(e) => {
                    let err_msg = format!("Failed to sync {} spot: {}", asset_name, e);
                    warn!("⚠️  {}", err_msg);
                    report.errors.push(err_msg);
                }
            }
        }

        // Step 3: Sync futures (if enabled)
        if self.filter_config.include_futures {
            info!("📈 Syncing {} futures...", asset_name);
            match self.sync_futures(asset).await {
                Ok(count) => {
                    report.futures_synced = count;
                    info!("✅ Synced {} futures contracts", count);
                }
                Err(e) => {
                    let err_msg = format!("Failed to sync {} futures: {}", asset_name, e);
                    warn!("⚠️  {}", err_msg);
                    report.errors.push(err_msg);
                }
            }
        }

        // Step 4: Sync options
        if self.filter_config.include_options {
            info!("🎯 Syncing {} options...", asset_name);
            match self.sync_options(asset).await {
                Ok((count, strikes, daily, hourly)) => {
                    report.options_synced = count;
                    report.strikes_covered = strikes;
                    report.total_daily_bars += daily;
                    report.total_hourly_bars += hourly;
                    info!("✅ Synced {} option contracts across {} strikes", count, report.strikes_covered.len());
                }
                Err(e) => {
                    let err_msg = format!("Failed to sync {} options: {}", asset_name, e);
                    warn!("⚠️  {}", err_msg);
                    report.errors.push(err_msg);
                }
            }
        }

        Ok(report)
    }

    /// Find underlying token for an asset using automatic extraction
    async fn find_underlying_token(&self, asset: UnderlyingAsset) -> Result<String> {
        let instruments = self.instrument_cache.get_all_instruments().await;
        let asset_name = asset.as_str();

        // Use TokenExtractor for intelligent token discovery
        let extractor = TokenExtractor::new(instruments);
        let asset_tokens = extractor.extract_asset_tokens(asset_name);

        asset_tokens.spot_token
            .ok_or_else(|| crate::error::TradingError::InstrumentNotFound(
                format!("{} underlying token not found", asset_name)
            ))
    }

    /// Sync underlying spot data
    async fn sync_underlying_data(&self, token: &str, symbol: &str) -> Result<(usize, usize)> {
        let to_date = Utc::now();
        
        // Download daily bars (last 365 days)
        let from_daily = to_date - Duration::days(365);
        let daily_bars = self.broker.get_candles(token, "ONE_DAY", from_daily, to_date).await?;
        let daily_count = daily_bars.len();
        
        // Store if we have a registered store
        if let Some(store) = self.bar_stores.get(symbol) {
            for bar in daily_bars {
                store.append(bar).await.ok();
            }
        }

        // Download hourly bars (last 30 days)
        let from_hourly = to_date - Duration::days(30);
        let hourly_bars = self.broker.get_candles(token, "ONE_HOUR", from_hourly, to_date).await?;
        let hourly_count = hourly_bars.len();
        
        if let Some(store) = self.bar_stores.get(&format!("{}_hourly", symbol)) {
            for bar in hourly_bars {
                store.append(bar).await.ok();
            }
        }

        Ok((daily_count, hourly_count))
    }

    /// Sync futures contracts
    async fn sync_futures(&self, asset: UnderlyingAsset) -> Result<usize> {
        let instruments = self.instrument_cache.get_all_instruments().await;
        let asset_name = asset.as_str();

        // Filter futures contracts
        let futures: Vec<&Instrument> = instruments.iter()
            .filter(|i| {
                i.name == asset_name
                && i.exch_seg == "NFO"
                && i.instrument_type == "FUTIDX"
            })
            .collect();

        info!("   Found {} futures contracts for {}", futures.len(), asset_name);

        let mut synced = 0;
        for (idx, future) in futures.iter().enumerate() {
            info!("   [{}/{}] Syncing {} (expiry: {})...", 
                  idx + 1, futures.len(), future.symbol, future.expiry);
            
            match self.sync_derivative_data(&future.token, &future.symbol).await {
                Ok(_) => synced += 1,
                Err(e) => {
                    warn!("⚠️  Failed to sync {}: {}", future.symbol, e);
                }
            }

            // Rate limiting
            if idx < futures.len() - 1 {
                sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        Ok(synced)
    }

    /// Sync options contracts
    async fn sync_options(&self, asset: UnderlyingAsset) -> Result<(usize, Vec<i32>, usize, usize)> {
        let asset_name = asset.as_str();
        
        // Get current price estimate
        let current_price = self.estimate_current_price(asset).await;
        info!("   Estimated current {} price: {:.2}", asset_name, current_price);

        // Calculate ATM strike
        let strike_increment = asset.strike_increment();
        let atm_strike = ((current_price / strike_increment as f64).round() * strike_increment as f64) as i32;
        info!("   ATM strike: {}", atm_strike);

        // Calculate strike range
        let range = self.filter_config.strike_range;
        let min_strike = atm_strike - range;
        let max_strike = atm_strike + range;
        info!("   Strike range: {} to {} (±{})", min_strike, max_strike, range);

        // Get all instruments
        let instruments = self.instrument_cache.get_all_instruments().await;
        
        // Filter relevant options
        let mut options: Vec<Instrument> = instruments
            .into_iter()
            .filter(|inst| {
                // Must be the right underlying
                inst.name == asset_name
                // Must be in NFO (F&O segment)
                && inst.exch_seg == "NFO"
                // Must be an option (OPTIDX type or ends with CE/PE)
                && (inst.instrument_type == "OPTIDX" || inst.symbol.ends_with("CE") || inst.symbol.ends_with("PE"))
                // Must actually end with CE or PE
                && (inst.symbol.ends_with("CE") || inst.symbol.ends_with("PE"))
                // Must be within strike range
                && inst.strike as i32 >= min_strike
                && inst.strike as i32 <= max_strike
            })
            .collect();

        // Apply expiry filter
        options = self.apply_expiry_filter(options).await;

        // Sort by strike and option type
        options.sort_by(|a, b| {
            a.strike.partial_cmp(&b.strike)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.symbol.cmp(&b.symbol))
        });

        // Limit to configured count
        let max_count = self.filter_config.max_strikes_per_side * 2; // CE + PE
        if options.len() > max_count {
            info!("   Limiting to {} options (configured max)", max_count);
            options.truncate(max_count);
        }

        info!("✅ Selected {} option contracts for {}", options.len(), asset_name);

        // Log some examples
        if !options.is_empty() {
            info!("   Sample options:");
            for inst in options.iter().take(5) {
                info!("     - {} (strike: {}, expiry: {}, token: {})", 
                      inst.symbol, inst.strike, inst.expiry, inst.token);
            }
            if options.len() > 5 {
                info!("     ... and {} more", options.len() - 5);
            }
        }

        // Extract unique strikes
        let mut strikes: Vec<i32> = options.iter()
            .map(|o| o.strike as i32)
            .collect();
        strikes.sort();
        strikes.dedup();

        // Sync option data
        let mut synced = 0;
        let mut total_daily = 0;
        let mut total_hourly = 0;

        for (idx, option) in options.iter().enumerate() {
            info!("   [{}/{}] Syncing {} (strike: {}, expiry: {})...", 
                  idx + 1, options.len(), option.symbol, option.strike, option.expiry);
            
            match self.sync_option_data(option).await {
                Ok((daily, hourly)) => {
                    synced += 1;
                    total_daily += daily;
                    total_hourly += hourly;
                }
                Err(e) => {
                    warn!("⚠️  Failed to sync {}: {}", option.symbol, e);
                }
            }

            // Rate limiting
            if idx < options.len() - 1 {
                sleep(tokio::time::Duration::from_millis(300)).await;
            }
        }

        Ok((synced, strikes, total_daily, total_hourly))
    }

    /// Apply expiry filter to options
    async fn apply_expiry_filter(&self, mut options: Vec<Instrument>) -> Vec<Instrument> {
        match self.filter_config.expiry_filter {
            ExpiryFilter::NearestWeekly => {
                // Get the nearest expiry
                if let Some(nearest_expiry) = self.find_nearest_expiry(&options) {
                    options.retain(|o| o.expiry == nearest_expiry);
                    info!("   Filtered to nearest weekly expiry: {}", nearest_expiry);
                }
            }
            ExpiryFilter::NearestMonthly => {
                // Get the nearest monthly expiry (last Thursday of month)
                if let Some(nearest_monthly) = self.find_nearest_monthly_expiry(&options) {
                    options.retain(|o| o.expiry == nearest_monthly);
                    info!("   Filtered to nearest monthly expiry: {}", nearest_monthly);
                }
            }
            ExpiryFilter::Specific(date) => {
                let target_expiry = date.format("%d%b%Y").to_string().to_uppercase();
                options.retain(|o| o.expiry == target_expiry);
                info!("   Filtered to specific expiry: {}", target_expiry);
            }
            ExpiryFilter::AllActive => {
                // Keep all options (no filtering)
                info!("   Keeping all active expiries");
            }
        }

        options
    }

    /// Find nearest expiry date
    fn find_nearest_expiry(&self, options: &[Instrument]) -> Option<String> {
        let now = Utc::now().naive_utc().date();
        
        options.iter()
            .filter_map(|o| {
                NaiveDate::parse_from_str(&o.expiry, "%d%b%Y")
                    .ok()
                    .map(|date| (o.expiry.clone(), date))
            })
            .filter(|(_, date)| *date >= now)
            .min_by_key(|(_, date)| (*date - now).num_days())
            .map(|(expiry, _)| expiry)
    }

    /// Find nearest monthly expiry
    fn find_nearest_monthly_expiry(&self, options: &[Instrument]) -> Option<String> {
        // Monthly expiries are typically the last Thursday of the month
        // For simplicity, we'll take the expiry with the longest DTE
        let now = Utc::now().naive_utc().date();
        
        options.iter()
            .filter_map(|o| {
                NaiveDate::parse_from_str(&o.expiry, "%d%b%Y")
                    .ok()
                    .map(|date| (o.expiry.clone(), date))
            })
            .filter(|(_, date)| *date >= now)
            .max_by_key(|(_, date)| (*date - now).num_days())
            .map(|(expiry, _)| expiry)
    }

    /// Estimate current price for an asset
    async fn estimate_current_price(&self, asset: UnderlyingAsset) -> f64 {
        let asset_name = asset.as_str();
        
        // Try to get last bar from registered store
        if let Some(store) = self.bar_stores.get(asset_name) {
            if let Some(last_bar) = store.get_last().await {
                return last_bar.close;
            }
        }

        // Fallback to default price
        asset.default_price()
    }

    /// Sync derivative (futures/options) data
    async fn sync_derivative_data(&self, token: &str, symbol: &str) -> Result<(usize, usize)> {
        let to_date = Utc::now();
        
        // For derivatives, download last 60 days of daily data
        let from_daily = to_date - Duration::days(60);
        let daily_bars = match self.broker.get_candles(token, "ONE_DAY", from_daily, to_date).await {
            Ok(bars) => bars,
            Err(_) => Vec::new(),
        };
        let daily_count = daily_bars.len();

        // Download last 14 days of hourly data
        let from_hourly = to_date - Duration::days(14);
        let hourly_bars = match self.broker.get_candles(token, "ONE_HOUR", from_hourly, to_date).await {
            Ok(bars) => bars,
            Err(_) => Vec::new(),
        };
        let hourly_count = hourly_bars.len();

        // Store if we have a registered store for this symbol
        if let Some(store) = self.bar_stores.get(symbol) {
            for bar in daily_bars {
                store.append(bar).await.ok();
            }
        }

        if let Some(store) = self.bar_stores.get(&format!("{}_hourly", symbol)) {
            for bar in hourly_bars {
                store.append(bar).await.ok();
            }
        }

        Ok((daily_count, hourly_count))
    }

    /// Sync option data (shorter history)
    async fn sync_option_data(&self, instrument: &Instrument) -> Result<(usize, usize)> {
        let to_date = Utc::now();
        
        // For options, download last 30 days of daily data
        let from_daily = to_date - Duration::days(30);
        let daily_bars = match self.broker.get_candles(&instrument.token, "ONE_DAY", from_daily, to_date).await {
            Ok(bars) => bars,
            Err(_) => Vec::new(),
        };
        let daily_count = daily_bars.len();

        // Download last 7 days of hourly data
        let from_hourly = to_date - Duration::days(7);
        let hourly_bars = match self.broker.get_candles(&instrument.token, "ONE_HOUR", from_hourly, to_date).await {
            Ok(bars) => bars,
            Err(_) => Vec::new(),
        };
        let hourly_count = hourly_bars.len();

        // Store if we have a registered store for this option
        if let Some(store) = self.bar_stores.get(&instrument.symbol) {
            for bar in daily_bars {
                store.append(bar).await.ok();
            }
        }

        if let Some(store) = self.bar_stores.get(&format!("{}_hourly", instrument.symbol)) {
            for bar in hourly_bars {
                store.append(bar).await.ok();
            }
        }

        Ok((daily_count, hourly_count))
    }

    /// Save multi-asset sync report
    async fn save_multi_asset_report(&self, report: &MultiAssetSyncReport) -> Result<()> {
        let filename = format!("{}/multi_asset_sync_report_{}.json", 
                              self.data_dir, 
                              report.timestamp.format("%Y%m%d_%H%M%S"));
        
        let json = serde_json::to_string_pretty(report)?;
        tokio::fs::write(&filename, json).await?;

        info!("💾 Saved multi-asset sync report to {}", filename);
        Ok(())
    }

    /// Get sync summary for a specific asset
    pub async fn get_asset_summary(&self, asset: UnderlyingAsset) -> String {
        let asset_name = asset.as_str();
        
        let mut summary = format!("📊 {} Data Summary:\n", asset_name);
        
        if let Some(store) = self.bar_stores.get(asset_name) {
            let count = store.memory_count().await;
            summary.push_str(&format!("   Underlying bars: {}\n", count));
        }
        
        summary.push_str(&format!("   Strike increment: {}\n", asset.strike_increment()));
        summary.push_str(&format!("   Lot size: {}\n", asset.lot_size()));
        
        summary
    }
}

