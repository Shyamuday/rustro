/// Calculate daily bias for all F&O underlyings
/// Usage: cargo run --release --bin calculate_daily_bias

use rustro::broker::{AngelOneClient, InstrumentCache};
use rustro::config::load_config;
use rustro::strategy::{DailyBiasCalculator, DailyBiasToken, BiasDirection};
use rustro::types::Bar;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("📊 Daily Bias Calculator");
    info!("========================");

    // Step 1: Load token list
    info!("\n📋 Step 1: Loading token list...");
    let token_json = tokio::fs::read_to_string("data/daily_bias_tokens.json").await?;
    let tokens: Vec<DailyBiasToken> = serde_json::from_str(&token_json)?;
    info!("✅ Loaded {} underlyings", tokens.len());

    // Step 2: Login to broker
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

    // Step 3: Download daily bars for all tokens
    info!("\n📥 Step 3: Downloading daily bars...");
    info!("   This will take a few minutes for {} underlyings...", tokens.len());
    
    let mut bars_map: HashMap<String, Vec<Bar>> = HashMap::new();
    let to_date = chrono::Utc::now();
    let from_date = to_date - chrono::Duration::days(365);

    for (idx, token) in tokens.iter().enumerate() {
        if idx % 10 == 0 {
            info!("   Progress: {}/{}", idx, tokens.len());
        }

        match broker.get_candles(&token.spot_token, "ONE_DAY", from_date, to_date).await {
            Ok(bars) => {
                if !bars.is_empty() {
                    bars_map.insert(token.spot_token.clone(), bars);
                }
            }
            Err(e) => {
                error!("   Failed to get bars for {}: {}", token.underlying, e);
            }
        }

        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    info!("✅ Downloaded bars for {}/{} underlyings", bars_map.len(), tokens.len());

    // Step 4: Calculate daily bias
    info!("\n🧮 Step 4: Calculating daily bias...");
    let calculator = DailyBiasCalculator::new(
        config.daily_adx_period,
        config.daily_adx_threshold,
    );

    let biases = calculator.calculate_all_bias(&tokens, &bars_map);
    
    // Step 5: Generate summary
    info!("\n📊 Step 5: Summary");
    info!("==================");
    
    let summary = DailyBiasCalculator::get_summary(&biases);
    info!("Total underlyings analyzed: {}", summary.total);
    info!("  CE (Bullish): {}", summary.ce_count);
    info!("  PE (Bearish): {}", summary.pe_count);
    info!("  NO_TRADE (Sideways): {}", summary.no_trade_count);

    // Step 6: Show tradeable underlyings
    info!("\n📈 CE Bias (Bullish - Trade Call Options):");
    info!("==========================================");
    let ce_biases = DailyBiasCalculator::filter_by_bias(&biases, BiasDirection::CE);
    for (idx, bias) in ce_biases.iter().take(10).enumerate() {
        info!("  [{}] {} - ADX: {:.2}, +DI: {:.2}, -DI: {:.2}, Close: {:.2}",
              idx + 1, bias.underlying, bias.adx, bias.plus_di, bias.minus_di, bias.close_price);
    }
    if ce_biases.len() > 10 {
        info!("  ... and {} more", ce_biases.len() - 10);
    }

    info!("\n📉 PE Bias (Bearish - Trade Put Options):");
    info!("==========================================");
    let pe_biases = DailyBiasCalculator::filter_by_bias(&biases, BiasDirection::PE);
    for (idx, bias) in pe_biases.iter().take(10).enumerate() {
        info!("  [{}] {} - ADX: {:.2}, +DI: {:.2}, -DI: {:.2}, Close: {:.2}",
              idx + 1, bias.underlying, bias.adx, bias.plus_di, bias.minus_di, bias.close_price);
    }
    if pe_biases.len() > 10 {
        info!("  ... and {} more", pe_biases.len() - 10);
    }

    // Step 7: Save results to JSON
    info!("\n💾 Step 6: Saving results...");
    tokio::fs::create_dir_all("data").await.ok();
    
    let results_json = serde_json::to_string_pretty(&biases)?;
    tokio::fs::write("data/daily_bias_results.json", &results_json).await?;
    info!("✅ Saved to: data/daily_bias_results.json");

    // Save filtered lists
    let ce_json = serde_json::to_string_pretty(&ce_biases)?;
    tokio::fs::write("data/daily_bias_ce.json", &ce_json).await?;
    
    let pe_json = serde_json::to_string_pretty(&pe_biases)?;
    tokio::fs::write("data/daily_bias_pe.json", &pe_json).await?;
    
    info!("✅ Saved filtered lists:");
    info!("   - data/daily_bias_ce.json ({} underlyings)", ce_biases.len());
    info!("   - data/daily_bias_pe.json ({} underlyings)", pe_biases.len());

    info!("\n✅ Daily Bias Calculation Complete!");
    info!("===================================");
    info!("Next steps:");
    info!("  1. Review: data/daily_bias_results.json");
    info!("  2. Monitor CE underlyings for hourly crossover");
    info!("  3. Monitor PE underlyings for hourly crossover");

    Ok(())
}

