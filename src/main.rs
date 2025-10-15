/// Main entry point for the trading bot
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_subscriber;

use rustro::{
    broker::{AngelOneClient, AngelWebSocket, InstrumentCache, PaperTradingBroker, TokenManager},
    config::load_config,
    data::{ConcurrentBarStore, MultiBarAggregator, Timeframe},
    error::{Result, TradingError},
    events::{Event, EventBus, EventPayload, EventType},
    orders::{OrderManager, OrderValidator},
    positions::{Position, PositionManager, PositionStatus},
    risk::RiskManager,
    strategy::{indicators::round_to_strike, AdxStrategy, EntrySignal},
    time::{get_market_timings, holidays::is_trading_day as is_trading_day_with_holidays},
    utils::*,
    Bar, Config, Direction, OptionType, Side,
};

/// Application state
pub struct TradingApp {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    token_manager: Arc<TokenManager>,
    broker_client: Arc<AngelOneClient>,
    paper_broker: Option<Arc<PaperTradingBroker>>,
    websocket: Option<Arc<AngelWebSocket>>,
    bar_aggregator: Arc<MultiBarAggregator>,
    instrument_cache: Arc<InstrumentCache>,
    order_validator: Arc<OrderValidator>,
    strategy: Arc<AdxStrategy>,
    order_manager: Arc<OrderManager>,
    position_manager: Arc<PositionManager>,
    risk_manager: Arc<RiskManager>,
    
    // Bar stores
    daily_bars: Arc<ConcurrentBarStore>,
    hourly_bars: Arc<ConcurrentBarStore>,
    
    // State
    session_uuid: String,
    nifty_token: Arc<RwLock<Option<String>>>,
    daily_analysis_done: Arc<RwLock<bool>>,
    last_hourly_check: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
    shutdown: Arc<RwLock<bool>>,
}

