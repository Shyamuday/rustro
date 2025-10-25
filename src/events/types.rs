/// Event definitions following the spec
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::types::{Direction, OptionType, SessionState, Side};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub timestamp_ms: i64,
    pub idempotency_key: String,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Initialization Events
    LogInitialized,
    ConfigLoaded,
    StorageReady,
    CredentialsLoaded,
    LoginApiCalled,
    TokenLoaded,
    TokenNotFound,
    TokensStored,
    TokenMonitorActive,
    BrokerClientReady,
    
    // Session Management
    TradingDayCheck,
    CalendarValidated,
    MarketSessionDetermined,
    MarketOpen,
    EntryWindowOpen,
    SessionRevalidationRequired,
    NoTradeModeActive,
    
    // Token Management
    TokenExpiryWarning,
    TokenInvalid,
    TokenRefreshStarted,
    TokenRefreshSuccess,
    TokenRefreshFailed,
    
    // Data Collection
    InstrumentMasterDownloaded,
    SubscriptionsInitialized,
    WebSocketConnected,
    WebSocketDisconnected,
    TickReceived,
    BarReady,
    DataGapDetected,
    DataGapRecoveryRequired,
    RecoveryStarted,
    RecoveryCompleted,
    RecoveryFailed,
    
    // Analysis & Strategy
    DailyAnalysisRequired,
    DailyDirectionDetermined,
    HourlyAnalysisRequired,
    HourlyAlignmentConfirmed,
    AlignmentLost,
    EntryFiltersEvaluated,
    SignalGenerated,
    NoTradeSignal,
    
    // Risk Management
    VixDataReceived,
    VixSpike,
    VixNormalResumed,
    DailyLossLimitBreached,
    RiskCheckPassed,
    RiskCheckFailed,
    
    // Orders & Positions
    OrderIntentCreated,
    OrderPlaced,
    OrderExecuted,
    OrderPartiallyFilled,
    OrderRejected,
    OrderFailed,
    OrderRetrying,
    PositionOpened,
    PositionUpdated,
    
    // Exit Management
    ExitSignalGenerated,
    StopLossTriggered,
    TrailingStopActivated,
    TrailingStopUpdated,
    TargetReached,
    EodMandatoryExit,
    PositionClosed,
    PositionsClosed,
    
    // System Events
    GracefulShutdownInitiated,
    ShutdownCompleted,
    FatalError,
    KillSwitchActivated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventPayload {
    // Initialization
    LogInitialized {
        log_level: String,
    },
    ConfigLoaded {
        config_hash: String,
        data_paths: Vec<String>,
    },
    StorageReady {
        data_root: String,
    },
    CredentialsLoaded {
        user_id: String,
    },
    LoginApiCalled {
        attempt_id: String,
    },
    TokenLoaded {
        jwt_expiry: DateTime<Utc>,
        feed_expiry: DateTime<Utc>,
    },
    TokensStored {
        stored_at: DateTime<Utc>,
    },
    TokenMonitorActive {
        check_interval_sec: u64,
    },
    BrokerClientReady {
        session_id: String,
    },
    
    // Session
    TradingDayCheck {
        date: String,
    },
    CalendarValidated {
        is_trading_day: bool,
        holidays: Vec<String>,
    },
    MarketSessionDetermined {
        session_state: SessionState,
        open_time: String,
        close_time: String,
    },
    MarketOpen {
        market_open_time: DateTime<Utc>,
    },
    EntryWindowOpen {
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    },
    SessionRevalidationRequired {
        reason: String,
    },
    NoTradeModeActive {
        reason: String,
    },
    
    // Token
    TokenExpiryWarning {
        expires_at: DateTime<Utc>,
        minutes_remaining: i64,
    },
    TokenInvalid {
        reason: String,
    },
    TokenRefreshStarted {
        attempt: u32,
    },
    TokenRefreshSuccess {
        new_expiry: DateTime<Utc>,
    },
    TokenRefreshFailed {
        reason: String,
        attempts: u32,
    },
    
    // Data
    InstrumentMasterDownloaded {
        instrument_count: usize,
        file_path: String,
    },
    SubscriptionsInitialized {
        symbols: Vec<String>,
        token_count: usize,
    },
    WebSocketConnected {
        connection_id: String,
    },
    WebSocketDisconnected {
        reason: String,
        reconnect_attempt: u32,
    },
    TickReceived {
        symbol: String,
        ltp: f64,
        volume: i64,
    },
    BarReady {
        symbol: String,
        timeframe: String,
        bar_time: DateTime<Utc>,
        bar_complete: bool,
    },
    DataGapDetected {
        symbol: String,
        timeframe: String,
        gap_start: DateTime<Utc>,
        gap_end: DateTime<Utc>,
        missing_bars: usize,
    },
    DataGapRecoveryRequired {
        symbol: String,
        timeframe: String,
    },
    RecoveryStarted {
        symbol: String,
        timeframe: String,
        bars_to_fetch: usize,
    },
    RecoveryCompleted {
        symbol: String,
        timeframe: String,
        bars_recovered: usize,
    },
    RecoveryFailed {
        symbol: String,
        reason: String,
    },
    
    // Analysis
    DailyAnalysisRequired {
        symbol: String,
        bar_time: DateTime<Utc>,
    },
    DailyDirectionDetermined {
        symbol: String,
        direction: Direction,
        daily_adx: f64,
        daily_plus_di: f64,
        daily_minus_di: f64,
        reason: String,
    },
    HourlyAnalysisRequired {
        symbol: String,
        bar_time: DateTime<Utc>,
    },
    HourlyAlignmentConfirmed {
        symbol: String,
        hourly_adx: f64,
        hourly_plus_di: f64,
        hourly_minus_di: f64,
        alignment_score: f64,
    },
    AlignmentLost {
        symbol: String,
        reason: String,
    },
    EntryFiltersEvaluated {
        symbol: String,
        passed: bool,
        filter_results: Vec<(String, bool)>,
    },
    SignalGenerated {
        symbol: String,
        underlying: String,
        direction: Direction,
        strike: i32,
        option_type: OptionType,
        side: Side,
        reason: String,
        underlying_ltp: f64,
        option_ltp: f64,
        vix: f64,
    },
    NoTradeSignal {
        symbol: String,
        reason: String,
    },
    
    // Risk
    VixDataReceived {
        vix: f64,
        timestamp: DateTime<Utc>,
    },
    VixSpike {
        vix: f64,
        threshold: f64,
        positions_to_exit: Vec<String>,
    },
    VixNormalResumed {
        vix: f64,
        threshold: f64,
    },
    DailyLossLimitBreached {
        daily_pnl: f64,
        limit: f64,
        positions_to_close: Vec<String>,
    },
    RiskCheckPassed {
        check_type: String,
    },
    RiskCheckFailed {
        check_type: String,
        reason: String,
    },
    
    // Orders
    OrderIntentCreated {
        order_id: String,
        symbol: String,
        side: Side,
        quantity: i32,
        intent_reason: String,
    },
    OrderPlaced {
        order_id: String,
        broker_order_id: String,
        symbol: String,
        quantity: i32,
        price: f64,
    },
    OrderExecuted {
        order_id: String,
        broker_order_id: String,
        fill_price: f64,
        fill_quantity: i32,
        fill_time: DateTime<Utc>,
    },
    OrderPartiallyFilled {
        order_id: String,
        filled_quantity: i32,
        remaining_quantity: i32,
    },
    OrderRejected {
        order_id: String,
        reason: String,
        broker_message: String,
    },
    OrderFailed {
        order_id: String,
        reason: String,
        retry_count: u32,
    },
    OrderRetrying {
        order_id: String,
        attempt: u32,
        max_retries: u32,
        backoff_sec: u64,
    },
    PositionOpened {
        position_id: String,
        symbol: String,
        quantity: i32,
        entry_price: f64,
        entry_reason: String,
    },
    PositionUpdated {
        position_id: String,
        current_price: f64,
        pnl: f64,
        pnl_pct: f64,
    },
    
    // Exit
    ExitSignalGenerated {
        position_id: String,
        primary_reason: String,
        secondary_reasons: Vec<String>,
        priority: u8,
    },
    StopLossTriggered {
        position_id: String,
        stop_loss: f64,
        current_price: f64,
    },
    TrailingStopActivated {
        position_id: String,
        activation_pnl_pct: f64,
        trail_stop: f64,
    },
    TrailingStopUpdated {
        position_id: String,
        new_trail_stop: f64,
        high_price: f64,
    },
    TargetReached {
        position_id: String,
        target: f64,
        current_price: f64,
    },
    EodMandatoryExit {
        time: DateTime<Utc>,
        positions_to_close: Vec<String>,
    },
    PositionClosed {
        position_id: String,
        exit_price: f64,
        exit_reason: String,
        pnl_gross: f64,
        pnl_gross_pct: f64,
    },
    PositionsClosed {
        position_ids: Vec<String>,
        reason: String,
    },
    
    // System
    GracefulShutdownInitiated {
        reason: String,
    },
    ShutdownCompleted {
        duration_sec: u64,
    },
    FatalError {
        error_code: String,
        message: String,
    },
    KillSwitchActivated {
        reason: String,
        manual: bool,
    },
    
    // Generic
    Empty,
}

