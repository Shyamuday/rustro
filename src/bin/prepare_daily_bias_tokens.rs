/// Prepare token list specifically for daily bias calculator
/// Output: Lightweight JSON with just spot tokens for all underlyings

use rustro::broker::{AngelOneClient, InstrumentCache, TokenExtractor};
use rustro::config::load_config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DailyBiasToken {
    underlying: String,
    spot_token: String,
    spot_symbol: String,
    asset_type: String,  // "INDEX" or "STOCK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("📊 Preparing Daily Bias Token List");
    info!("===================================");

    // Load config and login
    let config = Arc::new(load_config("config.toml")?);
    let token_manager = Arc::new(rustro::broker::TokenManager::new("data/tokens.json".to_string()));
    
    let broker = Arc::new(AngelOneClient::new(
        Arc::clone(&token_manager),
        config.angel_one_client_code.clone(),
        config.angel_one_password.clone(),
        config.angel_one_mpin.clone(),
        config.angel_one_totp_secret.clone(),
        config.angel_one_api_key.clone(),
    ));

    info!("🔑 Logging in...");
    broker.login().await?;

    info!("📥 Downloading instrument master...");
    let instrument_cache = Arc::new(InstrumentCache::new(broker.clone()));
    instrument_cache.refresh().await?;
    
    let instruments = instrument_cache.get_all_instruments().await;
    info!("✅ Downloaded {} instruments", instruments.len());

    let extractor = TokenExtractor::new(instruments);

    // Collect all underlyings
    let mut daily_bias_tokens = Vec::new();

    // Add indices
    info!("\n📈 Processing Indices...");
    let indices = vec!["NIFTY", "BANKNIFTY", "FINNIFTY"];
    for index in indices {
        let tokens = extractor.extract_asset_tokens(index);
        if let (Some(spot_token), Some(spot_symbol)) = (tokens.spot_token, tokens.spot_symbol) {
            daily_bias_tokens.push(DailyBiasToken {
                underlying: index.to_string(),
                spot_token,
                spot_symbol,
                asset_type: "INDEX".to_string(),
            });
            info!("  ✅ {}", index);
        } else {
            error!("  ❌ {} - No spot token", index);
        }
    }

    // Add F&O stocks
    info!("\n📊 Processing F&O Stocks...");
    let all_stocks = extractor.get_all_fno_stocks();
    info!("  Found {} F&O stocks", all_stocks.len());
    
    let mut processed = 0;
    let mut failed = 0;
    
    for (idx, stock) in all_stocks.iter().enumerate() {
        if idx % 20 == 0 {
            info!("  Progress: {}/{}", idx, all_stocks.len());
        }
        
        let tokens = extractor.extract_asset_tokens(stock);
        if let (Some(spot_token), Some(spot_symbol)) = (tokens.spot_token, tokens.spot_symbol) {
            daily_bias_tokens.push(DailyBiasToken {
                underlying: stock.clone(),
                spot_token,
                spot_symbol,
                asset_type: "STOCK".to_string(),
            });
            processed += 1;
        } else {
            failed += 1;
        }
    }

    info!("  ✅ Processed: {}", processed);
    if failed > 0 {
        info!("  ⚠️  Failed: {} (no spot token)", failed);
    }

    // Save to JSON
    info!("\n💾 Saving to JSON...");
    tokio::fs::create_dir_all("data").await.ok();
    
    let json = serde_json::to_string_pretty(&daily_bias_tokens)?;
    tokio::fs::write("data/daily_bias_tokens.json", &json).await?;
    
    let file_size_kb = json.len() as f64 / 1024.0;
    info!("✅ Saved to: data/daily_bias_tokens.json");
    info!("   File size: {:.1} KB", file_size_kb);

    // Summary
    info!("\n📊 Summary:");
    info!("   Total underlyings: {}", daily_bias_tokens.len());
    info!("   Indices: {}", daily_bias_tokens.iter().filter(|t| t.asset_type == "INDEX").count());
    info!("   Stocks: {}", daily_bias_tokens.iter().filter(|t| t.asset_type == "STOCK").count());

    // Show sample
    info!("\n📋 Sample (first 10):");
    for (idx, token) in daily_bias_tokens.iter().take(10).enumerate() {
        info!("  [{}] {} ({}) - token: {}", 
              idx + 1, 
              token.underlying, 
              token.asset_type,
              &token.spot_token[..8.min(token.spot_token.len())]);
    }

    info!("\n✅ Daily Bias Token List Ready!");
    info!("   Use this file for daily bias calculator");
    info!("   File: data/daily_bias_tokens.json");

    Ok(())
}

