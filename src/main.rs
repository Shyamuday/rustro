/// Main entry point for the trading bot
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_subscriber;
use chrono::Timelike;

use rustro::{
    broker::{AngelOneClient, AngelWebSocket, InstrumentCache, PaperTradingBroker, TokenExtractor, TokenManager},
    config::load_config,
    data::{ConcurrentBarStore, HistoricalDataSync, MultiBarAggregator, Timeframe},
    error::{Result, TradingError},
    events::{Event, EventBus, EventPayload, EventType},
    orders::{OrderManager, OrderValidator},
    positions::PositionManager,
    risk::RiskManager,
    strategy::{adx_strategy::EntrySignal, AdxStrategy, BiasDirection, DailyBias, DailyBiasCalculator, HourlyCrossoverMonitor},
    time::{get_market_timings, holidays::is_trading_day as is_trading_day_with_holidays},
    trading::PremarketSelector,
    utils::{calculate_days_to_expiry, generate_idempotency_key, is_in_entry_window},
    Config, Direction, OrderType, OptionType, Position, PositionStatus, Side,
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
    token_extractor: Arc<TokenExtractor>,
    _order_validator: Arc<OrderValidator>,
    strategy: Arc<AdxStrategy>,
    order_manager: Arc<OrderManager>,
    position_manager: Arc<PositionManager>,
    risk_manager: Arc<RiskManager>,
    
    // Hybrid strategy components
    daily_bias_calculator: Arc<DailyBiasCalculator>,
    premarket_selector: Arc<PremarketSelector>,
    hourly_crossover: Arc<HourlyCrossoverMonitor>,
    
    // Bar stores
    daily_bars: Arc<ConcurrentBarStore>,
    hourly_bars: Arc<ConcurrentBarStore>,
    
    // Historical data sync
    historical_sync: Arc<HistoricalDataSync>,
    
    // State
    session_uuid: String,
    nifty_token: Arc<RwLock<Option<String>>>,
    daily_biases: Arc<RwLock<Vec<DailyBias>>>,
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
            config.angel_one_mpin.clone(),
            config.angel_one_totp_secret.clone(),
            config.angel_one_api_key.clone(),
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
        
        // Create token extractor
        let token_extractor = Arc::new(TokenExtractor::new(Vec::new())); // Will be updated after instrument download
        
        // Create order validator
        let _order_validator = Arc::new(OrderValidator::new(Arc::clone(&config)));
        
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
        
        // Create hybrid strategy components
        let daily_bias_calculator = Arc::new(DailyBiasCalculator::new(
            config.daily_adx_period,
            config.daily_adx_threshold,
        ));
        let premarket_selector = Arc::new(PremarketSelector::new(Arc::clone(&token_extractor)));
        let hourly_crossover = Arc::new(HourlyCrossoverMonitor::new(
            config.hourly_adx_period,
            config.hourly_adx_threshold,
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
        
        // Create historical data sync
        let historical_sync = Arc::new(HistoricalDataSync::new(
            Arc::clone(&broker_client),
            Arc::clone(&instrument_cache),
            Arc::clone(&daily_bars),
            Arc::clone(&hourly_bars),
            Arc::clone(&config),
        ));
        
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
            token_extractor,
            _order_validator,
            strategy,
            order_manager,
            position_manager,
            risk_manager,
            daily_bias_calculator,
            premarket_selector,
            hourly_crossover,
            daily_bars,
            hourly_bars,
            historical_sync,
            session_uuid,
            nifty_token: Arc::new(RwLock::new(None)),
            daily_biases: Arc::new(RwLock::new(Vec::new())),
            daily_analysis_done: Arc::new(RwLock::new(false)),
            last_hourly_check: Arc::new(RwLock::new(None)),
            shutdown: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Setup event subscriptions for auto-triggering
    async fn setup_event_subscriptions(&self) {
        info!("📡 Setting up event subscriptions...");
        
        // Subscribe BarReady → Check for hourly crossover
        let hourly_crossover = Arc::clone(&self.hourly_crossover);
        let daily_biases = Arc::clone(&self.daily_biases);
        let event_bus = Arc::clone(&self.event_bus);
        
        self.event_bus.subscribe(
            EventType::BarReady,
            Arc::new(move |event| {
                let hourly_crossover = Arc::clone(&hourly_crossover);
                let daily_biases = Arc::clone(&daily_biases);
                let event_bus = Arc::clone(&event_bus);
                
                Box::pin(async move {
                    if let EventPayload::BarReady { symbol, timeframe, .. } = &event.payload {
                        if timeframe == "1h" {
                            info!("⏰ Hourly bar ready for {}, checking crossovers...", symbol);
                            
                            // Check crossovers for all biased underlyings
                            let biases = daily_biases.read().await;
                            for bias in biases.iter() {
                                if bias.bias == BiasDirection::NoTrade {
                                    continue;
                                }
                                
                                if let Ok(Some(signal)) = hourly_crossover.check_crossover(
                                    &bias.underlying,
                                    &bias.spot_token,
                                    bias.bias,
                                ).await {
                                    // Save crossover signal to JSON
                                    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                                    let signal_file = format!("data/crossover_signal_{}_{}.json", 
                                                             signal.underlying, timestamp);
                                    if let Ok(signal_json) = serde_json::to_string_pretty(&signal) {
                                        let _ = tokio::fs::write(&signal_file, &signal_json).await;
                                        info!("💾 Saved crossover signal to: {}", signal_file);
                                    }
                                    
                                    // Also append to daily signals log
                                    let daily_signals_file = format!("data/crossover_signals_{}.jsonl", 
                                                                    chrono::Utc::now().format("%Y%m%d"));
                                    if let Ok(signal_json) = serde_json::to_string(&signal) {
                                        use tokio::io::AsyncWriteExt;
                                        if let Ok(mut file) = tokio::fs::OpenOptions::new()
                                            .create(true)
                                            .append(true)
                                            .open(&daily_signals_file)
                                            .await 
                                        {
                                            let _ = file.write_all(format!("{}\n", signal_json).as_bytes()).await;
                                        }
                                    }
                                    
                                    // Publish SignalGenerated event
                                    let _ = event_bus.publish(Event::new(
                                        EventType::SignalGenerated,
                                        EventPayload::SignalGenerated {
                                            symbol: signal.underlying.clone(),
                                            underlying: signal.underlying.clone(),
                                            direction: match signal.direction {
                                                BiasDirection::CE => Direction::CE,
                                                BiasDirection::PE => Direction::PE,
                                                BiasDirection::NoTrade => Direction::NoTrade,
                                            },
                                            strike: 0, // Will be filled by premarket selector
                                            option_type: match signal.direction {
                                                BiasDirection::CE => OptionType::CE,
                                                BiasDirection::PE => OptionType::PE,
                                                BiasDirection::NoTrade => OptionType::CE,
                                            },
                                            side: Side::Buy,
                                            reason: format!("Hourly crossover aligned with daily bias"),
                                            underlying_ltp: signal.close_price,
                                            option_ltp: 0.0,
                                            vix: 0.0,
                                        },
                                    )).await;
                                }
                            }
                        }
                    }
                    Ok(())
                })
            }),
        ).await;
        
        info!("✅ Event subscriptions configured");
    }
    
    /// Start the trading bot
    pub async fn run(&self) -> Result<()> {
        info!("🏁 Trading bot starting main loop...");
        
        // Setup graceful shutdown handler
        self.setup_shutdown_handler().await;
        
        // Setup event subscriptions
        self.setup_event_subscriptions().await;
        
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
        
        // Save NIFTY token to JSON for hourly data
        if let Some(nifty_token) = self.nifty_token.read().await.as_ref() {
            let hourly_tokens = rustro::data::hourly_tokens::HourlyTokensManager::new(
                "data/hourly_data_tokens.json".to_string()
            );
            if let Err(e) = hourly_tokens.add_token("NIFTY", nifty_token, "NIFTY").await {
                warn!("⚠️  Failed to save hourly token to JSON: {}", e);
            } else {
                info!("💾 Saved NIFTY token to hourly_data_tokens.json");
            }
        }
        
        // Step 1: Sync historical data if needed (CRITICAL - must complete before analysis)
        if let Some(nifty_token) = self.nifty_token.read().await.as_ref() {
            if self.needs_data_sync().await {
                // Publish event: Data sync started
                self.event_bus.publish(Event::new(
                    EventType::HistoricalDataSyncStarted,
                    EventPayload::HistoricalDataSyncStarted {
                        symbol: "NIFTY".to_string(),
                        token: nifty_token.clone(),
                    },
                )).await?;
                
                info!("📊 Insufficient data - syncing historical data first...");
                match self.historical_sync.sync_historical_data(nifty_token, "NIFTY").await {
                    Ok(report) => {
                        info!("✅ Historical sync completed:");
                        info!("   Daily bars: {}", report.daily_bars_downloaded);
                        info!("   Hourly bars: {}", report.hourly_bars_downloaded);
                        
                        // Publish event: Data sync completed
                        self.event_bus.publish(Event::new(
                            EventType::HistoricalDataSyncCompleted,
                            EventPayload::HistoricalDataSyncCompleted {
                                symbol: "NIFTY".to_string(),
                                daily_bars_downloaded: report.daily_bars_downloaded,
                                hourly_bars_downloaded: report.hourly_bars_downloaded,
                                total_bars: report.daily_bars_downloaded + report.hourly_bars_downloaded,
                            },
                        )).await?;
                        
                        if !report.errors.is_empty() {
                            warn!("   Errors: {}", report.errors.len());
                        }
                    }
                    Err(e) => {
                        // Publish event: Data sync failed
                        self.event_bus.publish(Event::new(
                            EventType::HistoricalDataSyncFailed,
                            EventPayload::HistoricalDataSyncFailed {
                                symbol: "NIFTY".to_string(),
                                reason: e.to_string(),
                            },
                        )).await?;
                        
                        error!("❌ Historical sync failed: {}", e);
                        return Err(e);
                    }
                }
            } else {
                info!("✅ Sufficient historical data available");
            }
        }
        
        // Step 2: Verify data is ready and publish DataReady event
        let daily_count = self.daily_bars.total_count().await;
        let hourly_count = self.hourly_bars.total_count().await;
        let data_sufficient = self.has_sufficient_data().await;
        
        self.event_bus.publish(Event::new(
            EventType::DataReady,
            EventPayload::DataReady {
                symbol: "NIFTY".to_string(),
                daily_bars_count: daily_count,
                hourly_bars_count: hourly_count,
                data_sufficient,
            },
        )).await?;
        
        if !data_sufficient {
            return Err(TradingError::MissingData(
                format!("Data not ready: {} daily bars (need {}), {} hourly bars (need {})",
                       daily_count, self.config.daily_adx_period,
                       hourly_count, self.config.hourly_adx_period)
            ));
        }
        
        info!("✅ Data ready: {} daily bars, {} hourly bars", daily_count, hourly_count);
        
        info!("✅ Session initialized successfully");
        Ok(())
    }
    
    /// Check if we need to sync historical data
    async fn needs_data_sync(&self) -> bool {
        !self.has_sufficient_data().await
    }
    
    /// Check if we have sufficient data for analysis
    async fn has_sufficient_data(&self) -> bool {
        // Check if we have enough daily bars (need at least daily_adx_period)
        let daily_count = self.daily_bars.total_count().await;
        if daily_count < self.config.daily_adx_period as usize {
            info!("📊 Insufficient daily bars: have {}, need {}", daily_count, self.config.daily_adx_period);
            return false;
        }
        
        // Check if we have enough hourly bars (need at least hourly_adx_period)
        let hourly_count = self.hourly_bars.total_count().await;
        if hourly_count < self.config.hourly_adx_period as usize {
            info!("📊 Insufficient hourly bars: have {}, need {}", hourly_count, self.config.hourly_adx_period);
            return false;
        }
        
        info!("✅ Sufficient data: {} daily bars, {} hourly bars", daily_count, hourly_count);
        true
    }
    
    /// Run one trading cycle
    async fn run_trading_cycle(&self) -> Result<()> {
        let now = chrono::Utc::now();
        let now_ist = now.with_timezone(&chrono_tz::Asia::Kolkata);
        
        // Step 1: Fetch latest data
        self.fetch_and_update_bars().await?;
        
        // Step 2: Verify data is still sufficient (should be ready from initialization)
        if !self.has_sufficient_data().await {
            warn!("⚠️  Data became insufficient - waiting for data sync...");
            // Try to sync data if it became insufficient
            if let Some(nifty_token) = self.nifty_token.read().await.as_ref() {
                self.event_bus.publish(Event::new(
                    EventType::HistoricalDataSyncStarted,
                    EventPayload::HistoricalDataSyncStarted {
                        symbol: "NIFTY".to_string(),
                        token: nifty_token.clone(),
                    },
                )).await?;
                
                if let Err(e) = self.historical_sync.sync_historical_data(nifty_token, "NIFTY").await {
                    warn!("⚠️  Data sync failed: {}", e);
                    return Ok(()); // Skip this cycle, try again next time
                }
            }
            return Ok(());
        }
        
        // Step 3: Daily analysis (runs once at 9:30 AM)
        if now_ist.hour() >= 9 && now_ist.minute() >= 30 {
            let daily_done = self.daily_analysis_done.read().await;
            if !*daily_done {
                drop(daily_done);
                self.run_daily_analysis().await?;
            }
        }
        
        // Step 4: Hourly analysis (runs every hour after bar completes)
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
        
        // Step 5: Update open positions
        self.update_positions().await?;
        
        // Step 6: Check EOD exit (3:20 PM)
        if now_ist.hour() == 15 && now_ist.minute() >= 20 {
            self.eod_exit_positions().await?;
        }
        
        Ok(())
    }
    
    /// Fetch latest bars from broker
    async fn fetch_and_update_bars(&self) -> Result<()> {
        // Load tokens from JSON for hourly data
        let hourly_tokens = rustro::data::hourly_tokens::HourlyTokensManager::new(
            "data/hourly_data_tokens.json".to_string()
        );
        
        let tokens_map = hourly_tokens.get_tokens_map().await
            .map_err(|e| TradingError::FileError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load hourly tokens: {}", e)
            )))?;
        
        // Fetch hourly bars for each token
        for (underlying, token) in tokens_map {
            let to_date = chrono::Utc::now();
            let from_date = to_date - chrono::Duration::hours(2); // Last 2 hours
            
            match self.broker_client.get_candles(&token, "ONE_HOUR", from_date, to_date).await {
                Ok(bars) => {
                    let bars_count = bars.len();
                    if !bars.is_empty() {
                        for bar in bars {
                            if underlying == "NIFTY" {
                                self.hourly_bars.append(bar).await?;
                            }
                        }
                        info!("📊 Updated {} hourly bars for {}", bars_count, underlying);
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to fetch hourly bars for {}: {}", underlying, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Run daily direction analysis
    async fn run_daily_analysis(&self) -> Result<()> {
        info!("📊 Running daily bias calculation for all F&O underlyings...");
        
        // Verify we have sufficient data before calculating bias
        if !self.has_sufficient_data().await {
            warn!("⚠️  Insufficient data for daily bias calculation - data sync should have completed during initialization");
            return Err(TradingError::MissingData(
                "Insufficient daily bars for bias calculation. Please ensure historical data sync completed successfully.".to_string()
            ));
        }
        
        // TODO: Load daily_bias_tokens.json and fetch bars for all underlyings
        // For now, just do NIFTY as example
        let daily_bars_vec = self.daily_bars.get_recent(30).await?;
        
        if daily_bars_vec.len() < self.config.daily_adx_period {
            warn!("⚠️  Insufficient daily bars for analysis: have {}, need {}", 
                  daily_bars_vec.len(), self.config.daily_adx_period);
            return Err(TradingError::MissingData(
                format!("Need at least {} daily bars, but only have {}", 
                       self.config.daily_adx_period, daily_bars_vec.len())
            ));
        }
        
        // Calculate bias for NIFTY
        if let Some(nifty_token) = self.nifty_token.read().await.as_ref() {
            if let Some(bias) = self.daily_bias_calculator.calculate_bias(
                "NIFTY",
                nifty_token,
                &daily_bars_vec,
            ) {
                info!("✅ NIFTY daily bias: {} (ADX: {:.2})", bias.bias.as_str(), bias.adx);
                
                // Store bias in memory
                let mut biases = self.daily_biases.write().await;
                biases.clear();
                biases.push(bias.clone());
                
                // Save to JSON file for persistence
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let daily_bias_file = format!("data/daily_bias_{}.json", chrono::Utc::now().format("%Y%m%d"));
                let bias_json = serde_json::to_string_pretty(&biases.clone())?;
                tokio::fs::write(&daily_bias_file, &bias_json).await?;
                info!("💾 Saved daily bias to: {}", daily_bias_file);
                
                // Also save to latest file for easy access
                tokio::fs::write("data/daily_bias_latest.json", &bias_json).await?;
                
                // Publish event
                self.event_bus.publish(Event::new(
                    EventType::DailyDirectionDetermined,
                    EventPayload::DailyDirectionDetermined {
                        symbol: "NIFTY".to_string(),
                        direction: match bias.bias {
                            BiasDirection::CE => Direction::CE,
                            BiasDirection::PE => Direction::PE,
                            BiasDirection::NoTrade => Direction::NoTrade,
                        },
                        daily_adx: bias.adx,
                        daily_plus_di: bias.plus_di,
                        daily_minus_di: bias.minus_di,
                        reason: format!("Daily bias: {}", bias.bias.as_str()),
                    },
                )).await?;
            }
        }
        
        let mut done = self.daily_analysis_done.write().await;
        *done = true;
        
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
        
        let order_id: String;
        let filled_price: f64;

        if self.config.enable_paper_trading {
            if let Some(paper_broker) = &self.paper_broker {
                // Use paper trading broker
                order_id = paper_broker.place_order(
                    symbol.clone(),
                    signal.side,
                    quantity,
                    OrderType::Limit, // Assuming Limit for paper trades
                    Some(option_price),
                ).await?;
                filled_price = paper_broker.get_fill_price(&order_id).await.unwrap_or(option_price);
                info!("📝 [PAPER] Order executed: {} @ {:.2}", order_id, filled_price);
            } else {
                return Err(TradingError::ConfigError("Paper trading enabled but broker not initialized".to_string()));
            }
        } else {
            // Use live order manager
            order_id = self.order_manager.place_order(
                symbol.clone(),
                token.to_string(),
                signal.side,
                quantity,
                option_price,
                idempotency_key.clone(),
            ).await?;
            // In a live scenario, you would wait for a fill event.
            // For now, we assume it's filled at the requested price.
            filled_price = option_price;
            info!("✅ Live order placed: {}", order_id);
        }

        // Create and open the position with the correct fill price
        let position = Position {
            position_id: order_id.clone(),
            symbol,
            underlying: "NIFTY".to_string(),
            strike: signal.strike,
            option_type: signal.option_type,
            side: signal.side,
            quantity,
            entry_price: filled_price, // Use the actual filled price
            entry_time: chrono::Utc::now(),
            entry_time_ms: chrono::Utc::now().timestamp_millis(),
            underlying_entry: signal.underlying_ltp,
            stop_loss: filled_price * (1.0 - self.config.option_stop_loss_pct),
            target: None,
            trailing_stop: None,
            trailing_active: false,
            current_price: filled_price,
            pnl: 0.0,
            pnl_pct: 0.0,
            status: PositionStatus::Open,
            entry_reason: signal.reason,
            idempotency_key,
        };

        self.position_manager.open_position(position.clone()).await?;
        
        // Save position to JSON
        let position_file = format!("data/position_{}_{}.json", 
                                   position.symbol, 
                                   chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let position_json = serde_json::to_string_pretty(&position)?;
        tokio::fs::write(&position_file, &position_json).await?;
        info!("💾 Saved position to: {}", position_file);
        
        // Append to daily positions log
        let daily_positions_file = format!("data/positions_{}.jsonl", 
                                          chrono::Utc::now().format("%Y%m%d"));
        let position_json_line = serde_json::to_string(&position)?;
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&daily_positions_file)
            .await?;
        file.write_all(format!("{}\n", position_json_line).as_bytes()).await?;
        
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
                
                // Close position
                self.position_manager.close_position(
                    &position.position_id,
                    current_price,
                    exit_reason.clone(),
                ).await?;
                
                // Save closed position to JSON
                if let Some(closed_position) = self.position_manager.get_position(&position.position_id).await {
                    let exit_file = format!("data/exit_{}_{}.json", 
                                          closed_position.symbol,
                                          chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                    let exit_json = serde_json::to_string_pretty(&closed_position)?;
                    tokio::fs::write(&exit_file, &exit_json).await?;
                    info!("💾 Saved exit to: {}", exit_file);
                    
                    // Append to daily exits log
                    let daily_exits_file = format!("data/exits_{}.jsonl", 
                                                  chrono::Utc::now().format("%Y%m%d"));
                    let exit_json_line = serde_json::to_string(&closed_position)?;
                    use tokio::io::AsyncWriteExt;
                    let mut file = tokio::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&daily_exits_file)
                        .await?;
                    file.write_all(format!("{}\n", exit_json_line).as_bytes()).await?;
                }
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
        
        // Sync historical data during off-hours
        if let Some(nifty_token) = self.nifty_token.read().await.as_ref() {
            info!("📊 Starting historical data sync...");
            match self.historical_sync.sync_historical_data(nifty_token, "NIFTY").await {
                Ok(report) => {
                    info!("✅ Historical sync completed:");
                    info!("   Daily bars: {}", report.daily_bars_downloaded);
                    info!("   Hourly bars: {}", report.hourly_bars_downloaded);
                    if !report.errors.is_empty() {
                        warn!("   Errors: {}", report.errors.len());
                    }
                }
                Err(e) => {
                    warn!("⚠️  Historical sync failed: {}", e);
                }
            }
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
