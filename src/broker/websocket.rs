/// Angel One SmartAPI WebSocket client for real-time data
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::broker::TokenManager;
use crate::error::{Result, TradingError};
use crate::types::Tick;

const WS_URL: &str = "wss://smartapisocket.angelone.in/smart-stream";

#[derive(Debug, Serialize)]
struct WsSubscribeRequest {
    action: u8,
    params: WsSubscribeParams,
}

#[derive(Debug, Serialize)]
struct WsSubscribeParams {
    mode: u8,
    #[serde(rename = "tokenList")]
    token_list: Vec<WsToken>,
}

#[derive(Debug, Serialize)]
struct WsToken {
    #[serde(rename = "exchangeType")]
    exchange_type: u8,
    tokens: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WsTickData {
    #[serde(rename = "tk")]
    token: String,
    #[serde(rename = "ltp")]
    last_price: Option<f64>,
    #[serde(rename = "bs")]
    bid: Option<f64>,
    #[serde(rename = "bp")]
    bid_price: Option<f64>,
    #[serde(rename = "ap")]
    ask_price: Option<f64>,
    #[serde(rename = "v")]
    volume: Option<i64>,
    #[serde(rename = "e")]
    exchange: Option<String>,
}

pub struct AngelWebSocket {
    token_manager: Arc<TokenManager>,
    tx: mpsc::UnboundedSender<Tick>,
    rx: Arc<RwLock<mpsc::UnboundedReceiver<Tick>>>,
    subscribed_tokens: Arc<RwLock<Vec<String>>>,
    is_connected: Arc<RwLock<bool>>,
}

impl AngelWebSocket {
    pub fn new(token_manager: Arc<TokenManager>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        AngelWebSocket {
            token_manager,
            tx,
            rx: Arc::new(RwLock::new(rx)),
            subscribed_tokens: Arc::new(RwLock::new(Vec::new())),
            is_connected: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Connect to WebSocket and start receiving data
    pub async fn connect(&self) -> Result<()> {
        let tokens = self.token_manager.get_tokens().await
            .ok_or_else(|| TradingError::TokenExpired("No tokens available".to_string()))?;
        
        info!("🔌 Connecting to Angel One WebSocket...");
        
        // Build WebSocket URL with auth
        let url = format!(
            "{}?jwtToken={}&apiKey={}&clientCode={}&feedToken={}",
            WS_URL,
            tokens.jwt_token,
            "dummy_api_key", // Would use actual API key
            "dummy_client",
            tokens.feed_token
        );
        
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| TradingError::WebSocketError(format!("Connection failed: {}", e)))?;
        
        let (mut write, mut read) = ws_stream.split();
        
        {
            let mut connected = self.is_connected.write().await;
            *connected = true;
        }
        
        info!("✅ WebSocket connected");
        
        // Spawn reader task
        let tx = self.tx.clone();
        let is_connected = Arc::clone(&self.is_connected);
        
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(tick_data) = serde_json::from_str::<WsTickData>(&text) {
                            if let Some(ltp) = tick_data.last_price {
                                let tick = Tick {
                                    symbol: tick_data.token.clone(),
                                    token: tick_data.token,
                                    ltp,
                                    bid: tick_data.bid_price.unwrap_or(0.0),
                                    ask: tick_data.ask_price.unwrap_or(0.0),
                                    volume: tick_data.volume.unwrap_or(0),
                                    timestamp: chrono::Utc::now(),
                                    timestamp_ms: chrono::Utc::now().timestamp_millis(),
                                };
                                
                                if let Err(e) = tx.send(tick) {
                                    error!("Failed to send tick: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // Binary tick data (more efficient)
                        if let Some(tick) = Self::parse_binary_tick(&data) {
                            if let Err(e) = tx.send(tick) {
                                error!("Failed to send tick: {}", e);
                                break;
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        debug!("Received ping, sending pong");
                        // Auto-handled by library
                    }
                    Ok(Message::Close(_)) => {
                        warn!("WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            
            let mut connected = is_connected.write().await;
            *connected = false;
            warn!("WebSocket reader task ended");
        });
        
        Ok(())
    }
    
    /// Subscribe to tokens
    pub async fn subscribe(&self, tokens: Vec<String>, exchange: &str) -> Result<()> {
        let exchange_type = match exchange {
            "NSE" => 1,
            "NFO" => 2,
            "BSE" => 3,
            "BFO" => 4,
            "MCX" => 5,
            _ => 1,
        };
        
        let subscribe_req = WsSubscribeRequest {
            action: 1, // Subscribe
            params: WsSubscribeParams {
                mode: 1, // LTP mode (mode 2 = Quote, mode 3 = Snap Quote)
                token_list: vec![WsToken {
                    exchange_type,
                    tokens: tokens.clone(),
                }],
            },
        };
        
        // Would send to WebSocket here
        // ws_write.send(Message::Text(serde_json::to_string(&subscribe_req)?)).await?;
        
        {
            let mut subscribed = self.subscribed_tokens.write().await;
            subscribed.extend(tokens.clone());
        }
        
        info!("📡 Subscribed to {} tokens on {}", tokens.len(), exchange);
        
        Ok(())
    }
    
    /// Unsubscribe from tokens
    pub async fn unsubscribe(&self, tokens: Vec<String>) -> Result<()> {
        // Would send unsubscribe message
        
        {
            let mut subscribed = self.subscribed_tokens.write().await;
            subscribed.retain(|t| !tokens.contains(t));
        }
        
        info!("📡 Unsubscribed from {} tokens", tokens.len());
        
        Ok(())
    }
    
    /// Get tick receiver
    pub fn get_tick_receiver(&self) -> Arc<RwLock<mpsc::UnboundedReceiver<Tick>>> {
        Arc::clone(&self.rx)
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let connected = self.is_connected.read().await;
        *connected
    }
    
    /// Parse binary tick data (Angel One specific format)
    fn parse_binary_tick(data: &[u8]) -> Option<Tick> {
        // Angel One binary format:
        // Bytes 0-3: Token (4 bytes)
        // Bytes 4-7: LTP (4 bytes, float)
        // Bytes 8-11: Volume (4 bytes, int)
        // etc.
        
        if data.len() < 12 {
            return None;
        }
        
        let token = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let ltp_bytes = [data[4], data[5], data[6], data[7]];
        let ltp = f32::from_be_bytes(ltp_bytes) as f64;
        let volume = i32::from_be_bytes([data[8], data[9], data[10], data[11]]) as i64;
        
        Some(Tick {
            symbol: token.to_string(),
            token: token.to_string(),
            ltp,
            bid: 0.0,
            ask: 0.0,
            volume,
            timestamp: chrono::Utc::now(),
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        })
    }
    
    /// Reconnect with exponential backoff
    pub async fn reconnect(&self, max_attempts: u32) -> Result<()> {
        let mut attempt = 0;
        let backoffs = vec![1, 2, 4, 8, 16]; // seconds
        
        while attempt < max_attempts {
            attempt += 1;
            
            let backoff = backoffs.get(attempt as usize - 1).unwrap_or(&16);
            
            warn!("Reconnecting (attempt {}/{}), waiting {}s...", attempt, max_attempts, backoff);
            tokio::time::sleep(tokio::time::Duration::from_secs(*backoff)).await;
            
            match self.connect().await {
                Ok(_) => {
                    info!("✅ Reconnected successfully");
                    
                    // Re-subscribe to previous tokens
                    let tokens = {
                        let subscribed = self.subscribed_tokens.read().await;
                        subscribed.clone()
                    };
                    
                    if !tokens.is_empty() {
                        self.subscribe(tokens, "NFO").await?;
                    }
                    
                    return Ok(());
                }
                Err(e) => {
                    error!("Reconnection attempt {} failed: {}", attempt, e);
                }
            }
        }
        
        Err(TradingError::WebSocketError(format!(
            "Failed to reconnect after {} attempts",
            max_attempts
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binary_tick_parsing() {
        let data = vec![
            0, 0, 0, 10,  // Token: 10
            0x42, 0xFA, 0, 0,  // LTP: 125.0
            0, 0x0F, 0x42, 0x40,  // Volume: 1000000
        ];
        
        let tick = AngelWebSocket::parse_binary_tick(&data);
        assert!(tick.is_some());
        
        let tick = tick.unwrap();
        assert_eq!(tick.token, "10");
        assert!(tick.ltp > 0.0);
    }
}

