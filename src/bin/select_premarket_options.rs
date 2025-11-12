/// Select pre-market ATM options based on daily bias
/// Usage: cargo run --release --bin select_premarket_options

use rustro::broker::{AngelOneClient, InstrumentCache, TokenExtractor};
use rustro::config::load_config;
use rustro::strategy::DailyBias;
use rustro::trading::PremarketSelector;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("🎯 Pre-Market ATM Option Selector");
    info!("==================================");

    // Step 1: Load daily bias results
    info!("\n📋 Step 1: Loading daily bias results...");
    let bias_json = tokio::fs::read_to_string("data/daily_bias_results.json").await?;
    let biases: Vec<DailyBias> = serde_json::from_str(&bias_json)?;
    info!("✅ Loaded {} bias results", biases.len());

    // Filter tradeable (CE or PE)
    let tradeable: Vec<_> = biases
        .iter()
        .filter(|b| b.bias != rustro::strategy::BiasDirection::NoTrade)
        .cloned()
        .collect();
    info!("   Tradeable: {} (CE + PE)", tradeable.len());

    // Step 2: Login and get instrument master
    info!("\n🔑 Step 2: Logging in to Angel One...");
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

    broker.login().await?;
    info!("✅ Login successful");

    info!("\n📥 Step 3: Downloading instrument master...");
    let instrument_cache = Arc::new(InstrumentCache::new(broker.clone()));
    instrument_cache.refresh().await?;
    
    let instruments = instrument_cache.get_all_instruments().await;
    info!("✅ Downloaded {} instruments", instruments.len());

    // Step 4: Create token extractor and selector
    info!("\n🎯 Step 4: Selecting ATM options...");
    let extractor = Arc::new(TokenExtractor::new(instruments));
    let selector = PremarketSelector::new(extractor);

    let preselected = selector.select_all_premarket_options(&tradeable);

    // Step 5: Display results
    info!("\n📊 Step 5: Pre-Selected Options");
    info!("================================");

    // Group by bias
    let ce_options: Vec<_> = preselected
        .iter()
        .filter(|o| o.bias == rustro::strategy::BiasDirection::CE)
        .collect();
    
    let pe_options: Vec<_> = preselected
        .iter()
        .filter(|o| o.bias == rustro::strategy::BiasDirection::PE)
        .collect();

    info!("\n📈 CE Options (Bullish - Trade Calls):");
    info!("======================================");
    for (idx, opt) in ce_options.iter().take(10).enumerate() {
        if let Some((token, symbol)) = PremarketSelector::get_tradeable_option(opt) {
            info!("  [{}] {} @ {:.2} → Strike: {} | {} (token: {}) | Lot: {} | Exp: {}",
                  idx + 1,
                  opt.underlying,
                  opt.close_price,
                  opt.atm_strike.strike,
                  symbol,
                  &token[..8.min(token.len())],
                  opt.lot_size,
                  opt.expiry);
        }
    }
    if ce_options.len() > 10 {
        info!("  ... and {} more", ce_options.len() - 10);
    }

    info!("\n📉 PE Options (Bearish - Trade Puts):");
    info!("=====================================");
    for (idx, opt) in pe_options.iter().take(10).enumerate() {
        if let Some((token, symbol)) = PremarketSelector::get_tradeable_option(opt) {
            info!("  [{}] {} @ {:.2} → Strike: {} | {} (token: {}) | Lot: {} | Exp: {}",
                  idx + 1,
                  opt.underlying,
                  opt.close_price,
                  opt.atm_strike.strike,
                  symbol,
                  &token[..8.min(token.len())],
                  opt.lot_size,
                  opt.expiry);
        }
    }
    if pe_options.len() > 10 {
        info!("  ... and {} more", pe_options.len() - 10);
    }

    // Step 6: Save to JSON
    info!("\n💾 Step 6: Saving pre-selected options...");
    tokio::fs::create_dir_all("data").await.ok();
    
    let preselected_json = serde_json::to_string_pretty(&preselected)?;
    tokio::fs::write("data/premarket_options.json", &preselected_json).await?;
    info!("✅ Saved to: data/premarket_options.json");

    // Save filtered lists
    let ce_json = serde_json::to_string_pretty(&ce_options)?;
    tokio::fs::write("data/premarket_ce_options.json", &ce_json).await?;
    
    let pe_json = serde_json::to_string_pretty(&pe_options)?;
    tokio::fs::write("data/premarket_pe_options.json", &pe_json).await?;
    
    info!("✅ Saved filtered lists:");
    info!("   - data/premarket_ce_options.json ({} options)", ce_options.len());
    info!("   - data/premarket_pe_options.json ({} options)", pe_options.len());

    // Step 7: Summary
    info!("\n📊 Summary:");
    info!("===========");
    info!("Total pre-selected: {}", preselected.len());
    info!("  CE (Call) options: {}", ce_options.len());
    info!("  PE (Put) options: {}", pe_options.len());
    info!("");
    info!("✅ Pre-Market Selection Complete!");
    info!("");
    info!("Next steps:");
    info!("  1. Review: data/premarket_options.json");
    info!("  2. Monitor these options for hourly crossover");
    info!("  3. Enter trade when hourly signal aligns with daily bias");

    Ok(())
}

