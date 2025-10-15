/// Centralized error types for the trading bot
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingError {
    // Authentication Errors
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Token expired: {0}")]
    TokenExpired(String),
    
    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),
    
    // Network Errors
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("WebSocket connection failed: {0}")]
    WebSocketError(String),
    
    #[error("WebSocket disconnected: {0}")]
    WebSocketDisconnected(String),
    
    #[error("Network timeout: {0}")]
    NetworkTimeout(String),
    
    // Data Errors
    #[error("Data gap detected: {0}")]
    DataGap(String),
    
    #[error("Invalid bar data: {0}")]
    InvalidBarData(String),
    
    #[error("Missing data: {0}")]
    MissingData(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationError(#[from] serde_json::Error),
    
    // Order Errors
    #[error("Order placement failed: {0}")]
    OrderPlacementFailed(String),
    
    #[error("Order not found: {0}")]
    OrderNotFound(String),
    
    #[error("Order rejection: {0}")]
    OrderRejected(String),
    
    #[error("Insufficient margin: {0}")]
    InsufficientMargin(String),
    
    #[error("Freeze quantity breach: {0}")]
    FreezeQuantityBreach(String),
    
    #[error("Price band breach: {0}")]
    PriceBandBreach(String),
    
    // Position Errors
    #[error("Position not found: {0}")]
    PositionNotFound(String),
    
    #[error("Position limit exceeded: {0}")]
    PositionLimitExceeded(String),
    
    #[error("Position already exists: {0}")]
    DuplicatePosition(String),
    
    // Risk Errors
    #[error("Daily loss limit breached: {0}")]
    DailyLossLimit(String),
    
    #[error("VIX spike detected: {0}")]
    VixSpike(String),
    
    #[error("Risk check failed: {0}")]
    RiskCheckFailed(String),
    
    // Strategy Errors
    #[error("Invalid strategy state: {0}")]
    InvalidStrategyState(String),
    
    #[error("No trade signal: {0}")]
    NoTradeSignal(String),
    
    #[error("Strategy alignment lost: {0}")]
    AlignmentLost(String),
    
    // Configuration Errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    // File I/O Errors
    #[error("File I/O error: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("File write failed: {0}")]
    FileWriteFailed(String),
    
    // Market Session Errors
    #[error("Market closed: {0}")]
    MarketClosed(String),
    
    #[error("Outside entry window: {0}")]
    OutsideEntryWindow(String),
    
    #[error("Non-trading day: {0}")]
    NonTradingDay(String),
    
    // Broker Errors
    #[error("Broker API error: {code} - {message}")]
    BrokerApiError { code: String, message: String },
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Instrument not found: {0}")]
    InstrumentNotFound(String),
    
    // System Errors
    #[error("System shutdown: {0}")]
    SystemShutdown(String),
    
    #[error("Fatal error: {0}")]
    FatalError(String),
    
    #[error("Graceful exit: {0}")]
    GracefulExit(String),
    
    // Event Bus Errors
    #[error("Event dispatch failed: {0}")]
    EventDispatchFailed(String),
    
    #[error("Event handler error: {0}")]
    EventHandlerError(String),
    
    // Idempotency Errors
    #[error("Duplicate event detected: {0}")]
    DuplicateEvent(String),
    
    #[error("Idempotency key collision: {0}")]
    IdempotencyCollision(String),
    
    // Recovery Errors
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
    
    #[error("Recovery timeout: {0}")]
    RecoveryTimeout(String),
    
    // Generic Errors
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, TradingError>;

impl TradingError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            TradingError::NetworkTimeout(_)
                | TradingError::WebSocketDisconnected(_)
                | TradingError::DataGap(_)
                | TradingError::OrderPlacementFailed(_)
                | TradingError::RateLimitExceeded(_)
        )
    }
    
    /// Check if error requires immediate system shutdown
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            TradingError::FatalError(_)
                | TradingError::TokenRefreshFailed(_)
                | TradingError::SystemShutdown(_)
        )
    }
    
    /// Check if error requires graceful position exit
    pub fn requires_exit(&self) -> bool {
        matches!(
            self,
            TradingError::VixSpike(_)
                | TradingError::DailyLossLimit(_)
                | TradingError::TokenExpired(_)
                | TradingError::MarketClosed(_)
        )
    }
    
    /// Get error code for logging/monitoring
    pub fn error_code(&self) -> &str {
        match self {
            TradingError::AuthenticationFailed(_) => "AUTH_001",
            TradingError::TokenExpired(_) => "AUTH_002",
            TradingError::TokenRefreshFailed(_) => "AUTH_003",
            TradingError::HttpError(_) => "NET_001",
            TradingError::WebSocketError(_) => "NET_002",
            TradingError::WebSocketDisconnected(_) => "NET_003",
            TradingError::NetworkTimeout(_) => "NET_004",
            TradingError::DataGap(_) => "DATA_001",
            TradingError::InvalidBarData(_) => "DATA_002",
            TradingError::MissingData(_) => "DATA_003",
            TradingError::DeserializationError(_) => "DATA_004",
            TradingError::OrderPlacementFailed(_) => "ORDER_001",
            TradingError::OrderNotFound(_) => "ORDER_002",
            TradingError::OrderRejected(_) => "ORDER_003",
            TradingError::InsufficientMargin(_) => "ORDER_004",
            TradingError::FreezeQuantityBreach(_) => "ORDER_005",
            TradingError::PriceBandBreach(_) => "ORDER_006",
            TradingError::PositionNotFound(_) => "POS_001",
            TradingError::PositionLimitExceeded(_) => "POS_002",
            TradingError::DuplicatePosition(_) => "POS_003",
            TradingError::DailyLossLimit(_) => "RISK_001",
            TradingError::VixSpike(_) => "RISK_002",
            TradingError::RiskCheckFailed(_) => "RISK_003",
            TradingError::InvalidStrategyState(_) => "STRAT_001",
            TradingError::NoTradeSignal(_) => "STRAT_002",
            TradingError::AlignmentLost(_) => "STRAT_003",
            TradingError::ConfigError(_) => "CFG_001",
            TradingError::InvalidParameter(_) => "CFG_002",
            TradingError::FileError(_) => "FILE_001",
            TradingError::FileNotFound(_) => "FILE_002",
            TradingError::FileWriteFailed(_) => "FILE_003",
            TradingError::MarketClosed(_) => "MKT_001",
            TradingError::OutsideEntryWindow(_) => "MKT_002",
            TradingError::NonTradingDay(_) => "MKT_003",
            TradingError::BrokerApiError { .. } => "BROKER_001",
            TradingError::RateLimitExceeded(_) => "BROKER_002",
            TradingError::InstrumentNotFound(_) => "BROKER_003",
            TradingError::SystemShutdown(_) => "SYS_001",
            TradingError::FatalError(_) => "SYS_002",
            TradingError::GracefulExit(_) => "SYS_003",
            TradingError::EventDispatchFailed(_) => "EVENT_001",
            TradingError::EventHandlerError(_) => "EVENT_002",
            TradingError::DuplicateEvent(_) => "IDEM_001",
            TradingError::IdempotencyCollision(_) => "IDEM_002",
            TradingError::RecoveryFailed(_) => "REC_001",
            TradingError::RecoveryTimeout(_) => "REC_002",
            TradingError::InternalError(_) => "INT_001",
            TradingError::Other(_) => "GEN_001",
        }
    }
}

