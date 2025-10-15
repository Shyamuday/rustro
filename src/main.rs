/// Main entry point for the trading bot
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_subscriber;

use rustro::{
    broker::{AngelOneClient, TokenManager},
    config::load_config,
    data::HybridBarStore,
    error::{Result, TradingError},
    events::{EventBus, Event, EventType, EventPayload},
    orders::OrderManager,
    positions::PositionManager,
    risk::RiskManager,
    strategy::AdxStrategy,
    utils::*,
    Config,
};

/// Application state
pub struct TradingApp {
    config: Arc<Config>,
    event_bus: Arc<EventBus>,
    token_manager: Arc<TokenManager>,
    broker_client: Arc<AngelOneClient>,
    strategy: Arc<AdxStrategy>,
    order_manager: Arc<OrderManager>,
    position_manager: Arc<PositionManager>,
    risk_manager: Arc<RiskManager>,
    shutdown: Arc<RwLock<bool>>,
}

impl TradingApp {
    pub async fn new(config_path: &str) -> Result<Self> {
        // Initialize logging
        tracing_subscriber::fmt()
            .with_env_filter("rustro=debug,info")
            .init();
        
        info!("Starting trading bot...");
        
        // Load configuration
        let config = Arc::new(load_config(config_path)?);
        info!("Configuration loaded");
        
        // Create event bus
        let event_bus = Arc::new(EventBus::new("data/events.jsonl".to_string()));
        event_bus.start_processing().await;
        
        // Emit log initialized event
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
        
        let shutdown = Arc::new(RwLock::new(false));
        
        Ok(TradingApp {
            config,
            event_bus,
            token_manager,
            broker_client,
            strategy,
            order_manager,
            position_manager,
            risk_manager,
            shutdown,
        })
    }
    
    /// Start the trading bot
    pub async fn run(&self) -> Result<()> {
        info!("Trading bot starting...");
        
        // Setup graceful shutdown handler
        self.setup_shutdown_handler().await;
        
        // Initialize session
        self.initialize_session().await?;
        
        // Main trading loop
        loop {
            // Check shutdown flag
            {
                let shutdown = self.shutdown.read().await;
                if *shutdown {
                    info!("Shutdown signal received");
                    break;
                }
            }
            
            // Check if market is open
            let now = chrono::Utc::now();
            if !is_market_open(now) {
                info!("Market closed - waiting for market open");
                let next_open = next_market_open(now);
                let wait_duration = (next_open - now).to_std().unwrap_or(std::time::Duration::from_secs(60));
                tokio::time::sleep(wait_duration).await;
                continue;
            }
            
            // Market is open - run trading logic
            if let Err(e) = self.run_trading_cycle().await {
                error!("Trading cycle error: {} ({})", e, e.error_code());
                
                if e.is_fatal() {
                    error!("Fatal error encountered - initiating shutdown");
                    break;
                }
                
                if e.requires_exit() {
                    warn!("Risk event requires position exit");
                    let _ = self.position_manager.close_all_positions(e.to_string()).await;
                }
            }
            
            // Sleep before next cycle
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        // Shutdown sequence
        self.shutdown_sequence().await?;
        
        Ok(())
    }
    
    /// Initialize session (authentication, data loading)
    async fn initialize_session(&self) -> Result<()> {
        info!("Initializing session...");
        
        // Try to load existing tokens
        match self.token_manager.load_from_file().await {
            Ok(_) => {
                if self.token_manager.is_valid().await {
                    info!("Valid tokens loaded from file");
                    
                    self.event_bus.publish(Event::new(
                        EventType::TokenLoaded,
                        EventPayload::TokenLoaded {
                            jwt_expiry: self.token_manager.get_tokens().await.unwrap().jwt_expiry,
                            feed_expiry: self.token_manager.get_tokens().await.unwrap().feed_expiry,
                        },
                    )).await?;
                } else {
                    info!("Tokens expired - logging in");
                    self.broker_client.login().await?;
                }
            }
            Err(_) => {
                info!("No tokens found - logging in");
                self.broker_client.login().await?;
            }
        }
        
        self.event_bus.publish(Event::new(
            EventType::BrokerClientReady,
            EventPayload::BrokerClientReady {
                session_id: uuid::Uuid::new_v4().to_string(),
            },
        )).await?;
        
        info!("Session initialized successfully");
        Ok(())
    }
    
    /// Run one trading cycle
    async fn run_trading_cycle(&self) -> Result<()> {
        // This is a simplified placeholder - full implementation would:
        // 1. Fetch/aggregate bars
        // 2. Run daily/hourly analysis
        // 3. Check entry conditions
        // 4. Manage open positions
        // 5. Monitor risk
        
        // For now, just log that we're running
        info!("Trading cycle executing...");
        
        // Example: Update positions with current prices
        let open_positions = self.position_manager.get_open_positions().await;
        for position in open_positions {
            // Would fetch current LTP from broker/websocket
            // For now, skip actual updates
            info!("Position open: {} @ {:.2}", position.symbol, position.current_price);
        }
        
        Ok(())
    }
    
    /// Setup graceful shutdown handler
    async fn setup_shutdown_handler(&self) {
        let shutdown = Arc::clone(&self.shutdown);
        let event_bus = Arc::clone(&self.event_bus);
        
        tokio::spawn(async move {
            // Wait for Ctrl+C
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            
            info!("Ctrl+C received - initiating graceful shutdown");
            
            // Set shutdown flag
            {
                let mut flag = shutdown.write().await;
                *flag = true;
            }
            
            // Emit shutdown event
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
        info!("Starting shutdown sequence...");
        
        let start_time = chrono::Utc::now();
        
        // 1. Close all open positions
        let open_positions = self.position_manager.get_open_positions().await;
        if !open_positions.is_empty() {
            warn!("Closing {} open positions", open_positions.len());
            let _ = self.position_manager.close_all_positions("Shutdown".to_string()).await;
        }
        
        // 2. Cancel pending orders
        let active_orders = self.order_manager.get_active_orders().await;
        if !active_orders.is_empty() {
            warn!("{} active orders detected during shutdown", active_orders.len());
        }
        
        // 3. Save daily trades
        let trades = self.position_manager.get_daily_trades().await;
        info!("Completed {} trades today", trades.len());
        
        let trades_json = serde_json::to_string_pretty(&trades)?;
        tokio::fs::write(
            format!("data/trades_{}.json", chrono::Utc::now().format("%Y%m%d")),
            trades_json
        ).await?;
        
        // 4. Emit shutdown completed event
        let duration = (chrono::Utc::now() - start_time).num_seconds() as u64;
        self.event_bus.publish(Event::new(
            EventType::ShutdownCompleted,
            EventPayload::ShutdownCompleted {
                duration_sec: duration,
            },
        )).await?;
        
        info!("Shutdown sequence completed in {}s", duration);
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

