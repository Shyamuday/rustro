# Option Trading Bot - Critical Implementation Details Supplement

**This document provides critical details that complement the main structured guide. Use both documents together.**

---

## 1. Token Discovery & Instrument Master Management

### 1.1 Angel One Instrument Master (CSV Processing)

**Purpose**: Map strikes to broker tokens for order placement  
**Source**: Angel One SmartAPI instrument master  
**Update Frequency**: Daily (new contracts), Monthly (expiry rotation)

#### CSV Download & Structure

```rust
// Angel One CSV columns
struct InstrumentCSV {
    token: String,           // e.g., "99926009"
    symbol: String,          // e.g., "NIFTY23DEC23500CE"
    name: String,            // e.g., "NIFTY"
    expiry: String,          // e.g., "26Dec2024"
    strike: f64,             // e.g., 23500.00
    lotsize: u32,            // e.g., 50
    instrumenttype: String,  // e.g., "OPTIDX"
    exch_seg: String,        // e.g., "NFO"
    tick_size: f64,          // e.g., 0.05
}
```

**Download URL**:

```
https://margincalculator.angelbroking.com/OpenAPI_File/files/OpenAPIScripMaster.json
```

#### Token Map Construction Flow

```
┌─────────────────────────────────────────────────────────────┐
│         INSTRUMENT MASTER PROCESSING                         │
└─────────────────────────────────────────────────────────────┘

    System Startup OR Daily Refresh (4:00 PM)
            ↓
    Step 1: Download CSV from Angel One
    ├─ URL: Angel One instrument master endpoint
    ├─ Fallback: Use cached file from previous day
    └─ Validate: Check file size > 0, valid JSON/CSV format
            ↓
    Step 2: Parse and Filter
    ├─ Filter: exch_seg == "NFO" (F&O segment only)
    ├─ Filter: instrumenttype IN ["OPTIDX", "OPTSTK"]
    └─ Filter: name IN ["NIFTY", "BANKNIFTY", "FINNIFTY"]
            ↓
    Step 3: Identify CE/PE from Symbol
    ├─ IF symbol.ends_with("CE") → Call option
    ├─ ELSE IF symbol.ends_with("PE") → Put option
    └─ ELSE → Underlying (skip)
            ↓
    Step 4: Create Token Map
    key = (name, expiry, strike, option_type)
    value = {
        token: token,
        symbol: symbol,
        lot_size: lotsize,
        tick_size: tick_size
    }
            ↓
    Step 5: Store to JSON
    ├─ File: data/tokens/token_map_YYYYMMDD.json
    ├─ Format: HashMap<(String, String, f64, String), TokenInfo>
    └─ Compression: None (active use)
            ↓
    Step 6: Validate Map
    ├─ Check: Each strike has both CE and PE
    ├─ Check: Minimum strikes available (>100 per underlying)
    └─ Alert: If strikes missing for current ATM range

REUSE: Same process for all underlyings (NIFTY, BANKNIFTY, FINNIFTY)
```

#### Token Lookup Pattern (REUSABLE)

```rust
fn get_option_token(
    underlying: &str,      // "NIFTY"
    strike: f64,           // 23500.0
    option_type: &str,     // "CE" or "PE"
) -> Result<TokenInfo> {

    // Step 1: Get current week expiry
    let expiry = get_weekly_expiry(underlying)?;

    // Step 2: Create lookup key
    let key = (
        underlying.to_string(),
        expiry.format("%d%b%Y").to_string(),  // "26Dec2024"
        strike,
        option_type.to_string()
    );

    // Step 3: Lookup in token map
    let token_map = load_token_map()?;
    let token_info = token_map.get(&key)
        .ok_or("Token not found for strike")?;

    // Step 4: Validate token
    validate_token_active(token_info)?;

    Ok(token_info.clone())
}

// REUSE: Call this before every option order
```

---

## 2. Broker-Specific Constraints (Angel One SmartAPI)

### 2.1 RMS Rules & Limits

#### Freeze Quantity Limits

**Definition**: Maximum quantity per single order  
**Enforcement**: Broker-side rejection if exceeded