impl TradingApp {
    pub async fn new(config_path: &str) -> Result<Self> {
        // Initialize logging
        tracing_subscriber::fmt()
            .with_env_filter("rustro=info")
            .init();
        
        info!("🚀 Starting Rustro Trading Bot...");
        
        // Load configuration
        let config = Arc::new(load_config(config_path)?);
        info!("✅ Configuration loaded");
        
        // Create data directory
        tokio::fs::create_dir_all("data").await.ok();
        
        // Create event bus
        let event_bus = Arc::new(EventBus::new("data/events.jsonl".to_string()));
        event_bus.start_processing().await;
        
        event_bus.publish(Event::new(
            EventType::LogInitialized,
            EventPayload::LogInitialized {
                log_level: config.log_level.clone(),
            },
        )).await?;
        
        // Create token manager
        let token_manager = Arc::new(TokenManager::new("data/tokens.json".to_string()));
        
        // Create broker client
        let broker_client = Arc::new(AngelOneClient::new(
            Arc::clone(&token_manager),
            config.angel_one_client_code.clone(),
            config.angel_one_password.clone(),
            config.angel_one_totp_secret.clone(),
        ));
        
        // Create paper trading broker if enabled
        let paper_broker = if config.enable_paper_trading {
            info!("📝 Paper trading mode ENABLED");
            Some(Arc::new(PaperTradingBroker::new(true, 5.0))) // Auto-fill with 5bps slippage
        } else {
            info!("💰 Live trading mode");
            None
        };
        
        // Create WebSocket client (optional - can use REST fallback)
        let websocket = if !config.enable_paper_trading {
            info!("📡 WebSocket enabled for real-time data");
            Some(Arc::new(AngelWebSocket::new(Arc::clone(&token_manager))))
        } else {
            info!("📝 Paper mode - WebSocket disabled");
            None
        };
        
        // Create bar aggregator
        let bar_aggregator = Arc::new(MultiBarAggregator::new(Arc::clone(&event_bus)));
        
        // Create instrument cache
        let instrument_cache = Arc::new(InstrumentCache::new(Arc::clone(&broker_client)));
        
        // Create order validator
        let order_validator = Arc::new(OrderValidator::new(Arc::clone(&config)));
        
        // Create managers
        let strategy = Arc::new(AdxStrategy::new(Arc::clone(&config)));
        let order_manager = Arc::new(OrderManager::new(
            Arc::clone(&broker_client),
            Arc::clone(&event_bus),
            Arc::clone(&config),
        ));
        let position_manager = Arc::new(PositionManager::new(
            Arc::clone(&event_bus),
            Arc::clone(&config),
        ));
        let risk_manager = Arc::new(RiskManager::new(
            Arc::clone(&event_bus),
            Arc::clone(&config),
            Arc::clone(&position_manager),
        ));
        
        // Create bar stores
        let daily_bars = Arc::new(ConcurrentBarStore::new(
            "NIFTY".to_string(),
            "1d".to_string(),
            PathBuf::from("data/bars_nifty_daily.jsonl"),
            100, // Keep 100 days in memory
        ));
        
        let hourly_bars = Arc::new(ConcurrentBarStore::new(
            "NIFTY".to_string(),
            "1h".to_string(),
            PathBuf::from("data/bars_nifty_hourly.jsonl"),
            500, // Keep 500 hours in memory
        ));
        
        // Load existing bars from disk
        daily_bars.load_from_disk(100).await.ok();
        hourly_bars.load_from_disk(500).await.ok();
        
        let session_uuid = uuid::Uuid::new_v4().to_string();
        
        Ok(TradingApp {
            config,
            event_bus,
            token_manager,
            broker_client,
            paper_broker,
            websocket,
            bar_aggregator,
            instrument_cache,
            order_validator,
            strategy,
            order_manager,
            position_manager,
            risk_manager,
            daily_bars,
            hourly_bars,
            session_uuid,
            nifty_token: Arc::new(RwLock::new(None)),
            daily_analysis_done: Arc::new(RwLock::new(false)),
            last_hourly_check: Arc::new(RwLock::new(None)),
            shutdown: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the trading bot
    pub async fn run(&self) -> Result<()> {
        info!("🏁 Trading bot starting main loop...");
        
        // Setup graceful shutdown handler
        self.setup_shutdown_handler().await;
        
        // Initialize session (authenticate)
        self.initialize_session().await?;
        
        // Main trading loop
        loop {
            // Check shutdown flag
            {
                let shutdown = self.shutdown.read().await;
                if *shutdown {
                    info!("🛑 Shutdown signal received");
                    break;
                }
            }
            
            let now = chrono::Utc::now();
            let today = now.date_naive();
            
            // Check if today is a trading day (includes NSE holidays)
            if !is_trading_day_with_holidays(today) {
                info!("📅 Today is not a trading day (weekend or holiday) - waiting");
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                continue;
            }
            
            // Check if market is open
            let (market_open, market_close) = get_market_timings(now);
            
            if now < market_open {
                let wait_secs = (market_open - now).num_seconds().max(0) as u64;
                info!("⏰ Market opens at {} IST - waiting {} minutes", 
                      market_open.format("%H:%M:%S"),
                      wait_secs / 60);
                tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs.min(300))).await;
                continue;
            }
            
            if now >= market_close {
                info!("🌙 Market closed for the day");
                // Reset for next day
                self.end_of_day_sequence().await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                continue;
            }
            
            // Market is OPEN - run trading cycle
            if let Err(e) = self.run_trading_cycle().await {
                error!("❌ Trading cycle error: {} ({})", e, e.error_code());
                
                if e.is_fatal() {
                    error!("💀 Fatal error - initiating shutdown");
                    break;
                }
                
                if e.requires_exit() {
                    warn!("⚠️  Risk event requires position exit");
                    let _ = self.position_manager.close_all_positions(e.to_string()).await;
                }
            }
            
            // Sleep before next cycle (1 minute intervals)
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
        
        // Shutdown sequence
        self.shutdown_sequence().await?;
        
        Ok(())
    }
    
    /// Start tick processing loop
    async fn start_tick_processing(&self) {
        if let Some(ws) = &self.websocket {
            let rx = ws.get_tick_receiver();
            let aggregator = Arc::clone(&self.bar_aggregator);
            
            tokio::spawn(async move {
                let mut rx = rx.write().await;
                
                while let Some(tick) = rx.recv().await {
                    // Process tick through bar aggregators
                    if let Err(e) = aggregator.process_tick(tick).await {
                        error!("Error processing tick: {}", e);
                    }
                }
                
                warn!("Tick processing loop ended");
            });
            
            info!("✅ Tick processing loop started");
        }
    }
    
    /// Initialize session (authentication, data loading)
    async fn initialize_session(&self) -> Result<()> {
        info!("🔐 Initializing session...");
        
        // Try to load existing tokens
        match self.token_manager.load_from_file().await {
            Ok(_) => {
                if self.token_manager.is_valid().await {
                    info!("✅ Valid tokens loaded from file");
                    let tokens = self.token_manager.get_tokens().await.unwrap();
                    info!("🔑 Tokens expire at: {}", tokens.jwt_expiry);
                } else {
                    info!("🔄 Tokens expired - logging in");
                    self.broker_client.login().await?;
                }
            }
            Err(_) => {
                info!("🆕 No tokens found - logging in");
                self.broker_client.login().await?;
            }
        }
        
        self.event_bus.publish(Event::new(
            EventType::BrokerClientReady,
            EventPayload::BrokerClientReady {
                session_id: self.session_uuid.clone(),
            },
        )).await?;
        
        // Download instrument master
        if self.instrument_cache.needs_refresh().await {
            info!("📥 Downloading instrument master...");
            self.instrument_cache.refresh().await?;
            
            self.event_bus.publish(Event::new(
                EventType::InstrumentMasterDownloaded,
                EventPayload::InstrumentMasterDownloaded {
                    instrument_count: self.instrument_cache.size().await,
                    file_path: "memory".to_string(),
                },
            )).await?;
        }
        
        // Get NIFTY token
        let nifty_token = self.instrument_cache.get_nifty_token().await?;
        {
            let mut token = self.nifty_token.write().await;
            *token = Some(nifty_token.clone());
        }
        info!("✅ NIFTY token: {}", nifty_token);
        
        // Setup bar aggregators
        self.bar_aggregator.add_aggregator(
            "NIFTY".to_string(),
            Timeframe::OneHour,
            Arc::clone(&self.hourly_bars),
        ).await;
        
        self.bar_aggregator.add_aggregator(
            "NIFTY".to_string(),
            Timeframe::OneDay,
            Arc::clone(&self.daily_bars),
        ).await;
        
        // Connect WebSocket if available
        if let Some(ws) = &self.websocket {
            match ws.connect().await {
                Ok(_) => {
                    // Subscribe to NIFTY
                    ws.subscribe(vec![nifty_token.clone()], "NFO").await?;
                    
                    // Start tick processing loop
                    self.start_tick_processing().await;
                    
                    info!("✅ WebSocket connected and subscribed");
                }
                Err(e) => {
                    warn!("⚠️  WebSocket connection failed: {} - using REST fallback", e);
                }
            }
        }
        
        info!("✅ Session initialized successfully");
        Ok(())
    }
    
    /// Run one trading cycle
    async fn run_trading_cycle(&self) -> Result<()> {
        let now = chrono::Utc::now();
        let now_ist = now.with_timezone(&chrono_tz::Asia::Kolkata);
        
        // Step 1: Fetch latest data
        self.fetch_and_update_bars().await?;
        
        // Step 2: Daily analysis (runs once at 9:30 AM)
        if now_ist.hour() >= 9 && now_ist.minute() >= 30 {
            let daily_done = self.daily_analysis_done.read().await;
            if !*daily_done {
                drop(daily_done);
                self.run_daily_analysis().await?;
            }
        }
        
        // Step 3: Hourly analysis (runs every hour after bar completes)
        if now_ist.minute() >= 15 {
            let last_check = self.last_hourly_check.read().await;
            let should_run = match *last_check {
                None => true,
                Some(last_time) => {
                    let diff = (now - last_time).num_minutes();
                    diff >= 60 // Run if 60+ minutes since last check
                }
            };
            
            if should_run {
                drop(last_check);
                self.run_hourly_analysis().await?;
            }
        }
        
        // Step 4: Update open positions
        self.update_positions().await?;
        
        // Step 5: Check EOD exit (3:20 PM)
        if now_ist.hour() == 15 && now_ist.minute() >= 20 {
            self.eod_exit_positions().await?;
        }
        
        Ok(())
    }
    
    /// Fetch latest bars from broker
    async fn fetch_and_update_bars(&self) -> Result<()> {
        // Note: In production, you would:
        // 1. Get NIFTY token from instrument master
        // 2. Fetch recent candles
        // 3. Update bar stores
        
        // For now, using placeholder - would need actual NIFTY token
        // let candles = self.broker_client.get_candles("99926000", "ONE_HOUR", from, to).await?;
        
        Ok(())
    }
    
    /// Run daily direction analysis
    async fn run_daily_analysis(&self) -> Result<()> {
        info!("📊 Running daily direction analysis...");
        
        let daily_bars_vec = self.daily_bars.get_recent(30).await?;
        
        if daily_bars_vec.len() < self.config.daily_adx_period {
            warn!("⚠️  Insufficient daily bars for analysis");
            return Ok(());
        }
        
        let direction = self.strategy.analyze_daily(&daily_bars_vec).await?;
        
        self.event_bus.publish(Event::new(
            EventType::DailyDirectionDetermined,
            EventPayload::DailyDirectionDetermined {
                symbol: "NIFTY".to_string(),
                direction,
                daily_adx: 0.0, // Would be from actual calculation
                daily_plus_di: 0.0,
                daily_minus_di: 0.0,
                reason: format!("Daily direction: {}", direction.as_str()),
            },
        )).await?;
        
        let mut done = self.daily_analysis_done.write().await;
        *done = true;
        
        info!("✅ Daily direction determined: {}", direction.as_str());
        
        Ok(())
    }
    
    /// Run hourly alignment check and entry logic
    async fn run_hourly_analysis(&self) -> Result<()> {
        info!("🔍 Running hourly analysis...");
        
        let hourly_bars_vec = self.hourly_bars.get_recent(30).await?;
        
        if hourly_bars_vec.len() < self.config.hourly_adx_period {
            warn!("⚠️  Insufficient hourly bars for analysis");
            return Ok(());
        }
        
        // Check alignment
        let aligned = self.strategy.analyze_hourly(&hourly_bars_vec).await?;
        
        if !aligned {
            info!("❌ Hourly not aligned with daily");
            let mut last_check = self.last_hourly_check.write().await;
            *last_check = Some(chrono::Utc::now());
            return Ok(());
        }
        
        // Check if we're in entry window
        let now = chrono::Utc::now();
        if !is_in_entry_window(now, &self.config.entry_window_start, &self.config.entry_window_end) {
            info!("⏰ Outside entry window");
            let mut last_check = self.last_hourly_check.write().await;
            *last_check = Some(chrono::Utc::now());
            return Ok(());
        }
        
        // Pre-entry risk check
        if let Err(e) = self.risk_manager.pre_entry_risk_check().await {
            warn!("⚠️  Risk check failed: {}", e);
            let mut last_check = self.last_hourly_check.write().await;
            *last_check = Some(chrono::Utc::now());
            return Ok(());
        }
        
        // Get current VIX (placeholder - would fetch from broker)
        let vix = self.risk_manager.get_current_vix().await.unwrap_or(20.0);
        
        // Get underlying LTP (placeholder - would fetch from broker)
        let underlying_ltp = 19500.0; // Placeholder
        
        // Evaluate entry
        if let Some(signal) = self.strategy.evaluate_entry(&hourly_bars_vec, underlying_ltp, vix).await? {
            info!("🎯 Entry signal generated!");
            self.execute_entry(signal).await?;
        }
        
        let mut last_check = self.last_hourly_check.write().await;
        *last_check = Some(chrono::Utc::now());
        
        Ok(())
    }
    
    /// Execute entry based on signal
    async fn execute_entry(&self, signal: EntrySignal) -> Result<()> {
        info!("📈 Executing entry: {:?} @ {}", signal.option_type, signal.strike);
        
        // Calculate position size
        let vix = self.risk_manager.get_current_vix().await.unwrap_or(20.0);
        let dte = calculate_days_to_expiry(chrono::Utc::now());
        let quantity = self.risk_manager.calculate_position_size(1_000_000.0, vix, dte);
        
        // Generate idempotency key
        let idempotency_key = generate_idempotency_key(&[
            &self.session_uuid,
            "NIFTY",
            signal.option_type.as_str(),
            &signal.strike.to_string(),
            &chrono::Utc::now().timestamp_millis().to_string(),
        ]);
        
        // Get actual token and symbol from instrument cache
        let (token, symbol) = self.instrument_cache
            .find_option_token("NIFTY", signal.strike, signal.option_type, None)
            .await?;
        
        info!("📍 Using instrument: {} (token: {})", symbol, token);
        
        // Placeholder option price
        let option_price = 125.0;
        
        // Place order (this will use retry logic automatically)
        let order_id = self.order_manager.place_order(
            symbol.clone(),
            token.to_string(),
            signal.side,
            quantity,
            option_price,
            idempotency_key.clone(),
        ).await?;
        
        info!("✅ Order placed: {}", order_id);
        
        // Create position
        let position = Position {
            position_id: order_id.clone(),
            symbol,
            underlying: "NIFTY".to_string(),
            strike: signal.strike,
            option_type: signal.option_type,
            side: signal.side,
            quantity,
            entry_price: option_price,
            entry_time: chrono::Utc::now(),
            entry_time_ms: chrono::Utc::now().timestamp_millis(),
            underlying_entry: signal.underlying_ltp,
            stop_loss: option_price * (1.0 - self.config.option_stop_loss_pct),
            target: None,
            trailing_stop: None,
            trailing_active: false,
            current_price: option_price,
            pnl: 0.0,
            pnl_pct: 0.0,
            status: PositionStatus::Open,
            entry_reason: signal.reason,
            idempotency_key,
        };
        
        self.position_manager.open_position(position).await?;
        
        Ok(())
    }
    
    /// Update open positions with current prices
    async fn update_positions(&self) -> Result<()> {
        let positions = self.position_manager.get_open_positions().await;
        
        for position in positions {
            // Fetch current price (placeholder - would fetch from broker)
            let current_price = position.entry_price * 1.02; // Placeholder: 2% up
            
            // Update position
            if let Some(exit_reason) = self.position_manager.update_position(
                &position.position_id,
                current_price,
            ).await? {
                // Exit signal generated
                info!("🚪 Exit signal for {}: {}", position.position_id, exit_reason);
                self.position_manager.close_position(
                    &position.position_id,
                    current_price,
                    exit_reason,
                ).await?;
            }
        }
        
        Ok(())
    }
    
    /// EOD mandatory exit
    async fn eod_exit_positions(&self) -> Result<()> {
        let positions = self.position_manager.get_open_positions().await;
        
        if positions.is_empty() {
            return Ok(());
        }
        
        info!("🌆 EOD: Closing {} open positions", positions.len());
        
        self.position_manager.close_all_positions("EOD_MANDATORY_EXIT".to_string()).await?;
        
        Ok(())
    }
    
    /// End of day cleanup
    async fn end_of_day_sequence(&self) -> Result<()> {
        info!("🌙 Running end of day sequence...");
        
        // Save trades
        let trades = self.position_manager.get_daily_trades().await;
        if !trades.is_empty() {
            let trades_json = serde_json::to_string_pretty(&trades)?;
            let filename = format!("data/trades_{}.json", chrono::Utc::now().format("%Y%m%d"));
            tokio::fs::write(filename, trades_json).await?;
            info!("💾 Saved {} trades", trades.len());
        }
        
        // Reset daily state
        {
            let mut done = self.daily_analysis_done.write().await;
            *done = false;
        }
        {
            let mut last_check = self.last_hourly_check.write().await;
            *last_check = None;
        }
        
        self.position_manager.reset_daily_pnl().await;
        self.risk_manager.reset_daily().await;
        self.strategy.reset().await;
        
        info!("✅ EOD sequence completed");
        Ok(())
    }
    
    /// Setup graceful shutdown handler
    async fn setup_shutdown_handler(&self) {
        let shutdown = Arc::clone(&self.shutdown);
        let event_bus = Arc::clone(&self.event_bus);
        
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            
            info!("⚠️  Ctrl+C received - initiating graceful shutdown");
            
            {
                let mut flag = shutdown.write().await;
                *flag = true;
            }
            
            let _ = event_bus.publish(Event::new(
                EventType::GracefulShutdownInitiated,
                EventPayload::GracefulShutdownInitiated {
                    reason: "User requested (Ctrl+C)".to_string(),
                },
            )).await;
        });
    }
    
