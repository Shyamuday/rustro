/// Automatic token extraction from instrument master
/// Intelligently identifies underlying, futures, and options tokens

use std::collections::HashMap;
use tracing::{info, warn};

use crate::types::Instrument;

/// Token information for an underlying asset
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetTokens {
    pub underlying_name: String,
    pub spot_token: Option<String>,
    pub spot_symbol: Option<String>,
    pub futures: Vec<FutureToken>,
    pub options: Vec<OptionToken>,
}

/// Future contract token
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FutureToken {
    pub token: String,
    pub symbol: String,
    pub expiry: String,
    pub lot_size: i32,
}

/// Option contract token
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OptionToken {
    pub token: String,
    pub symbol: String,
    pub strike: f64,
    pub option_type: String, // "CE" or "PE"
    pub expiry: String,
    pub lot_size: i32,
}

/// Automatic token extractor
pub struct TokenExtractor {
    instruments: Vec<Instrument>,
}

impl TokenExtractor {
    pub fn new(instruments: Vec<Instrument>) -> Self {
        Self { instruments }
    }

    /// Extract all tokens for a given underlying (NIFTY, BANKNIFTY, FINNIFTY)
    pub fn extract_asset_tokens(&self, underlying: &str) -> AssetTokens {
        info!("🔍 Extracting tokens for {}...", underlying);

        let underlying_upper = underlying.to_uppercase();

        // Find spot/index token
        let (spot_token, spot_symbol) = self.find_spot_token(&underlying_upper);

        // Find futures
        let futures = self.find_futures(&underlying_upper);

        // Find options
        let options = self.find_options(&underlying_upper);

        info!("✅ Extracted tokens for {}:", underlying);
        info!("   Spot: {} (token: {})", 
              spot_symbol.as_ref().unwrap_or(&"NOT FOUND".to_string()),
              spot_token.as_ref().unwrap_or(&"NOT FOUND".to_string()));
        info!("   Futures: {} contracts", futures.len());
        info!("   Options: {} contracts", options.len());

        AssetTokens {
            underlying_name: underlying_upper,
            spot_token,
            spot_symbol,
            futures,
            options,
        }
    }

    /// Extract tokens for all major indices
    pub fn extract_all_indices(&self) -> HashMap<String, AssetTokens> {
        let mut result = HashMap::new();

        let indices = vec!["NIFTY", "BANKNIFTY", "FINNIFTY"];

        for index in indices {
            let tokens = self.extract_asset_tokens(index);
            result.insert(index.to_string(), tokens);
        }

        result
    }

    /// Get list of all F&O stocks (stocks with futures and options)
    pub fn get_all_fno_stocks(&self) -> Vec<String> {
        let mut stocks: Vec<String> = self.instruments.iter()
            .filter(|i| {
                // Must be in NFO segment
                i.exch_seg == "NFO"
                // Must be a stock (FUTSTK or OPTSTK)
                && (i.instrument_type == "FUTSTK" || i.instrument_type == "OPTSTK")
            })
            .map(|i| i.name.clone())
            .collect();

        // Remove duplicates and sort
        stocks.sort();
        stocks.dedup();

        info!("📋 Found {} F&O stocks", stocks.len());
        stocks
    }

    /// Get popular F&O stocks (top traded)
    pub fn get_popular_fno_stocks(&self) -> Vec<String> {
        // Popular F&O stocks in Indian market
        vec![
            "RELIANCE".to_string(),
            "TCS".to_string(),
            "HDFCBANK".to_string(),
            "INFY".to_string(),
            "ICICIBANK".to_string(),
            "HINDUNILVR".to_string(),
            "ITC".to_string(),
            "SBIN".to_string(),
            "BHARTIARTL".to_string(),
            "KOTAKBANK".to_string(),
            "LT".to_string(),
            "AXISBANK".to_string(),
            "BAJFINANCE".to_string(),
            "ASIANPAINT".to_string(),
            "MARUTI".to_string(),
            "TITAN".to_string(),
            "SUNPHARMA".to_string(),
            "WIPRO".to_string(),
            "ULTRACEMCO".to_string(),
            "TATAMOTORS".to_string(),
        ]
    }

