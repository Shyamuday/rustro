/// Pre-market ATM option selector
/// Selects ATM strike based on previous day close and daily bias

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use crate::broker::TokenExtractor;
use crate::strategy::{BiasDirection, DailyBias};

/// ATM strike information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmStrike {
    pub strike: i32,
    pub distance_from_price: f64,
}

/// Pre-selected option for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreSelectedOption {
    pub underlying: String,
    pub spot_token: String,
    pub bias: BiasDirection,
    pub close_price: f64,
    pub atm_strike: AtmStrike,
    
    // Option tokens based on bias
    pub ce_token: Option<String>,
    pub ce_symbol: Option<String>,
    pub pe_token: Option<String>,
    pub pe_symbol: Option<String>,
    
    pub lot_size: i32,
    pub expiry: String,
}

/// Pre-market ATM selector
pub struct PremarketSelector {
    token_extractor: Arc<TokenExtractor>,
}

impl PremarketSelector {
    pub fn new(token_extractor: Arc<TokenExtractor>) -> Self {
        Self { token_extractor }
    }

    /// Select ATM strike based on close price
    pub fn select_atm_strike(
        &self,
        underlying: &str,
        close_price: f64,
    ) -> Option<AtmStrike> {
        // Get strike increment for this underlying
        let strike_increment = self.get_strike_increment(underlying);
        
        // Calculate ATM strike (round to nearest increment)
        let atm_strike = ((close_price / strike_increment as f64).round() 
                         * strike_increment as f64) as i32;
        
        let distance = (atm_strike as f64 - close_price).abs();
        
        info!("   {} @ {:.2} → ATM strike: {} (distance: {:.2})",
              underlying, close_price, atm_strike, distance);
        
        Some(AtmStrike {
            strike: atm_strike,
            distance_from_price: distance,
        })
    }

    /// Get strike increment for underlying
    fn get_strike_increment(&self, underlying: &str) -> i32 {
        match underlying {
            "NIFTY" | "FINNIFTY" => 50,
            "BANKNIFTY" => 100,
            // For stocks, try to detect from available strikes
            _ => self.detect_strike_increment(underlying).unwrap_or(50),
        }
    }

    /// Detect strike increment from available options
    fn detect_strike_increment(&self, underlying: &str) -> Option<i32> {
        let tokens = self.token_extractor.extract_asset_tokens(underlying);
        
        if tokens.options.len() < 2 {
            return None;
        }

        // Get first two strikes and calculate difference
        let mut strikes: Vec<i32> = tokens.options
            .iter()
            .map(|o| o.strike as i32)
            .collect();
        strikes.sort();
        strikes.dedup();

        if strikes.len() >= 2 {
            Some(strikes[1] - strikes[0])
        } else {
            None
        }
    }

    /// Select pre-market option for a bias
    pub fn select_premarket_option(
        &self,
        bias: &DailyBias,
    ) -> Option<PreSelectedOption> {
        // Skip NoTrade bias
        if bias.bias == BiasDirection::NoTrade {
            return None;
        }

        // Calculate ATM strike
        let atm_strike = self.select_atm_strike(&bias.underlying, bias.close_price)?;

        // Extract tokens for this underlying
        let tokens = self.token_extractor.extract_asset_tokens(&bias.underlying);

        // Select appropriate expiry
        let selected_expiry = self.select_nearest_expiry(&bias.underlying, &tokens.options)?;
        
        info!("   Selected expiry: {} for {}", selected_expiry, bias.underlying);

        // Find options at ATM strike with selected expiry
        let atm_options: Vec<_> = tokens.options
            .iter()
            .filter(|o| o.strike as i32 == atm_strike.strike && o.expiry == selected_expiry)
            .collect();

        if atm_options.is_empty() {
            warn!("{}: No options found at strike {} for expiry {}", 
                  bias.underlying, atm_strike.strike, selected_expiry);
            return None;
        }

        // Get CE and PE tokens
        let ce_option = atm_options.iter().find(|o| o.option_type == "CE");
        let pe_option = atm_options.iter().find(|o| o.option_type == "PE");

        // Get lot size
        let lot_size = atm_options.first()?.lot_size;

        Some(PreSelectedOption {
            underlying: bias.underlying.clone(),
            spot_token: bias.spot_token.clone(),
            bias: bias.bias,
            close_price: bias.close_price,
            atm_strike,
            ce_token: ce_option.map(|o| o.token.clone()),
            ce_symbol: ce_option.map(|o| o.symbol.clone()),
            pe_token: pe_option.map(|o| o.token.clone()),
            pe_symbol: pe_option.map(|o| o.symbol.clone()),
            lot_size,
            expiry: selected_expiry,
        })
    }
    
