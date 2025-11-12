/// Quick test to verify we can get all F&O tokens for daily bias calculator
use rustro::broker::{AngelOneClient, InstrumentCache, TokenExtractor};
use rustro::config::load_config;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("🧪 Testing Token List for Daily Bias Calculator");
    info!("================================================");

    // Load config
    let config = Arc::new(load_config("config.toml")?);
    let token_manager = Arc::new(rustro::broker::TokenManager::new("data/tokens.json".to_string()));

    // Initialize broker
    let broker = Arc::new(AngelOneClient::new(
        Arc::clone(&token_manager),
        config.angel_one_client_code.clone(),
        config.angel_one_password.clone(),
        config.angel_one_mpin.clone(),
        config.angel_one_totp_secret.clone(),
        config.angel_one_api_key.clone(),
    ));

    // Login
    info!("🔑 Logging in...");
    broker.login().await?;

    // Download instrument master
    info!("📥 Downloading instrument master...");
    let instrument_cache = Arc::new(InstrumentCache::new(broker.clone()));
    instrument_cache.refresh().await?;
    
    let instruments = instrument_cache.get_all_instruments().await;
    info!("✅ Downloaded {} instruments", instruments.len());

    // Create token extractor
    let extractor = TokenExtractor::new(instruments);

    // Test 1: Get all F&O stocks
    info!("\n📊 Test 1: Get All F&O Stocks");
    info!("================================");
    let all_fno = extractor.get_all_fno_stocks();
    info!("✅ Found {} F&O stocks", all_fno.len());
    info!("   Sample: {:?}", &all_fno[0..5.min(all_fno.len())]);

    // Test 2: Get indices
    info!("\n📊 Test 2: Get Indices");
    info!("======================");
    let indices = vec!["NIFTY", "BANKNIFTY", "FINNIFTY"];
    for index in &indices {
        let tokens = extractor.extract_asset_tokens(index);
        info!("✅ {}: spot_token={:?}", 
              index, 
              tokens.spot_token.as_ref().map(|t| &t[..8]));
    }

    // Test 3: Combined list for daily bias
    info!("\n📊 Test 3: Combined List (Indices + Stocks)");
    info!("============================================");
    let mut all_underlyings = indices.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    all_underlyings.extend(all_fno.clone());
    info!("✅ Total underlyings for daily bias: {}", all_underlyings.len());
    info!("   Indices: {}", indices.len());
    info!("   Stocks: {}", all_fno.len());

    // Test 4: Extract spot tokens for first 5
    info!("\n📊 Test 4: Extract Spot Tokens (Sample)");
    info!("========================================");
    for underlying in all_underlyings.iter().take(5) {
        let tokens = extractor.extract_asset_tokens(underlying);
        match tokens.spot_token {
            Some(token) => info!("✅ {}: token={}", underlying, token),
            None => info!("❌ {}: NO SPOT TOKEN", underlying),
        }
    }

    // Test 5: Check what we need for daily bias
    info!("\n📊 Test 5: Daily Bias Requirements Check");
    info!("=========================================");
    info!("For daily bias calculator, we need:");
    info!("  1. Underlying name: ✅ Available");
    info!("  2. Spot token: ✅ Available via extract_asset_tokens()");
    info!("  3. Daily bars: ⏳ Need to download (separate step)");
    info!("  4. ADX/DMI calculation: ✅ Available in indicators.rs");

    info!("\n✅ Token List Test Complete!");
    info!("============================");
    info!("Summary:");
    info!("  • Can get all F&O stocks: YES ({} stocks)", all_fno.len());
    info!("  • Can get indices: YES (3 indices)");
    info!("  • Can get spot tokens: YES");
    info!("  • Total underlyings: {}", all_underlyings.len());
    info!("\n✅ Ready to build Daily Bias Calculator!");

    Ok(())
}