```rust
fn get_freeze_quantity(underlying: &str) -> u32 {
    match underlying {
        "NIFTY" => 36_000,        // 720 lots × 50
        "BANKNIFTY" => 14_400,    // 960 lots × 15
        "FINNIFTY" => 40_000,     // 1000 lots × 40
        _ => 0,
    }
}

fn check_freeze_quantity(underlying: &str, quantity: u32) -> Result<()> {
    let freeze_qty = get_freeze_quantity(underlying);
    if quantity > freeze_qty {
        return Err(format!(
            "Order quantity {} exceeds freeze limit {}",
            quantity, freeze_qty
        ));
    }
    Ok(())
}

// REUSE: Call before every order
```

#### Price Band Validation (±20%)

```rust
fn validate_price_band(ltp: f64, order_price: f64) -> Result<()> {
    let lower_band = ltp * 0.80;
    let upper_band = ltp * 1.20;

    if order_price < lower_band || order_price > upper_band {
        return Err(format!(
            "Order price {} outside band [{}, {}]",
            order_price, lower_band, upper_band
        ));
    }
    Ok(())
}

// REUSE: Call before every order
```

#### Lot Size & Tick Size Validation

```rust
fn validate_lot_size(underlying: &str, quantity: u32) -> Result<()> {
    let lot_size = get_lot_size(underlying);
    if quantity % lot_size != 0 {
        return Err(format!(
            "Quantity {} not multiple of lot size {}",
            quantity, lot_size
        ));
    }
    Ok(())
}

fn validate_tick_size(price: f64) -> Result<()> {
    let tick_size = 0.05;
    let remainder = (price % tick_size).abs();
    if remainder > 0.001 {  // Floating point tolerance
        return Err(format!(
            "Price {} not multiple of tick size {}",
            price, tick_size
        ));
    }
    Ok(())
}

// REUSE: Call before every order
```

### 2.2 Angel One API Rate Limits

**REST API Limits** (verify with latest SmartAPI docs):

- Orders: 10 requests/second
- Market Data: 3 requests/second
- Historical: 3 requests/second

**WebSocket Limits**:

- Max subscriptions: 100 symbols
- Reconnect: Max 3 reconnects/minute

```rust
struct RateLimiter {
    tokens: u32,
    max_tokens: u32,
    refill_rate: Duration,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(max_requests_per_sec: u32) -> Self {
        Self {
            tokens: max_requests_per_sec,
            max_tokens: max_requests_per_sec,
            refill_rate: Duration::from_secs(1),
            last_refill: Instant::now(),
        }
    }

    fn acquire(&mut self) -> Result<()> {
        self.refill_tokens();

        if self.tokens > 0 {
            self.tokens -= 1;
            Ok(())
        } else {
            Err("Rate limit exceeded")
        }
    }

    fn refill_tokens(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_refill) >= self.refill_rate {
            self.tokens = self.max_tokens;
            self.last_refill = now;
        }
    }
}

// REUSE: Wrap all broker API calls with rate limiter
```

---

## 3. Holiday Calendar & Market Hours (NSE Specific)

### 3.1 NSE Holiday Calendar Management

**Source**: NSE official API  
**Update Frequency**: Annual (start of year), Monthly (for changes)

```rust
struct HolidayCalendar {
    holidays: HashSet<NaiveDate>,
    last_updated: DateTime<Utc>,
}

impl HolidayCalendar {
    async fn fetch_from_nse() -> Result<Self> {
        let url = "https://www.nseindia.com/api/holiday-master?type=trading";

        // Required headers for NSE API
        let headers = [
            ("User-Agent", "Mozilla/5.0"),
            ("Accept", "application/json"),
        ];

        let response = reqwest::Client::new()
            .get(url)
            .headers(headers.into_iter().collect())
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let holidays = Self::parse_holidays(&data)?;

        Ok(Self {
            holidays,
            last_updated: Utc::now(),
        })
    }

    fn is_trading_day(&self, date: NaiveDate) -> bool {
        // Check 1: Weekend
        if date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun {
            return false;
        }

        // Check 2: Holiday
        if self.holidays.contains(&date) {
            return false;
        }

        true
    }

    fn save_to_file(&self, path: &str) -> Result<()> {
        let holidays_vec: Vec<String> = self.holidays
            .iter()
            .map(|d| d.format("%Y-%m-%d").to_string())
            .collect();

        let json = serde_json::to_string_pretty(&holidays_vec)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

// REUSE: Check at system startup
```

### 3.2 Market Hours Validation

