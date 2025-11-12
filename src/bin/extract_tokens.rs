/// Standalone utility to extract and display tokens automatically
/// Usage: cargo run --bin extract_tokens --release
/// 
/// This tool will:
/// 1. Download instrument master from Angel One
/// 2. Automatically identify NIFTY, BANKNIFTY, FINNIFTY tokens
/// 3. Extract all futures and options for each underlying
/// 4. Export token mapping to JSON file for reference

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

    info!("🔍 Automatic Token Extraction Utility");
    info!("=====================================");
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

    // Print summary statistics
    info!("📊 Analyzing instrument master...");
    info!("");
    extractor.print_summary();
    info!("");

    // Extract tokens for all major indices
    info!("🎯 Extracting tokens for major indices...");
    info!("==========================================");
    info!("");

    let indices = vec!["NIFTY", "BANKNIFTY", "FINNIFTY"];

    for index in &indices {
        info!("📈 {}:", index);
        info!("   {}", "=".repeat(index.len() + 3));
        
        let asset_tokens = extractor.extract_asset_tokens(index);

        // Display spot token
        if let Some(token) = &asset_tokens.spot_token {
            info!("   ✅ Spot Token: {}", token);
            if let Some(symbol) = &asset_tokens.spot_symbol {
                info!("      Symbol: {}", symbol);
            }
        } else {
            info!("   ❌ Spot Token: NOT FOUND");
        }

        // Display futures summary
        info!("");
        info!("   📊 Futures Contracts: {}", asset_tokens.futures.len());
        if !asset_tokens.futures.is_empty() {
            for (idx, future) in asset_tokens.futures.iter().take(3).enumerate() {
                info!("      [{}] {} (token: {}, expiry: {}, lot: {})",
                      idx + 1, future.symbol, future.token, future.expiry, future.lot_size);
            }
            if asset_tokens.futures.len() > 3 {
                info!("      ... and {} more", asset_tokens.futures.len() - 3);
            }
        }

        // Display options summary
        info!("");
        info!("   🎯 Option Contracts: {}", asset_tokens.options.len());
        if !asset_tokens.options.is_empty() {
            let ce_count = asset_tokens.options.iter()
                .filter(|o| o.option_type == "CE")
                .count();
            let pe_count = asset_tokens.options.iter()
                .filter(|o| o.option_type == "PE")
                .count();
            
            info!("      CE: {}, PE: {}", ce_count, pe_count);

            // Show strike range
            if let (Some(min_strike), Some(max_strike)) = (
                asset_tokens.options.iter().map(|o| o.strike as i32).min(),
                asset_tokens.options.iter().map(|o| o.strike as i32).max(),
            ) {
                info!("      Strike range: {} to {}", min_strike, max_strike);
            }

            // Show examples
            info!("      Sample options:");
            for (idx, option) in asset_tokens.options.iter().take(5).enumerate() {
                info!("         [{}] {} (token: {}, strike: {}, expiry: {})",
                      idx + 1, option.symbol, option.token, option.strike, option.expiry);
            }
            if asset_tokens.options.len() > 5 {
                info!("         ... and {} more", asset_tokens.options.len() - 5);
            }
        }

        info!("");
    }

    // Demonstrate ATM option extraction
    info!("🎯 Example: Getting ATM options for current market price");
    info!("========================================================");
    info!("");

    let nifty_price = 23500.0; // Example price
    let banknifty_price = 49000.0;
    let finnifty_price = 22000.0;

    for (index, price, increment) in [
        ("NIFTY", nifty_price, 50),
        ("BANKNIFTY", banknifty_price, 100),
        ("FINNIFTY", finnifty_price, 50),
    ] {
        info!("📊 {} at {:.2}:", index, price);
        let atm_options = extractor.get_atm_options(index, price, increment, 3);
        info!("   Found {} ATM options (±3 strikes)", atm_options.len());
        
        for opt in atm_options.iter().take(6) {
            info!("      {} (strike: {}, {})", opt.symbol, opt.strike, opt.option_type);
        }
        info!("");
    }

    // Demonstrate nearest expiry extraction
    info!("📅 Example: Getting nearest expiry options");
    info!("==========================================");
    info!("");

    for index in &indices {
        info!("📈 {} nearest expiry:", index);
        let nearest_options = extractor.get_nearest_expiry_options(index);
        
        if !nearest_options.is_empty() {
            if let Some(expiry) = nearest_options.first().map(|o| &o.expiry) {
                info!("   Expiry: {}", expiry);
                info!("   Options: {}", nearest_options.len());
                
                let ce_count = nearest_options.iter().filter(|o| o.option_type == "CE").count();
                let pe_count = nearest_options.iter().filter(|o| o.option_type == "PE").count();
                info!("   CE: {}, PE: {}", ce_count, pe_count);
            }
        } else {
            info!("   No options found");
        }
        info!("");
    }

    // Export to JSON file
    info!("💾 Exporting token mapping to file...");
    let export_file = "data/extracted_tokens.json";
    tokio::fs::create_dir_all("data").await.ok();
    
    match extractor.export_tokens_to_file(export_file).await {
        Ok(_) => {
            info!("✅ Token mapping exported to: {}", export_file);
            info!("   You can use this file as reference for token IDs");
        }
        Err(e) => {
            error!("❌ Failed to export tokens: {}", e);
        }
    }

    info!("");
    info!("✅ Token extraction complete!");
    info!("");
    info!("💡 Key Takeaways:");
    info!("   • All tokens are automatically identified from instrument master");
    info!("   • No manual token lookup needed");
    info!("   • The system intelligently finds NIFTY, BANKNIFTY, FINNIFTY");
    info!("   • Futures and options are automatically categorized");
    info!("   • Use TokenExtractor in your code for automatic token discovery");

    Ok(())
}

