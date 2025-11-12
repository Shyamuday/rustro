/// Standalone utility to sync historical data for all assets
/// Usage: cargo run --bin sync_multi_asset --release

use rustro::broker::{AngelOneClient, InstrumentCache};
use rustro::config::load_config;
use rustro::data::{
    ConcurrentBarStore, FilterConfig, ExpiryFilter, MultiAssetHistoricalSync, UnderlyingAsset,
};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("🚀 Multi-Asset Historical Data Sync Utility");
    info!("============================================");

    // Load configuration
    info!("📋 Loading configuration...");
    let config = Arc::new(load_config("config.toml")?);

    // Initialize token manager
    let token_manager = Arc::new(rustro::broker::TokenManager::new("data/tokens.json".to_string()));

    // Initialize broker client
    info!("🔐 Initializing broker client...");
    let broker = Arc::new(AngelOneClient::new(
        Arc::clone(&token_manager),
        config.angel_one_client_code.clone(),
        config.angel_one_password.clone(),
        config.angel_one_mpin.clone(),
        config.angel_one_totp_secret.clone(),
        config.angel_one_api_key.clone(),
    ));

    // Login to broker
    info!("🔑 Logging in to Angel One...");
    match broker.login().await {
        Ok(_) => info!("✅ Login successful"),
        Err(e) => {
            error!("❌ Login failed: {}", e);
            return Err(e.into());
        }
    }

    // Initialize instrument cache
    info!("📥 Downloading instrument master...");
    let instrument_cache = Arc::new(InstrumentCache::new(broker.clone()));
    instrument_cache.refresh().await?;
    info!("✅ Cached {} instruments", instrument_cache.size().await);

    // Create bar stores for each asset
    let mut syncer = MultiAssetHistoricalSync::new(broker.clone(), instrument_cache.clone(), config.clone());

    // Register bar stores for underlying indices
    for asset in UnderlyingAsset::all() {
        let asset_name = asset.as_str();
        
        // Daily store
        let daily_file = std::path::PathBuf::from(format!("data/bars/{}_daily.jsonl", asset_name.to_lowercase()));
        let daily_store = Arc::new(ConcurrentBarStore::new(
            asset_name.to_string(),
            "1D".to_string(),
            daily_file,
            10000,
        ));
        syncer.register_bar_store(asset_name.to_string(), daily_store);

        // Hourly store
        let hourly_file = std::path::PathBuf::from(format!("data/bars/{}_hourly.jsonl", asset_name.to_lowercase()));
        let hourly_store = Arc::new(ConcurrentBarStore::new(
            asset_name.to_string(),
            "1H".to_string(),
            hourly_file,
            10000,
        ));
        syncer.register_bar_store(format!("{}_hourly", asset_name), hourly_store);
    }

    // Configure filter
    let filter_config = FilterConfig {
        include_spot: true,
        include_futures: false, // Set to true if you want futures
        include_options: true,
        strike_range: 200, // ±200 points from ATM
        max_strikes_per_side: 9, // 9 strikes per side (CE/PE)
        expiry_filter: ExpiryFilter::NearestWeekly, // Only nearest weekly expiry
    };

    syncer = syncer.with_filter_config(filter_config);

    // Sync all assets
    info!("");
    info!("🔄 Starting multi-asset synchronization...");
    info!("   This may take several minutes depending on API rate limits");
    info!("");

    match syncer.sync_all_assets().await {
        Ok(report) => {
            info!("");
            info!("✅ Multi-asset sync completed successfully!");
            info!("============================================");
            info!("📊 Summary:");
            info!("   Duration: {}s", report.duration_sec);
            info!("   Total instruments: {}", report.total_instruments);
            info!("   Total bars downloaded: {}", report.total_bars_downloaded);
            info!("   Success rate: {:.1}%", report.success_rate);
            info!("");

            // Per-asset details
            for asset_report in &report.assets_synced {
                info!("📈 {}:", asset_report.asset);
                info!("   Underlying bars: {}", asset_report.underlying_bars);
                info!("   Futures synced: {}", asset_report.futures_synced);
                info!("   Options synced: {}", asset_report.options_synced);
                info!("   Daily bars: {}", asset_report.total_daily_bars);
                info!("   Hourly bars: {}", asset_report.total_hourly_bars);
                info!("   Strikes covered: {:?}", asset_report.strikes_covered);
                
                if !asset_report.errors.is_empty() {
                    info!("   ⚠️  Errors: {}", asset_report.errors.len());
                    for err in &asset_report.errors {
                        info!("      - {}", err);
                    }
                }
                info!("");
            }

            info!("💾 Detailed report saved to: data/bars/multi_asset_sync_report_*.json");
        }
        Err(e) => {
            error!("❌ Multi-asset sync failed: {}", e);
            return Err(e.into());
        }
    }

    info!("✅ All done! Historical data is ready for backtesting and live trading.");

    Ok(())
}