```rust
enum MarketSession {
    Holiday,
    PreMarket,
    Open,
    PostMarket,
    Closed,
}

fn get_market_session(now: DateTime<Tz>) -> MarketSession {
    let calendar = load_holiday_calendar();
    let today = now.date_naive();

    // Check 1: Trading day?
    if !calendar.is_trading_day(today) {
        return MarketSession::Holiday;
    }

    // Check 2: Time of day
    let time = now.time();
    match time {
        t if t >= NaiveTime::from_hms(9, 0, 0)
          && t < NaiveTime::from_hms(9, 15, 0) => {
            MarketSession::PreMarket
        }
        t if t >= NaiveTime::from_hms(9, 15, 0)
          && t < NaiveTime::from_hms(15, 30, 0) => {
            MarketSession::Open
        }
        t if t >= NaiveTime::from_hms(15, 30, 0)
          && t < NaiveTime::from_hms(16, 0, 0) => {
            MarketSession::PostMarket
        }
        _ => MarketSession::Closed,
    }
}

fn can_place_new_orders(session: &MarketSession, time: NaiveTime) -> bool {
    matches!(session, MarketSession::Open)
        && time >= NaiveTime::from_hms(10, 0, 0)
        && time < NaiveTime::from_hms(14, 30, 0)
}

// REUSE: Check before every order and at system startup
```

---

## 4. Data Quality & Synchronization

### 4.1 WebSocket Gap Detection & Auto-Recovery

```rust
struct GapDetector {
    last_tick_time: HashMap<String, Instant>,
    gap_threshold: Duration,
}

impl GapDetector {
    fn new() -> Self {
        Self {
            last_tick_time: HashMap::new(),
            gap_threshold: Duration::from_secs(60),
        }
    }

    fn on_tick(&mut self, symbol: &str) {
        self.last_tick_time.insert(symbol.to_string(), Instant::now());
    }

    async fn check_gaps(&self) -> Vec<String> {
        let now = Instant::now();
        let mut gaps = Vec::new();

        for (symbol, last_time) in &self.last_tick_time {
            if now.duration_since(*last_time) > self.gap_threshold {
                gaps.push(symbol.clone());
            }
        }

        gaps
    }

    async fn recover_gap(&self, symbol: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<OHLCV>> {
        // Fetch missing data from REST API
        let candles = fetch_historical_candles(symbol, start, end).await?;

        // Validate fetched data
        for candle in &candles {
            validate_candle(candle)?;
        }

        Ok(candles)
    }
}

// REUSE: Run every minute during trading hours
```

### 4.2 Opening Gap Handling

```rust
async fn handle_opening_gap() -> Result<()> {
    // Get yesterday's close
    let yesterday_close = get_previous_day_close("NIFTY").await?;

    // Get today's open
    let today_open = get_current_ltp("NIFTY").await?;

    // Calculate gap
    let gap_pct = (today_open - yesterday_close) / yesterday_close;

    if gap_pct.abs() > 0.02 {  // >2% gap
        log::warn!("Significant gap detected: {:.2}%", gap_pct * 100.0);

        // 1. Recalculate ATM
        let new_atm = calculate_atm("NIFTY", today_open)?;

        // 2. Cancel old subscriptions
        unsubscribe_all_strikes().await?;

        // 3. Subscribe to wider range
        let strike_range = if gap_pct > 0.0 {
            // Gap up: subscribe higher strikes
            (new_atm - 100.0, new_atm + 200.0)
        } else {
            // Gap down: subscribe lower strikes
            (new_atm - 200.0, new_atm + 100.0)
        };

        subscribe_strike_range("NIFTY", strike_range).await?;

        // 4. Wait for stabilization
        tokio::time::sleep(Duration::from_secs(300)).await;  // 5 minutes
    }

    Ok(())
}

// REUSE: Call at market open (9:15 AM)
```

---

## 5. Critical Missing Pieces Summary

### 5.1 What's in Structured Doc ✅

- System architecture & module dependencies
- Core component interfaces (Token Manager, Data Manager, etc.)
- Multi-timeframe ADX strategy flow
- Strike selection (ATM calculation)
- Entry/exit signal generation
- Risk management framework
- Order execution with idempotency
- Configuration templates
- Deployment checklist
- Reference tables

### 5.2 What's in This Supplement ✅

