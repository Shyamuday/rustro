/// Core type definitions for the trading bot
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OHLCV Bar data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub timestamp: DateTime<Utc>,
    pub timestamp_ms: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub bar_complete: bool,
}

/// Live tick data from WebSocket
#[derive(Debug, Clone)]
pub struct Tick {
    pub symbol: String,
    pub token: String,
    pub ltp: f64,
    pub bid: f64,
    pub ask: f64,
    pub volume: i64,
    pub timestamp: DateTime<Utc>,
    pub timestamp_ms: i64,
}

/// Position data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_id: String,
    pub symbol: String,
    pub underlying: String,
    pub strike: i32,
    pub option_type: OptionType,
    pub side: Side,
    pub quantity: i32,
    pub entry_price: f64,
    pub entry_time: DateTime<Utc>,
    pub entry_time_ms: i64,
    pub underlying_entry: f64,
    pub stop_loss: f64,
    pub target: Option<f64>,
    pub trailing_stop: Option<f64>,
    pub trailing_active: bool,
    pub current_price: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub status: PositionStatus,
    pub entry_reason: String,
    pub idempotency_key: String,
}

/// Order data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub broker_order_id: Option<String>,
    pub position_id: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: i32,
    pub limit_price: Option<f64>,
    pub fill_price: Option<f64>,
    pub fill_quantity: i32,
    pub fill_time: Option<DateTime<Utc>>,
    pub status: OrderStatus,
    pub attempts: u32,
    pub retry_count: u32,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Trade result (completed position)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub position_id: String,
    pub symbol: String,
    pub underlying: String,
    pub strike: i32,
    pub option_type: OptionType,
    pub quantity: i32,
    pub entry_time: DateTime<Utc>,
    pub entry_price: f64,
    pub entry_reason: String,
    pub exit_time: DateTime<Utc>,
    pub exit_price: f64,
    pub exit_reason: String,
    pub secondary_reasons: Vec<String>,
    pub pnl_gross: f64,
    pub pnl_gross_pct: f64,
    pub pnl_net: f64,
    pub brokerage: f64,
    pub duration_sec: i64,
    pub high_price: f64,
    pub low_price: f64,
    pub vix_at_entry: f64,
    pub vix_at_exit: f64,
}

/// Option type (Call or Put)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    CE,  // Call European
    PE,  // Put European
}

impl OptionType {
    pub fn as_str(&self) -> &str {
        match self {
            OptionType::CE => "CE",
            OptionType::PE => "PE",
        }
    }
}

/// Trade side (Buy or Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

impl OrderType {
    pub fn as_str(&self) -> &str {
        match self {
            OrderType::Limit => "LIMIT",
            OrderType::Market => "MARKET",
        }
    }
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Submitted,
    PartiallyFilled,
    Filled,
    Rejected,
    Cancelled,
    Failed,
}

/// Position status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closing,
    Closed,
}

/// Daily direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    CE,
    PE,
    NoTrade,
}

impl Direction {
    pub fn as_str(&self) -> &str {
        match self {
            Direction::CE => "CE",
            Direction::PE => "PE",
            Direction::NoTrade => "NO_TRADE",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CE" => Some(Direction::CE),
            "PE" => Some(Direction::PE),
            "NO_TRADE" => Some(Direction::NoTrade),
            _ => None,
        }
    }
}

/// Market session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    PreOpen,
    Open,
    Closed,
    PostMarket,
}

impl SessionState {
    pub fn as_str(&self) -> &str {
        match self {
            SessionState::PreOpen => "PREOPEN",
            SessionState::Open => "OPEN",
            SessionState::Closed => "CLOSED",
            SessionState::PostMarket => "POST_MARKET",
        }
    }
}

/// Exit priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExitPriority {
    Mandatory = 1,
    Risk = 2,
    Profit = 3,
    Technical = 4,
}

/// Instrument data from broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub token: String,
    pub symbol: String,
    pub name: String,
    pub expiry: String,
    pub strike: f64,
    pub lotsize: i32,
    pub instrument_type: String,
    pub exch_seg: String,
    pub tick_size: f64,
}

