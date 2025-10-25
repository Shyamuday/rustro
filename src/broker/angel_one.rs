/// Angel One SmartAPI REST client
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::broker::tokens::{TokenManager, Tokens};
use crate::error::{Result, TradingError};
use crate::types::{Bar, Instrument, OrderType, Side};

const BASE_URL: &str = "https://apiconnect.angelbroking.com";

#[derive(Debug, Serialize)]
struct LoginRequest {
    #[serde(rename = "clientcode")]
    client_code: String,
    password: String,
    totp: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    status: bool,
    message: String,
    #[serde(rename = "errorcode")]
    _error_code: Option<String>,
    data: Option<LoginData>,
}

#[derive(Debug, Deserialize)]
struct LoginData {
    #[serde(rename = "jwtToken")]
    jwt_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "feedToken")]
    feed_token: String,
}

#[derive(Debug, Serialize)]
struct OrderRequest {
    variety: String,
    #[serde(rename = "tradingsymbol")]
    trading_symbol: String,
    #[serde(rename = "symboltoken")]
    symbol_token: String,
    #[serde(rename = "transactiontype")]
    transaction_type: String,
    exchange: String,
    #[serde(rename = "ordertype")]
    order_type: String,
    #[serde(rename = "producttype")]
    product_type: String,
    duration: String,
    price: String,
    #[serde(rename = "squareoff")]
    square_off: String,
    #[serde(rename = "stoploss")]
    stop_loss: String,
    quantity: String,
}

#[derive(Debug, Deserialize)]
struct OrderResponse {
    status: bool,
    message: String,
    #[serde(rename = "errorcode")]
    error_code: Option<String>,
    data: Option<OrderResponseData>,
}

#[derive(Debug, Deserialize)]
struct OrderResponseData {
    #[serde(rename = "orderid")]
    order_id: String,
}

#[derive(Debug, Serialize)]
struct CandleRequest {
    exchange: String,
    #[serde(rename = "symboltoken")]
    symbol_token: String,
    interval: String,
    #[serde(rename = "fromdate")]
    from_date: String,
    #[serde(rename = "todate")]
    to_date: String,
}

#[derive(Debug, Deserialize)]
struct CandleResponse {
    status: bool,
    message: String,
    #[serde(rename = "errorcode")]
    _error_code: Option<String>,
    data: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct LtpResponse {
    status: bool,
    message: String,
    data: Option<LtpData>,
}

#[derive(Debug, Deserialize)]
struct LtpData {
    ltp: f64,
}

/// Angel One SmartAPI client
pub struct AngelOneClient {
    client: Client,
    token_manager: Arc<TokenManager>,
    client_code: String,
    password: String,
    totp_secret: String,
}

impl AngelOneClient {
    pub fn new(
        token_manager: Arc<TokenManager>,
        client_code: String,
        password: String,
        totp_secret: String,
    ) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        
        AngelOneClient {
            client,
            token_manager,
            client_code,
            password,
            totp_secret,
        }
    }
    
    /// Authenticate and get tokens
    pub async fn login(&self) -> Result<Tokens> {
        info!("Attempting login to Angel One");
        
        // Generate TOTP
        let totp = self.generate_totp()?;
        
        let login_req = LoginRequest {
            client_code: self.client_code.clone(),
            password: self.password.clone(),
            totp,
        };
        
        let response = self.client
            .post(&format!("{}/rest/auth/angelbroking/user/v1/loginByPassword", BASE_URL))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&login_req)
            .send()
            .await?;
        
        let status = response.status();
        let body = response.text().await?;
        
        debug!("Login response status: {}, body: {}", status, body);
        
        let login_response: LoginResponse = serde_json::from_str(&body)
            .map_err(|e| TradingError::AuthenticationFailed(format!("Parse error: {}", e)))?;
        
        if !login_response.status {
            return Err(TradingError::AuthenticationFailed(format!(
                "Login failed: {}",
                login_response.message
            )));
        }
        
        let data = login_response.data
            .ok_or_else(|| TradingError::AuthenticationFailed("No data in login response".to_string()))?;
        
        // Token expiry: JWT and Feed tokens expire at 3:30 AM next day
        let now = Utc::now();
        let expiry = self.calculate_token_expiry(now);
        
        let tokens = Tokens {
            jwt_token: data.jwt_token,
            feed_token: data.feed_token,
            jwt_expiry: expiry,
            feed_expiry: expiry,
            refresh_token: Some(data.refresh_token),
        };
        
        self.token_manager.set_tokens(tokens.clone()).await?;
        
