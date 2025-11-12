/// Standalone utility to extract all F&O stocks and their options
/// Usage: cargo run --bin extract_fno_stocks --release
/// 
/// This tool will:
/// 1. Download instrument master from Angel One
/// 2. Identify all F&O stocks (stocks with futures/options)
/// 3. Extract futures and options for each stock
/// 4. Export comprehensive list to JSON file

use rustro::broker::{AngelOneClient, InstrumentCache, TokenExtractor};
use rustro::config::load_config;
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

    info!("📋 F&O Stocks Extraction Utility");
    info!("=================================");
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
    info!("   This may take a minute...");
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
    info!("✅ Found {} F&O stocks in total", all_fno_stocks.len());
    info!("");

    // Display first 20 stocks
    info!("📊 Sample F&O Stocks (first 20):");
    for (idx, stock) in all_fno_stocks.iter().take(20).enumerate() {
        info!("   [{}] {}", idx + 1, stock);
    }
    if all_fno_stocks.len() > 20 {
        info!("   ... and {} more", all_fno_stocks.len() - 20);
    }
    info!("");

    // Get popular F&O stocks
    let popular_stocks = extractor.get_popular_fno_stocks();
    info!("⭐ Popular F&O Stocks ({}):", popular_stocks.len());
    for (idx, stock) in popular_stocks.iter().enumerate() {
        info!("   [{}] {}", idx + 1, stock);
    }
    info!("");

    // Extract tokens for popular stocks
    info!("🎯 Extracting detailed information for popular F&O stocks...");
    info!("   This will take a few moments...");
    info!("");

    let popular_tokens = extractor.extract_popular_fno_stock_tokens();

    // Display detailed information for each popular stock
    info!("📈 Detailed Information:");
    info!("========================");
    info!("");

    for (stock, tokens) in popular_tokens.iter() {
        info!("📊 {}:", stock);
        
        // Spot token
        if let Some(token) = &tokens.spot_token {
            info!("   ✅ Spot Token: {}", token);
            if let Some(symbol) = &tokens.spot_symbol {
                info!("      Symbol: {}", symbol);
            }
        } else {
            info!("   ❌ Spot Token: NOT FOUND");
        }

        // Futures
        info!("   📈 Futures: {}", tokens.futures.len());
        if !tokens.futures.is_empty() {
            for (idx, future) in tokens.futures.iter().take(2).enumerate() {
                info!("      [{}] {} (token: {}, expiry: {}, lot: {})",
                      idx + 1, future.symbol, future.token, future.expiry, future.lot_size);
            }
            if tokens.futures.len() > 2 {
                info!("      ... and {} more", tokens.futures.len() - 2);
            }
        }

        // Options
        info!("   🎯 Options: {}", tokens.options.len());
        if !tokens.options.is_empty() {
            let ce_count = tokens.options.iter().filter(|o| o.option_type == "CE").count();
            let pe_count = tokens.options.iter().filter(|o| o.option_type == "PE").count();
            info!("      CE: {}, PE: {}", ce_count, pe_count);

            // Show strike range
            if let (Some(min_strike), Some(max_strike)) = (
                tokens.options.iter().map(|o| o.strike as i32).min(),
                tokens.options.iter().map(|o| o.strike as i32).max(),
            ) {
                info!("      Strike range: {} to {}", min_strike, max_strike);
            }

            // Show sample options
            info!("      Sample options:");
            for (idx, option) in tokens.options.iter().take(4).enumerate() {
                info!("         [{}] {} (token: {}, strike: {}, {})",
                      idx + 1, option.symbol, option.token, option.strike, option.option_type);
            }
            if tokens.options.len() > 4 {
                info!("         ... and {} more", tokens.options.len() - 4);
            }
        }

        info!("");
    }

    // Export all F&O stocks list
    info!("💾 Exporting F&O stocks list...");
    tokio::fs::create_dir_all("data").await.ok();
    
    let all_stocks_json = serde_json::to_string_pretty(&all_fno_stocks)?;
    tokio::fs::write("data/all_fno_stocks.json", all_stocks_json).await?;
    info!("✅ All F&O stocks list saved to: data/all_fno_stocks.json");

    // Export popular stocks with detailed tokens
    let popular_tokens_json = serde_json::to_string_pretty(&popular_tokens)?;
    tokio::fs::write("data/popular_fno_stocks_tokens.json", popular_tokens_json).await?;
    info!("✅ Popular F&O stocks tokens saved to: data/popular_fno_stocks_tokens.json");

    // Option 1: Extract ALL F&O stocks (this will take time)
    info!("");
    info!("❓ Do you want to extract ALL {} F&O stocks?", all_fno_stocks.len());
    info!("   This will take 5-10 minutes and create a large JSON file.");
    info!("   To extract all, run: cargo run --bin extract_all_fno_stocks --release");
    info!("");

    // Summary
    info!("✅ F&O Stock Extraction Complete!");
    info!("");
    info!("📊 Summary:");
    info!("   Total F&O stocks: {}", all_fno_stocks.len());
    info!("   Popular stocks extracted: {}", popular_tokens.len());
    info!("");
    info!("📁 Files Created:");
    info!("   • data/all_fno_stocks.json - List of all F&O stocks");
    info!("   • data/popular_fno_stocks_tokens.json - Detailed tokens for popular stocks");
    info!("");
    info!("💡 Next Steps:");
    info!("   1. Review the popular stocks tokens file");
    info!("   2. Add your preferred stocks to config");
    info!("   3. Use TokenExtractor to get options for any stock");
    info!("");
    info!("📖 Example Usage:");
    info!("   let tokens = extractor.extract_asset_tokens(\"RELIANCE\");");
    info!("   let options = extractor.get_atm_options(\"RELIANCE\", 2500.0, 50, 5);");

    Ok(())
}