- **Instrument master CSV processing** (token discovery)
- **Token map construction & lookup** (strike → broker token)
- **Broker-specific constraints** (freeze qty, price bands, lot/tick size)
- **Angel One API rate limits** (REST & WebSocket)
- **NSE holiday calendar integration** (API, parsing, storage)
- **Market hours validation** (session detection, order timing)
- **Gap detection & recovery** (WebSocket gaps, opening gaps)
- **Data synchronization patterns** (REST fallback, validation)

### 5.3 Critical Items Still Needed (From Original) ⚠️

The following details from the original document should be referenced when needed:

1. **Indicator Formulas** (RSI, EMA calculations) - Section 19 of original
2. **Kill-Switch Implementation** - Section 14.2 of original
3. **Paper Trading Simulation** - Section 13.13 of original
4. **Reconciliation Procedures** - Section 13.3 of original
5. **Crash Recovery Process** - Section 13.14 of original
6. **Notification System** - Section 10.10 of original
7. **Performance Metrics** - Section 10.9 of original

---

## 6. Integration Guide

### How to Use Both Documents Together

```
┌─────────────────────────────────────────────────────────────┐
│              DOCUMENT USAGE FLOWCHART                        │
└─────────────────────────────────────────────────────────────┘

Need to understand system architecture?
    → Use: STRUCTURED document Section 1-2

Need to implement a specific component?
    → Use: STRUCTURED document Section 2 (Component Library)

Need trading strategy logic?
    → Use: STRUCTURED document Section 3 (Decision Trees)

Need broker-specific details?
    → Use: THIS SUPPLEMENT Section 2 (Broker Constraints)

Need token/instrument management?
    → Use: THIS SUPPLEMENT Section 1 (Token Discovery)

Need market hours/holiday logic?
    → Use: THIS SUPPLEMENT Section 3 (Holiday Calendar)

Need detailed indicator formulas?
    → Use: ORIGINAL document Section 19

Need operational procedures (kill-switch, recovery)?
    → Use: ORIGINAL document Sections 13-14

Need deployment/testing strategy?
    → Use: STRUCTURED document Section 7
```

### 6.1 Implementation Checklist (Combined)

```
□ ARCHITECTURE (Structured Doc)
  □ System startup flow
  □ Module dependencies
  □ Component interfaces

□ TOKEN MANAGEMENT (This Supplement)
  □ CSV download & parsing
  □ Token map construction
  □ Strike → token lookup

□ BROKER INTEGRATION (This Supplement)
  □ RMS constraint validation
  □ Rate limiting
  □ API wrapper with retry

□ TRADING LOGIC (Structured Doc)
  □ Multi-timeframe ADX
  □ Strike selection (ATM)
  □ Entry/exit signals

□ RISK CONTROLS (Structured Doc + Supplement)
  □ Pre-order validation (9 checks)
  □ Circuit breakers
  □ Position sizing

□ DATA MANAGEMENT (Structured Doc + Supplement)
  □ Storage hierarchy
  □ Gap detection & recovery
  □ Holiday calendar integration

□ ORDER EXECUTION (Structured Doc)
  □ Idempotency
  □ Order lifecycle
  □ Fill monitoring

□ OPERATIONS (Original Doc)
  □ Kill-switch
  □ Paper trading
  □ Reconciliation
  □ Crash recovery
```

---

## 7. Quick Decision Matrix

| Need to Find...      | Check Document | Section         |
| -------------------- | -------------- | --------------- |
| Component interface  | Structured     | 2.x             |
| Trading flow logic   | Structured     | 3.x             |
| Broker API details   | Supplement     | 2.1-2.2         |
| Token/strike mapping | Supplement     | 1.1-1.2         |
| Holiday calendar     | Supplement     | 3.1             |
| Indicator formulas   | Original       | 19.x            |
| Kill-switch          | Original       | 14.2            |
| Deployment guide     | Structured     | 7.x             |
| Risk parameters      | Structured     | 5.x, Appendix C |

---

**CONCLUSION**: Together, these documents provide:

- ✅ **Structured Doc**: Architecture, components, strategy logic
- ✅ **This Supplement**: Broker integration, token management, operational details
- ✅ **Original Doc**: Detailed formulas, procedures, edge cases

**No conflicts exist** - each document serves a distinct purpose. Use the decision matrix above to find information quickly.