    /// Extract tokens for all F&O stocks
    pub fn extract_all_fno_stock_tokens(&self) -> HashMap<String, AssetTokens> {
        let mut result = HashMap::new();
        let stocks = self.get_all_fno_stocks();

        info!("🔍 Extracting tokens for {} F&O stocks...", stocks.len());

        for (idx, stock) in stocks.iter().enumerate() {
            if idx % 10 == 0 {
                info!("   Progress: {}/{}", idx, stocks.len());
            }
            let tokens = self.extract_asset_tokens(stock);
            result.insert(stock.clone(), tokens);
        }

        info!("✅ Extracted tokens for {} F&O stocks", result.len());
        result
    }

    /// Extract tokens for popular F&O stocks only
    pub fn extract_popular_fno_stock_tokens(&self) -> HashMap<String, AssetTokens> {
        let mut result = HashMap::new();
        let stocks = self.get_popular_fno_stocks();

        info!("🔍 Extracting tokens for {} popular F&O stocks...", stocks.len());

        for stock in stocks {
            let tokens = self.extract_asset_tokens(&stock);
            result.insert(stock.clone(), tokens);
        }

        info!("✅ Extracted tokens for {} popular F&O stocks", result.len());
        result
    }

    /// Find spot/index token for underlying
    fn find_spot_token(&self, underlying: &str) -> (Option<String>, Option<String>) {
        // Strategy 1: Look for exact name match with INDEX instrument type
        if let Some(inst) = self.instruments.iter().find(|i| {
            i.name == underlying && i.instrument_type == "INDEX"
        }) {
            info!("   Found INDEX: {} (token: {})", inst.symbol, inst.token);
            return (Some(inst.token.clone()), Some(inst.symbol.clone()));
        }

        // Strategy 2: Look for OPTIDX in NSE (common for option underlyings)
        if let Some(inst) = self.instruments.iter().find(|i| {
            i.name == underlying && i.instrument_type == "OPTIDX" && i.exch_seg == "NSE"
        }) {
            info!("   Found OPTIDX: {} (token: {})", inst.symbol, inst.token);
            return (Some(inst.token.clone()), Some(inst.symbol.clone()));
        }

        // Strategy 3: Look for stock in NSE (for F&O stocks)
        if let Some(inst) = self.instruments.iter().find(|i| {
            (i.name == underlying || i.symbol == underlying) 
            && i.exch_seg == "NSE" 
            && i.instrument_type == "EQUITY"
        }) {
            info!("   Found EQUITY: {} (token: {})", inst.symbol, inst.token);
            return (Some(inst.token.clone()), Some(inst.symbol.clone()));
        }

        // Strategy 4: Look for symbol starting with underlying name
        if let Some(inst) = self.instruments.iter().find(|i| {
            i.symbol.starts_with(underlying) && 
            (i.instrument_type == "INDEX" || i.instrument_type == "OPTIDX" || i.instrument_type == "EQUITY")
        }) {
            info!("   Found by symbol prefix: {} (token: {})", inst.symbol, inst.token);
            return (Some(inst.token.clone()), Some(inst.symbol.clone()));
        }

        // Strategy 5: Special handling for specific indices
        let special_patterns = match underlying {
            "NIFTY" => vec!["NIFTY 50", "Nifty 50", "NIFTY50"],
            "BANKNIFTY" => vec!["NIFTY BANK", "Nifty Bank", "BANKNIFTY"],
            "FINNIFTY" => vec!["NIFTY FIN SERVICE", "Nifty Fin Service", "FINNIFTY"],
            _ => vec![],
        };

        for pattern in special_patterns {
            if let Some(inst) = self.instruments.iter().find(|i| {
                i.symbol.contains(pattern) || i.name.contains(pattern)
            }) {
                info!("   Found by special pattern '{}': {} (token: {})", 
                      pattern, inst.symbol, inst.token);
                return (Some(inst.token.clone()), Some(inst.symbol.clone()));
            }
        }

        warn!("⚠️  Could not find spot token for {}", underlying);
        (None, None)
    }