    /// Select nearest expiry based on days to expiry (DTE)
    /// - For indices: Skip if DTE < 2 (avoid expiry day margin)
    /// - For stocks: Skip if DTE < 7 (avoid increasing margin)
    fn select_nearest_expiry(
        &self,
        underlying: &str,
        options: &[crate::broker::OptionToken],
    ) -> Option<String> {
        if options.is_empty() {
            return None;
        }

        // Get unique expiries and sort
        let mut expiries: Vec<String> = options
            .iter()
            .map(|o| o.expiry.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        expiries.sort();

        let is_index = matches!(underlying, "NIFTY" | "BANKNIFTY" | "FINNIFTY" | "MIDCPNIFTY");
        let now = chrono::Utc::now().date_naive();
        
        // DTE thresholds
        let min_dte_index = 2;  // Skip if < 2 days for indices
        let min_dte_stock = 7;  // Skip if < 7 days for stocks
        
        for (idx, expiry_str) in expiries.iter().enumerate() {
            // Parse expiry date from string (format: "14NOV2024" or "28NOV2024")
            if let Some(expiry_date) = self.parse_expiry_date(expiry_str) {
                let dte = (expiry_date - now).num_days();
                
                if is_index {
                    // For indices: Need at least 2 DTE
                    if dte >= min_dte_index {
                        info!("   {} - Selected expiry: {} (DTE: {} days)", 
                              underlying, expiry_str, dte);
                        return Some(expiry_str.clone());
                    } else {
                        info!("   {} - Skipped expiry: {} (DTE: {} < {} days)", 
                              underlying, expiry_str, dte, min_dte_index);
                    }
                } else {
                    // For stocks: Need at least 7 DTE
                    if dte >= min_dte_stock {
                        info!("   {} - Selected expiry: {} (DTE: {} days)", 
                              underlying, expiry_str, dte);
                        return Some(expiry_str.clone());
                    } else {
                        info!("   {} - Skipped expiry: {} (DTE: {} < {} days)", 
                              underlying, expiry_str, dte, min_dte_stock);
                    }
                }
            }
        }
        
        // Fallback: If all expiries are too close, use the farthest one
        warn!("   {} - All expiries too close, using farthest: {:?}", 
              underlying, expiries.last());
        expiries.last().cloned()
    }
    
    /// Parse expiry date from string format (e.g., "14NOV2024" -> NaiveDate)
    fn parse_expiry_date(&self, expiry_str: &str) -> Option<chrono::NaiveDate> {
        // Expected format: "14NOV2024" or "28NOV2024"
        if expiry_str.len() < 9 {
            return None;
        }
        
        // Extract day, month, year
        let day_str = &expiry_str[0..2];
        let month_str = &expiry_str[2..5];
        let year_str = &expiry_str[5..9];
        
        let day: u32 = day_str.parse().ok()?;
        let year: i32 = year_str.parse().ok()?;
        
        // Map month abbreviation to number
        let month = match month_str.to_uppercase().as_str() {
            "JAN" => 1, "FEB" => 2, "MAR" => 3, "APR" => 4,
            "MAY" => 5, "JUN" => 6, "JUL" => 7, "AUG" => 8,
            "SEP" => 9, "OCT" => 10, "NOV" => 11, "DEC" => 12,
            _ => return None,
        };
        
        chrono::NaiveDate::from_ymd_opt(year, month, day)
    }

    /// Select pre-market options for all biases
    pub fn select_all_premarket_options(
        &self,
        biases: &[DailyBias],
    ) -> Vec<PreSelectedOption> {
        info!("🎯 Selecting pre-market ATM options...");
        
        let mut results = Vec::new();
        let mut ce_count = 0;
        let mut pe_count = 0;

        for bias in biases {
            if bias.bias == BiasDirection::NoTrade {
                continue;
            }

            if let Some(option) = self.select_premarket_option(bias) {
                match option.bias {
                    BiasDirection::CE => ce_count += 1,
                    BiasDirection::PE => pe_count += 1,
                    _ => {}
                }
                results.push(option);
            }
        }

        info!("✅ Selected {} options: {} CE, {} PE", results.len(), ce_count, pe_count);
        results
    }

    /// Get option to trade based on bias
    pub fn get_tradeable_option(option: &PreSelectedOption) -> Option<(String, String)> {
        match option.bias {
            BiasDirection::CE => {
                if let (Some(token), Some(symbol)) = (&option.ce_token, &option.ce_symbol) {
                    Some((token.clone(), symbol.clone()))
                } else {
                    None
                }
            }
            BiasDirection::PE => {
                if let (Some(token), Some(symbol)) = (&option.pe_token, &option.pe_symbol) {
                    Some((token.clone(), symbol.clone()))
                } else {
                    None
                }
            }
            BiasDirection::NoTrade => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atm_calculation() {
        // NIFTY at 23,547.50 with increment 50 → ATM = 23,550
        let close = 23547.50;
        let increment = 50;
        let atm = ((close / increment as f64).round() * increment as f64) as i32;
        assert_eq!(atm, 23550);

        // BANKNIFTY at 48,923.75 with increment 100 → ATM = 48,900
        let close = 48923.75;
        let increment = 100;
        let atm = ((close / increment as f64).round() * increment as f64) as i32;
        assert_eq!(atm, 48900);
    }
}

