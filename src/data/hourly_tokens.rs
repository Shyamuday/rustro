/// Hourly data tokens management
/// Stores tokens that need hourly bar data for analysis
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyDataToken {
    pub underlying: String,
    pub token: String,
    pub symbol: String,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyTokensConfig {
    pub tokens: Vec<HourlyDataToken>,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct HourlyTokensManager {
    file_path: String,
}

impl HourlyTokensManager {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    /// Load tokens from JSON file
    pub async fn load(&self) -> Result<HourlyTokensConfig, Box<dyn std::error::Error>> {
        if !fs::metadata(&self.file_path).await.is_ok() {
            // File doesn't exist, return empty config
            return Ok(HourlyTokensConfig {
                tokens: Vec::new(),
                last_sync: None,
            });
        }

        let content = fs::read_to_string(&self.file_path).await?;
        let config: HourlyTokensConfig = serde_json::from_str(&content)?;
        
        info!("📂 Loaded {} hourly data tokens from {}", config.tokens.len(), self.file_path);
        Ok(config)
    }

    /// Save tokens to JSON file
    pub async fn save(&self, config: &HourlyTokensConfig) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(&self.file_path, json).await?;
        
        info!("💾 Saved {} hourly data tokens to {}", config.tokens.len(), self.file_path);
        Ok(())
    }

    /// Add or update a token
    pub async fn add_token(
        &self,
        underlying: &str,
        token: &str,
        symbol: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.load().await?;
        
        // Remove existing token for this underlying if present
        config.tokens.retain(|t| t.underlying != underlying);
        
        // Add new token
        config.tokens.push(HourlyDataToken {
            underlying: underlying.to_string(),
            token: token.to_string(),
            symbol: symbol.to_string(),
            last_updated: Some(chrono::Utc::now()),
        });
        
        config.last_sync = Some(chrono::Utc::now());
        
        self.save(&config).await?;
        Ok(())
    }

    /// Get all tokens as a map (underlying -> token)
    pub async fn get_tokens_map(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let config = self.load().await?;
        let mut map = HashMap::new();
        
        for token_info in config.tokens {
            map.insert(token_info.underlying, token_info.token);
        }
        
        Ok(map)
    }

    /// Get token for a specific underlying
    pub async fn get_token(&self, underlying: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let config = self.load().await?;
        
        Ok(config.tokens
            .iter()
            .find(|t| t.underlying == underlying)
            .map(|t| t.token.clone()))
    }

    /// Get all tokens
    pub async fn get_all_tokens(&self) -> Result<Vec<HourlyDataToken>, Box<dyn std::error::Error>> {
        let config = self.load().await?;
        Ok(config.tokens)
    }
}