impl Event {
    pub fn new(event_type: EventType, payload: EventPayload) -> Self {
        let now = Utc::now();
        let idempotency_key = format!(
            "{}:{}:{}",
            event_type.as_str(),
            now.timestamp_millis(),
            uuid::Uuid::new_v4()
        );
        
        Event {
            event_type,
            timestamp: now,
            timestamp_ms: now.timestamp_millis(),
            idempotency_key,
            payload,
        }
    }
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::LogInitialized => "LOG_INITIALIZED",
            EventType::ConfigLoaded => "CONFIG_LOADED",
            EventType::StorageReady => "STORAGE_READY",
            EventType::CredentialsLoaded => "CREDENTIALS_LOADED",
            EventType::LoginApiCalled => "LOGIN_API_CALLED",
            EventType::TokenLoaded => "TOKEN_LOADED",
            EventType::TokenNotFound => "TOKEN_NOT_FOUND",
            EventType::TokensStored => "TOKENS_STORED",
            EventType::TokenMonitorActive => "TOKEN_MONITOR_ACTIVE",
            EventType::BrokerClientReady => "BROKER_CLIENT_READY",
            EventType::TradingDayCheck => "TRADING_DAY_CHECK",
            EventType::CalendarValidated => "CALENDAR_VALIDATED",
            EventType::MarketSessionDetermined => "MARKET_SESSION_DETERMINED",
            EventType::MarketOpen => "MARKET_OPEN",
            EventType::EntryWindowOpen => "ENTRY_WINDOW_OPEN",
            EventType::SessionRevalidationRequired => "SESSION_REVALIDATION_REQUIRED",
            EventType::NoTradeModeActive => "NO_TRADE_MODE_ACTIVE",
            EventType::TokenExpiryWarning => "TOKEN_EXPIRY_WARNING",
            EventType::TokenInvalid => "TOKEN_INVALID",
            EventType::TokenRefreshStarted => "TOKEN_REFRESH_STARTED",
            EventType::TokenRefreshSuccess => "TOKEN_REFRESH_SUCCESS",
            EventType::TokenRefreshFailed => "TOKEN_REFRESH_FAILED",
            EventType::InstrumentMasterDownloaded => "INSTRUMENT_MASTER_DOWNLOADED",
            EventType::SubscriptionsInitialized => "SUBSCRIPTIONS_INITIALIZED",
            EventType::WebSocketConnected => "WEBSOCKET_CONNECTED",
            EventType::WebSocketDisconnected => "WEBSOCKET_DISCONNECTED",
            EventType::TickReceived => "TICK_RECEIVED",
            EventType::BarReady => "BAR_READY",
            EventType::DataGapDetected => "DATA_GAP_DETECTED",
            EventType::DataGapRecoveryRequired => "DATA_GAP_RECOVERY_REQUIRED",
            EventType::RecoveryStarted => "RECOVERY_STARTED",
            EventType::RecoveryCompleted => "RECOVERY_COMPLETED",
            EventType::RecoveryFailed => "RECOVERY_FAILED",
            EventType::DailyAnalysisRequired => "DAILY_ANALYSIS_REQUIRED",
            EventType::DailyDirectionDetermined => "DAILY_DIRECTION_DETERMINED",
            EventType::HourlyAnalysisRequired => "HOURLY_ANALYSIS_REQUIRED",
            EventType::HourlyAlignmentConfirmed => "HOURLY_ALIGNMENT_CONFIRMED",
            EventType::AlignmentLost => "ALIGNMENT_LOST",
            EventType::EntryFiltersEvaluated => "ENTRY_FILTERS_EVALUATED",
            EventType::SignalGenerated => "SIGNAL_GENERATED",
            EventType::NoTradeSignal => "NO_TRADE_SIGNAL",
            EventType::VixDataReceived => "VIX_DATA_RECEIVED",
            EventType::VixSpike => "VIX_SPIKE",
            EventType::VixNormalResumed => "VIX_NORMAL_RESUMED",
            EventType::DailyLossLimitBreached => "DAILY_LOSS_LIMIT_BREACHED",
            EventType::RiskCheckPassed => "RISK_CHECK_PASSED",
            EventType::RiskCheckFailed => "RISK_CHECK_FAILED",
            EventType::OrderIntentCreated => "ORDER_INTENT_CREATED",
            EventType::OrderPlaced => "ORDER_PLACED",
            EventType::OrderExecuted => "ORDER_EXECUTED",
            EventType::OrderPartiallyFilled => "ORDER_PARTIALLY_FILLED",
            EventType::OrderRejected => "ORDER_REJECTED",
            EventType::OrderFailed => "ORDER_FAILED",
            EventType::OrderRetrying => "ORDER_RETRYING",
            EventType::PositionOpened => "POSITION_OPENED",
            EventType::PositionUpdated => "POSITION_UPDATED",
            EventType::ExitSignalGenerated => "EXIT_SIGNAL_GENERATED",
            EventType::StopLossTriggered => "STOP_LOSS_TRIGGERED",
            EventType::TrailingStopActivated => "TRAILING_STOP_ACTIVATED",
            EventType::TrailingStopUpdated => "TRAILING_STOP_UPDATED",
            EventType::TargetReached => "TARGET_REACHED",
            EventType::EodMandatoryExit => "EOD_MANDATORY_EXIT",
            EventType::PositionClosed => "POSITION_CLOSED",
            EventType::PositionsClosed => "POSITIONS_CLOSED",
            EventType::GracefulShutdownInitiated => "GRACEFUL_SHUTDOWN_INITIATED",
            EventType::ShutdownCompleted => "SHUTDOWN_COMPLETED",
            EventType::FatalError => "FATAL_ERROR",
            EventType::KillSwitchActivated => "KILL_SWITCH_ACTIVATED",
        }
    }
}