/// Configuration for the trading bot
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Time Windows
    pub entry_window_start: String,
    pub entry_window_end: String,
    pub eod_exit_time: String,
    pub market_close_time: String,
    
    // Bar Processing
    pub bar_ready_grace_sec: u64,
    
    // Risk Parameters
    pub option_stop_loss_pct: f64,
    pub trail_activate_pnl_pct: f64,
    pub trail_gap_pct: f64,
    pub max_positions: usize,
    pub daily_loss_limit_pct: f64,
    pub consecutive_loss_limit: usize,
    
    // VIX Circuit Breaker
    pub vix_threshold: f64,
    pub vix_spike_threshold: f64,
    pub vix_resume_threshold: f64,
    
    // Position Sizing
    pub base_position_size_pct: f64,
    pub vix_mult_anchors: VixMultipliers,
    pub dte_mult: DteMultipliers,
    
    // Order Retry
    pub order_retry_steps_pct: Vec<f64>,
    pub order_max_retries: u32,
    pub order_retry_backoffs_sec: Vec<u64>,
    pub retry_cap_sec: u64,
    
    // Token Management
    pub token_expiry_warning_min: i64,
    pub token_grace_to_flatten_sec: u64,
    pub token_check_interval_sec: u64,
    
    // Data Quality
    pub data_gap_threshold_sec: u64,
    pub data_gap_check_interval_sec: u64,
    pub recovery_timeout_sec: u64,
    
    // Broker Constraints
    pub freeze_quantity: BrokerLimits,
    pub lot_size: LotSizes,
    pub tick_size: f64,
    pub price_band_pct: f64,
    
    // Rate Limiting
    pub rate_limit_orders: u32,
    pub rate_limit_market_data: u32,
    pub rate_limit_historical: u32,
    
    // WebSocket
    pub ws_ping_interval_sec: u64,
    pub ws_pong_timeout_sec: u64,
    pub ws_reconnect_backoff_sec: Vec<u64>,
    pub ws_max_reconnects_per_minute: u32,
    
    // Strategy
    pub daily_adx_period: usize,
    pub daily_adx_threshold: f64,
    pub hourly_adx_period: usize,
    pub hourly_adx_threshold: f64,
    pub rsi_period: usize,
    pub rsi_oversold: f64,
    pub rsi_overbought: f64,
    pub ema_period: usize,
    
    // Strike Selection
    pub strike_increment: i32,
    pub initial_strike_range: i32,
    pub strike_subscription_count: usize,
    
    // Feature Flags
    pub strategy_invalidate_on_recompute: bool,
    pub use_trailing_stop: bool,
    pub use_underlying_soft_check: bool,
    pub enable_paper_trading: bool,
    
    // Logging
    pub log_level: String,
    pub log_rotation: String,
    pub log_retention_days: u32,
    pub audit_trail_enabled: bool,
    
    // Broker Credentials
    pub angel_one_client_code: String,
    pub angel_one_password: String,
    pub angel_one_mpin: Option<String>,
    pub angel_one_totp_secret: String,
    pub angel_one_api_key: String,
    pub angel_one_secret_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VixMultipliers {
    pub vix_12_or_below: f64,
    pub vix_20: f64,
    pub vix_30: f64,
    pub vix_30_or_above: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DteMultipliers {
    pub gte_5_days: f64,
    pub days_2_to_4: f64,
    pub day_1: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerLimits {
    pub nifty: i32,
    pub banknifty: i32,
    pub finnifty: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LotSizes {
    pub nifty: i32,
    pub banknifty: i32,
    pub finnifty: i32,
}

impl Config {
    pub fn get_lot_size(&self, underlying: &str) -> i32 {
        match underlying.to_uppercase().as_str() {
            "NIFTY" => self.lot_size.nifty,
            "BANKNIFTY" => self.lot_size.banknifty,
            "FINNIFTY" => self.lot_size.finnifty,
            _ => 50, // Default to NIFTY
        }
    }
    
    pub fn get_freeze_quantity(&self, underlying: &str) -> i32 {
        match underlying.to_uppercase().as_str() {
            "NIFTY" => self.freeze_quantity.nifty,
            "BANKNIFTY" => self.freeze_quantity.banknifty,
            "FINNIFTY" => self.freeze_quantity.finnifty,
            _ => 36000, // Default to NIFTY
        }
    }
}