    /// Find all futures contracts for underlying
    fn find_futures(&self, underlying: &str) -> Vec<FutureToken> {
        let futures: Vec<FutureToken> = self.instruments.iter()
            .filter(|i| {
                // Must match underlying name
                i.name == underlying
                // Must be in NFO segment
                && i.exch_seg == "NFO"
                // Must be a futures contract
                && (i.instrument_type == "FUTIDX" || i.instrument_type == "FUTSTK")
            })
            .map(|i| FutureToken {
                token: i.token.clone(),
                symbol: i.symbol.clone(),
                expiry: i.expiry.clone(),
                lot_size: i.lotsize,
            })
            .collect();

        if !futures.is_empty() {
            info!("   Found {} futures contracts", futures.len());
            // Show first few examples
            for (idx, fut) in futures.iter().take(3).enumerate() {
                info!("     [{}] {} (expiry: {}, lot: {})", 
                      idx + 1, fut.symbol, fut.expiry, fut.lot_size);
            }
            if futures.len() > 3 {
                info!("     ... and {} more", futures.len() - 3);
            }
        }

        futures
    }

    /// Find all option contracts for underlying
    fn find_options(&self, underlying: &str) -> Vec<OptionToken> {
        let options: Vec<OptionToken> = self.instruments.iter()
            .filter(|i| {
                // Must match underlying name
                i.name == underlying
                // Must be in NFO segment
                && i.exch_seg == "NFO"
                // Must be an option (OPTIDX or OPTSTK)
                && (i.instrument_type == "OPTIDX" || i.instrument_type == "OPTSTK")
                // Must end with CE or PE
                && (i.symbol.ends_with("CE") || i.symbol.ends_with("PE"))
            })
            .map(|i| {
                let option_type = if i.symbol.ends_with("CE") {
                    "CE".to_string()
                } else {
                    "PE".to_string()
                };

                OptionToken {
                    token: i.token.clone(),
                    symbol: i.symbol.clone(),
                    strike: i.strike,
                    option_type,
                    expiry: i.expiry.clone(),
                    lot_size: i.lotsize,
                }
            })
            .collect();

        if !options.is_empty() {
            info!("   Found {} option contracts", options.len());
            
            // Count by option type
            let ce_count = options.iter().filter(|o| o.option_type == "CE").count();
            let pe_count = options.iter().filter(|o| o.option_type == "PE").count();
            info!("     CE: {}, PE: {}", ce_count, pe_count);

            // Show strike range
            if let (Some(min_strike), Some(max_strike)) = (
                options.iter().map(|o| o.strike as i32).min(),
                options.iter().map(|o| o.strike as i32).max(),
            ) {
                info!("     Strike range: {} to {}", min_strike, max_strike);
            }

            // Show first few examples
            for (idx, opt) in options.iter().take(3).enumerate() {
                info!("     [{}] {} (strike: {}, expiry: {})", 
                      idx + 1, opt.symbol, opt.strike, opt.expiry);
            }
            if options.len() > 3 {
                info!("     ... and {} more", options.len() - 3);
            }
        }

        options
    }

    /// Get options for a specific strike range
    pub fn get_options_in_range(
        &self,
        underlying: &str,
        min_strike: i32,
        max_strike: i32,
        expiry: Option<&str>,
    ) -> Vec<OptionToken> {
        let underlying_upper = underlying.to_uppercase();
        let all_options = self.find_options(&underlying_upper);

        let filtered: Vec<OptionToken> = all_options
            .into_iter()
            .filter(|opt| {
                let strike = opt.strike as i32;
                let in_range = strike >= min_strike && strike <= max_strike;
                
                let expiry_match = if let Some(exp) = expiry {
                    opt.expiry == exp
                } else {
                    true
                };

                in_range && expiry_match
            })
            .collect();

        info!("   Filtered to {} options in range {} to {}", 
              filtered.len(), min_strike, max_strike);

        filtered
    }

    /// Get nearest expiry options
    pub fn get_nearest_expiry_options(&self, underlying: &str) -> Vec<OptionToken> {
        let underlying_upper = underlying.to_uppercase();
        let all_options = self.find_options(&underlying_upper);

        if all_options.is_empty() {
            return Vec::new();
        }

        // Find nearest expiry
        let mut expiries: Vec<String> = all_options
            .iter()
            .map(|o| o.expiry.clone())
            .collect();
        expiries.sort();
        expiries.dedup();

        if let Some(nearest_expiry) = expiries.first() {
            info!("   Nearest expiry: {}", nearest_expiry);
            
            let filtered: Vec<OptionToken> = all_options
                .into_iter()
                .filter(|o| o.expiry == *nearest_expiry)
                .collect();

            info!("   Found {} options for nearest expiry", filtered.len());
            return filtered;
        }

        Vec::new()
    }

