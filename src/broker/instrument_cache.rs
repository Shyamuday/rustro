/// Instrument cache for fast token lookups
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, NaiveDate, Utc};
use tracing::info;

use crate::broker::AngelOneClient;
use crate::error::{Result, TradingError};
use crate::types::{Instrument, OptionType};

/// Instrument cache for fast lookups
pub struct InstrumentCache {
    broker: Arc<AngelOneClient>,
    instruments: Arc<RwLock<Vec<Instrument>>>,
    token_map: Arc<RwLock<HashMap<String, Instrument>>>,
    last_updated: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl InstrumentCache {
    pub fn new(broker: Arc<AngelOneClient>) -> Self {
        InstrumentCache {
            broker,
            instruments: Arc::new(RwLock::new(Vec::new())),
            token_map: Arc::new(RwLock::new(HashMap::new())),
            last_updated: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Download and cache instrument master
    pub async fn refresh(&self) -> Result<()> {
        info!("📥 Downloading instrument master...");
        
        let instruments = self.broker.download_instrument_master().await?;
        
        // Build token map for fast lookups
        let mut token_map = HashMap::new();
        for inst in &instruments {
            token_map.insert(inst.symbol.clone(), inst.clone());
            token_map.insert(inst.token.clone(), inst.clone());
        }
        
        // Update cache
        {
            let mut cache = self.instruments.write().await;
            *cache = instruments.clone();
        }
        {
            let mut map = self.token_map.write().await;
            *map = token_map;
        }
        {
            let mut updated = self.last_updated.write().await;
            *updated = Some(Utc::now());
        }
        
        info!("✅ Cached {} instruments", instruments.len());
        Ok(())
    }
    
    /// Find NIFTY underlying token
    pub async fn get_nifty_token(&self) -> Result<String> {
        let instruments = self.instruments.read().await;
        
        let nifty = instruments.iter()
            .find(|i| i.name == "NIFTY" && i.instrument_type == "OPTIDX")
            .or_else(|| instruments.iter().find(|i| i.symbol.starts_with("NIFTY") && i.instrument_type == "INDEX"))
            .ok_or_else(|| TradingError::InstrumentNotFound("NIFTY not found".to_string()))?;
        
        Ok(nifty.token.clone())
    }
    
    /// Find option token by strike and type
    pub async fn find_option_token(
        &self,
        underlying: &str,
        strike: i32,
        option_type: OptionType,
        expiry: Option<NaiveDate>,
    ) -> Result<(String, String)> {
        let instruments = self.instruments.read().await;
        
        // Filter by underlying, strike, option type, and NFO exchange
        let mut candidates: Vec<&Instrument> = instruments.iter()
            .filter(|i| {
                i.name == underlying
                    && i.strike as i32 == strike
                    && i.exch_seg == "NFO"
                    && i.symbol.ends_with(option_type.as_str())
            })
            .collect();
        
        if candidates.is_empty() {
            return Err(TradingError::InstrumentNotFound(format!(
                "No option found: {} {} {}",
                underlying, strike, option_type.as_str()
            )));
        }
        
        // If expiry specified, filter by expiry
        if let Some(target_expiry) = expiry {
            candidates.retain(|i| {
                if let Ok(inst_expiry) = NaiveDate::parse_from_str(&i.expiry, "%d%b%Y") {
                    inst_expiry == target_expiry
                } else {
                    false
                }
            });
        } else {
            // Get nearest expiry (weekly)
            candidates.sort_by_key(|i| &i.expiry);
        }
        
        let instrument = candidates.first()
            .ok_or_else(|| TradingError::InstrumentNotFound(format!(
                "No matching expiry: {} {} {}",
                underlying, strike, option_type.as_str()
            )))?;
        
        info!(
            "🎯 Found option: {} (token: {}, expiry: {}, lot: {})",
            instrument.symbol,
            instrument.token,
            instrument.expiry,
            instrument.lotsize
        );
        
        Ok((instrument.token.clone(), instrument.symbol.clone()))
    }
    
    /// Get instrument by token
    pub async fn get_by_token(&self, token: &str) -> Option<Instrument> {
        let map = self.token_map.read().await;
        map.get(token).cloned()
    }
    
    /// Get instrument by symbol
    pub async fn get_by_symbol(&self, symbol: &str) -> Option<Instrument> {
        let map = self.token_map.read().await;
        map.get(symbol).cloned()
    }
    
    /// Get all NIFTY options for a specific expiry
    pub async fn get_nifty_options_chain(
        &self,
        expiry: Option<NaiveDate>,
    ) -> Vec<Instrument> {
        let instruments = self.instruments.read().await;
        
        let mut options: Vec<Instrument> = instruments.iter()
            .filter(|i| i.name == "NIFTY" && i.exch_seg == "NFO")
            .cloned()
            .collect();
        
        if let Some(target_expiry) = expiry {
            options.retain(|i| {
                if let Ok(inst_expiry) = NaiveDate::parse_from_str(&i.expiry, "%d%b%Y") {
                    inst_expiry == target_expiry
                } else {
                    false
                }
            });
        }
        
        options.sort_by_key(|i| i.strike as i32);
        options
    }
    
    /// Check if cache needs refresh (daily)
    pub async fn needs_refresh(&self) -> bool {
        let last_updated = self.last_updated.read().await;
        
        match *last_updated {
            None => true,
            Some(last) => {
                let now = Utc::now();
                let diff = now - last;
                diff.num_hours() >= 24 // Refresh daily
            }
        }
    }
    
    /// Get cache size
    pub async fn size(&self) -> usize {
        let instruments = self.instruments.read().await;
        instruments.len()
    }
    
    /// Get all instruments (for historical sync and analysis)
    pub async fn get_all_instruments(&self) -> Vec<Instrument> {
        let instruments = self.instruments.read().await;
        instruments.clone()
    }
}
