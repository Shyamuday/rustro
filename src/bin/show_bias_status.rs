/// Show daily bias status - count CE/PE/NoTrade
use rustro::strategy::{BiasDirection, DailyBias};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📊 Daily Bias Status");
    println!("====================\n");

    // Try to load latest bias data
    let bias_file = "data/daily_bias_latest.json";
    
    if !std::path::Path::new(bias_file).exists() {
        println!("❌ No daily bias data found yet.");
        println!("\n💡 To calculate daily bias, run:");
        println!("   cargo run --release --bin calculate_daily_bias");
        return Ok(());
    }

    // Load bias data
    let bias_json = tokio::fs::read_to_string(bias_file).await?;
    let biases: Vec<DailyBias> = serde_json::from_str(&bias_json)?;

    if biases.is_empty() {
        println!("⚠️  No bias data available");
        return Ok(());
    }

    // Count by bias type
    let mut ce_count = 0;
    let mut pe_count = 0;
    let mut no_trade_count = 0;
    let mut ce_list = Vec::new();
    let mut pe_list = Vec::new();

    for bias in &biases {
        match bias.bias {
            BiasDirection::CE => {
                ce_count += 1;
                ce_list.push(bias);
            }
            BiasDirection::PE => {
                pe_count += 1;
                pe_list.push(bias);
            }
            BiasDirection::NoTrade => {
                no_trade_count += 1;
            }
        }
    }

    // Show summary
    println!("📈 Summary:");
    println!("   Total Underlyings: {}", biases.len());
    println!("   CE (Bullish): {} ({:.1}%)", ce_count, (ce_count as f64 / biases.len() as f64) * 100.0);
    println!("   PE (Bearish): {} ({:.1}%)", pe_count, (pe_count as f64 / biases.len() as f64) * 100.0);
    println!("   NO_TRADE (Sideways): {} ({:.1}%)", no_trade_count, (no_trade_count as f64 / biases.len() as f64) * 100.0);

    // Show CE underlyings
    if !ce_list.is_empty() {
        println!("\n📈 CE Bias (Bullish - Trade Call Options):");
        println!("   Count: {}", ce_count);
        for (idx, bias) in ce_list.iter().take(10).enumerate() {
            println!("   [{}] {} - ADX: {:.2}, +DI: {:.2}, -DI: {:.2}, Close: {:.2}",
                     idx + 1, bias.underlying, bias.adx, bias.plus_di, bias.minus_di, bias.close_price);
        }
        if ce_list.len() > 10 {
            println!("   ... and {} more", ce_list.len() - 10);
        }
    }

    // Show PE underlyings
    if !pe_list.is_empty() {
        println!("\n📉 PE Bias (Bearish - Trade Put Options):");
        println!("   Count: {}", pe_count);
        for (idx, bias) in pe_list.iter().take(10).enumerate() {
            println!("   [{}] {} - ADX: {:.2}, +DI: {:.2}, -DI: {:.2}, Close: {:.2}",
                     idx + 1, bias.underlying, bias.adx, bias.plus_di, bias.minus_di, bias.close_price);
        }
        if pe_list.len() > 10 {
            println!("   ... and {} more", pe_list.len() - 10);
        }
    }

    // Market sentiment
    println!("\n🎯 Market Sentiment:");
    if ce_count > pe_count * 2 {
        println!("   Strong Bullish ({}x more CE than PE)", ce_count as f64 / pe_count.max(1) as f64);
    } else if pe_count > ce_count * 2 {
        println!("   Strong Bearish ({}x more PE than CE)", pe_count as f64 / ce_count.max(1) as f64);
    } else if ce_count > pe_count {
        println!("   Moderately Bullish");
    } else if pe_count > ce_count {
        println!("   Moderately Bearish");
    } else {
        println!("   Neutral/Mixed");
    }

    println!("\n✅ Status check complete!");

    Ok(())
}