    /// Get ATM (At-The-Money) options for a given price
    pub fn get_atm_options(
        &self,
        underlying: &str,
        current_price: f64,
        strike_increment: i32,
        range_strikes: usize,
    ) -> Vec<OptionToken> {
        let underlying_upper = underlying.to_uppercase();
        
        // Calculate ATM strike
        let atm_strike = ((current_price / strike_increment as f64).round() 
                         * strike_increment as f64) as i32;
        
        info!("   Current price: {:.2}, ATM strike: {}", current_price, atm_strike);

        // Calculate range
        let min_strike = atm_strike - (strike_increment * range_strikes as i32);
        let max_strike = atm_strike + (strike_increment * range_strikes as i32);

        info!("   Strike range: {} to {} (±{} strikes)", 
              min_strike, max_strike, range_strikes);

        self.get_options_in_range(&underlying_upper, min_strike, max_strike, None)
    }

    /// Export tokens to JSON file for reference
    pub async fn export_tokens_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let all_tokens = self.extract_all_indices();
        
        let json = serde_json::to_string_pretty(&all_tokens)?;
        tokio::fs::write(filename, json).await?;
        
        info!("💾 Exported token mapping to {}", filename);
        Ok(())
    }

    /// Print summary statistics
    pub fn print_summary(&self) {
        info!("📊 Instrument Master Summary:");
        info!("   Total instruments: {}", self.instruments.len());

        // Count by exchange
        let nse_count = self.instruments.iter().filter(|i| i.exch_seg == "NSE").count();
        let nfo_count = self.instruments.iter().filter(|i| i.exch_seg == "NFO").count();
        let bse_count = self.instruments.iter().filter(|i| i.exch_seg == "BSE").count();
        
        info!("   NSE: {}, NFO: {}, BSE: {}", nse_count, nfo_count, bse_count);

        // Count by instrument type
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for inst in &self.instruments {
            *type_counts.entry(inst.instrument_type.clone()).or_insert(0) += 1;
        }

        info!("   By type:");
        for (inst_type, count) in type_counts.iter() {
            if *count > 100 {
                info!("     {}: {}", inst_type, count);
            }
        }

        // Count major indices
        for index in &["NIFTY", "BANKNIFTY", "FINNIFTY"] {
            let count = self.instruments.iter()
                .filter(|i| i.name == *index)
                .count();
            info!("   {} instruments: {}", index, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_instruments() -> Vec<Instrument> {
        vec![
            Instrument {
                token: "99926000".to_string(),
                symbol: "NIFTY 50".to_string(),
                name: "NIFTY".to_string(),
                expiry: "".to_string(),
                strike: 0.0,
                lotsize: 50,
                instrument_type: "INDEX".to_string(),
                exch_seg: "NSE".to_string(),
                tick_size: 0.05,
            },
            Instrument {
                token: "12345".to_string(),
                symbol: "NIFTY23DEC2350000CE".to_string(),
                name: "NIFTY".to_string(),
                expiry: "28DEC2023".to_string(),
                strike: 23500.0,
                lotsize: 50,
                instrument_type: "OPTIDX".to_string(),
                exch_seg: "NFO".to_string(),
                tick_size: 0.05,
            },
        ]
    }

    #[test]
    fn test_extract_spot_token() {
        let instruments = create_test_instruments();
        let extractor = TokenExtractor::new(instruments);
        
        let (token, symbol) = extractor.find_spot_token("NIFTY");
        assert!(token.is_some());
        assert_eq!(token.unwrap(), "99926000");
    }

    #[test]
    fn test_extract_options() {
        let instruments = create_test_instruments();
        let extractor = TokenExtractor::new(instruments);
        
        let options = extractor.find_options("NIFTY");
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].strike, 23500.0);
        assert_eq!(options[0].option_type, "CE");
    }
}

