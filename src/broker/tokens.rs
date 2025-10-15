/// Token management for Angel One SmartAPI
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::{Result, TradingError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub jwt_token: String,
    pub feed_token: String,
    pub jwt_expiry: DateTime<Utc>,
    pub feed_expiry: DateTime<Utc>,
    pub refresh_token: Option<String>,
}

impl Tokens {
    pub fn is_jwt_expired(&self) -> bool {
        Utc::now() >= self.jwt_expiry
    }
    
    pub fn is_feed_expired(&self) -> bool {
        Utc::now() >= self.feed_expiry
    }
    
    pub fn minutes_until_jwt_expiry(&self) -> i64 {
        (self.jwt_expiry - Utc::now()).num_minutes()
    }
    
    pub fn minutes_until_feed_expiry(&self) -> i64 {
        (self.feed_expiry - Utc::now()).num_minutes()
    }
}

/// Token manager with thread-safe access
pub struct TokenManager {
    tokens: Arc<RwLock<Option<Tokens>>>,
    token_file_path: String,
}

impl TokenManager {
    pub fn new(token_file_path: String) -> Self {
        TokenManager {
            tokens: Arc::new(RwLock::new(None)),
            token_file_path,
        }
    }
    
    /// Get current tokens (clone)
    pub async fn get_tokens(&self) -> Option<Tokens> {
        let tokens = self.tokens.read().await;
        tokens.clone()
    }
    
    /// Set tokens and persist to disk
    pub async fn set_tokens(&self, tokens: Tokens) -> Result<()> {
        // Persist to file
        self.save_tokens_to_file(&tokens).await?;
        
        // Update in memory
        let mut t = self.tokens.write().await;
        *t = Some(tokens);
        
        debug!("Tokens updated and persisted");
        Ok(())
    }
    
    /// Load tokens from file
    pub async fn load_from_file(&self) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.token_file_path).await?;
        let tokens: Tokens = serde_json::from_str(&content)?;
        
        let mut t = self.tokens.write().await;
        *t = Some(tokens);
        
        debug!("Tokens loaded from file");
        Ok(())
    }
    
    /// Save tokens to file
    async fn save_tokens_to_file(&self, tokens: &Tokens) -> Result<()> {
        let json = serde_json::to_string_pretty(tokens)?;
        tokio::fs::write(&self.token_file_path, json).await?;
        Ok(())
    }
    
    /// Check if tokens need refresh (warning threshold)
    pub async fn needs_refresh(&self, warning_minutes: i64) -> bool {
        if let Some(tokens) = self.get_tokens().await {
            tokens.minutes_until_jwt_expiry() < warning_minutes
                || tokens.minutes_until_feed_expiry() < warning_minutes
        } else {
            true
        }
    }
    
    /// Check if tokens are valid
    pub async fn is_valid(&self) -> bool {
        if let Some(tokens) = self.get_tokens().await {
            !tokens.is_jwt_expired() && !tokens.is_feed_expired()
        } else {
            false
        }
    }
    
    /// Clear tokens
    pub async fn clear(&self) {
        let mut t = self.tokens.write().await;
        *t = None;
        
        // Delete file
        let _ = tokio::fs::remove_file(&self.token_file_path).await;
        
        warn!("Tokens cleared");
    }
}