    /// Shutdown sequence
    async fn shutdown_sequence(&self) -> Result<()> {
        info!("🛑 Starting shutdown sequence...");
        
        let start_time = chrono::Utc::now();
        
        // Close all open positions
        let open_positions = self.position_manager.get_open_positions().await;
        if !open_positions.is_empty() {
            warn!("⚠️  Closing {} open positions", open_positions.len());
            let _ = self.position_manager.close_all_positions("Shutdown".to_string()).await;
        }
        
        // Save daily trades
        let trades = self.position_manager.get_daily_trades().await;
        if !trades.is_empty() {
            let trades_json = serde_json::to_string_pretty(&trades)?;
            tokio::fs::write(
                format!("data/trades_{}.json", chrono::Utc::now().format("%Y%m%d")),
                trades_json
            ).await?;
            info!("💾 Saved {} trades", trades.len());
        }
        
        let duration = (chrono::Utc::now() - start_time).num_seconds() as u64;
        
        self.event_bus.publish(Event::new(
            EventType::ShutdownCompleted,
            EventPayload::ShutdownCompleted {
                duration_sec: duration,
            },
        )).await?;
        
        info!("✅ Shutdown completed in {}s", duration);
        info!("👋 Goodbye!");
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.toml".to_string());
    
    let app = TradingApp::new(&config_path).await?;
    
    app.run().await?;
    
    Ok(())
}