        info!("Login successful, tokens expire at: {}", expiry);
        Ok(tokens)
    }
    
    /// Calculate token expiry (3:30 AM next day IST)
    fn calculate_token_expiry(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        use chrono::TimeZone;
        use chrono_tz::Asia::Kolkata;
        
        let now_ist = now.with_timezone(&Kolkata);
        let today_330am = Kolkata.with_ymd_and_hms(
            now_ist.year(),
            now_ist.month(),
            now_ist.day(),
            3,
            30,
            0
        ).unwrap();
        
        let expiry_ist = if now_ist < today_330am {
            today_330am
        } else {
            today_330am + chrono::Duration::days(1)
        };
        
        expiry_ist.with_timezone(&Utc)
    }
    
    /// Generate TOTP for authentication
    fn generate_totp(&self) -> Result<String> {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        type HmacSha1 = Hmac<Sha1>;
        
        // Decode base32 secret
        let secret = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &self.totp_secret)
            .ok_or_else(|| TradingError::AuthenticationFailed("Invalid TOTP secret".to_string()))?;
        
        // Get current time step (30 second intervals)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let time_step = timestamp / 30;
        
        // Generate HMAC-SHA1
        let mut mac = HmacSha1::new_from_slice(&secret)
            .map_err(|e| TradingError::AuthenticationFailed(format!("HMAC error: {}", e)))?;
        mac.update(&time_step.to_be_bytes());
        let hash = mac.finalize().into_bytes();
        
        // Dynamic truncation
        let offset = (hash[hash.len() - 1] & 0x0f) as usize;
        let code = u32::from_be_bytes([
            hash[offset] & 0x7f,
            hash[offset + 1],
            hash[offset + 2],
            hash[offset + 3],
        ]);
        
        let totp = format!("{:06}", code % 1_000_000);
        Ok(totp)
    }
    
    /// Place an order
    pub async fn place_order(
        &self,
        symbol: &str,
        token: &str,
        side: Side,
        quantity: i32,
        order_type: OrderType,
        price: Option<f64>,
    ) -> Result<String> {
        let tokens = self.token_manager.get_tokens().await
            .ok_or_else(|| TradingError::TokenExpired("No tokens available".to_string()))?;
        
        let order_req = OrderRequest {
            variety: "NORMAL".to_string(),
            trading_symbol: symbol.to_string(),
            symbol_token: token.to_string(),
            transaction_type: side.as_str().to_string(),
            exchange: "NFO".to_string(),
            order_type: order_type.as_str().to_string(),
            product_type: "CARRYFORWARD".to_string(),
            duration: "DAY".to_string(),
            price: price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string()),
            square_off: "0".to_string(),
            stop_loss: "0".to_string(),
            quantity: quantity.to_string(),
        };
        
        debug!("Placing order: {:?}", order_req);
        
        let response = self.client
            .post(&format!("{}/rest/secure/angelbroking/order/v1/placeOrder", BASE_URL))
            .header("Authorization", format!("Bearer {}", tokens.jwt_token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-UserType", "USER")
            .header("X-SourceID", "WEB")
            .header("X-ClientLocalIP", "127.0.0.1")
            .header("X-ClientPublicIP", "127.0.0.1")
            .header("X-MACAddress", "00:00:00:00:00:00")
            .header("X-PrivateKey", &self.client_code)
            .json(&order_req)
            .send()
            .await?;
        
        let status = response.status();
        let body = response.text().await?;
        
        debug!("Order response status: {}, body: {}", status, body);
        
        let order_response: OrderResponse = serde_json::from_str(&body)
            .map_err(|e| TradingError::OrderPlacementFailed(format!("Parse error: {}", e)))?;
        
        if !order_response.status {
            return Err(TradingError::OrderPlacementFailed(format!(
                "Order failed: {} (code: {})",
                order_response.message,
                order_response.error_code.unwrap_or_default()
            )));
        }
        
        let order_id = order_response.data
            .ok_or_else(|| TradingError::OrderPlacementFailed("No order ID in response".to_string()))?
            .order_id;
        
        info!("Order placed successfully: {}", order_id);
        Ok(order_id)
    }
    
    /// Get historical candle data
    pub async fn get_candles(
        &self,
        symbol_token: &str,
        interval: &str,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<Vec<Bar>> {
        let tokens = self.token_manager.get_tokens().await
            .ok_or_else(|| TradingError::TokenExpired("No tokens available".to_string()))?;
        
        let candle_req = CandleRequest {
            exchange: "NFO".to_string(),
            symbol_token: symbol_token.to_string(),
            interval: interval.to_string(),
            from_date: from_date.format("%Y-%m-%d %H:%M").to_string(),
            to_date: to_date.format("%Y-%m-%d %H:%M").to_string(),
        };
        
        debug!("Fetching candles: {:?}", candle_req);
        
        let response = self.client
            .post(&format!("{}/rest/secure/angelbroking/historical/v1/getCandleData", BASE_URL))
            .header("Authorization", format!("Bearer {}", tokens.jwt_token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-PrivateKey", &self.client_code)
            .json(&candle_req)
            .send()
            .await?;
        
        let body = response.text().await?;
        let candle_response: CandleResponse = serde_json::from_str(&body)
            .map_err(|e| TradingError::DeserializationError(e))?;
        
        if !candle_response.status {
            return Err(TradingError::MissingData(format!(
                "Candle fetch failed: {}",
                candle_response.message
            )));
        }
        
        let data = candle_response.data
            .ok_or_else(|| TradingError::MissingData("No candle data".to_string()))?;
        
        // Parse candles: [timestamp, open, high, low, close, volume]
        let bars: Vec<Bar> = data
            .iter()
            .filter_map(|candle| {
                if candle.len() >= 6 {
                    // Parse timestamp (format: "YYYY-MM-DD HH:MM:SS+0530")
                    let ts_str = candle[0].replace("+0530", "").trim().to_string();
                    let naive_dt = NaiveDateTime::parse_from_str(&ts_str, "%Y-%m-%d %H:%M:%S").ok()?;
                    let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
                    
                    Some(Bar {
                        timestamp,
                        timestamp_ms: timestamp.timestamp_millis(),
                        open: candle[1].parse().ok()?,
                        high: candle[2].parse().ok()?,
                        low: candle[3].parse().ok()?,
                        close: candle[4].parse().ok()?,
                        volume: candle[5].parse().ok()?,
                        bar_complete: true,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        debug!("Fetched {} candles", bars.len());
        Ok(bars)
    }
    
    /// Get LTP for a symbol
    pub async fn get_ltp(&self, symbol_token: &str) -> Result<f64> {
        let tokens = self.token_manager.get_tokens().await
            .ok_or_else(|| TradingError::TokenExpired("No tokens available".to_string()))?;
        
        let payload = serde_json::json!({
            "exchange": "NFO",
            "symboltoken": symbol_token,
            "tradingsymbol": ""
        });
        
        let response = self.client
            .post(&format!("{}/rest/secure/angelbroking/order/v1/getLtpData", BASE_URL))
            .header("Authorization", format!("Bearer {}", tokens.jwt_token))
            .header("Content-Type", "application/json")
            .header("X-PrivateKey", &self.client_code)
            .json(&payload)
            .send()
            .await?;
        
        let body = response.text().await?;
        let ltp_response: LtpResponse = serde_json::from_str(&body)?;
        
        if !ltp_response.status {
            return Err(TradingError::MissingData(format!(
                "LTP fetch failed: {}",
                ltp_response.message
            )));
        }
        
        let ltp = ltp_response.data
            .ok_or_else(|| TradingError::MissingData("No LTP data".to_string()))?
            .ltp;
        
        Ok(ltp)
    }
    
    /// Download instrument master CSV
    pub async fn download_instrument_master(&self) -> Result<Vec<Instrument>> {
        info!("Downloading instrument master");
        
        let url = "https://margincalculator.angelbroking.com/OpenAPI_File/files/OpenAPIScripMaster.json";
        
        let response = self.client
            .get(url)
            .send()
            .await?;
        
        let body = response.text().await?;
        
        #[derive(Debug, Deserialize)]
        struct RawInstrument {
            token: String,
            symbol: String,
            name: String,
            expiry: String,
            strike: String,
            lotsize: String,
            instrumenttype: String,
            exch_seg: String,
            tick_size: String,
        }
        
        let raw_instruments: Vec<RawInstrument> = serde_json::from_str(&body)?;
        
        let instruments: Vec<Instrument> = raw_instruments
            .into_iter()
            .filter_map(|raw| {
                Some(Instrument {
                    token: raw.token,
                    symbol: raw.symbol,
                    name: raw.name,
                    expiry: raw.expiry,
                    strike: raw.strike.parse().ok()?,
                    lotsize: raw.lotsize.parse().ok()?,
                    instrument_type: raw.instrumenttype,
                    exch_seg: raw.exch_seg,
                    tick_size: raw.tick_size.parse().unwrap_or(0.05),
                })
            })
            .collect();
        
        info!("Downloaded {} instruments", instruments.len());
        Ok(instruments)
    }
    
    /// Refresh token (if refresh token available)
    pub async fn refresh_token(&self) -> Result<Tokens> {
        warn!("Token refresh not yet implemented for Angel One");
        // Angel One doesn't support token refresh - must re-login
        self.login().await
    }
}
