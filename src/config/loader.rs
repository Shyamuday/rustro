/// Configuration loading from TOML file
use std::path::Path;
use crate::error::{Result, TradingError};
use crate::types::Config;

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| TradingError::ConfigError(format!("Failed to read config file: {}", e)))?;
    
    let config: Config = toml::from_str(&content)
        .map_err(|e| TradingError::ConfigError(format!("Failed to parse config: {}", e)))?;
    
    // Validate config
    validate_config(&config)?;
    
    Ok(config)
}

fn validate_config(config: &Config) -> Result<()> {
    // Validate time windows
    if config.entry_window_start.is_empty() {
        return Err(TradingError::ConfigError("entry_window_start is empty".to_string()));
    }
    
    // Validate risk parameters
    if config.option_stop_loss_pct <= 0.0 || config.option_stop_loss_pct > 1.0 {
        return Err(TradingError::ConfigError(
            format!("Invalid option_stop_loss_pct: {}", config.option_stop_loss_pct)
        ));
    }
    
    if config.daily_loss_limit_pct <= 0.0 {
        return Err(TradingError::ConfigError(
            format!("Invalid daily_loss_limit_pct: {}", config.daily_loss_limit_pct)
        ));
    }
    
    // Validate VIX thresholds
    if config.vix_spike_threshold <= config.vix_resume_threshold {
        return Err(TradingError::ConfigError(
            "vix_spike_threshold must be > vix_resume_threshold".to_string()
        ));
    }
    
    // Validate periods
    if config.daily_adx_period < 2 || config.hourly_adx_period < 2 {
        return Err(TradingError::ConfigError("ADX periods must be >= 2".to_string()));
    }
    
    Ok(())
}

