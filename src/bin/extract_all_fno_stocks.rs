/// Utility to extract ALL F&O stocks with full details
/// Usage: cargo run --bin extract_all_fno_stocks --release
/// 
/// WARNING: This will take 5-10 minutes and create a large JSON file
/// Only run this if you need the complete list of all F&O stocks

use rustro::broker::{AngelOneClient, InstrumentCache, TokenExtractor};
use rustro::config::load_config;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("🚨 COMPLETE F&O Stocks Extraction Utility");
    info!("==========================================");
    info!("");
    warn!("⚠️  WARNING: This will extract ALL F&O stocks");
    warn!("⚠️  Estimated time: 5-10 minutes");
    warn!("⚠️  Output file size: ~50-100 MB");
    info!("");

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

    // Initialize instrument cache and download master
    info!("");
    info!("📥 Downloading instrument master from Angel One...");
    let instrument_cache = Arc::new(InstrumentCache::new(broker.clone()));
    instrument_cache.refresh().await?;
    
    let total_instruments = instrument_cache.size().await;
    info!("✅ Downloaded {} instruments", total_instruments);
    info!("");

    // Get all instruments
    let instruments = instrument_cache.get_all_instruments().await;

    // Create token extractor
    let extractor = TokenExtractor::new(instruments);

    // Get list of all F&O stocks
    info!("🔍 Identifying F&O stocks...");
    let all_fno_stocks = extractor.get_all_fno_stocks();
    info!("✅ Found {} F&O stocks", all_fno_stocks.len());
    info!("");

    // Extract tokens for ALL stocks
    info!("🎯 Extracting detailed information for ALL {} F&O stocks...", all_fno_stocks.len());
    info!("   This will take several minutes. Please be patient...");
    info!("");

    let start_time = std::time::Instant::now();
    let all_tokens = extractor.extract_all_fno_stock_tokens();
    let elapsed = start_time.elapsed();

    info!("");
    info!("✅ Extraction complete in {:.1} seconds", elapsed.as_secs_f64());
    info!("");

    // Calculate statistics
    let mut total_futures = 0;
    let mut total_options = 0;
    let mut stocks_with_spot = 0;
    let mut stocks_with_futures = 0;
    let mut stocks_with_options = 0;

    for tokens in all_tokens.values() {
        if tokens.spot_token.is_some() {
            stocks_with_spot += 1;
        }
        if !tokens.futures.is_empty() {
            stocks_with_futures += 1;
            total_futures += tokens.futures.len();
        }
        if !tokens.options.is_empty() {
            stocks_with_options += 1;
            total_options += tokens.options.len();
        }
    }

    info!("📊 Statistics:");
    info!("   Total F&O stocks: {}", all_tokens.len());
    info!("   Stocks with spot token: {}", stocks_with_spot);
    info!("   Stocks with futures: {}", stocks_with_futures);
    info!("   Stocks with options: {}", stocks_with_options);
    info!("   Total futures contracts: {}", total_futures);
    info!("   Total option contracts: {}", total_options);
    info!("");

    // Show top 10 stocks by option count
    let mut stocks_by_options: Vec<_> = all_tokens.iter()
        .map(|(name, tokens)| (name, tokens.options.len()))
        .collect();
    stocks_by_options.sort_by(|a, b| b.1.cmp(&a.1));

    info!("🏆 Top 10 stocks by option count:");
    for (idx, (stock, count)) in stocks_by_options.iter().take(10).enumerate() {
        info!("   [{}] {}: {} options", idx + 1, stock, count);
    }
    info!("");

    // Export to JSON
    info!("💾 Exporting to JSON file...");
    tokio::fs::create_dir_all("data").await.ok();
    
    let json = serde_json::to_string_pretty(&all_tokens)?;
    let file_size_mb = json.len() as f64 / 1024.0 / 1024.0;
    
    tokio::fs::write("data/all_fno_stocks_complete.json", json).await?;
    info!("✅ Complete F&O stocks data saved to: data/all_fno_stocks_complete.json");
    info!("   File size: {:.2} MB", file_size_mb);
    info!("");

    // Also save a summary (just stock names and counts)
    #[derive(serde::Serialize)]
    struct StockSummary {
        stock: String,
        has_spot: bool,
        futures_count: usize,
        options_count: usize,
        ce_count: usize,
        pe_count: usize,
    }

    let summary: Vec<StockSummary> = all_tokens.iter()
        .map(|(name, tokens)| {
            let ce_count = tokens.options.iter().filter(|o| o.option_type == "CE").count();
            let pe_count = tokens.options.iter().filter(|o| o.option_type == "PE").count();
            
            StockSummary {
                stock: name.clone(),
                has_spot: tokens.spot_token.is_some(),
                futures_count: tokens.futures.len(),
                options_count: tokens.options.len(),
                ce_count,
                pe_count,
            }
        })
        .collect();

    let summary_json = serde_json::to_string_pretty(&summary)?;
    tokio::fs::write("data/all_fno_stocks_summary.json", summary_json).await?;
    info!("✅ Summary saved to: data/all_fno_stocks_summary.json");
    info!("");

    info!("✅ Complete F&O Stock Extraction Finished!");
    info!("");
    info!("📁 Files Created:");
    info!("   • data/all_fno_stocks_complete.json - Full details ({:.2} MB)", file_size_mb);
    info!("   • data/all_fno_stocks_summary.json - Summary with counts");
    info!("");
    info!("💡 Usage:");
    info!("   1. Review the summary file for overview");
    info!("   2. Search the complete file for specific stocks");
    info!("   3. Use TokenExtractor in your code for dynamic extraction");

    Ok(())
}

