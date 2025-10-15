# Option Trading Bot Flow - Enhanced Implementation Guide

## 1. System Initialization & Startup

### 1.1 Project Startup Sequence

- **Step 1: Trading Day & Market Hours Validation**
  - **Trading Day Check**: Verify current day is Monday-Friday
  - **Holiday Calendar**: Check against NSE holiday list for current year
  - **Market Hours Validation**:
    - **Trading Hours**: 9:15 AM - 3:30 PM (Monday to Friday)
    - **Pre-market**: 9:00 AM - 9:15 AM (preparation phase)
    - **Post-market**: 3:30 PM - 4:00 PM (settlement phase)
    - **Weekend Check**: Skip trading on Saturday & Sunday
  - **Trading Mode Decision**:
    - **If trading day + market hours**: Proceed with full trading mode
    - **If non-trading day**: Continue with data management mode (no live trading)
- **Step 2: Token Validation & Session Check**
  - Load stored access token from configuration
  - Test token validity with simple API call (Angel One SmartAPI profile/portfolio endpoint)
  - **Session Duration Check**: Verify token will remain valid until market close (3:30 PM)
  - **If token invalid or expires before market close**: Proceed to auto-login
  - **If token valid for entire trading day**: Continue with system initialization
- **Step 3: SmartAPI Session Generation**
  - **REST Authentication**: Use Angel One SmartAPI session generation endpoint (approved flow only)
  - **Credentials**: Provide client code, password, and TOTP per SmartAPI spec (no browser automation)
  - **Tokens**: Obtain `jwtToken` for REST and `feedToken` for WebSocket streaming
  - **Token Storage**: Save tokens to secure OS storage (e.g., Windows Credential Manager), not plaintext
- **Step 4: Final Validation**
  - Test new token with API call
  - Verify all required permissions
  - Initialize SmartAPI clients (REST and WebSocket) with valid tokens
  - Confirm system ready for trading

### 1.2 Configuration Loading

- Load trading parameters from config file
- Set position sizing rules
- Configure risk management limits
- Load strategy parameters
- Set up logging and monitoring

### 1.3 Token Management & Authentication (Angel One SmartAPI)

- This section supersedes token/auth notes in 1.1; follow this SmartAPI-centric flow.

- **Token Validation Process**:
  - Load stored access token from secure storage
  - Test token validity with Angel One SmartAPI account/profile endpoint
  - **Critical: Session Duration Check**: Verify token will remain valid until market close (3:30 PM)
  - **If token invalid or expires before market close**: Generate a new session per SmartAPI
  - **If token valid for entire trading day**: Continue with system initialization
- **SmartAPI Session Workflow**:
  - **REST Authentication**: Use Angel One SmartAPI session generation endpoint
  - **Credentials**: Provide client code, password, and TOTP per SmartAPI spec
  - **Tokens**: Obtain `jwtToken` for REST and `feedToken` for WebSocket streaming
  - **Secure Storage**: Store tokens in OS keychain/DPAPI; never plaintext or logs
- **Critical: Daily Token/Session Handling**:
  - **Token Expiry**: SmartAPI sessions typically expire daily; verify current policy in docs
  - **Pre-Market Validation**: Check token validity before 9:15 AM; refresh proactively if near expiry
  - **Session Duration Check**: Verify token will remain valid until 3:30 PM
  - **During Trading**: Monitor REST auth and WS `feedToken` health every 5 minutes
  - **Emergency Response**: Pause trading if token/WS auth expires during market hours
  - **User Notification**: Alert user when token refresh is required per TOS
- **Continuous Session Monitoring**:
  - **Pre-Trading Check**: Ensure token valid for entire trading day
  - **Mid-Session Check**: Monitor token status every 5 minutes
  - **Proactive Re-login**: Renew session if token expires before market close
  - **Trading Continuity**: Prevent disruption during active trading hours
- **Error Handling**:
  - Handle login failures gracefully
  - Retry mechanism for network issues
  - Clear error messages for user guidance
  - Fallback to manual token entry if automation fails

### 1.4 Startup Checklist & Validation

- **Pre-Startup Checks**:
  - Verify system date and time accuracy
  - Check internet connectivity
  - Validate configuration files exist
  - Ensure required dependencies are installed
  - Check disk space availability
- **Final System Validation**:
  - Test all API endpoints
  - Verify WebSocket connection
  - Check data feed availability
  - Confirm all systems ready for trading

### 1.5 Non-Trading Day Operations

- **Data Management Mode** (Weekends, Holidays, After Hours):
  - **Historical Data Download**: Download missing historical data
  - **Data Gap Detection**: Identify and log missing data periods
  - **Data Validation**: Verify data integrity and completeness
  - **Token Refresh**: Update access tokens if needed
  - **System Maintenance**: Run JSON file cleanup and archival
  - **Backup Operations**: Create data backups and snapshots
  - **Strategy Analysis**: Run backtesting on historical data
  - **Report Generation**: Create performance and data quality reports
- **Data Gap Detection Process**:
  - Compare expected vs actual data points
  - Identify missing timeframes and instruments
  - Log gaps with timestamps and reasons
  - Prioritize critical data gaps for download
  - Generate gap report for manual review
- **Maintenance Tasks**:
  - Clean up temporary files
  - Optimize JSON file storage (compression, archival)
  - Update instrument lists
  - Refresh configuration files
  - Test system components
- **Monitoring & Alerts**:
  - Check system health status
  - Monitor data quality metrics
  - Send alerts for critical issues
  - Log all operations for audit trail

### 1.6 System Health Check

- Verify JSON file storage directories exist and are writable
- Check disk space availability
- Validate network connectivity
- Test WebSocket connection capability
- Run system diagnostics

## 2. Data Management & Synchronization

### 2.1 Data Source Strategy & JSON Storage

- **Historical API**:

  - Download 1 year of daily data for underlying stocks/indices (ADX calculation)
  - Download 3 months of 1-hour data for medium-term analysis
  - **JSON File Storage**: Store data in simple JSON files (no database)
  - **Compression Format**: Use gzip (.gz) for archived files (older than 7 days)
  - **Active Files**: Keep as plain JSON for easy access and debugging
  - **File Structure** (Hybrid Organization):
    - `raw/[symbol]_today.json` (current day - plain JSON)
    - `raw/[symbol]_yesterday.json` (previous day, delete after 2 days - plain JSON)
    - `timeframes/[symbol]/1m.json` (3 months - compress files older than 7 days to .json.gz)
    - `timeframes/[symbol]/1h.json` (3 months - compress files older than 7 days to .json.gz)
    - `timeframes/[symbol]/daily.json` (1 year - compress files older than 7 days to .json.gz)
  - **When to use**: System startup, ADX calculation, backtesting

- **Market Data API**:

  - Get current LTP for specific instruments
  - Fetch option chain data periodically
  - Get margin and position details
  - Check instrument status and validity
  - **When to use**: Top-of-book strikes every 1-2 minutes; full chain every 5-10 minutes; on-demand queries; validation

- **WebSocket**:

  - Live price updates for active trading
  - Real-time option chain changes
  - Instant order book updates
  - Live P&L and position monitoring
  - **When to use**: During active trading hours, real-time decisions

#### 2.1.1 JSON Schemas (storage format)

```json
{
  "raw_tick": {
    "ts_exchange": "2025-10-15T09:15:00.123+05:30", // exchange timestamp if available
    "ts_utc": "2025-10-15T03:45:00.123Z", // system UTC timestamp
    "symbol": "NIFTY",
    "token": 12345,
    "ltp": 23456.7,
    "bid": 23456.5,
    "ask": 23456.9,
    "volume": 1000
  },
  "bar_1m": {
    "start_ts_utc": "2025-10-15T03:45:00Z",
    "end_ts_utc": "2025-10-15T03:46:00Z",
    "o": 23450.0,
    "h": 23470.0,
    "l": 23445.0,
    "c": 23465.0,
    "v": 12000
  }
}
```

- **Time convention**: Store times in UTC; preserve broker exchange timestamps when provided; NTP-sync system clock.
- **Data fidelity**: WebSocket ticks are snapshots; reconcile 1m OHLC with official historical candles at EOD or on reconnect. Never interpolate options.

#### 2.1.0 SmartAPI-only Data Policy (Live Trading)

- All live market data, quotes, and order events will be sourced exclusively from Angel One SmartAPI (REST + WebSocket)
- Minute bars for both underlyings and options are constructed from SmartAPI WebSocket ticks; no third‑party feeds
- Historical candles: use SmartAPI historical endpoints where available; otherwise rely on locally persisted bars built from SmartAPI ticks
- Option chain: build from SmartAPI instrument master + live LTP via REST/WS; subscribe to a rotating subset of strikes as per rate/subscription limits
- OI/Greeks: if SmartAPI returns these in quotes, consume them; otherwise proceed without them (strategy must not depend on non‑available fields)

### 2.2 Underlying-Options Data Sync Strategy

- **Primary Data Source**: Use underlying (NIFTY, BANKNIFTY) for trend analysis

  - Download 1-2 years of daily underlying data
  - Calculate daily/weekly/monthly trends
  - Generate trend signals from underlying data
  - Use for higher timeframe trend confirmation

- **Options Data Integration**:

  - Download 3 months of options data for current trading
  - Sync options data with underlying trends
  - Map options strikes to underlying price levels
  - Update options data when underlying trends change

- **Data Synchronization Process**:
  - **Daily Sync**: Update underlying daily data every night
  - **Options Refresh**: Update options data monthly (new contracts)
  - **Trend Alignment**: Ensure options strategy aligns with underlying trends
  - **Token Mapping**: Keep options tokens synced with underlying price levels

### 2.3 Timeframe Construction from Tick Data

- **Market Time Boundaries**: Use exact market time (9:15 AM - 3:30 PM)
- **Raw Tick Processing**:
  - **Tick Storage**: Store 2 days of raw ticks in JSON files
  - **Gap Detection**: Compare current day open with previous day close
  - **1-minute bars**: 9:15-9:16, 9:16-9:17, 9:17-9:18... 3:29-3:30
- **Timeframe Construction Flow**:
  - **Step 1**: Raw ticks → 1-minute bars → Store in `timeframes/[symbol]/1m.json`
  - **Step 2**: 1-minute bars → 5-minute bars → Store in `timeframes/[symbol]/5m.json`
  - **Step 3**: 1-minute bars → 15-minute bars → Store in `timeframes/[symbol]/15m.json`
  - **Step 4**: 1-minute bars → 1-hour bars → Store in `timeframes/[symbol]/1h.json`
  - **Step 5**: 1-hour bars → Daily bars → Store in `timeframes/[symbol]/daily.json`
- **Data Retention**:
  - **Raw Ticks**: 2 days only (for gap detection)
  - **1-minute bars**: 3 months
  - **5-minute bars**: 3 months
  - **15-minute bars**: 3 months
  - **1-hour bars**: 3 months
  - **Daily bars**: 1 year
- **JSON File Structure** (Hybrid Organization with Daily Rotation):
  - `raw/NIFTY_today.json` (current day)
  - `raw/NIFTY_yesterday.json` (previous day, delete after 2 days)
  - `raw/BANKNIFTY_today.json` (current day)
  - `raw/BANKNIFTY_yesterday.json` (previous day, delete after 2 days)
  - `timeframes/NIFTY/1m.json` (3 months)
  - `timeframes/NIFTY/5m.json` (3 months)
  - `timeframes/NIFTY/15m.json` (3 months)
  - `timeframes/NIFTY/1h.json` (3 months)
  - `timeframes/NIFTY/daily.json` (1 year)
  - `timeframes/BANKNIFTY/1m.json` (3 months)
  - `timeframes/BANKNIFTY/1h.json` (3 months)
  - `timeframes/BANKNIFTY/daily.json` (1 year)

### 2.4 Missing Data Handling & Gap Detection

- **Gap Detection Process**:
  - **Previous Day Close**: Get closing price from `timeframes/[symbol]/daily.json`
  - **Current Day Open**: Get opening price from raw ticks
  - **Gap Calculation**: (Open - Previous Close) / Previous Close \* 100
  - **Gap Threshold**: Trigger handling if gap >2% (approximately 100+ points for NIFTY)
- **Gap Response Strategy**:
  - **Immediate Token Refresh**: Cancel all existing subscriptions
  - **New ATM Calculation**: Recalculate ATM based on gap-adjusted price
  - **Emergency Token Pool**: Subscribe to 10-15 strikes around new ATM
  - **Wider Strike Range**: Use ±100 points instead of ±50 for gap scenarios
- **Data Recovery**: Request missing data from broker API
- **Options Data Strategy**:
  - **No Interpolation**: Never interpolate option prices (discrete jumps)
  - **Discard Incomplete Bars**: Remove bars with missing data points
  - **Wait for Complete Data**: Only use fully complete bars for analysis
  - **Strike Validation**: Ensure all strikes in bar have valid data
- **Underlying Data Strategy**:
  - **Interpolation Allowed**: Linear interpolation for underlying prices only
  - **Small Gaps**: Fill gaps <5 minutes for underlying data
  - **Large Gaps**: Discard bars with gaps >10 minutes

### 2.5 Angel One SmartAPI Rate Limits & Management

- **Rate Limits** (verify with latest SmartAPI docs):
  - **REST (market/order)**: Enforce per-second and per-minute limits
  - **WebSocket**: Connection and subscription limits apply
  - **Historical**: Throttled; batch and cache where possible
- **Rate Limit Handling**:
  - **Request Queuing**: Queue requests to stay within limits
  - **Exponential Backoff**: Retry with increasing delays on 429/5xx
  - **Request Batching**: Combine multiple requests where possible
  - **Priority System**: Critical requests (orders) get priority
- **Optimization Strategies**:
  - **Cache Data**: Store frequently accessed data locally
  - **Batch Requests**: Use SmartAPI bulk quote endpoints where available
  - **WebSocket Priority**: Use WebSocket for real-time data
  - **Scheduled Updates**: Update non-critical data per cadence above

## 3. Token Management & ATM Selection

**⚠️ STRATEGY OVERVIEW: Multi-Timeframe ADX Approach**

```
┌──────────────────────────────────────────────────────────┐
│     COMPLETE STRATEGY FLOW (Daily + Hourly)              │
└──────────────────────────────────────────────────────────┘

Step 1: DAILY Timeframe Analysis (9:15 AM)
───────────────────────────────────────────
Purpose: Determine WHICH side to trade (CE or PE)
Data: Last 30 DAILY bars
Indicators: ADX, +DI, -DI (14-period)

IF Daily ADX < 25:
    → NO TRADE (weak trend)

ELSE IF Daily +DI > Daily -DI:
    → Direction = CE (bullish trend)
    → Wait for hourly confirmation

ELSE IF Daily -DI > Daily +DI:
    → Direction = PE (bearish trend)
    → Wait for hourly confirmation

Step 2: HOURLY Timeframe Analysis (Dynamic Sync)
────────────────────────────────────────────────
Purpose: Determine WHEN to enter (NOT to reverse direction!)
Data: Last 30 HOURLY bars (properly aligned)
Indicators: ADX, +DI, -DI (14-period)

⚠️ CRITICAL: Daily trend is MASTER for entire day
             Hourly only confirms entry timing
             NEVER trade against daily trend!

⚠️ CANDLE TIMING: Hourly candles MUST be market-hour aligned
   Market Hours: 9:15 AM - 3:30 PM (6 hours 15 minutes)

Hourly Candle Schedule (2 Options):
────────────────────────────────────

Option A: 1-Hour Fixed Intervals (Recommended)
├─ 09:15 - 10:15 (1st hourly candle)
├─ 10:15 - 11:15 (2nd hourly candle)
├─ 11:15 - 12:15 (3rd hourly candle)
├─ 12:15 - 13:15 (4th hourly candle)
├─ 13:15 - 14:15 (5th hourly candle)
└─ 14:15 - 15:15 (6th hourly candle, closes at 15:30)

Option B: Regular Hour Boundaries
├─ 09:15 - 10:00 (45 min, partial)
├─ 10:00 - 11:00 (1st full hour)
├─ 11:00 - 12:00 (2nd full hour)
├─ 12:00 - 13:00 (3rd full hour)
├─ 13:00 - 14:00 (4th full hour)
├─ 14:00 - 15:00 (5th full hour)
└─ 15:00 - 15:30 (30 min, partial)

Recommended: Option A (Fixed 60-min intervals from 9:15)
Reason: No partial candles, consistent timing

Real-Time Candle Construction:
───────────────────────────────

FUNCTION BUILD_HOURLY_CANDLE(start_time, end_time):

    // Get all tick data from Angel One API
    ticks = GET_TICKS_FROM_API(
        symbol: "NIFTY",
        start: start_time,
        end: end_time
    )

    // Construct OHLC
    candle = {
        open: ticks[0].ltp,           // First tick LTP
        high: MAX(ticks.ltp),         // Highest LTP
        low: MIN(ticks.ltp),          // Lowest LTP
        close: ticks[last].ltp,       // Last tick LTP
        volume: SUM(ticks.volume),    // Total volume
        timestamp: end_time           // Candle close time
    }

    RETURN candle

Update Schedule:
────────────────
Every candle close (10:15, 11:15, 12:15, etc.):
1. Construct completed hourly candle
2. Calculate ADX, +DI, -DI on last 30 hourly bars
3. Check alignment with daily direction
4. Generate entry/exit signals

Mid-Candle Updates (Optional):
────────────────────────────────
Every 1 minute during candle formation:
1. Update current candle (partial)
2. Calculate indicators with partial candle
3. Prepare for potential signals
4. Don't trade until candle closes ✅

IF daily_direction == CE:
    // Wait for hourly candle to close
    IF hourly_candle_closed:
        hourly_adx = CALCULATE_HOURLY_ADX(last_30_candles)
        hourly_plus_di = CALCULATE_HOURLY_PLUS_DI(last_30_candles)
        hourly_minus_di = CALCULATE_HOURLY_MINUS_DI(last_30_candles)

        IF hourly_adx >= 25 AND hourly_plus_di > hourly_minus_di:
            → Hourly CONFIRMS daily bullish ✅
            → WAIT for CE crossover signal
            → Then BUY CE ✅
        ELSE:
            → NO TRADE ⏳ (Wait for hourly to align back)
            → DO NOT switch to PE!

ELSE IF daily_direction == PE:
    IF hourly_candle_closed:
        hourly_adx = CALCULATE_HOURLY_ADX(last_30_candles)
        hourly_plus_di = CALCULATE_HOURLY_PLUS_DI(last_30_candles)
        hourly_minus_di = CALCULATE_HOURLY_MINUS_DI(last_30_candles)

        IF hourly_adx >= 25 AND hourly_minus_di > hourly_plus_di:
            → Hourly CONFIRMS daily bearish ✅
            → WAIT for PE crossover signal
            → Then BUY PE ✅
        ELSE:
            → NO TRADE ⏳ (Wait for hourly to align back)
            → DO NOT switch to CE!

Real-Time Data Flow from Angel One API:
────────────────────────────────────────

⚠️ CRITICAL: WebSocket has frequent gaps!
   PRIMARY: REST API for candle data (reliable)
   SECONDARY: WebSocket for real-time price alerts only

┌─────────────────────────────────────────────────────────┐
│  REST API → Historical Candles → Indicators (PRIMARY)  │
│  WebSocket → Live LTP → Alerts/Monitoring (SECONDARY)  │
└─────────────────────────────────────────────────────────┘

Recommended Data Strategy:
──────────────────────────

✅ PRIMARY: Use REST API for all candle data
   - More reliable (no gaps)
   - Official exchange data
   - Consistent OHLCV values
   - Better for indicator calculation

❌ SECONDARY: WebSocket only for monitoring
   - Real-time LTP for alerts
   - Position P&L tracking
   - ATM strike updates
   - NOT for building candles

Step-by-Step Data Processing (REST API Focused):
─────────────────────────────────────────────────

1. Initial Load (9:00 AM):
   ├─ Fetch last 30 DAILY candles via REST API
   ├─ Fetch last 30 HOURLY candles via REST API
   ├─ Calculate initial indicators
   └─ Store in memory + JSON files

2. Real-Time Updates (Every Hourly Candle Close):

   At 10:15:00 (candle closes):
   ├─ Fetch completed hourly candle via REST API
   ├─ Validate OHLCV data
   ├─ Add to hourly_candles array
   ├─ Recalculate indicators
   └─ Generate entry/exit signals

3. WebSocket (Parallel, for monitoring only):
   ├─ Subscribe to NIFTY/BANKNIFTY LTP
   ├─ Update current price every tick
   ├─ Check stop loss in real-time
   ├─ Update ATM strike calculation
   └─ Alert on significant moves

4. Between Candle Closes (10:16 - 11:14):
   ├─ Monitor current price via WebSocket LTP
   ├─ Check for stop loss hits
   ├─ Track position P&L
   ├─ Wait for next candle (11:15)
   └─ Don't trade on partial data

5. At Next Candle Close (11:15:00):
   ├─ Fetch new completed candle via REST API
   ├─ Update indicators
   ├─ Check alignment (daily + hourly)
   └─ Generate new signals

Complete Algorithm:
───────────────────

```

GLOBAL:
tick_buffer = {} // Store ticks for each symbol
hourly_candles = [] // Store completed hourly candles
current_candle_start = "09:15:00"

FUNCTION ON_WEBSOCKET_TICK(tick):

    // Store tick in buffer
    symbol = tick.symbol  // e.g., "NIFTY"

    IF symbol NOT IN tick_buffer:
        tick_buffer[symbol] = []

    tick_buffer[symbol].append({
        ltp: tick.ltp,
        volume: tick.volume,
        timestamp: tick.timestamp
    })

    // Check if candle should close
    current_time = GET_CURRENT_TIME()

    IF IS_CANDLE_CLOSE_TIME(current_time):  // 10:15, 11:15, etc.
        BUILD_AND_SAVE_CANDLE(symbol, current_candle_start, current_time)

        // Reset for next candle
        tick_buffer[symbol] = []
        current_candle_start = current_time

FUNCTION BUILD_AND_SAVE_CANDLE(symbol, start_time, end_time):

    ticks = tick_buffer[symbol]

    IF ticks.length == 0:
        LOG_WARN("No ticks received for candle!")
        RETURN

    // Construct OHLC from ticks
    candle = {
        symbol: symbol,
        timeframe: "1H",
        open: ticks[0].ltp,
        high: MAX([t.ltp for t in ticks]),
        low: MIN([t.ltp for t in ticks]),
        close: ticks[last].ltp,
        volume: SUM([t.volume for t in ticks]),
        timestamp: end_time,
        tick_count: ticks.length
    }

    // Save to hourly candles array
    hourly_candles.append(candle)

    // Save to JSON file (persistent storage)
    SAVE_CANDLE_TO_FILE(candle)

    LOG_INFO("Hourly candle completed:", candle)

    // Trigger indicator calculation
    IF hourly_candles.length >= 30:
        CALCULATE_HOURLY_INDICATORS()

FUNCTION IS_CANDLE_CLOSE_TIME(current_time):

    candle_close_times = [
        "10:15:00",
        "11:15:00",
        "12:15:00",
        "13:15:00",
        "14:15:00",
        "15:15:00"
    ]

    // Check if current time matches any close time
    // Allow 5-second window (10:15:00 to 10:15:05)
    FOR close_time IN candle_close_times:
        IF current_time >= close_time AND
           current_time < close_time + 5_seconds:
            RETURN TRUE

    RETURN FALSE

FUNCTION CALCULATE_HOURLY_INDICATORS():

    // Use last 30 hourly candles
    bars = hourly_candles[-30:]

    // Calculate indicators
    adx = CALCULATE_ADX(bars, period=14)
    plus_di = CALCULATE_PLUS_DI(bars, period=14)
    minus_di = CALCULATE_MINUS_DI(bars, period=14)

    // Store results
    hourly_indicators = {
        adx: adx,
        plus_di: plus_di,
        minus_di: minus_di,
        timestamp: GET_CURRENT_TIME()
    }

    // Check alignment with daily
    CHECK_DAILY_HOURLY_ALIGNMENT()

```

**UPDATED: Hybrid Strategy (REST Primary + WebSocket Secondary):**

```

⚠️ CRITICAL CHANGES Based on WebSocket Gap Issues:

1. PRIMARY DATA SOURCE: REST API (Historical endpoint)

   - Download initial 30 hourly candles when token selected
   - Fetch new completed candle at every hourly close
   - More reliable, no gaps, official exchange data

2. SECONDARY: WebSocket (Monitoring only)

   - Real-time LTP for stop loss checks
   - Position P&L tracking
   - Gap detection (if >1 minute → fetch from REST)

3. AUTO-RECOVERY: Gap >1 Minute
   - Detect: current_time - last_tick_timestamp > 60s
   - Action: Fetch missing candles from REST API immediately
   - Insert sorted, recalculate indicators

Complete Updated Flow:
──────────────────────

ON_TOKEN_SELECTED(token, symbol):
historical_data = FETCH_REST_API(symbol, 30 candles)
VALIDATE_AND_STORE(historical_data)
CALCULATE_INDICATORS(symbol)
SUBSCRIBE_WEBSOCKET(token) // For monitoring only
last_tick_timestamp[symbol] = NOW()

ON_WEBSOCKET_TICK(tick):
gap = NOW() - last_tick_timestamp[symbol]
IF gap > 60_seconds:
FILL_GAP_FROM_REST(symbol, last_tick, NOW()) // Auto-fix
last_tick_timestamp[symbol] = NOW()
// Use LTP for stop loss checks, ATM updates

ON_HOURLY_CANDLE_CLOSE(time):
SLEEP(30_seconds) // Wait for REST API to have data
new_candle = FETCH_REST_API(symbol, time, "60m")
VALIDATE_AND_ADD(new_candle)
RECALCULATE_INDICATORS(symbol)
GENERATE_SIGNALS()

Benefits of This Approach:
───────────────────────────
✅ No gaps (REST API is reliable)
✅ Official exchange data
✅ Auto-recovery if WebSocket fails
✅ WebSocket used for real-time monitoring only
✅ Best of both worlds

```

**ANGEL ONE SmartAPI: Recommended Approach**

```

═══════════════════════════════════════════════════════════
✅ BEST PRACTICE for Angel One SmartAPI
═══════════════════════════════════════════════════════════

PRIMARY: REST API for ALL candle data
SECONDARY: WebSocket for LTP monitoring only

Why This Works Best for Angel One:
───────────────────────────────────

1. REST API Has Historical Endpoint ✅

   - Endpoint: /rest/secure/angelbroking/historical/v1/getCandleData
   - Supports: 1h, 1d intervals (we need these)
   - Rate Limits: 3 req/sec (we use <1 req/hour)
   - Reliability: 99%+ uptime, no gaps
   - Data Quality: Official exchange data

2. WebSocket Issues (Common) ⚠️

   - Frequent gaps (as you mentioned)
   - Disconnection/reconnection delays
   - Cannot fetch historical ticks after disconnect
   - Max 100 symbol limit

3. Our Usage is Very Light:
   - Initial load: 2 API calls (daily + hourly)
   - Per hour: 1 API call (new candle)
   - Total: ~8 calls/day (well within limits!)

Implementation for Angel One:
─────────────────────────────

9:00 AM: Load historical data via REST API
10:15 AM: Fetch new hourly candle via REST API
11:15 AM: Fetch new hourly candle via REST API
... continues every hour

WebSocket: Only for live LTP (stop loss, alerts)

No gaps, no sync issues, 100% reliable! ✅

```

Handling Edge Cases:
────────────────────

1. Missing Ticks (WebSocket Disconnect):
   ├─ If no ticks for >60 seconds → Reconnect
   ├─ Fetch missing data via REST API
   └─ Reconstruct candle from REST historical data

2. Partial Candles (System Restart):
   ├─ On restart at 10:30 AM
   ├─ Current candle (10:15-11:15) is partial
   ├─ Use partial candle for monitoring
   └─ Wait for 11:15 for confirmed signal

3. Market Close (3:30 PM):
   ├─ Last candle: 14:15 - 15:15
   ├─ Extends to 15:30 (extra 15 minutes)
   └─ Build final candle at 15:30

4. Tick Timestamp Sync:
   ├─ ALWAYS use exchange timestamp from tick
   ├─ NOT local system time
   ├─ Handle timezone (IST)
   └─ Validate timestamp sequence

Data Quality & Synchronization (CRITICAL):
───────────────────────────────────────────

⚠️ RULE: NEVER trade without fully synchronized, validated data!

Complete Data Sync Strategy:
─────────────────────────────

Step 1: Initial Data Loading (Before 9:15 AM)
──────────────────────────────────────────────

At 9:00 AM (15 minutes before market open):

1. Load Historical Daily Data:
   ├─ Fetch last 30 DAILY candles from Angel One REST API
   ├─ Validate: Each candle has complete OHLCV
   ├─ Calculate: Daily ADX, +DI, -DI
   └─ Store: In memory + JSON file

2. Load Historical Hourly Data:
   ├─ Fetch last 30 HOURLY candles from Angel One REST API
   ├─ Validate: Candles are properly aligned (9:15-10:15, etc.)
   ├─ Calculate: Hourly ADX, +DI, -DI
   └─ Store: In memory + JSON file

3. Data Validation Before Trading:
   ├─ Check: All 30 daily candles present
   ├─ Check: All 30 hourly candles present
   ├─ Check: No gaps in data
   ├─ Check: Timestamps are sequential
   └─ Status: READY or NOT_READY

```

FUNCTION INITIALIZE_DATA_BEFORE_TRADING():

    LOG_INFO("Starting data initialization at 9:00 AM...")

    // Step 1: Fetch Daily Data
    daily_candles = FETCH_DAILY_HISTORICAL(
        symbol: "NIFTY",
        count: 30
    )

    IF daily_candles.length < 30:
        LOG_ERROR("Insufficient daily data:", daily_candles.length)
        RETURN "NOT_READY"

    // Validate daily data
    IF NOT VALIDATE_CANDLE_ARRAY(daily_candles):
        LOG_ERROR("Daily data validation failed")
        RETURN "NOT_READY"

    // Step 2: Fetch Hourly Data
    hourly_candles = FETCH_HOURLY_HISTORICAL(
        symbol: "NIFTY",
        count: 30,
        interval: "60m"
    )

    IF hourly_candles.length < 30:
        LOG_ERROR("Insufficient hourly data:", hourly_candles.length)
        RETURN "NOT_READY"

    // Validate hourly data
    IF NOT VALIDATE_CANDLE_ARRAY(hourly_candles):
        LOG_ERROR("Hourly data validation failed")
        RETURN "NOT_READY"

    // Step 3: Calculate Initial Indicators
    daily_indicators = CALCULATE_DAILY_INDICATORS(daily_candles)
    hourly_indicators = CALCULATE_HOURLY_INDICATORS(hourly_candles)

    // Step 4: Store in memory
    STORE_IN_MEMORY(daily_candles, hourly_candles)

    // Step 5: Save to persistent storage
    SAVE_TO_JSON(daily_candles, "data/daily_candles_YYYYMMDD.json")
    SAVE_TO_JSON(hourly_candles, "data/hourly_candles_YYYYMMDD.json")

    LOG_INFO("Data initialization complete ✅")
    RETURN "READY"

```

Step 2: Real-Time Sync During Trading Hours
────────────────────────────────────────────

At every hourly candle close (10:15, 11:15, etc.):

1. Build Candle from WebSocket Ticks
2. Validate Constructed Candle
3. Cross-Check with REST API (for sync)
4. Compare: WebSocket candle vs REST candle
5. If mismatch → Use REST candle (official source)
6. Update indicators with new candle
7. Generate signals

```

FUNCTION ON_HOURLY_CANDLE_CLOSE(time):

    LOG_INFO("Hourly candle closed at:", time)

    // Build from WebSocket ticks
    ws_candle = BUILD_CANDLE_FROM_TICKS(tick_buffer)

    // Validate basic OHLC
    IF NOT VALIDATE_CANDLE(ws_candle):
        LOG_ERROR("WebSocket candle validation failed")
        // Try REST API as fallback
        ws_candle = FETCH_REST_CANDLE(time)

    // Cross-check with REST API (official source)
    rest_candle = FETCH_REST_CANDLE_FOR_VALIDATION(time)

    // Compare WebSocket vs REST
    sync_status = COMPARE_CANDLES(ws_candle, rest_candle)

    IF sync_status == "MISMATCH":
        LOG_WARN("WebSocket vs REST mismatch detected!")
        LOG_WARN("WS:", ws_candle)
        LOG_WARN("REST:", rest_candle)

        // Use REST as source of truth
        final_candle = rest_candle

        // Alert: Data sync issue
        SEND_ALERT("Data sync issue detected, using REST API")
    ELSE:
        LOG_INFO("Data sync OK ✅")
        final_candle = ws_candle

    // Add to candle array
    hourly_candles.append(final_candle)

    // Keep only last 30 candles in memory
    IF hourly_candles.length > 30:
        hourly_candles = hourly_candles[-30:]

    // Save to persistent storage
    SAVE_CANDLE_TO_JSON(final_candle)

    // Update indicators
    UPDATE_INDICATORS(hourly_candles)

    // Check if ready for trading
    IF hourly_candles.length >= 30:
        trading_ready = TRUE

    RETURN final_candle

```

Step 3: Continuous Data Quality Monitoring
───────────────────────────────────────────

Monitor every minute during trading hours:

```

FUNCTION MONITOR_DATA_QUALITY():

    EVERY 1 MINUTE:

        // Check 1: WebSocket Connection
        IF NOT websocket.is_connected():
            LOG_ERROR("WebSocket disconnected!")
            RECONNECT_WEBSOCKET()

        // Check 2: Tick Flow
        last_tick_time = GET_LAST_TICK_TIMESTAMP()
        time_since_last_tick = NOW() - last_tick_time

        IF time_since_last_tick > 60_seconds:
            LOG_ERROR("No ticks for", time_since_last_tick, "seconds!")
            TRIGGER_DATA_RESYNC()

        // Check 3: Candle Continuity
        IF GAPS_DETECTED_IN_CANDLES():
            LOG_ERROR("Gap detected in candle sequence!")
            FILL_GAPS_FROM_REST_API()

        // Check 4: Indicator Freshness
        indicator_age = NOW() - hourly_indicators.timestamp
        IF indicator_age > 3600_seconds:  // 1 hour old
            LOG_WARN("Indicators not updated for 1 hour!")
            RECALCULATE_INDICATORS()

```

Data Quality Validation Functions:
───────────────────────────────────

FUNCTION VALIDATE_CANDLE(candle):

    // Check 1: Required fields present
    IF NOT candle.has_fields(["open", "high", "low", "close", "volume", "timestamp"]):
        LOG_ERROR("Missing required fields in candle")
        RETURN FALSE

    // Check 2: OHLC relationship
    IF candle.high < candle.low:
        LOG_ERROR("Invalid: High < Low")
        RETURN FALSE

    IF candle.open > candle.high OR candle.open < candle.low:
        LOG_ERROR("Invalid: Open outside H-L range")
        RETURN FALSE

    IF candle.close > candle.high OR candle.close < candle.low:
        LOG_ERROR("Invalid: Close outside H-L range")
        RETURN FALSE

    // Check 3: Volume sanity
    IF candle.volume < 0:
        LOG_ERROR("Invalid: Negative volume")
        RETURN FALSE

    // Check 4: Price sanity (not zero or negative)
    IF candle.open <= 0 OR candle.high <= 0 OR
       candle.low <= 0 OR candle.close <= 0:
        LOG_ERROR("Invalid: Zero or negative price")
        RETURN FALSE

    // Check 5: Reasonable price movement
    range_pct = (candle.high - candle.low) / candle.open * 100
    IF range_pct > 5:  // More than 5% in 1 hour
        LOG_WARN("Unusual: Price range", range_pct, "%")
        // Don't fail, just warn

    RETURN TRUE

FUNCTION VALIDATE_CANDLE_ARRAY(candles):

    // Check 1: Sufficient count
    IF candles.length < 30:
        LOG_ERROR("Insufficient candles:", candles.length)
        RETURN FALSE

    // Check 2: Each candle is valid
    FOR candle IN candles:
        IF NOT VALIDATE_CANDLE(candle):
            RETURN FALSE

    // Check 3: Timestamps are sequential
    FOR i FROM 1 TO candles.length - 1:
        IF candles[i].timestamp <= candles[i-1].timestamp:
            LOG_ERROR("Non-sequential timestamps at index", i)
            RETURN FALSE

    // Check 4: No gaps (for hourly: 60 min between candles)
    FOR i FROM 1 TO candles.length - 1:
        time_diff = candles[i].timestamp - candles[i-1].timestamp
        expected_diff = 3600  // 60 minutes in seconds

        IF ABS(time_diff - expected_diff) > 300:  // 5 min tolerance
            LOG_ERROR("Gap detected between candles:", time_diff, "seconds")
            RETURN FALSE

    // Check 5: Price continuity (no wild jumps)
    FOR i FROM 1 TO candles.length - 1:
        gap_pct = ABS(candles[i].open - candles[i-1].close) / candles[i-1].close * 100

        IF gap_pct > 3:  // More than 3% gap
            LOG_WARN("Large gap between candles:", gap_pct, "%")
            // Don't fail, just warn (could be genuine gap-up/down)

    LOG_INFO("Candle array validation passed ✅")
    RETURN TRUE

FUNCTION COMPARE_CANDLES(ws_candle, rest_candle):

    // Allow small tolerance for floating point differences
    TOLERANCE = 0.05  // 5 paisa tolerance

    // Compare each field
    open_match = ABS(ws_candle.open - rest_candle.open) < TOLERANCE
    high_match = ABS(ws_candle.high - rest_candle.high) < TOLERANCE
    low_match = ABS(ws_candle.low - rest_candle.low) < TOLERANCE
    close_match = ABS(ws_candle.close - rest_candle.close) < TOLERANCE

    IF open_match AND high_match AND low_match AND close_match:
        RETURN "MATCH"
    ELSE:
        RETURN "MISMATCH"

FUNCTION TRIGGER_DATA_RESYNC():

    LOG_INFO("Starting emergency data resync...")

    // Stop trading temporarily
    PAUSE_TRADING()

    // Reconnect WebSocket
    RECONNECT_WEBSOCKET()

    // Fetch last 30 hourly candles from REST API
    hourly_candles = FETCH_HOURLY_HISTORICAL(
        symbol: "NIFTY",
        count: 30,
        interval: "60m"
    )

    // Validate
    IF VALIDATE_CANDLE_ARRAY(hourly_candles):
        // Replace in-memory candles
        REPLACE_IN_MEMORY(hourly_candles)

        // Recalculate indicators
        UPDATE_INDICATORS(hourly_candles)

        // Resume trading
        RESUME_TRADING()

        LOG_INFO("Data resync complete ✅")
    ELSE:
        LOG_ERROR("Data resync failed! Manual intervention required.")
        SEND_ALERT("CRITICAL: Data resync failed")

FUNCTION FILL_GAPS_FROM_REST_API():

    LOG_INFO("Filling gaps in candle data...")

    // Detect missing timestamps
    missing_timestamps = DETECT_MISSING_CANDLES(hourly_candles)

    FOR timestamp IN missing_timestamps:
        // Fetch missing candle from REST API
        missing_candle = FETCH_REST_CANDLE(timestamp)

        IF missing_candle:
            // Insert at correct position
            INSERT_CANDLE_AT_POSITION(missing_candle, hourly_candles)
            LOG_INFO("Filled gap at:", timestamp)
        ELSE:
            LOG_ERROR("Could not fetch missing candle for:", timestamp)

    // Re-sort by timestamp
    hourly_candles = SORT_BY_TIMESTAMP(hourly_candles)

    // Validate after filling
    IF VALIDATE_CANDLE_ARRAY(hourly_candles):
        LOG_INFO("Gaps filled successfully ✅")
    ELSE:
        LOG_ERROR("Gap filling incomplete")
```

Data Quality Metrics Dashboard:
────────────────────────────────

Track continuously:
├─ WebSocket uptime: 99.9% required
├─ Tick frequency: >1 tick/second during market hours
├─ Candle completion rate: 100% (all hourly candles)
├─ Data sync accuracy: <0.1% mismatch rate
├─ Gap incidents: 0 per day
├─ Resync events: <3 per day
└─ Indicator calculation latency: <100ms

Critical Alerts:
────────────────

1. WebSocket disconnected >30 seconds → CRITICAL
2. No ticks for >60 seconds → CRITICAL
3. Candle validation failed → CRITICAL
4. WS vs REST mismatch >3 times/hour → WARNING
5. Gap detected in data → WARNING
6. Indicators not updated for >1 hour → CRITICAL

Step 3: Crossover Signal (Entry Confirmation)
──────────────────────────────────────────────
After Daily + Hourly both align, wait for crossover:

For CE Entry (Bullish):
├─ Daily: Bullish ✅
├─ Hourly: Bullish ✅
└─ Wait for: Price crosses above key level OR
+DI crosses above -DI OR
ADX starts rising

For PE Entry (Bearish):
├─ Daily: Bearish ✅
├─ Hourly: Bearish ✅
└─ Wait for: Price crosses below key level OR
-DI crosses above +DI OR
ADX starts rising

Step 4: Strike Selection (Real-Time)
────────────────────────────────────
Rule 1: ALWAYS trade ATM (At-The-Money)
Rule 2: Use current week expiry
Rule 3: Verify liquidity (OI > 1000)

Example: NIFTY at 23,500
→ Select 23500 strike
→ Use nearest Thursday expiry
→ If CE: NIFTY28DEC23500CE
→ If PE: NIFTY28DEC23500PE

Key Points:
───────────
✅ Daily defines DIRECTION for ENTIRE day (master)
✅ Hourly confirms TIMING (must align with daily)
✅ Crossover gives ENTRY signal (final confirmation)
✅ NEVER trade against daily trend (even if hourly changes)
✅ ATM strike for best liquidity
✅ Current week expiry for tight spreads

```

### 3.1 Token Discovery & JSON File Creation

#### 3.1.1 Angel One CSV Structure

Angel One CSV columns (example):

```

token,symbol,name,expiry,strike,lotsize,instrumenttype,exch_seg,tick_size
99926000,NIFTY,NIFTY 50,,0.00,50,OPTIDX,NFO,0.05
99926009,NIFTY23DEC23500CE,NIFTY,26Dec2024,23500.00,50,OPTIDX,NFO,0.05
99926010,NIFTY23DEC23500PE,NIFTY,26Dec2024,23500.00,50,OPTIDX,NFO,0.05
99926011,NIFTY23DEC23550CE,NIFTY,26Dec2024,23550.00,50,OPTIDX,NFO,0.05

```

**Key Fields**:

- `token`: Numeric ID for Angel One API (e.g., 99926009)
- `symbol`: Trading symbol (e.g., NIFTY23DEC23500CE)
- `name`: Underlying name (e.g., NIFTY)
- `expiry`: Expiry date (e.g., 26Dec2024)
- `strike`: Strike price (e.g., 23500.00)
- `instrumenttype`: OPTIDX (index options), OPTSTK (stock options)
- `exch_seg`: Exchange segment (NFO for F&O)

#### 3.1.2 CSV Download & Parsing

```

Algorithm: Download & Parse Angel One CSV

1.  Download CSV:

    - URL: https://margincalculator.angelbroking.com/OpenAPI_File/files/OpenAPIScripMaster.json
    - Alternative: Use Angel One's searchScrip API endpoint
    - Save to: data/tokens/angelone_master_YYYYMMDD.csv

2.  Parse CSV:

    - Read CSV line by line
    - Skip header row
    - Parse each line into structured format

3.  Filter Options by Type:
    index_options = [] // For NIFTY, BANKNIFTY, FINNIFTY, etc.
    stock_options = [] // For RELIANCE, TCS, INFY, etc.

    FOR each row IN csv:
    // Skip non-option instruments
    IF exch_seg != "NFO":
    CONTINUE

        // Filter Index Options
        IF instrumenttype == "OPTIDX":
            // Index options: NIFTY, BANKNIFTY, FINNIFTY, MIDCPNIFTY, SENSEX
            ADD to index_options

        // Filter Stock Options
        ELSE IF instrumenttype == "OPTSTK":
            // Stock options: RELIANCE, TCS, INFY, etc.
            ADD to stock_options

4.  Separate Index Options (Most liquid, preferred):
    indices_map = HashMap<(underlying, expiry, strike), (ce_token, pe_token)>

    FOR each option IN index_options:
    underlying = option.name // "NIFTY", "BANKNIFTY", etc.
    expiry = PARSE_EXPIRY(option.expiry)
    strike = option.strike

        key = (underlying, expiry, strike)

        // Identify CE or PE from symbol
        IF option.symbol ENDS_WITH "CE":
            indices_map[key].ce_token = option.token
            indices_map[key].ce_symbol = option.symbol
        ELSE IF option.symbol ENDS_WITH "PE":
            indices_map[key].pe_token = option.token
            indices_map[key].pe_symbol = option.symbol

        // Store additional info
        indices_map[key].lot_size = option.lotsize
        indices_map[key].tick_size = option.tick_size

5.  Separate Stock Options (If trading F&O stocks):
    stocks_map = HashMap<(underlying, expiry, strike), (ce_token, pe_token)>

    FOR each option IN stock_options:
    underlying = option.name // "RELIANCE", "TCS", etc.
    expiry = PARSE_EXPIRY(option.expiry)
    strike = option.strike

        key = (underlying, expiry, strike)

        IF option.symbol ENDS_WITH "CE":
            stocks_map[key].ce_token = option.token
            stocks_map[key].ce_symbol = option.symbol
        ELSE IF option.symbol ENDS_WITH "PE":
            stocks_map[key].pe_token = option.token
            stocks_map[key].pe_symbol = option.symbol

        stocks_map[key].lot_size = option.lotsize
        stocks_map[key].tick_size = option.tick_size

6.  Save to Separate JSON Files:
    // Save index options
    Write indices_map to: data/tokens/index_options_YYYYMMDD.json

    // Save stock options (if needed)
    Write stocks_map to: data/tokens/stock_options_YYYYMMDD.json

````

#### 3.1.2.1 Filtering Examples with Real CSV Data

**Example CSV Rows:**

```csv
token,symbol,name,expiry,strike,lotsize,instrumenttype,exch_seg,tick_size
99926000,NIFTY,NIFTY 50,,0.00,50,OPTIDX,NFO,0.05
99926009,NIFTY23DEC23500CE,NIFTY,26Dec2024,23500.00,50,OPTIDX,NFO,0.05
99926010,NIFTY23DEC23500PE,NIFTY,26Dec2024,23500.00,50,OPTIDX,NFO,0.05
48926001,BANKNIFTY,NIFTY BANK,,0.00,15,OPTIDX,NFO,0.05
48926009,BANKNIFTY23DEC48500CE,BANKNIFTY,27Dec2024,48500.00,15,OPTIDX,NFO,0.05
48926010,BANKNIFTY23DEC48500PE,BANKNIFTY,27Dec2024,48500.00,15,OPTIDX,NFO,0.05
35926001,RELIANCE,RELIANCE,,0.00,250,OPTSTK,NFO,0.05
35926009,RELIANCE23DEC2900CE,RELIANCE,28Dec2024,2900.00,250,OPTSTK,NFO,0.05
35926010,RELIANCE23DEC2900PE,RELIANCE,28Dec2024,2900.00,250,OPTSTK,NFO,0.05
````

**Filtering Algorithm (Detailed):**

```
Step 1: Identify Instrument Type
────────────────────────────────

FOR each row:
    // Check exchange segment first
    IF exch_seg == "NFO":

        // Check instrument type
        IF instrumenttype == "OPTIDX":
            type = "INDEX_OPTION"

        ELSE IF instrumenttype == "OPTSTK":
            type = "STOCK_OPTION"

        ELSE:
            SKIP (not an option)

Step 2: Identify Option Type (CE or PE)
────────────────────────────────────────

Function IDENTIFY_OPTION_TYPE(symbol):
    IF symbol ENDS_WITH "CE":
        RETURN "CALL"

    ELSE IF symbol ENDS_WITH "PE":
        RETURN "PUT"

    ELSE:
        RETURN "UNKNOWN"  // Underlying, not an option

Examples:
    "NIFTY23DEC23500CE" → CALL (CE)
    "NIFTY23DEC23500PE" → PUT (PE)
    "RELIANCE23DEC2900CE" → CALL (CE)
    "RELIANCE23DEC2900PE" → PUT (PE)
    "NIFTY" → UNKNOWN (underlying itself)

Step 3: Filter by Underlying Name
──────────────────────────────────

// Index Options (Priority for trading)
INDEX_UNDERLYINGS = ["NIFTY", "BANKNIFTY", "FINNIFTY", "MIDCPNIFTY", "SENSEX"]

// F&O Stocks (Optional, if trading stocks)
FNO_STOCKS = ["RELIANCE", "TCS", "INFY", "HDFC", "ICICIBANK", ...]

Function IS_INDEX_OPTION(name):
    RETURN name IN INDEX_UNDERLYINGS

Function IS_STOCK_OPTION(name):
    RETURN name IN FNO_STOCKS

Step 4: Pair CE and PE Tokens
──────────────────────────────

// Group by unique combination
key = (underlying_name, expiry_date, strike_price)

Example:
    key1 = ("NIFTY", "2024-12-26", 23500)
        ├─ CE token: 99926009
        └─ PE token: 99926010

    key2 = ("BANKNIFTY", "2024-12-27", 48500)
        ├─ CE token: 48926009
        └─ PE token: 48926010

    key3 = ("RELIANCE", "2024-12-28", 2900)
        ├─ CE token: 35926009
        └─ PE token: 35926010
```

**Complete Filtering Code Logic:**

```
Algorithm: Filter and Categorize Options

INPUT: csv_rows (list of all CSV rows)
OUTPUT: index_options_map, stock_options_map

INITIALIZE:
    index_options_map = {}
    stock_options_map = {}

    INDEX_UNDERLYINGS = ["NIFTY", "BANKNIFTY", "FINNIFTY", "MIDCPNIFTY", "SENSEX"]

PROCESS:
    FOR each row IN csv_rows:

        // Step 1: Filter by exchange (NFO only)
        IF row.exch_seg != "NFO":
            CONTINUE

        // Step 2: Identify option type
        option_type = NONE
        IF row.symbol ENDS_WITH "CE":
            option_type = "CE"
        ELSE IF row.symbol ENDS_WITH "PE":
            option_type = "PE"
        ELSE:
            CONTINUE  // Not an option (underlying itself)

        // Step 3: Parse expiry date
        expiry = PARSE_EXPIRY_DATE(row.expiry)  // "26Dec2024" → "2024-12-26"

        // Step 4: Create unique key
        key = (row.name, expiry, row.strike)

        // Step 5: Categorize by instrument type
        IF row.instrumenttype == "OPTIDX":
            // Index Option
            IF row.name NOT IN INDEX_UNDERLYINGS:
                CONTINUE  // Unknown index, skip

            IF key NOT IN index_options_map:
                index_options_map[key] = {
                    underlying: row.name,
                    expiry: expiry,
                    strike: row.strike,
                    lot_size: row.lotsize,
                    tick_size: row.tick_size
                }

            IF option_type == "CE":
                index_options_map[key].ce_token = row.token
                index_options_map[key].ce_symbol = row.symbol
            ELSE:
                index_options_map[key].pe_token = row.token
                index_options_map[key].pe_symbol = row.symbol

        ELSE IF row.instrumenttype == "OPTSTK":
            // Stock Option
            IF key NOT IN stock_options_map:
                stock_options_map[key] = {
                    underlying: row.name,
                    expiry: expiry,
                    strike: row.strike,
                    lot_size: row.lotsize,
                    tick_size: row.tick_size
                }

            IF option_type == "CE":
                stock_options_map[key].ce_token = row.token
                stock_options_map[key].ce_symbol = row.symbol
            ELSE:
                stock_options_map[key].pe_token = row.token
                stock_options_map[key].pe_symbol = row.symbol

VALIDATE:
    // Ensure both CE and PE exist for each key
    FOR each key IN index_options_map:
        IF ce_token IS NULL OR pe_token IS NULL:
            LOG_WARN("Incomplete pair for:", key)
            DELETE key from index_options_map

    // Same for stock options
    FOR each key IN stock_options_map:
        IF ce_token IS NULL OR pe_token IS NULL:
            LOG_WARN("Incomplete pair for:", key)
            DELETE key from stock_options_map

RETURN:
    index_options_map, stock_options_map
```

**Statistics After Filtering (Example):**

```
Filtering Results:
──────────────────
Total CSV rows: 50,000
NFO instruments: 35,000
Index options (OPTIDX): 15,000
  ├─ NIFTY: 6,000 options (3,000 CE + 3,000 PE)
  ├─ BANKNIFTY: 4,500 options (2,250 CE + 2,250 PE)
  ├─ FINNIFTY: 2,000 options (1,000 CE + 1,000 PE)
  ├─ MIDCPNIFTY: 1,500 options
  └─ SENSEX: 1,000 options

Stock options (OPTSTK): 20,000
  ├─ RELIANCE: 500 options (250 CE + 250 PE)
  ├─ TCS: 400 options (200 CE + 200 PE)
  ├─ INFY: 400 options (200 CE + 200 PE)
  └─ Others: 18,700 options

Unique strikes per index:
  ├─ NIFTY: ~150 strikes per expiry
  ├─ BANKNIFTY: ~120 strikes per expiry
  └─ FINNIFTY: ~80 strikes per expiry
```

#### 3.1.3 Strike Selection Strategy - Which CE/PE to Trade?

**Problem Statement:**

For any underlying (e.g., NIFTY at 23456), there are 100+ CE and 100+ PE options available:

```
Available NIFTY Options (Current LTP: 23456):
├─ 23000 CE/PE (Deep ITM/OTM)
├─ 23100 CE/PE
├─ 23200 CE/PE
├─ 23300 CE/PE
├─ 23350 CE/PE
├─ 23400 CE/PE ← Close to ATM
├─ 23450 CE/PE ← ATM (At-The-Money) ⭐ TRADE THIS
├─ 23500 CE/PE ← Close to ATM
├─ 23550 CE/PE
├─ 23600 CE/PE
├─ 23700 CE/PE
└─ 24000 CE/PE (Deep OTM/ITM)

Question: Which strike should we trade?
Answer: ATM (At-The-Money) strike = 23450
```

**Solution: ATM (At-The-Money) Strategy**

**Why ATM?**

1. ✅ **Maximum liquidity** - Highest trading volume
2. ✅ **Tight spreads** - Smallest bid-ask spread
3. ✅ **Balanced risk/reward** - Not too expensive, not too cheap
4. ✅ **Good delta** - ~0.5 delta, moves with underlying
5. ✅ **Best for intraday** - Captures price movement efficiently

**Strike Selection Logic:**

```
Algorithm: Which Strike to Trade?

RULE 1: Always Trade ATM (At-The-Money) Strike
────────────────────────────────────────────────

ATM Definition:
  The strike price CLOSEST to the current underlying price

Calculation:
  1. Get current underlying price (LTP)
  2. Round to nearest strike increment
  3. That's your ATM strike

Example 1 (NIFTY):
  Current LTP: 23456.75
  Strike increment: 50
  ATM Strike = ROUND(23456.75 / 50) * 50 = 23450 ✅

Example 2 (BANKNIFTY):
  Current LTP: 48678.25
  Strike increment: 100
  ATM Strike = ROUND(48678.25 / 100) * 100 = 48700 ✅

Example 3 (NIFTY near midpoint):
  Current LTP: 23475.00 (exactly between 23450 and 23500)
  ATM Strike = ROUND(23475.00 / 50) * 50 = 23500 ✅
  (Rounds up at midpoint)

RULE 2: Trade Current Week Expiry (Most Liquid)
────────────────────────────────────────────────

For maximum liquidity:
  - NIFTY: Current week Thursday expiry
  - BANKNIFTY: Current week Wednesday expiry
  - FINNIFTY: Current week Tuesday expiry

DO NOT trade:
  ❌ Next week expiry (less liquid)
  ❌ Monthly expiry (unless no weekly available)
  ❌ Far month expiry (very illiquid)

RULE 3: CE or PE Based on Strategy Signal
──────────────────────────────────────────

From your ADX-based strategy:

IF ADX > 25 AND +DI > -DI:
    direction = "CE"  // Buy Call (bullish)
    strike = ATM_STRIKE
    token = GET_TOKEN(underlying, expiry, strike, "CE")

ELSE IF ADX > 25 AND -DI > +DI:
    direction = "PE"  // Buy Put (bearish)
    strike = ATM_STRIKE
    token = GET_TOKEN(underlying, expiry, strike, "PE")

ELSE:
    NO TRADE  // ADX < 25, no clear trend
```

**Complete Token Selection Algorithm:**

```
Algorithm: Select Exact Token for Trading

INPUT:
  - underlying: "NIFTY" (current price: 23456.75)
  - strategy_signal: "BUY_CE" (from ADX analysis)

OUTPUT:
  - token: 99926009 (exact Angel One token to trade)

STEPS:

Step 1: Calculate ATM Strike
─────────────────────────────
current_ltp = GET_UNDERLYING_LTP("NIFTY")  // 23456.75
strike_increment = 50  // for NIFTY

atm_strike = ROUND(current_ltp / strike_increment) * strike_increment
// atm_strike = ROUND(23456.75 / 50) * 50 = 23450

Step 2: Get Current Week Expiry
────────────────────────────────
today = "2024-12-23" (Monday)
expiry = NEXT_THURSDAY(today)  // "2024-12-26" (Thursday)

Step 3: Determine CE or PE
───────────────────────────
IF strategy_signal == "BUY_CE":
    option_type = "CE"
ELSE IF strategy_signal == "BUY_PE":
    option_type = "PE"

Step 4: Load Token Map
──────────────────────
tokens = LOAD_JSON("data/tokens/index_options_20241223.json")

Step 5: Find Exact Token
─────────────────────────
key = ("NIFTY", "2024-12-26", 23450)

IF option_type == "CE":
    token = tokens[key].ce_token      // 99926009
    symbol = tokens[key].ce_symbol    // "NIFTY26DEC23450CE"
    lot_size = tokens[key].lot_size   // 50
ELSE:
    token = tokens[key].pe_token      // 99926010
    symbol = tokens[key].pe_symbol    // "NIFTY26DEC23450PE"
    lot_size = tokens[key].lot_size   // 50

Step 6: Validate Token
──────────────────────
quote = FETCH_QUOTE(token)  // Call Angel One API

IF quote.ltp == 0 OR quote.volume == 0:
    LOG_ERROR("Token inactive or illiquid:", token)
    RETURN NULL

IF quote.oi < 1000:  // Open Interest too low
    LOG_WARN("Low OI for token:", token)
    // Consider fallback to ±1 strike

Step 7: Return Selected Token
──────────────────────────────
RETURN {
    token: 99926009,
    symbol: "NIFTY26DEC23450CE",
    strike: 23450,
    lot_size: 50,
    current_premium: quote.ltp  // e.g., 150.50
}
```

**Real-World Example:**

```
Scenario: NIFTY Trading at 23456.75 on Monday

Step-by-Step:
─────────────
1. Current LTP: 23456.75
2. ATM Strike: 23450 (rounded to nearest 50)
3. Expiry: This Thursday (26-Dec-2024)
4. Strategy: ADX = 32, +DI > -DI → BUY CE ✅

Available Options:
├─ 23400 CE @ ₹185 (ITM, expensive)
├─ 23450 CE @ ₹150 ⭐ ATM - TRADE THIS
├─ 23500 CE @ ₹120 (OTM, cheaper but less movement)

Selected Token:
├─ Token: 99926009
├─ Symbol: NIFTY26DEC23450CE
├─ Strike: 23450
├─ Premium: ₹150
├─ Lot Size: 50
└─ Total Cost: ₹7,500 (150 × 50)

Why 23450 CE?
✅ ATM (maximum liquidity)
✅ Current week expiry (most liquid)
✅ Strategy says CE (bullish signal)
✅ High OI and volume
✅ Tight bid-ask spread
```

**ATM Tracking & Dynamic Token Updates:**

```
Algorithm: When to Update/Change Option Token

═══════════════════════════════════════════════════════════
CRITICAL RULES:
═══════════════════════════════════════════════════════════
Rule 1: NEVER change token while in position (hold until exit)
Rule 2: UPDATE ATM calculation every 10 seconds (for next trade)
Rule 3: CHECK trend change every 1 minute (hourly ADX)
Rule 4: ALWAYS exit before entering opposite direction

═══════════════════════════════════════════════════════════
Scenario 1: Price Moves (ATM Strike Changes)
═══════════════════════════════════════════════════════════

MONITOR underlying price EVERY 10 SECONDS

Current Situation:
├─ Open Position: NIFTY 23450 CE @ ₹150 (bought at 9:30 AM)
├─ Entry Price: 23,456
└─ Now: NIFTY moves to 23,520 (moved up 64 points)

ATM Check:
├─ Old ATM: 23450 (1 strike away)
├─ New ATM: 23500 (current ATM)
└─ Price moved >1 strike ✅

Action Plan:
┌──────────────────────────────────────────┐
│ If IN position:                          │
│   ✅ HOLD current 23450 CE               │
│   ✅ Do NOT exit/change                  │
│   ✅ Update ATM to 23500 (for future)    │
│   ✅ Subscribe to 23500 CE data          │
│   ✅ Wait for exit signal                │
│                                          │
│ If NOT in position (ready for entry):   │
│   ✅ Use NEW ATM: 23500                  │
│   ✅ Next entry: NIFTY 23500 CE          │
└──────────────────────────────────────────┘

Real Example:
─────────────
09:30 AM: BUY NIFTY 23450 CE @ ₹150
10:00 AM: Price → 23,520 (ATM now 23500)
         Action: HOLD 23450 CE (don't change!)
11:00 AM: Exit signal hits
         Action: SELL 23450 CE @ ₹180
11:05 AM: New entry signal
         Action: BUY NIFTY 23500 CE @ ₹170 (NEW ATM!)

Key Point: Always use CURRENT ATM for NEW entries,
           but don't exit existing positions just because ATM changed.

═══════════════════════════════════════════════════════════
Scenario 2: Trend Reversal (CE to PE or PE to CE)
═══════════════════════════════════════════════════════════

CHECK trend EVERY 1 MINUTE (using HOURLY ADX)

Current Situation:
├─ Open Position: NIFTY 23450 CE @ ₹150
├─ Entry: Daily bullish + Hourly bullish
└─ Now: Hourly trend reversing

Hourly ADX Check (Every 1 Min):
────────────────────────────────
10:00 AM Check:
├─ Daily ADX: 32, +DI > -DI (still bullish) ✅
├─ Hourly ADX: 30, +DI > -DI (still bullish) ✅
└─ Action: HOLD CE position

11:00 AM Check:
├─ Daily ADX: 32, +DI > -DI (still bullish) ✅
├─ Hourly ADX: 28, -DI > +DI (BEARISH now!) ⚠️
└─ Action: TREND CHANGED!

Position Change Flow:
─────────────────────
Step 1: EXIT current position FIRST
  → SELL NIFTY 23450 CE @ ₹165
  → Wait for order confirmation
  → Close position completely

Step 2: WAIT for clean slate
  → Ensure no open positions
  → Check P&L booked
  → Verify balance updated

Step 3: Calculate NEW ATM
  → Current NIFTY: 23,489
  → New ATM: 23500
  → New direction: PE (bearish)

Step 4: ENTER new position
  → BUY NIFTY 23500 PE @ ₹140
  → Place LIMIT order
  → Wait for fill confirmation

═══════════════════════════════════════════════════════════
Scenario 3: Daily Trend Changes (Major Reversal)
═══════════════════════════════════════════════════════════

CHECK daily trend ONCE per day (9:15 AM)

Next Day Scenario:
──────────────────
Monday 9:15 AM:
├─ Daily ADX: 32, +DI > -DI (Bullish)
└─ Trading: CE options

Tuesday 9:15 AM:
├─ Daily ADX: 35, -DI > +DI (BEARISH now!) 🔄
└─ Trading: Switch to PE options from today

Action:
1. If holding CE from Monday:
   → Exit CE position first
   → Book profit/loss

2. New direction = PE:
   → Look for hourly PE entry signals
   → Use ATM PE strikes
   → Trade PE only today

═══════════════════════════════════════════════════════════
Update Frequency Summary
═══════════════════════════════════════════════════════════

┌─────────────────────┬──────────────┬─────────────────────┐
│ Check               │ Frequency    │ Action              │
├─────────────────────┼──────────────┼─────────────────────┤
│ Daily ADX           │ Once/day     │ Set CE or PE        │
│                     │ (9:15 AM)    │ direction           │
├─────────────────────┼──────────────┼─────────────────────┤
│ Hourly ADX          │ Every 1 min  │ Trigger position    │
│                     │              │ change if needed    │
├─────────────────────┼──────────────┼─────────────────────┤
│ ATM Strike          │ Every 10 sec │ Update for next     │
│                     │              │ entry (not current) │
├─────────────────────┼──────────────┼─────────────────────┤
│ Stop Loss           │ Real-time    │ Exit immediately    │
│                     │              │ if hit              │
├─────────────────────┼──────────────┼─────────────────────┤
│ VIX Spike           │ Real-time    │ Exit all if >30     │
└─────────────────────┴──────────────┴─────────────────────┘

═══════════════════════════════════════════════════════════
Critical Rules to Avoid Mistakes
═══════════════════════════════════════════════════════════

❌ NEVER: Hold both CE and PE at same time
✅ ALWAYS: Exit first, then enter opposite

❌ NEVER: Change token mid-position (because ATM changed)
✅ ALWAYS: Hold existing strike until exit signal

❌ NEVER: Enter new position before exit confirmed
✅ ALWAYS: Wait for exit order fill confirmation

❌ NEVER: Ignore daily trend and trade on hourly only
✅ ALWAYS: Both daily and hourly must align

❌ NEVER: Keep same direction if daily trend changes
✅ ALWAYS: Check daily trend every morning (9:15 AM)
```

**Quick Decision Matrix: When to Change Token**

```
┌───────────────────────────────────────────────────────────────┐
│           SITUATION → ACTION MATRIX                           │
└───────────────────────────────────────────────────────────────┘

╔════════════════════════╦══════════════╦═════════════════════╗
║ Situation              ║ In Position? ║ Action              ║
╠════════════════════════╬══════════════╬═════════════════════╣
║ Price moves 50 points  ║ YES          ║ HOLD current token  ║
║ (ATM changed)          ║              ║ Update ATM for next ║
║                        ╠══════════════╬═════════════════════╣
║                        ║ NO           ║ Use NEW ATM token   ║
╠════════════════════════╬══════════════╬═════════════════════╣
║ Hourly trend reverses  ║ YES          ║ EXIT current first  ║
║ (CE → PE or PE → CE)   ║              ║ Then BUY opposite   ║
║                        ╠══════════════╬═════════════════════╣
║                        ║ NO           ║ Wait for entry      ║
║                        ║              ║ signal              ║
╠════════════════════════╬══════════════╬═════════════════════╣
║ Daily trend changes    ║ YES          ║ EXIT at 9:15 AM     ║
║ (next day reversal)    ║              ║ Switch direction    ║
║                        ╠══════════════╬═════════════════════╣
║                        ║ NO           ║ Trade new direction ║
║                        ║              ║ from today          ║
╠════════════════════════╬══════════════╬═════════════════════╣
║ Stop loss hit          ║ YES          ║ EXIT immediately    ║
║                        ║              ║ (market order)      ║
║                        ╠══════════════╬═════════════════════╣
║                        ║ NO           ║ No action           ║
╠════════════════════════╬══════════════╬═════════════════════╣
║ VIX spikes > 30        ║ YES          ║ EXIT all positions  ║
║                        ║              ║ immediately         ║
║                        ╠══════════════╬═════════════════════╣
║                        ║ NO           ║ Pause new trades    ║
╠════════════════════════╬══════════════╬═════════════════════╣
║ Target profit hit      ║ YES          ║ EXIT (book profit)  ║
║                        ║              ║ Wait for new signal ║
║                        ╠══════════════╬═════════════════════╣
║                        ║ NO           ║ No action           ║
╚════════════════════════╩══════════════╩═════════════════════╝

Key Principles:
───────────────
1️⃣  Update ATM calculation continuously (every 10 sec)
2️⃣  Don't exit just because ATM changed
3️⃣  Exit ONLY on: signal change, stop loss, target, or VIX spike
4️⃣  ALWAYS use CURRENT ATM for NEW entries
5️⃣  NEVER hold CE and PE together
```

**Complete Day Timeline Example (CORRECTED):**

```
Full Trading Day: When Tokens Change (Daily = Bullish)
───────────────────────────────────────────────────────

09:15 AM - Daily Analysis
├─ Daily ADX: 32, +DI > -DI (Bullish)
├─ Daily Direction: CE ONLY for today ✅
└─ Set: Trade CE only, NO PE today

09:20 AM - Hourly Check
├─ Hourly ADX: 28, +DI > -DI (Bullish)
├─ Hourly aligned with daily ✅
└─ Ready to trade CE

09:30 AM - Entry Signal
├─ NIFTY: 23,456
├─ ATM: 23450
├─ Crossover: +DI crosses above -DI ✅
└─ ACTION: BUY NIFTY26DEC23450CE @ ₹150

10:00 AM - Price moves up
├─ NIFTY: 23,523 (+67 points)
├─ ATM: NOW 23500 (moved 1 strike!) ⚠️
├─ Hourly: Still bullish ✅
├─ Current position: 23450 CE (in profit)
└─ ACTION: HOLD 23450 CE ✅
           Update ATM to 23500 (for next trade)
           Subscribe to 23500 CE data

10:30 AM - Continue monitoring
├─ NIFTY: 23,545
├─ ATM: 23550 (moved again)
├─ Hourly: Still bullish
└─ ACTION: HOLD 23450 CE ✅ (don't exit!)
           Update ATM to 23550

11:00 AM - HOURLY CONFLICTS! ⚠️
├─ NIFTY: 23,512 (dropped)
├─ ATM: 23500
├─ Daily: Still bullish (32, +DI > -DI) ✅
├─ Hourly: NOW bearish! (28, -DI > +DI) ❌
├─ Conflict: Daily says CE, Hourly says PE
└─ ACTION:
           Step 1: SELL 23450 CE @ ₹180 ✅
           Step 2: Book profit: +₹1,500
           Step 3: DO NOT buy PE! ❌
           Step 4: WAIT for hourly to align back ⏳

11:30 AM - Still Waiting (No Position)
├─ NIFTY: 23,478
├─ ATM: Still 23500
├─ Daily: Bullish ✅
├─ Hourly: Still bearish ❌
└─ ACTION: WAIT ⏳ (NO TRADE, no position)

12:00 PM - Price drops more
├─ NIFTY: 23,389
├─ ATM: NOW 23400
├─ Daily: Bullish ✅
├─ Hourly: Still bearish ❌
└─ ACTION: WAIT ⏳ (NO TRADE)
           Update ATM to 23400 (for future)

01:00 PM - Hourly Aligns Back! ✅
├─ NIFTY: 23,423
├─ ATM: 23400
├─ Daily: Bullish ✅
├─ Hourly: NOW bullish! (ADX 26, +DI > -DI) ✅
├─ Both aligned again ✅
└─ Ready to trade CE again

01:15 PM - Re-entry Signal
├─ NIFTY: 23,445
├─ ATM: 23450
├─ Crossover: Price crosses above key level ✅
└─ ACTION: BUY NIFTY26DEC23450CE @ ₹165 ✅
           (Same direction CE, new strike)

02:30 PM - Continue in CE
├─ NIFTY: 23,489
├─ ATM: Still 23450
├─ Hourly: Still bullish ✅
├─ Current P&L: +₹1,200
└─ ACTION: HOLD 23450 CE ✅

03:15 PM - Near close
├─ NIFTY: 23,512
├─ Position: 23450 CE
├─ Current value: ₹185 (+12%)
└─ ACTION: Prepare to square off

03:25 PM - Square off
├─ Time: 5 minutes to close
├─ Rule: No overnight positions
└─ ACTION: SELL 23450 CE @ ₹186 ✅
           Book profit: +₹1,050
           Day complete

═══════════════════════════════════════════════════════════
Summary of Token Changes Today (CORRECTED):
═══════════════════════════════════════════════════════════

Total Trades: 2 (BOTH CE, NO PE!) ✅

Trade 1: NIFTY 23450 CE (9:30 AM - 11:00 AM)
├─ Reason: Daily + Hourly both bullish
├─ Entry: ₹150
├─ Exit: ₹180 (hourly conflict)
└─ P&L: +₹1,500

No Trade Period: 11:00 AM - 1:15 PM ⏳
├─ Reason: Hourly bearish (conflicts with daily)
├─ Did NOT trade PE ❌
└─ Waited for hourly to align back with daily

Trade 2: NIFTY 23450 CE (1:15 PM - 3:25 PM)
├─ Reason: Hourly aligned back with daily
├─ Entry: ₹165
├─ Exit: ₹186 (square off)
└─ P&L: +₹1,050

Total Day P&L: +₹2,550 (before charges)

Key Observations (CORRECTED):
──────────────────────────────
✅ Daily set direction: CE ONLY for entire day
✅ Exited when hourly conflicted
❌ Did NOT switch to PE (daily was bullish!)
✅ Waited for hourly to align back
✅ Re-entered CE when aligned again
✅ Changed strike when ATM moved (for new entry)
✅ HELD strike when ATM changed mid-position
✅ Only 2 trades (both same direction)
```

**Fallback Strategy (If ATM Not Available):**

```
Priority Order:
1. ATM strike ⭐ (always preferred)
2. ATM + 1 strike (if ATM illiquid)
3. ATM - 1 strike (if ATM illiquid)
4. Skip trade (if no liquid strikes available)

Example:
ATM = 23450
1st choice: 23450 CE ✅
2nd choice: 23500 CE (if 23450 has low volume)
3rd choice: 23400 CE (if both above unavailable)
```

#### 3.1.3.1 CE vs PE Decision Logic - Which Side to Trade?

**The Million Dollar Question: Should I trade CE or PE?**

```
Answer: Let ADX + Directional Indicators decide

Simple Rule:
├─ Strong uptrend (+DI > -DI) → Trade CE (Call) 📈
├─ Strong downtrend (-DI > +DI) → Trade PE (Put) 📉
└─ No clear trend (ADX < 25) → NO TRADE ❌
```

**Complete Decision Algorithm:**

```
Algorithm: CE or PE Decision

INPUT:
  - underlying: "NIFTY" or "BANKNIFTY" or Stock name
  - bars: Last 30 DAILY bars for underlying ⭐ IMPORTANT

OUTPUT:
  - decision: "BUY_CE", "BUY_PE", or "NO_TRADE"

CRITICAL: Multi-Timeframe Strategy
───────────────────────────────────
DAILY Timeframe → Determine Trend Direction (CE or PE)
HOURLY Timeframe → Entry/Exit Signals

Timeframe Usage:
├─ DAILY (1D): Trend direction (which side: CE or PE?)
└─ HOURLY (1H): Entry/exit timing (when to trade?)

Why Multi-Timeframe?
✅ Daily: Filters out noise, reliable trend
✅ Hourly: Better entry/exit timing
✅ Avoids false signals from intraday whipsaws
✅ Professional institutional approach

STEPS:

Step 1: Calculate Daily ADX (Trend Direction)
──────────────────────────────────────────────
// Use DAILY bars to determine CE or PE
daily_bars = GET_DAILY_BARS(underlying, count=30)  // Last 30 days
daily_adx = CALCULATE_ADX(daily_bars, period=14)
daily_plus_di = CALCULATE_PLUS_DI(daily_bars, period=14)
daily_minus_di = CALCULATE_MINUS_DI(daily_bars, period=14)

// Daily ADX tells us: Is there a trend?
// ADX > 25: Strong trend exists ✅
// ADX < 20: Weak/no trend ❌

Step 2: Determine Direction from Daily
───────────────────────────────────────
IF daily_adx < 25:
    RETURN "NO_TRADE"  // No clear daily trend

IF daily_plus_di > daily_minus_di:
    trend_direction = "CE"  // Daily uptrend
ELSE IF daily_minus_di > daily_plus_di:
    trend_direction = "PE"  // Daily downtrend
ELSE:
    RETURN "NO_TRADE"  // Unclear direction

Step 3: Wait for Hourly Entry Signal
─────────────────────────────────────
// Now use HOURLY bars for entry timing
hourly_bars = GET_HOURLY_BARS(underlying, count=30)  // Last 30 hours
hourly_adx = CALCULATE_ADX(hourly_bars, period=14)
hourly_plus_di = CALCULATE_PLUS_DI(hourly_bars, period=14)
hourly_minus_di = CALCULATE_MINUS_DI(hourly_bars, period=14)

// Hourly must CONFIRM daily trend
IF trend_direction == "CE":
    // Daily says CE, check hourly confirmation
    IF hourly_adx >= 25 AND hourly_plus_di > hourly_minus_di:
        RETURN "BUY_CE"  // Both daily and hourly bullish ✅
    ELSE:
        RETURN "WAIT"  // Daily bullish but hourly not ready

ELSE IF trend_direction == "PE":
    // Daily says PE, check hourly confirmation
    IF hourly_adx >= 25 AND hourly_minus_di > hourly_plus_di:
        RETURN "BUY_PE"  // Both daily and hourly bearish ✅
    ELSE:
        RETURN "WAIT"  // Daily bearish but hourly not ready

Step 3: Decision Matrix
───────────────────────

IF adx < 20:
    // No clear trend, too risky
    RETURN "NO_TRADE"
    LOG_INFO("ADX too low:", adx, "- No clear trend")

ELSE IF adx >= 20 AND adx < 25:
    // Trend forming but not strong enough
    RETURN "NO_TRADE"
    LOG_INFO("ADX moderate:", adx, "- Wait for stronger trend")

ELSE IF adx >= 25:
    // Strong trend exists, now check direction

    IF plus_di > minus_di:
        // Bullish trend (+DI dominates)
        RETURN "BUY_CE"
        LOG_INFO("Bullish signal: ADX=", adx, "+DI=", plus_di, "-DI=", minus_di)

    ELSE IF minus_di > plus_di:
        // Bearish trend (-DI dominates)
        RETURN "BUY_PE"
        LOG_INFO("Bearish signal: ADX=", adx, "+DI=", minus_di, "-DI=", minus_di)

    ELSE:
        // Equal DI, no clear direction
        RETURN "NO_TRADE"
        LOG_INFO("Conflicting signals: +DI ≈ -DI")

Step 4: Additional Filters (Optional but Recommended)
──────────────────────────────────────────────────────

// Filter 1: Volume Check
volume_avg = CALCULATE_VOLUME_AVG(bars, period=20)
current_volume = bars[-1].volume

IF current_volume < (volume_avg * 0.8):
    RETURN "NO_TRADE"
    LOG_INFO("Volume too low, skip trade")

// Filter 2: RSI Extremes (Avoid overbought/oversold)
rsi = CALCULATE_RSI(bars, period=14)

IF decision == "BUY_CE" AND rsi > 70:
    RETURN "NO_TRADE"
    LOG_WARN("RSI overbought:", rsi, "- Skip CE")

IF decision == "BUY_PE" AND rsi < 30:
    RETURN "NO_TRADE"
    LOG_WARN("RSI oversold:", rsi, "- Skip PE")

// Filter 3: VIX Check
vix = GET_VIX()

IF vix > 30:
    RETURN "NO_TRADE"
    LOG_WARN("VIX too high:", vix, "- Market too volatile")

RETURN decision
```

**Visual Decision Matrix:**

```
┌─────────────────────────────────────────────────────────┐
│                    ADX DECISION MATRIX                   │
└─────────────────────────────────────────────────────────┘

ADX < 20:
├─ Trend: WEAK/RANGING
└─ Action: NO TRADE ❌
   Reason: No clear trend, choppy market

ADX 20-25:
├─ Trend: FORMING
└─ Action: NO TRADE ⚠️
   Reason: Trend not strong enough yet

ADX 25-40:
├─ Trend: STRONG ✅
├─ Check Direction:
│  ├─ IF +DI > -DI → Trade CE 📈 (Bullish)
│  └─ IF -DI > +DI → Trade PE 📉 (Bearish)

ADX > 40:
├─ Trend: VERY STRONG 🔥
├─ Check Direction:
│  ├─ IF +DI > -DI → Trade CE 📈 (Strong bullish)
│  └─ IF -DI > +DI → Trade PE 📉 (Strong bearish)
└─ Caution: Trend may be exhausting, watch for reversal
```

**Multi-Timeframe Strategy Example:**

```
Complete Example: How Daily + Hourly Work Together
───────────────────────────────────────────────────

Morning Analysis (9:15 AM):
───────────────────────────

Step 1: Check DAILY Trend (30 days)
────────────────────────────────────
NIFTY Daily Data:
├─ Daily ADX: 32 ✅ (Strong trend)
├─ Daily +DI: 28
├─ Daily -DI: 16
└─ Result: +DI > -DI → Daily UPTREND

Decision: Trend direction = CE ✅
But WAIT for hourly confirmation!

Step 2: Check HOURLY Signal (9:15 AM)
──────────────────────────────────────
NIFTY Hourly Data:
├─ Hourly ADX: 22 ⚠️ (Below 25)
├─ Hourly +DI: 20
├─ Hourly -DI: 18
└─ Result: Hourly not ready

Decision: WAIT ⏳
Daily says CE, but hourly trend not formed yet

Step 3: Wait for Hourly Confirmation
─────────────────────────────────────

10:00 AM Check:
├─ Hourly ADX: 26 ✅ (Above 25 now!)
├─ Hourly +DI: 28
├─ Hourly -DI: 18
└─ Result: Hourly +DI > -DI

Decision: BUY CE NOW! ✅
├─ Daily: Bullish (ADX 32, +DI > -DI)
├─ Hourly: Bullish (ADX 26, +DI > -DI)
└─ Both aligned!

Action: BUY NIFTY26DEC23450CE @ ₹150

Why This Works:
───────────────
✅ Daily trend filters noise (big picture)
✅ Hourly confirms momentum (timing)
✅ Both aligned = high probability trade
✅ Avoids false intraday signals
```

**Multi-Timeframe Decision Matrix:**

```
┌──────────────────────────────────────────────────────┐
│           DAILY vs HOURLY COMBINATIONS               │
└──────────────────────────────────────────────────────┘

Scenario 1: Both Bullish (IDEAL) ✅
├─ Daily: ADX 32, +DI > -DI (Bullish)
├─ Hourly: ADX 26, +DI > -DI (Bullish)
└─ Action: BUY CE ✅ (High confidence)

Scenario 2: Daily Bullish, Hourly Weak ⏳
├─ Daily: ADX 32, +DI > -DI (Bullish)
├─ Hourly: ADX 18, weak trend
└─ Action: WAIT ⏳ (Don't trade yet)

Scenario 3: Daily Bullish, Hourly Bearish ❌
├─ Daily: ADX 32, +DI > -DI (Bullish)
├─ Hourly: ADX 28, -DI > +DI (Bearish)
└─ Action: NO TRADE ❌ (Conflicting signals)

Scenario 4: Both Bearish (IDEAL) ✅
├─ Daily: ADX 35, -DI > +DI (Bearish)
├─ Hourly: ADX 27, -DI > +DI (Bearish)
└─ Action: BUY PE ✅ (High confidence)

Scenario 5: Daily Weak ❌
├─ Daily: ADX 18 (Weak trend)
├─ Hourly: Doesn't matter
└─ Action: NO TRADE ❌ (No daily trend)
```

**Timeframe Summary:**

```
┌─────────────────────────────────────────────────┐
│        TIMEFRAME USAGE REFERENCE                 │
└─────────────────────────────────────────────────┘

DAILY Timeframe (1D):
├─ Purpose: Determine trend direction
├─ Data: Last 30 daily bars
├─ Indicators: ADX, +DI, -DI (14-period)
├─ Output: CE (bullish) or PE (bearish) or NO_TRADE
└─ Update: Once per day (morning check)

HOURLY Timeframe (1H):
├─ Purpose: Entry/exit timing
├─ Data: Last 30 hourly bars
├─ Indicators: ADX, +DI, -DI (14-period)
├─ Output: Confirm daily + timing
└─ Update: Every hour

Trade Decision Flow:
────────────────────
1. Check DAILY → Get direction (CE or PE)
2. Check HOURLY → Confirm + find entry
3. Both aligned → TRADE ✅
4. Not aligned → WAIT ⏳
```

**Real-World Multi-Timeframe Examples:**

```
Example 1: Perfect Bullish Setup (Trade CE)
────────────────────────────────────────────

NIFTY Monday 9:15 AM:

DAILY Check:
├─ Daily ADX: 32 ✅ (Strong trend)
├─ Daily +DI: 28 📈
├─ Daily -DI: 16 📉
└─ Result: Daily UPTREND → Look for CE

HOURLY Check (9:15 AM):
├─ Hourly ADX: 22 ⚠️ (Below threshold)
└─ Result: WAIT (not ready yet)

HOURLY Check (10:00 AM):
├─ Hourly ADX: 26 ✅ (Above 25)
├─ Hourly +DI: 28 📈
├─ Hourly -DI: 18 📉
└─ Result: Hourly confirms UPTREND ✅

Decision: BUY CE NOW! ✅
├─ Daily: Bullish ✅
├─ Hourly: Bullish ✅
└─ Both aligned!

Action: BUY NIFTY26DEC23450CE @ ₹150

Example 2: Perfect Bearish Setup (Trade PE)
────────────────────────────────────────────

NIFTY Tuesday 9:15 AM:

DAILY Check:
├─ Daily ADX: 35 ✅ (Strong trend)
├─ Daily +DI: 15 📈
├─ Daily -DI: 29 📉
└─ Result: Daily DOWNTREND → Look for PE

HOURLY Check (9:15 AM):
├─ Hourly ADX: 28 ✅ (Above 25)
├─ Hourly +DI: 16 📈
├─ Hourly -DI: 30 📉
└─ Result: Hourly confirms DOWNTREND ✅

Decision: BUY PE NOW! ✅
├─ Daily: Bearish ✅
├─ Hourly: Bearish ✅
└─ Both aligned from start!

Action: BUY NIFTY26DEC23500PE @ ₹140

Example 3: No Trade (Weak Daily Trend)
───────────────────────────────────────

NIFTY Wednesday 2:00 PM:

DAILY Check:
├─ Daily ADX: 18 ❌ (Weak trend)
├─ Daily +DI: 20
├─ Daily -DI: 22
└─ Result: No clear daily trend

Decision: NO TRADE ❌
├─ Daily: Weak (ADX < 25)
└─ Don't check hourly (daily must be strong first)

Why No Trade?
- No clear daily trend (ADX only 18)
- Market ranging/choppy
- High risk of false signals

Action:
WAIT for daily ADX > 25

Example 4: No Trade (Conflicting Signals)
──────────────────────────────────────────

NIFTY Thursday 11:00 AM:

DAILY Check:
├─ Daily ADX: 30 ✅ (Strong trend)
├─ Daily +DI: 27 📈
├─ Daily -DI: 17 📉
└─ Result: Daily UPTREND → Look for CE

HOURLY Check (11:00 AM):
├─ Hourly ADX: 28 ✅ (Strong)
├─ Hourly +DI: 18 📈
├─ Hourly -DI: 30 📉
└─ Result: Hourly DOWNTREND ⚠️

Decision: NO TRADE ❌
├─ Daily: Bullish (wants CE) ✅
├─ Hourly: Bearish (wants PE) ❌
└─ Conflicting signals!

Why No Trade?
- Daily says BUY CE (uptrend)
- Hourly says BUY PE (downtrend)
- Conflicting signals = high risk
- Wait for both to align

Action:
WAIT for clearer direction
```

**Indicator Calculation (From Your Doc):**

```
ADX Calculation:
────────────────
1. Calculate True Range (TR)
2. Calculate +DM and -DM (directional movement)
3. Smooth TR, +DM, -DM over 14 periods
4. Calculate +DI = (+DM / TR) × 100
5. Calculate -DI = (-DM / TR) × 100
6. Calculate DX = |+DI - -DI| / (+DI + -DI) × 100
7. Calculate ADX = 14-period average of DX

Use library: `ta` or `yata` (DO NOT implement manually)
```

**Decision Summary Table:**

| ADX | +DI | -DI | +DI vs -DI | Decision        | Reasoning            |
| --- | --- | --- | ---------- | --------------- | -------------------- |
| 32  | 28  | 16  | +DI > -DI  | **BUY CE** ✅   | Strong bullish trend |
| 35  | 15  | 29  | -DI > +DI  | **BUY PE** ✅   | Strong bearish trend |
| 18  | 20  | 22  | N/A        | **NO TRADE** ❌ | Weak trend           |
| 26  | 23  | 22  | +DI ≈ -DI  | **NO TRADE** ⚠️ | Unclear direction    |
| 42  | 35  | 18  | +DI > -DI  | **BUY CE** 🔥   | Very strong bullish  |
| 38  | 16  | 32  | -DI > +DI  | **BUY PE** 🔥   | Very strong bearish  |

**Quick Decision Flowchart:**

```
Start
  ↓
Is ADX ≥ 25?
  ├─ NO → NO TRADE ❌ (Weak trend)
  └─ YES ↓
         Is +DI > -DI?
           ├─ YES → BUY CE ✅ (Bullish)
           ├─ NO ↓
           │    Is -DI > +DI?
           │      ├─ YES → BUY PE ✅ (Bearish)
           │      └─ NO → NO TRADE ⚠️ (Unclear)
```

**Additional Confirmation Filters:**

```
1. Volume Confirmation
──────────────────────
IF volume < 80% of average:
    → Skip trade (low conviction)

2. RSI Filter
─────────────
IF BUY CE signal AND RSI > 70:
    → Skip (overbought, risk of reversal)

IF BUY PE signal AND RSI < 30:
    → Skip (oversold, risk of reversal)

3. VIX Filter
─────────────
IF VIX > 30:
    → Skip all trades (too volatile)

4. Time of Day
──────────────
IF time < 9:30 AM:
    → Skip (wait for market to settle)

IF time > 3:00 PM:
    → Skip (avoid expiry day risks)
```

**Common Mistakes to Avoid:**

```
❌ WRONG: "NIFTY is going up, buy CE"
✅ RIGHT: "ADX = 32, +DI = 28 > -DI = 16, buy CE"

❌ WRONG: Trade based on gut feeling
✅ RIGHT: Trade based on indicator signals

❌ WRONG: Trade when ADX < 20 (weak trend)
✅ RIGHT: Wait for ADX ≥ 25 (strong trend)

❌ WRONG: Trade both CE and PE together (hedging)
✅ RIGHT: Trade only one direction at a time

❌ WRONG: Ignore VIX and volume
✅ RIGHT: Check all confirmation filters
```

#### 3.1.3.2 When Hourly Conflicts with Daily (Market Updates)

**Problem: Daily says CE, but Hourly becomes bearish**

```
Scenario:
─────────
9:15 AM: Daily bullish (+DI > -DI), set direction = CE for today
9:30 AM: Hourly bullish, buy NIFTY 23450 CE
11:00 AM: Hourly becomes bearish (-DI > +DI)
Question: Do we switch to PE? Answer: NO! ❌
```

**Solution: Exit and WAIT for Hourly to Align Back with Daily**

```
Algorithm: Handle Hourly Conflict with Daily

CRITICAL RULE: Daily trend is MASTER
                Only trade in daily direction
                NEVER trade against daily trend

Step 1: Set Daily Direction (9:15 AM)
──────────────────────────────────────
daily_adx = CALCULATE_DAILY_ADX()
daily_plus_di = CALCULATE_DAILY_PLUS_DI()
daily_minus_di = CALCULATE_DAILY_MINUS_DI()

IF daily_adx >= 25:
    IF daily_plus_di > daily_minus_di:
        daily_direction = "CE"  // Trade CE ONLY today
    ELSE IF daily_minus_di > daily_plus_di:
        daily_direction = "PE"  // Trade PE ONLY today
ELSE:
    daily_direction = "NO_TRADE"  // Skip today

Step 2: Monitor Hourly Alignment (Every 1 Minute)
──────────────────────────────────────────────────
WHILE market_open:
    EVERY 1 MINUTE:
        // Get hourly indicators
        hourly_adx = CALCULATE_HOURLY_ADX()
        hourly_plus_di = CALCULATE_HOURLY_PLUS_DI()
        hourly_minus_di = CALCULATE_HOURLY_MINUS_DI()

        // Check if hourly aligns with daily
        hourly_aligned = CHECK_ALIGNMENT(
            daily_direction,
            hourly_adx,
            hourly_plus_di,
            hourly_minus_di
        )

        // Take action based on alignment
        IF hourly_aligned:
            ALLOW_TRADING()
        ELSE:
            IF in_position:
                EXIT_POSITION()  // Exit and wait
            ELSE:
                WAIT()  // Don't enter until aligned

Step 3: Alignment Check Logic
──────────────────────────────
Function CHECK_ALIGNMENT(daily_dir, h_adx, h_plus, h_minus):

    // Case 1: Daily is CE (bullish)
    IF daily_dir == "CE":
        // Hourly must also be bullish
        IF h_adx >= 25 AND h_plus > h_minus:
            RETURN TRUE  // Aligned ✅
        ELSE:
            RETURN FALSE  // Conflict ❌ (exit and wait)

    // Case 2: Daily is PE (bearish)
    ELSE IF daily_dir == "PE":
        // Hourly must also be bearish
        IF h_adx >= 25 AND h_minus > h_plus:
            RETURN TRUE  // Aligned ✅
        ELSE:
            RETURN FALSE  // Conflict ❌ (exit and wait)

Step 4: Position Management
────────────────────────────
Function MANAGE_POSITION(aligned, daily_dir):

    // Case 1: Holding position, hourly conflicts
    IF in_position AND NOT aligned:
        LOG_WARN("Hourly conflicts with daily, exiting...")
        EXIT_POSITION()
        current_position = NULL
        WAIT_FOR_REALIGNMENT()

    // Case 2: No position, hourly aligned with daily
    ELSE IF NOT in_position AND aligned:
        // Wait for crossover signal
        IF CROSSOVER_SIGNAL_DETECTED(daily_dir):
            ENTER_POSITION(daily_dir)

    // Case 3: No position, hourly NOT aligned
    ELSE IF NOT in_position AND NOT aligned:
        LOG_INFO("Waiting for hourly to align with daily...")
        WAIT()  // Don't enter any trade
```

**Market Update Frequency & Triggers:**

```
Algorithm: When to Check for Signal Changes

1. Regular Monitoring (Every 1 Minute)
──────────────────────────────────────
SCHEDULE every 1 minute:
    bars = GET_LATEST_BARS("NIFTY", timeframe="1h", count=30)

    adx = CALCULATE_ADX(bars, period=14)
    plus_di, minus_di = CALCULATE_DI(bars, period=14)

    current_signal = DETERMINE_SIGNAL(adx, plus_di, minus_di)

    IF current_signal != previous_signal:
        HANDLE_SIGNAL_CHANGE(current_signal)

2. VIX Spike Trigger (Immediate)
─────────────────────────────────
ON vix_update:
    IF vix > 30 OR vix_spike > 5_points:
        LOG_WARN("VIX spike detected:", vix)
        EXIT_ALL_POSITIONS()  // Emergency exit
        PAUSE_TRADING()

3. Price Movement Trigger (Every 10 seconds)
─────────────────────────────────────────────
SCHEDULE every 10 seconds:
    current_price = GET_UNDERLYING_LTP("NIFTY")

    // Check if ATM changed
    new_atm = CALCULATE_ATM(current_price)
    IF new_atm != current_atm:
        LOG_INFO("ATM changed:", current_atm, "→", new_atm)
        UPDATE_ATM_SUBSCRIPTION(new_atm)
        // Don't exit position, just update for next trade

4. Stop Loss Trigger (Real-time)
─────────────────────────────────
ON tick_update:
    position = GET_OPEN_POSITION()
    IF position:
        current_pnl = CALCULATE_POSITION_PNL(position)

        IF current_pnl < position.stop_loss:
            LOG_WARN("Stop loss hit:", current_pnl)
            EXIT_POSITION_IMMEDIATELY()
```

**Real-World Example (CORRECTED LOGIC):**

```
Timeline: NIFTY Trading Day (Daily = Bullish, Trade CE ONLY)
────────────────────────────────────────────────────────────

09:15 AM: Daily Analysis
├─ Daily ADX: 32 ✅
├─ Daily +DI: 28, -DI: 18
├─ Daily Direction: BULLISH (CE only for today)
└─ Set: daily_direction = "CE"

09:20 AM: Hourly Check
├─ Hourly ADX: 28 ✅
├─ Hourly +DI: 27, -DI: 19
├─ Hourly: BULLISH (aligned with daily) ✅
└─ Ready to trade CE

09:30 AM: Wait for Crossover
├─ NIFTY: 23,450
├─ Crossover: +DI crosses above -DI ✅
└─ Action: BUY NIFTY26DEC23450CE @ ₹150

10:00 AM: Update (Holding CE)
├─ NIFTY: 23,512
├─ Daily: Still bullish ✅
├─ Hourly: Still bullish ✅
├─ Current P&L: +₹2,500 (profit)
└─ Action: HOLD CE position ✅

11:00 AM: Hourly Becomes Bearish ⚠️
├─ NIFTY: 23,489 (dropped)
├─ Daily: Still bullish (+DI > -DI) ✅
├─ Hourly: NOW bearish (-DI > +DI) ❌
├─ Conflict: Daily says CE, Hourly says PE
├─ Current P&L on CE: +₹750
└─ Action:
    1. EXIT CE (hourly conflicts) ✅
    2. SELL NIFTY26DEC23450CE @ ₹165
    3. Book profit: +₹750
    4. DO NOT buy PE! ❌ (daily is still bullish)
    5. WAIT for hourly to become bullish again ⏳

11:30 AM: Still Waiting (No Position)
├─ NIFTY: 23,445
├─ Daily: Still bullish ✅
├─ Hourly: Still bearish ❌
├─ Not aligned
└─ Action: WAIT ⏳ (NO TRADE)

12:00 PM: Still Waiting
├─ NIFTY: 23,460
├─ Daily: Bullish ✅
├─ Hourly: Still bearish (ADX 22) ❌
└─ Action: WAIT ⏳ (NO TRADE)

01:00 PM: Hourly Aligns Back! ✅
├─ NIFTY: 23,478
├─ Daily: Bullish ✅
├─ Hourly: NOW bullish again! (ADX 26, +DI > -DI) ✅
├─ Aligned: Both bullish ✅
└─ Ready to trade CE again

01:15 PM: Wait for Crossover
├─ NIFTY: 23,489
├─ Crossover: Price crosses above resistance ✅
└─ Action: BUY NIFTY26DEC23500CE @ ₹155

02:00 PM: Update (Holding CE)
├─ NIFTY: 23,523
├─ Daily: Bullish ✅
├─ Hourly: Bullish ✅
├─ Current P&L: +₹1,200
└─ Action: HOLD CE position ✅

03:25 PM: Square Off Before Close
├─ Time: 5 minutes to close
├─ Rule: No overnight positions
└─ Action: SELL NIFTY26DEC23500CE @ ₹178
           Book profit: +₹1,150

═══════════════════════════════════════════════════════════
Summary of Day (Daily = Bullish, Trade CE ONLY):
═══════════════════════════════════════════════════════════

Total Trades: 2 (BOTH CE, NO PE!)

Trade 1: NIFTY 23450 CE (9:30 AM - 11:00 AM)
├─ Reason: Daily + Hourly both bullish
├─ Entry: ₹150
├─ Exit: ₹165 (hourly conflict)
└─ P&L: +₹750

No Trade Period: 11:00 AM - 1:00 PM ⏳
├─ Reason: Hourly bearish (conflicts with daily)
├─ Did NOT trade PE (daily still bullish!)
└─ Waited for hourly to align back

Trade 2: NIFTY 23500 CE (1:15 PM - 3:25 PM)
├─ Reason: Hourly aligned back with daily
├─ Entry: ₹155
├─ Exit: ₹178 (square off)
└─ P&L: +₹1,150

Total Day P&L: +₹1,900 (before charges)

Key Observations:
─────────────────
✅ Daily set direction: CE ONLY for entire day
✅ Exited when hourly conflicted (didn't switch to PE!)
✅ Waited for hourly to align back with daily
✅ Re-entered CE when both aligned again
❌ NEVER traded PE (daily was bullish all day)
✅ Only 2 trades (both CE), avoided false signals
```

**Critical Rules (UPDATED):**

```
┌──────────────────────────────────────────────────────────┐
│     NEVER TRADE AGAINST DAILY TREND                      │
└──────────────────────────────────────────────────────────┘

Rule 1: Daily Trend = Master Direction
───────────────────────────────────────
✅ If Daily = Bullish → Trade CE ONLY for entire day
✅ If Daily = Bearish → Trade PE ONLY for entire day
❌ NEVER switch between CE and PE during same day

Rule 2: Hourly = Entry Timing (NOT Direction)
──────────────────────────────────────────────
✅ Hourly MUST align with daily before entry
✅ If hourly conflicts → EXIT and WAIT
❌ If hourly conflicts → DO NOT trade opposite

Rule 3: Exit When Hourly Conflicts
───────────────────────────────────
IF in_position AND hourly_conflicts_with_daily:
    → EXIT position ✅
    → WAIT for hourly to align back ⏳
    → DO NOT enter opposite direction ❌

Rule 4: Re-enter When Aligned Back
───────────────────────────────────
IF hourly_aligns_back_with_daily:
    → Wait for crossover signal
    → Re-enter SAME direction (CE or PE)
    → Not opposite direction!

Rule 5: Daily Direction Changes Next Day Only
──────────────────────────────────────────────
✅ Check daily trend at 9:15 AM every day
✅ If today = Bullish, tomorrow might be Bearish
✅ Change direction ONLY at start of new day
❌ Do NOT change direction during same day

Example Scenarios:
──────────────────

Scenario 1: Daily Bullish, Hourly Becomes Bearish
├─ Daily: Bullish (CE direction)
├─ Hourly: Becomes bearish
├─ Action: EXIT CE, WAIT (don't trade PE!)
└─ Re-enter: CE when hourly bullish again

Scenario 2: Daily Bearish, Hourly Becomes Bullish
├─ Daily: Bearish (PE direction)
├─ Hourly: Becomes bullish
├─ Action: EXIT PE, WAIT (don't trade CE!)
└─ Re-enter: PE when hourly bearish again

Scenario 3: Next Day Reversal
├─ Monday: Daily bullish → Trade CE all day
├─ Tuesday: Daily bearish → Trade PE all day
├─ Switch happens: At 9:15 AM Tuesday
└─ Not during Monday!
```

**Corrected Position Management:**

```
1. ALWAYS exit before entering opposite direction
   ❌ WRONG: Hold CE and PE together (hedging not allowed)
   ✅ RIGHT: Exit CE → Wait → Enter PE

2. Use LIMIT orders for entry, MARKET orders for exit
   ✅ Entry: LIMIT order (get good price)
   ✅ Exit: MARKET order (exit fast when signal changes)

3. Minimum holding period: 15 minutes
   Don't switch positions too frequently
   Avoid whipsaws and excessive costs

4. Maximum switches per day: 3
   If signal changes >3 times → STOP TRADING
   Market is choppy, no clear trend

5. Gap between exit and entry: 30 seconds
   Ensure exit is confirmed before new entry
   Avoid simultaneous CE and PE positions
```

**Signal Change Validation:**

```
Algorithm: Confirm Signal Change (Avoid False Signals)

Function IS_VALID_SIGNAL_CHANGE(new_signal):

    // Require 2 consecutive confirmations
    IF new_signal == last_signal:
        confirmation_count += 1
    ELSE:
        confirmation_count = 1
        last_signal = new_signal

    // Only act after 2 confirmations (2 minutes)
    IF confirmation_count >= 2:
        RETURN TRUE
    ELSE:
        LOG_INFO("Signal change pending confirmation:", new_signal)
        RETURN FALSE

Example:
────────
10:00 AM: Signal changes CE → PE (count: 1, wait)
10:01 AM: Signal still PE (count: 2, confirmed! ✅)
         → Exit CE, Enter PE

vs.

10:00 AM: Signal changes CE → PE (count: 1, wait)
10:01 AM: Signal back to CE (count: 1, false signal ❌)
         → Don't exit, continue holding CE
```

**Position Tracking:**

```
Data Structure: Track Current Position

current_position = {
    symbol: "NIFTY26DEC23450CE",
    token: 99926009,
    direction: "CE",
    entry_time: "2024-12-23 09:15:00",
    entry_price: 150.00,
    quantity: 50,
    current_price: 165.00,
    pnl: +750.00,
    stop_loss: -375.00,  // 50% of entry
    target: +1125.00,    // 150% of entry
    atm_at_entry: 23450
}

Update this EVERY tick for P&L monitoring
```

#### 3.1.4 Dynamic Token Selection Algorithm (Implementation)

#### 3.1.4 Strike Increments Reference

| Underlying | Strike Increment | Example Strikes                   |
| ---------- | ---------------- | --------------------------------- |
| NIFTY      | 50               | 23400, 23450, 23500, 23550, 23600 |
| BANKNIFTY  | 100              | 48900, 49000, 49100, 49200, 49300 |
| FINNIFTY   | 50               | 21800, 21850, 21900, 21950, 22000 |
| MIDCPNIFTY | 25               | 10000, 10025, 10050, 10075, 10100 |
| SENSEX     | 100              | 77000, 77100, 77200, 77300, 77400 |

#### 3.1.5 Expiry Date Calculation

```
Algorithm: Calculate Expiry Dates

1. Weekly Expiry (Most liquid):
   - NIFTY: Every Thursday
   - BANKNIFTY: Every Wednesday
   - FINNIFTY: Every Tuesday

   Function GET_WEEKLY_EXPIRY(underlying, today):
       IF underlying == "NIFTY":
           RETURN NEXT_THURSDAY(today)
       ELSE IF underlying == "BANKNIFTY":
           RETURN NEXT_WEDNESDAY(today)
       ELSE IF underlying == "FINNIFTY":
           RETURN NEXT_TUESDAY(today)

2. Monthly Expiry:
   - Last Thursday of every month for all indices

   Function GET_MONTHLY_EXPIRY(today):
       RETURN LAST_THURSDAY_OF_MONTH(today)

3. Expiry Date Format in CSV:
   - Format: "26Dec2024" (Angel One format)
   - Convert to: "2024-12-26" for internal use
```

#### 3.1.6 JSON Storage Format

```json
{
  "metadata": {
    "created_at": "2024-12-15T10:00:00Z",
    "source": "angel_one_csv",
    "csv_date": "2024-12-15"
  },
  "tokens": {
    "NIFTY_2024-12-26_23500": {
      "underlying": "NIFTY",
      "expiry": "2024-12-26",
      "strike": 23500,
      "ce_token": 99926009,
      "ce_symbol": "NIFTY23DEC23500CE",
      "pe_token": 99926010,
      "pe_symbol": "NIFTY23DEC23500PE",
      "lot_size": 50
    },
    "NIFTY_2024-12-26_23550": {
      "underlying": "NIFTY",
      "expiry": "2024-12-26",
      "strike": 23550,
      "ce_token": 99926011,
      "ce_symbol": "NIFTY23DEC23550CE",
      "pe_token": 99926012,
      "pe_symbol": "NIFTY23DEC23550PE",
      "lot_size": 50
    }
  }
}
```

#### 3.1.7 Token Refresh Strategy & Expiry Handling

**When to Re-Download CSV:**

1. **Daily (Mandatory)**: Every morning before 9:15 AM
   - Fresh tokens for the day
   - New expiry series added by exchange
   - Corporate actions, symbol changes
2. **After Weekly Expiry**: Immediately after 3:30 PM on expiry days
   - NIFTY: Every Thursday after 3:30 PM
   - BANKNIFTY: Every Wednesday after 3:30 PM
   - FINNIFTY: Every Tuesday after 3:30 PM
   - Reason: New weekly expiry becomes current, old one expires
3. **After Monthly Expiry**: Last Thursday of month after 3:30 PM
   - Next month's contracts become active
   - New strikes may be introduced
4. **On Startup**: If bot starts mid-day
   - Check CSV age: if >6 hours old, re-download
   - Validate token freshness before trading
5. **On Failure**: If token lookup fails repeatedly
   - Force CSV re-download
   - Possible: Exchange added new strikes, symbol changes

**CSV Age Check Algorithm:**

```
Algorithm: Should Re-Download CSV?

INPUT:
  - last_csv_download_time: Timestamp of last CSV download
  - current_time: Current timestamp
  - today_is_expiry_day: Boolean (is today weekly/monthly expiry?)

STEPS:

1. Check if CSV exists:
   IF csv_file NOT EXISTS:
       RETURN TRUE  // Must download

2. Calculate CSV age:
   csv_age_hours = (current_time - last_csv_download_time) / 3600

3. Check daily refresh (before market open):
   IF current_time < MARKET_OPEN (9:15 AM):
       IF csv_age_hours >= 12:  // Older than 12 hours
           RETURN TRUE  // Download fresh CSV

4. Check expiry day (after market close):
   IF today_is_expiry_day == TRUE:
       IF current_time > MARKET_CLOSE (3:30 PM):
           IF last_csv_download_time < MARKET_CLOSE:
               RETURN TRUE  // Download post-expiry CSV

5. Check stale CSV (during trading):
   IF csv_age_hours > 24:  // CSV is more than 1 day old
       RETURN TRUE  // CSV too old, refresh

6. Otherwise:
   RETURN FALSE  // Use existing CSV

OUTPUT: Boolean (true = download, false = use cached)
```

**Expiry Date Detection Algorithm:**

```
Algorithm: Is Today an Expiry Day?

INPUT:
  - today: Current date
  - underlying: "NIFTY", "BANKNIFTY", "FINNIFTY"

STEPS:

1. Get day of week:
   day_of_week = GET_DAY_OF_WEEK(today)

2. Check weekly expiry:
   IF underlying == "NIFTY":
       IF day_of_week == THURSDAY:
           RETURN TRUE (weekly expiry)

   ELSE IF underlying == "BANKNIFTY":
       IF day_of_week == WEDNESDAY:
           RETURN TRUE (weekly expiry)

   ELSE IF underlying == "FINNIFTY":
       IF day_of_week == TUESDAY:
           RETURN TRUE (weekly expiry)

3. Check monthly expiry:
   IF IS_LAST_THURSDAY_OF_MONTH(today):
       RETURN TRUE (monthly expiry)

4. Otherwise:
   RETURN FALSE (not an expiry day)
```

**Expiry Calendar (Always Updated):**

| Week   | NIFTY (Thu) | BANKNIFTY (Wed) | FINNIFTY (Tue) | Monthly (Last Thu)    |
| ------ | ----------- | --------------- | -------------- | --------------------- |
| Week 1 | 02-Jan-2025 | 01-Jan-2025     | 31-Dec-2024    | -                     |
| Week 2 | 09-Jan-2025 | 08-Jan-2025     | 07-Jan-2025    | -                     |
| Week 3 | 16-Jan-2025 | 15-Jan-2025     | 14-Jan-2025    | -                     |
| Week 4 | 23-Jan-2025 | 22-Jan-2025     | 21-Jan-2025    | -                     |
| Week 5 | 30-Jan-2025 | 29-Jan-2025     | 28-Jan-2025    | 30-Jan-2025 (Monthly) |

**Token Validation After Download:**

```
Algorithm: Validate Downloaded CSV

1. Check file size:
   IF csv_size < 1 MB:
       LOG_ERROR("CSV too small, possibly corrupt")
       USE_FALLBACK_CSV()

2. Parse and count:
   option_count = COUNT_OPTIONS_IN_CSV()

   IF option_count < 100:
       LOG_ERROR("Too few options in CSV")
       USE_FALLBACK_CSV()

3. Check expiry dates:
   expiry_dates = GET_UNIQUE_EXPIRY_DATES()

   // Should have at least 2-3 expiry dates (current + next)
   IF LENGTH(expiry_dates) < 2:
       LOG_WARN("Only one expiry in CSV, possible issue")

4. Verify current expiry exists:
   current_expiry = GET_CURRENT_WEEKLY_EXPIRY()

   IF current_expiry NOT IN expiry_dates:
       LOG_ERROR("Current week expiry missing from CSV")
       USE_FALLBACK_CSV()

5. Check for each underlying:
   FOR underlying IN ["NIFTY", "BANKNIFTY", "FINNIFTY"]:
       count = COUNT_OPTIONS(underlying, current_expiry)

       IF count < 50:  // Should have at least 50 strikes (25 CE + 25 PE)
           LOG_ERROR("Too few strikes for", underlying)
           USE_FALLBACK_CSV()

6. Success:
   LOG_INFO("CSV validated successfully")
   SAVE_AS_CURRENT_CSV()
   BACKUP_OLD_CSV()  // Keep as fallback
```

**Fallback Strategy:**

```
1. Primary: Use today's CSV
2. Fallback 1: Use yesterday's CSV (if today's fails)
3. Fallback 2: Use Angel One searchScrip API (real-time)
4. Fallback 3: Manual intervention (halt trading)

Priority Order:
  - Today's CSV (after 9:15 AM download)
  - Yesterday's CSV (if today's download fails)
  - API-based token search (if both CSV fail)
  - Stop trading (if all fail)
```

**CSV Storage Structure:**

```
data/tokens/
├── angelone_master_20250115.csv      # Today's CSV
├── angelone_master_20250114.csv      # Yesterday's CSV (backup)
├── tokens_20250115.json              # Today's parsed tokens
├── tokens_20250114.json              # Yesterday's parsed tokens (backup)
└── tokens_current.json               # Symlink/copy of active tokens
```

**Automatic CSV Rotation:**

```
Algorithm: Rotate Old CSVs

1. After successful download:
   - Rename today's CSV → angelone_master_YYYYMMDD.csv
   - Parse and save → tokens_YYYYMMDD.json
   - Create symlink → tokens_current.json

2. Cleanup old files:
   - Keep current day's CSV
   - Keep previous day's CSV (backup)
   - Delete CSVs older than 2 days
   - Compress CSVs older than 7 days (for audit)

3. On expiry day:
   - Keep expiry day's CSV for 30 days (audit trail)
   - Compress and archive
```

**Critical Rules:**

1. ✅ **NEVER trade with CSV older than 24 hours**
2. ✅ **Always re-download after expiry (post 3:30 PM)**
3. ✅ **Validate CSV before trading**
4. ✅ **Keep yesterday's CSV as backup**
5. ✅ **If download fails, use fallback or halt trading**

### 3.2 Underlying Classification

- **Index Options (NIFTY, BANKNIFTY)**:
  - Cash settled (no physical delivery)
  - Lower margin requirements
  - Monthly + weekly expiry available
  - Higher liquidity and volume
- **Stock Options (RELIANCE, TCS, etc.)**:
  - Physical settlement (delivery required)
  - Higher margin requirements near expiry
  - Expiry cadence varies by symbol; many have weekly expiries. Verify availability per symbol.
  - Lower liquidity, higher spreads

### 3.3 ADX-Based Token Categorization & ATM Management

- **Stock Categorization Process**:
  - **Category 1 - Buy CE**: ADX > 25, +DI > -DI, volume > average
  - **Category 2 - Buy PE**: ADX > 25, -DI > +DI, volume > average
  - **Category 3 - No Trade**: ADX < 20 or +DI ≈ -DI or low volume
- **Token Pool Management by Category**:
  - **Category 1 Tokens**: Maintain CE tokens for bullish stocks
  - **Category 2 Tokens**: Maintain PE tokens for bearish stocks
  - **Category 3 Tokens**: Remove from active trading pool
  - **Daily Rebalancing**: Update token pools based on new ADX values
- **ATM Calculation & Monitoring**:
  - **ATM Definition**: Nearest listed strike to the underlying LTP
  - **Strike Increments**: NIFTY 50 pts, BANKNIFTY 100 pts, FINNIFTY 50 pts (verify per contract)
  - **Price Monitoring**: Track underlying price changes every 5-10 seconds
  - **ATM Update Trigger**: When price moves >1 strike from current ATM
  - **Strike Range**: Monitor 5-10 strikes around current ATM
- **Category-Specific Token Selection**:
  - **CE Token Selection**: For Category 1 stocks, select CE strikes at ATM
  - **PE Token Selection**: For Category 2 stocks, select PE strikes at ATM
  - **Liquidity Check**: Verify selected tokens have sufficient OI/volume
  - **Token Validation**: Ensure tokens are active and tradeable
- **Dynamic Token Switching**:
  - **Category Changes**: Move tokens between CE/PE pools based on ADX changes
  - **ATM Updates**: Adjust strike selections based on price movements
  - **Gradual Replacement**: Update token list over 1-2 minutes
  - **No Trade Interruption**: Switch tokens without stopping ongoing trades
  - **Position Continuity**: Maintain existing positions while updating token pool

### 3.4 Gap-Up/Gap-Down Handling

- **Gap Detection**: Check for opening gaps >100 points at 9:15 AM
  - Compare current price with previous day's close
  - Calculate gap percentage: (Open - Previous Close) / Previous Close \* 100
  - Trigger gap handling if gap >2% (approximately 100+ points for NIFTY)
- **Gap Response Strategy**:
  - **Immediate Token Refresh**: Cancel all existing subscriptions
  - **New ATM Calculation**: Recalculate ATM based on gap-adjusted price
  - **Emergency Token Pool**: Subscribe to 10-15 strikes around new ATM
  - **Wider Strike Range**: Use ±100 points instead of ±50 for gap scenarios
  - **Liquidity Priority**: Focus on highest OI/volume strikes only
- **Gap Recovery Process**:
  - **Wait for Stability**: Wait 2-3 minutes for price to stabilize
  - **Reassess ATM**: Recalculate ATM after initial volatility
  - **Optimize Token Pool**: Reduce to normal ±50 point range
  - **Resume Normal Trading**: Continue with standard ATM management

## 4. Volatility Management & Risk Control

### 4.1 Volatility Detection

- **VIX Monitoring**: Track INDIA VIX levels and intraday changes
  - Resolve correct Angel One symbol/token for INDIA VIX (often "INDIAVIX"); verify via instruments dump
  - Fetch live VIX value using SmartAPI quote/LTP endpoint
  - Subscribe to VIX via WebSocket using `feedToken`
  - **VIX Spike Detection**: Monitor VIX changes >5 points in 10 minutes
  - **VIX Circuit Breaker**: Stop trading if VIX jumps >7 points in 5 minutes
- **Intraday Volatility Thresholds**:
  - **Normal VIX**: 12-18 (continue normal trading)
  - **Elevated VIX**: 18-25 (reduce position sizes by 50%)
  - **High VIX**: 25-30 (reduce position sizes by 75%, tighter stops)
  - **Extreme VIX**: >30 (pause trading, close existing positions)
- **Flash Spike Detection**:
  - **VIX Spike**: >5 point increase in 10 minutes
  - **Price Spike**: >2% move in 5 minutes
  - **Volume Spike**: >300% of average volume in 15 minutes
  - **Circuit Breaker**: Immediate trading halt on flash spikes

### 4.2 High Volatility Response Strategy

- **Position Sizing**: Reduce position sizes by 50-75%
- **Timeframe Switch**: Move to lower timeframes (15min → 5min, 1hr → 15min)
- **Strike Selection**: Focus on ATM options only (avoid OTM)
- **Entry Timing**: Wait for pullbacks, avoid chasing moves
- **Stop Loss**: Tighten stops to 0.5-1% of underlying
- **Target Management**: Use dynamic targets (2-5x risk)

### 4.3 Circuit Breaker Logic

- **Level 1 (VIX 18-25)**: Reduce position sizes by 50%
- **Level 2 (VIX 25-30)**: Reduce position sizes by 75%, tighten stops
- **Level 3 (VIX >30)**: Pause new trades, close existing positions
- **Flash Spike Response**: Immediate trading halt, close all positions
- **Recovery Process**: Wait for VIX to drop below threshold + 2 points
- **Gradual Resume**: Start with 25% position sizes, gradually increase

### 4.4 Risk Controls

- **Maximum Positions**: Limit to 2-3 concurrent positions
- **Daily Loss Limit**: Set strict daily loss limits
- **Position Duration**: Reduce holding time to 15-30 minutes
- **Delta Management**: Monitor and hedge delta exposure
- **Margin Monitoring**: Increase margin buffer by 25%
- **Global Kill-Switch**: On trigger, cancel open orders, close positions safely, and halt trading

## 5. Strategy Analysis & Signal Generation

### 5.1 Timeframe Selection

- **Choose appropriate timeframe** (1min, 5min, 15min, 1hr, daily)
- **Match timeframe to strategy type** (scalping, swing, positional)
- **Consider option expiry timeline** vs strategy duration
- **High Volatility Actions**:
  - Switch to lower timeframes (15min → 5min, 1hr → 15min)
  - Reduce position sizes by 50%
  - **Dynamic Risk-Reward**:
    - Tight stop-loss levels (0.5-1% of underlying)
    - Big dynamic targets (2-5x risk or more)
    - Trail stop-loss as price moves favorably
    - Scale out positions at multiple target levels

### 5.2 Multi-Timeframe Analysis

- **Timeframe Hierarchy**:
  - **Trend Confirmation (Higher Timeframe)**:
    - Daily chart for overall trend direction
    - 1-hour chart for intermediate trend
    - 15-minute chart for short-term trend
  - **Entry/Exit (Lower Timeframe)**:
    - 5-minute chart for precise entries
    - 1-minute chart for scalping entries
    - Match trading timeframe to strategy type

### 5.3 CE vs PE Selection

- **Buy CE (Call Options) when**:
  - Bullish trend confirmed on higher timeframe
  - Support level holding strong on 15min/1hr
  - Volume increasing on up moves
  - RSI oversold and turning up on 5min
  - Breakout above resistance levels
- **Buy PE (Put Options) when**:
  - Bearish trend confirmed on higher timeframe
  - Resistance level holding strong on 15min/1hr
  - Volume increasing on down moves
  - RSI overbought and turning down on 5min
  - Breakdown below support levels
- **Avoid both when**:
  - Sideways/consolidation market
  - Low volatility environment
  - No clear trend direction

### 5.4 ADX-Based Stock Categorization & Option Selection

**⚠️ CRITICAL: Multi-Timeframe Strategy**

- **DAILY timeframe**: Determine trend direction (CE or PE)
- **HOURLY timeframe**: Find entry/exit signals
- **Both must align**: Daily + Hourly confirmation required

- **Historical Data Requirement**: Download 1-2 years of historical data for underlying stocks/indices
- **Daily ADX Calculation**: Calculate Average Directional Index on DAILY timeframe for trend direction
- **Hourly ADX Calculation**: Calculate ADX on HOURLY timeframe for entry timing

#### Step 1: Daily Trend Direction (Which side: CE or PE?)

- **Stock Categorization Process** (using DAILY bars):

  - **Category 1 - Buy CE**: Daily ADX > 25, Daily +DI > -DI
  - **Category 2 - Buy PE**: Daily ADX > 25, Daily -DI > +DI
  - **Category 3 - No Trade**: Daily ADX < 25 or unclear direction

- **Category 1 - Buy CE Stocks** (Daily Uptrend):

  - **Criteria**: Daily ADX > 25, Daily +DI > -DI
  - **Option Type**: Look for CE (Call) options
  - **ATM Selection**: Select CE strikes closest to current stock price
  - **Strike Range**: ±50 points around current price for liquidity
  - **Next Step**: Wait for hourly confirmation before entry

- **Category 2 - Buy PE Stocks** (Daily Downtrend):

  - **Criteria**: Daily ADX > 25, Daily -DI > +DI
  - **Option Type**: Look for PE (Put) options
  - **ATM Selection**: Select PE strikes closest to current stock price
  - **Strike Range**: ±50 points around current price for liquidity
  - **Next Step**: Wait for hourly confirmation before entry

- **Category 3 - No Trade Stocks**:
  - **Criteria**: Daily ADX < 25 or +DI ≈ -DI
  - **Action**: Skip trading for these stocks
  - **Reason**: No clear daily trend

#### Step 2: Hourly Entry Confirmation (When to trade?)

- **Entry Signal Requirements** (using HOURLY bars):

  - Hourly ADX > 25 (strong hourly trend)
  - Hourly direction MUST match daily direction
  - For CE: Both daily and hourly +DI > -DI
  - For PE: Both daily and hourly -DI > +DI
  - Volume confirmation on hourly timeframe

- **Entry Logic**:

  ```
  IF Daily says CE:
    WAIT until Hourly ADX > 25 AND Hourly +DI > -DI
    THEN BUY CE

  IF Daily says PE:
    WAIT until Hourly ADX > 25 AND Hourly -DI > +DI
    THEN BUY PE

  IF Daily and Hourly conflict:
    NO TRADE (wait for alignment)
  ```

- **ATM Strike Selection Process**:

  - **Current Price**: Get real-time LTP of underlying stock
  - **Strike Calculation**: Find nearest available strike price
  - **CE Selection**: For Category 1 stocks, select CE at ATM
  - **PE Selection**: For Category 2 stocks, select PE at ATM
  - **Liquidity Check**: Ensure selected strikes have sufficient OI/volume

- **Daily Rebalancing**:

  - **Update Categories**: Recalculate DAILY ADX once per day (morning)
  - **Category Changes**: Move stocks between categories based on new daily ADX values
  - **Hourly Monitoring**: Check HOURLY ADX every hour for entry signals
  - **ATM Adjustments**: Adjust strike selections based on price movements

- **ADX Implementation Details**:
  - **Period**: 14-period ADX calculation (standard)
  - **Daily Data Source**: Daily OHLCV data for last 30 days minimum
  - **Hourly Data Source**: Hourly OHLCV data for last 30 hours minimum
  - **Calculation**: True Range, +DI, -DI, and ADX values for both timeframes
  - **Update Frequency**:
    - Daily ADX: Once per day (9:15 AM)
    - Hourly ADX: Every hour during market hours
  - **Multi-Timeframe Confirmation**: Daily defines direction, Hourly confirms timing

### 5.5 Concrete Strategy Logic & Execution Rules

#### 5.5.1 Entry Decision Framework

- **Pre-Entry Validation Checklist**:

  - Market hours validation (9:15 AM - 3:30 PM)
  - VIX within acceptable range (12-30)
  - Category status valid (not NoTrade)
  - Sufficient account balance and margin
  - Token validity and liquidity check (OI > 1000)
  - Maximum position limit not exceeded (2-3 concurrent positions)
  - No entry after 2:30 PM (insufficient time for trade development)

- **Bullish Entry Conditions (Call Options)**:

  - **Higher Timeframe Alignment**: Daily ADX > 25 and +DI > -DI (bullish trend)
  - **Intermediate Trend**: 1-hour chart shows higher highs and higher lows
  - **Support Validation**: 15-minute low staying above EMA-20
  - **Entry Trigger Options**:
    - **Pullback Entry**: 5-minute RSI < 40 and price bounces off 9-EMA
    - **Breakout Entry**: Current price breaks above 1-hour high with volume
  - **Volume Confirmation**: Current volume > 120% of average volume
  - **Risk-Reward Setup**: Stop loss at 1% below entry, target at 3% above entry

- **Bearish Entry Conditions (Put Options)**:
  - **Higher Timeframe Alignment**: Daily ADX > 25 and -DI > +DI (bearish trend)
  - **Intermediate Trend**: 1-hour chart shows lower highs and lower lows
  - **Resistance Validation**: 15-minute high staying below EMA-20
  - **Entry Trigger Options**:
    - **Pullback Entry**: 5-minute RSI > 60 and price rejects from 9-EMA
    - **Breakdown Entry**: Current price breaks below 1-hour low with volume
  - **Volume Confirmation**: Current volume > 120% of average volume
  - **Risk-Reward Setup**: Stop loss at 1% above entry, target at 3% below entry

#### 5.5.2 Position Sizing Decision Matrix

- **Base Position Size**: 2% of total account balance per trade
- **Volatility Adjustments**:
  - VIX < 15: Increase position by 25% (low volatility environment)
  - VIX 15-20: Standard position size (normal market)
  - VIX 20-25: Reduce position by 25% (elevated volatility)
  - VIX 25-30: Reduce position by 50% (high volatility)
  - VIX > 30: Reduce position by 75% or skip trading (extreme volatility)
- **Time Decay Adjustments**:
  - More than 14 days to expiry: Standard position size
  - 7-14 days to expiry: Reduce position by 25%
  - Less than 7 days to expiry: Reduce position by 50%
- **Liquidity Adjustments**:
  - OI > 5000: Standard position size
  - OI 1000-5000: Reduce position by 25%
  - OI 500-1000: Reduce position by 50%
  - OI < 500: Skip trade (insufficient liquidity)
- **Multiple Position Adjustments**:
  - 1 position: Standard position size
  - 2 positions: Reduce each by 20%
  - 3 positions: Reduce each by 40%
  - Maximum 3 concurrent positions allowed

#### 5.5.3 Exit Decision Framework (Priority Ordered)

- **Priority 1 - Mandatory Exits** (highest priority, execute immediately):

  - Market close approaching (exit all positions 30 minutes before 3:30 PM)
  - Expiry approaching (exit all positions 3 days before expiry)
  - Account daily loss limit reached (exit all and halt trading)
  - VIX spike > 5 points in 10 minutes (exit all positions)
  - Token/session expiry imminent (exit all positions)

- **Priority 2 - Risk-Based Exits**:

  - Stop loss hit (1% move against position in underlying)
  - Consecutive losing trades (after 3 losses, reduce position size or halt)
  - Margin approaching 80% utilization (close weakest position)

- **Priority 3 - Profit-Based Exits**:

  - Primary target reached (3% move in favor, close 100% of position)
  - Partial profit taking at 1:1 risk-reward (close 50%, trail stop on remaining)
  - Trailing stop hit after moving to breakeven

- **Priority 4 - Technical Exits**:

  - Trend reversal on 5-minute chart (price crosses EMA-9 opposite direction + RSI confirmation)
  - Volume drying up (volume drops below 50% of average for 15 minutes)
  - Higher timeframe trend change (daily ADX category shifts)

- **Priority 5 - Time-Based Exits**:
  - Position held over 60 minutes with negative P&L (time decay risk)
  - Position held over 2 hours with minimal profit (opportunity cost)

#### 5.5.4 Trailing Stop Loss Management

- **Breakeven Move**: After 1% profit (1:1 risk-reward), move stop loss to entry price + 0.5%
- **Aggressive Trailing**: After 2% profit (1:2 risk-reward), trail stop at 1.5% below current price
- **Lock Profits**: After 3% profit (target reached), trail stop at 2% below current price
- **Update Frequency**: Check and update trailing stop every 30 seconds during active position

#### 5.5.5 Re-Entry Rules After Stop-Out

- **Cooling Period**: Wait minimum 30 minutes after stop-out before considering re-entry
- **Time Restriction**: No re-entry after 2:30 PM (insufficient time remaining)
- **Price Movement Validation**: Underlying must move >1% from previous entry price
- **Category Revalidation**: Ensure stock still in same ADX category (CE or PE)
- **Market Condition Check**: Ensure VIX hasn't spiked, volume still healthy
- **Maximum Attempts**: Maximum 2 re-entries per symbol per day
- **Stop Loss Adjustment**: Use wider stop loss (1.5%) on re-entry to avoid whipsaws

#### 5.5.6 Correlation and Concentration Limits

- **Maximum per Underlying**: Only 1 option position per underlying (NIFTY/BANKNIFTY/Stock)
- **Sector Concentration**: Maximum 50% of positions from same sector
- **Direction Concentration**: Maximum 70% of positions in same direction (all CE or all PE)
- **Expiry Concentration**: Diversify across at least 2 different expiry dates if holding 3+ positions

## 6. Order Management & Safety

### 6.1 Order Generation

- Generate option orders (buy/sell calls/puts)
- Set appropriate limit prices
- Create unique order IDs for idempotency
- Validate order parameters before submission

### 6.2 Order Safety Measures

- **Idempotent Order IDs**: Use unique order IDs to prevent duplicates
  - Generate UUID-based order IDs
  - Store order ID mapping in JSON file (state/orders.json)
  - Check for existing orders before retry
- **Order Verification Process**:
  - **Place Order**: Submit order via Angel One SmartAPI
  - **Get Order ID**: Store returned order ID
  - **Verify Fill**: Check order book for execution confirmation
  - **Status Validation**: Verify order status (COMPLETE, REJECTED, PENDING)
  - **Position Update**: Update position only after confirmed fill
- **Auto-Cancel Logic**:
  - **Pending Order Timeout**: Auto-cancel after 30-60 seconds
  - **Market Hours Check**: Cancel all pending orders at 3:25 PM
  - **Volatility Check**: Cancel orders if VIX spikes >5 points
  - **Price Movement Check**: Cancel if underlying moves >1% from order price
- **Order Retry Strategy**:
  - **Exponential Backoff**: Retry with increasing delays (1s, 2s, 4s)
  - **Max Retries**: Limit to 3 retry attempts
  - **Error Handling**: Log and handle API errors gracefully
  - **Duplicate Prevention**: Use app-side deduplication by hashing order intent to avoid duplicates on retries

### 6.3 Order Execution Confirmation

- Place orders through broker API
- Verify order placement success
- Monitor order status via WebSocket
- Confirm execution via order book
- Update position tracking

## 7. Position Monitoring & Risk Management

### 7.1 Position Monitoring

- Track open positions in real-time
- Monitor P&L changes
- Update stop-loss levels
- Check for early exit conditions
- Close stock options before expiry to avoid delivery

### 7.2 Risk Management

- Execute stop-loss orders
- Adjust position sizes
- Hedge delta exposure if needed
- Close positions at expiry
- Handle settlement differences (cash vs physical)

### 7.3 Performance Tracking

- Log all trades and outcomes
- Calculate daily/weekly P&L
- Track win rate and average returns
- Generate performance reports

## 8. Error Handling & System Management

### 8.1 Error Handling

- Monitor for API failures
- Handle network disconnections
- Manage order rejections
- Implement circuit breakers
- **Missing Data Handling**:
  - If token not found in CSV/API, use cached JSON file
  - If cached data is stale (>7 days), skip trading that token
  - If no data available, use broker's instrument list API
  - Fallback to manual token list if all else fails
  - Log all missing tokens for manual review

### 8.2 Market Closure Handling

- **Weekend Mode**: System maintenance and data backup
- **Holiday Mode**: Skip trading, update holiday calendar
- **Early Closure**: Handle special market hours (e.g., Diwali)
- **Emergency Closure**: Handle unexpected market shutdowns
- **Position Management**: Close positions before market close

### 8.3 Market Off-Time Actions

- **Data Backup**: Backup all trading data and logs
- **System Maintenance**: Update software, clear temp files
- **Strategy Analysis**: Review performance, update parameters
- **Holiday Calendar Update**: Check for upcoming holidays
- **Position Review**: Analyze closed positions and P&L
- **Next Day Preparation**: Load tomorrow's trading plan

### 8.4 Market On-Time Actions

- **Session Validation**: Check if market is open
- **Data Sync**: Sync with latest market data
- **Position Check**: Verify all positions are active
- **Strategy Activation**: Start trading based on signals

## 9. System Shutdown

- Close all open positions
- Save trading logs
- Update performance metrics
- Prepare for next trading session

---

## 10. Enhanced Smart Decision Making & Operations

### 10.1 Intelligent Market Condition Assessment

- **Market Regime Detection**:
  - **Trending Market**: ADX > 25, clear directional bias
  - **Ranging Market**: ADX < 20, sideways price action
  - **Volatile Market**: VIX > 25, high intraday swings
  - **Low Volatility**: VIX < 15, compressed price action
  - **Gap Market**: Opening gap > 2% from previous close
- **Smart Operation Selection Based on Market Regime**:
  - **Trending Market**: Use momentum strategies, hold positions longer
  - **Ranging Market**: Use mean reversion, shorter holding periods
  - **Volatile Market**: Reduce position sizes, use wider stops
  - **Low Volatility**: Increase position sizes, use tighter stops
  - **Gap Market**: Wait for stabilization, use wider strike ranges

### 10.2 Dynamic Position Sizing Logic

- **Base Position Size Calculation**:
  - **Account Size**: Use 2-5% of account per trade
  - **Volatility Adjustment**: Reduce size by 25% for each VIX level increase
  - **Time Decay Adjustment**: Reduce size by 50% in last week of expiry
  - **Liquidity Adjustment**: Reduce size by 25% for low OI strikes
- **Smart Position Sizing Triggers**:
  - **High VIX (>25)**: Reduce all positions by 75%
  - **Low VIX (<15)**: Increase positions by 25%
  - **Near Expiry (<7 days)**: Reduce positions by 50%
  - **Low Liquidity**: Reduce positions by 25%
  - **Multiple Positions**: Reduce each position by 20% for each additional position

### 10.3 Intelligent Entry Timing

- **Pre-Entry Validation Checklist**:
  - **Market Hours**: Ensure within trading hours (9:15 AM - 3:30 PM)
  - **Token Validity**: Verify token is active and tradeable
  - **Liquidity Check**: Ensure minimum OI > 1000 contracts
  - **Volatility Check**: VIX within acceptable range (12-30)
  - **Trend Confirmation**: Higher timeframe trend aligns with trade direction
  - **Support/Resistance**: Price near key levels for better risk-reward
- **Smart Entry Triggers**:
  - **Breakout Entry**: Price breaks above resistance with volume
  - **Pullback Entry**: Price pulls back to support in uptrend
  - **Reversal Entry**: RSI divergence with price action
  - **Gap Fill Entry**: Price moves to fill opening gap
  - **Time-based Entry**: Enter at specific market hours (10:30 AM, 2:30 PM)

### 10.4 Advanced Exit Management

- **Dynamic Stop Loss Management**:
  - **Initial Stop**: 1% of underlying price
  - **Trailing Stop**: Move stop to breakeven after 0.5% profit
  - **Volatility Stop**: Adjust stop based on VIX levels
  - **Time Stop**: Close position if no movement in 30 minutes
  - **Support/Resistance Stop**: Place stop beyond key levels
- **Smart Target Management**:
  - **Risk-Reward Ratio**: Minimum 1:2 risk-reward ratio
  - **Dynamic Targets**: Adjust targets based on volatility
  - **Partial Profit Taking**: Close 50% at 1:1, trail remaining 50%
  - **Time-based Exit**: Close all positions 30 minutes before market close
  - **Volatility Exit**: Close positions if VIX spikes >5 points

### 10.5 Intelligent Risk Management

- **Portfolio Risk Assessment**:
  - **Maximum Drawdown**: Limit to 5% of account per day
  - **Correlation Check**: Avoid highly correlated positions
  - **Delta Exposure**: Monitor total delta exposure
  - **Margin Utilization**: Keep margin usage below 70%
  - **Position Concentration**: Limit single stock exposure to 20%
- **Smart Risk Controls**:
  - **Daily Loss Limit**: Stop trading after 3% daily loss
  - **Consecutive Loss Limit**: Stop after 3 consecutive losses
  - **Volatility Circuit Breaker**: Pause trading if VIX > 30
  - **Liquidity Circuit Breaker**: Avoid trading if OI < 500
  - **Time Decay Protection**: Close positions 3 days before expiry

### 10.6 Adaptive Strategy Selection

- **Strategy Selection Matrix**:
  - **High VIX + Trending**: Use momentum strategies
  - **High VIX + Ranging**: Use volatility strategies
  - **Low VIX + Trending**: Use trend-following strategies
  - **Low VIX + Ranging**: Use mean reversion strategies
  - **Gap Market**: Use gap-fill strategies
- **Strategy Performance Tracking**:
  - **Win Rate**: Track success rate for each strategy
  - **Average Return**: Monitor average profit per trade
  - **Maximum Drawdown**: Track worst losing streak
  - **Sharpe Ratio**: Measure risk-adjusted returns
  - **Strategy Rotation**: Switch strategies based on performance

### 10.7 Smart Data Management

- **Intelligent Data Prioritization**:
  - **Critical Data**: Real-time prices for active positions
  - **Important Data**: Option chain data for ATM calculation
  - **Background Data**: Historical data for analysis
  - **Maintenance Data**: Token lists and instrument data
- **Smart Data Refresh Logic**:
  - **Real-time Data**: Update every 1-5 seconds during trading
  - **Option Chain**: Top-of-book strikes every 1-2 minutes; full chain every 5-10 minutes
  - **Historical Data**: Update daily after market close
  - **Token Data**: Update monthly or when new contracts available
- **Data Quality Validation**:
  - **Price Validation**: Check for unrealistic price movements
  - **Volume Validation**: Verify volume data consistency
  - **Gap Detection**: Identify and handle data gaps
  - **Outlier Detection**: Flag and handle price outliers

### 10.8 Intelligent Error Recovery

- **Error Classification**:
  - **Critical Errors**: Token expiry, network failure, API errors
  - **Warning Errors**: Data gaps, low liquidity, high volatility
  - **Info Errors**: Minor delays, temporary issues
- **Smart Error Response**:
  - **Critical Errors**: Pause trading, notify user, attempt recovery
  - **Warning Errors**: Reduce position sizes, increase monitoring
  - **Info Errors**: Log and continue with normal operations
- **Recovery Strategies**:
  - **Token Recovery**: Automatic re-login with user notification
  - **Network Recovery**: Retry with exponential backoff
  - **Data Recovery**: Request missing data from alternative sources
  - **Position Recovery**: Verify and reconcile all positions

### 10.9 Performance Optimization

- **Smart Resource Management**:
  - **CPU Usage**: Optimize calculations during market hours
  - **Memory Usage**: Clean up unused data structures
  - **Network Usage**: Batch API requests to reduce overhead
  - **Storage Usage**: Compress and archive old data
- **Intelligent Caching**:
  - **Price Cache**: Cache frequently accessed prices
  - **Token Cache**: Cache instrument and token data
  - **Strategy Cache**: Cache calculated indicators
  - **Result Cache**: Cache backtesting results
- **Performance Monitoring**:
  - **Response Time**: Monitor API response times
  - **Throughput**: Track data processing speed
  - **Error Rate**: Monitor error frequency
  - **Resource Usage**: Track CPU, memory, and network usage

### 10.10 Smart Notification System

- **Notification Prioritization**:
  - **Critical**: Token expiry, system errors, large losses
  - **Important**: Position updates, strategy changes, market alerts
  - **Info**: Daily reports, performance updates, maintenance
- **Smart Notification Delivery**:
  - **Real-time**: Critical notifications via popup/email
  - **Scheduled**: Daily reports via email
  - **On-demand**: User-requested information
  - **Contextual**: Notifications based on current market conditions
- **Notification Content**:
  - **Action Required**: Clear instructions for user action
  - **Context**: Relevant market conditions and background
  - **Impact**: Potential impact on positions and performance
  - **Timeline**: When action needs to be taken

## Implementation Checklist

### Critical Safety Features

- [ ] No interpolation for options data
- [ ] Idempotent order management
- [ ] Circuit breakers for volatility spikes
- [ ] Gap-up/gap-down handling
- [ ] Auto-cancel stale orders
- [ ] Position verification before updates

### Token Management (Critical)

- [ ] Daily token expiry detection
- [ ] Pre-market token validation
- [ ] User notification system for token refresh
- [ ] Manual token input mechanism
- [ ] Token validation during market hours
- [ ] Emergency trading halt on token expiry
- [ ] Position protection before token expiry

### Data Management

- [ ] Underlying data for trend analysis (1-2 years)
- [ ] Options data for current trading (3 months)
- [ ] Real-time WebSocket data for execution
- [ ] Proper timeframe construction
- [ ] Data quality validation

### Risk Management

- [ ] VIX-based circuit breakers
- [ ] Dynamic position sizing
- [ ] Timeframe switching
- [ ] Flash spike detection
- [ ] Daily loss limits

### Order Safety

- [ ] Unique order IDs
- [ ] Order verification process
- [ ] Auto-cancel logic
- [ ] Retry strategy with backoff
- [ ] Position updates only after confirmed fills

### Enhanced Smart Features

- [ ] Market regime detection
- [ ] Dynamic position sizing logic
- [ ] Intelligent entry timing
- [ ] Advanced exit management
- [ ] Intelligent risk management
- [ ] Adaptive strategy selection
- [ ] Smart data management
- [ ] Intelligent error recovery
- [ ] Performance optimization
- [ ] Smart notification system

## 11. Critical Missing Components & Additional Considerations

### 11.1 Regulatory Compliance & Legal Framework

- **SEBI Compliance Requirements**:
  - **Position Limits**: Monitor and enforce SEBI position limits for options
  - **Margin Requirements**: Ensure adequate margin for all positions
  - **Reporting Requirements**: Generate required regulatory reports
  - **Audit Trail**: Maintain complete audit trail of all trades
  - **Risk Management**: Implement SEBI-mandated risk management systems
- **Tax Compliance**:
  - **P&L Calculation**: Calculate taxable P&L for each trade
  - **STT (Securities Transaction Tax)**: Track and calculate STT for options
  - **Capital Gains**: Classify gains as short-term or long-term
  - **Tax Reporting**: Generate tax reports for filing
- **Legal Documentation**:
  - **Terms of Service**: Clear terms for automated trading
  - **Risk Disclosures**: Comprehensive risk disclosure statements
  - **User Agreements**: Legal agreements for bot usage
  - **Data Privacy**: GDPR/Data Protection compliance

### 11.2 Advanced Technical Indicators & Analysis

- **Greeks Management**:
  - **Delta Monitoring**: Track and hedge delta exposure
  - **Gamma Risk**: Monitor gamma for large position sizes
  - **Theta Decay**: Account for time decay in position management
  - **Vega Sensitivity**: Monitor volatility sensitivity
  - **Rho Impact**: Consider interest rate sensitivity
- **Advanced Technical Analysis**:
  - **Volume Profile**: Analyze volume at different price levels
  - **Market Profile**: Use market profile for better entries
  - **Order Flow Analysis**: Analyze bid-ask spreads and order flow
  - **Support/Resistance Levels**: Dynamic S/R level calculation
  - **Fibonacci Retracements**: Automated Fibonacci level detection
- **Sentiment Analysis**:
  - **Put-Call Ratio**: Monitor PCR for market sentiment
  - **Open Interest Analysis**: Track OI changes for trend confirmation
  - **Implied Volatility**: Compare IV with historical volatility
  - **Skew Analysis**: Monitor volatility skew patterns

### 11.3 Portfolio Management & Diversification

- **Portfolio Construction**:
  - **Asset Allocation**: Diversify across different underlyings
  - **Correlation Analysis**: Avoid highly correlated positions
  - **Sector Diversification**: Spread risk across sectors
  - **Expiry Diversification**: Mix different expiry dates
- **Portfolio Risk Metrics**:
  - **Value at Risk (VaR)**: Calculate portfolio VaR
  - **Maximum Drawdown**: Track maximum portfolio drawdown
  - **Sharpe Ratio**: Monitor risk-adjusted returns
  - **Beta Calculation**: Measure portfolio beta vs market
- **Rebalancing Logic**:
  - **Time-based Rebalancing**: Rebalance at fixed intervals
  - **Threshold-based Rebalancing**: Rebalance when allocations drift
  - **Volatility-based Rebalancing**: Adjust based on market volatility
  - **Performance-based Rebalancing**: Adjust based on strategy performance

### 11.4 Advanced Order Types & Execution

- **Smart Order Routing**:
  - **Iceberg Orders**: Break large orders into smaller chunks
  - **TWAP Orders**: Time-weighted average price execution
  - **VWAP Orders**: Volume-weighted average price execution
  - **Implementation Shortfall**: Minimize market impact
- **Advanced Order Types**:
  - **Bracket Orders**: Automatic profit booking and stop loss
  - **Cover Orders**: Orders with stop loss
  - **After Market Orders**: Orders for next day execution
  - **Good Till Date Orders**: Orders valid for specific period
- **Execution Algorithms**:
  - **Slippage Minimization**: Minimize execution slippage
  - **Market Impact Reduction**: Reduce market impact of large orders
  - **Timing Optimization**: Optimize order timing
  - **Liquidity Seeking**: Find best execution venues

### 11.5 Backtesting & Strategy Validation

- **Comprehensive Backtesting Framework**:
  - **Historical Data Validation**: Ensure data quality for backtesting
  - **Strategy Parameter Optimization**: Optimize strategy parameters
  - **Walk-forward Analysis**: Test strategy robustness over time
  - **Monte Carlo Simulation**: Test strategy under various scenarios
- **Performance Metrics**:
  - **Total Return**: Calculate total strategy returns
  - **Annualized Return**: Calculate annualized returns
  - **Volatility**: Measure strategy volatility
  - **Maximum Drawdown**: Track maximum losses
  - **Win Rate**: Calculate percentage of winning trades
  - **Profit Factor**: Ratio of gross profit to gross loss
- **Strategy Validation**:
  - **Out-of-Sample Testing**: Test on unseen data
  - **Cross-Validation**: Validate across different time periods
  - **Stress Testing**: Test under extreme market conditions
  - **Regime Testing**: Test across different market regimes

### 11.6 Machine Learning & AI Integration

- **Predictive Models**:
  - **Price Prediction**: ML models for price direction
  - **Volatility Forecasting**: Predict future volatility
  - **Volume Prediction**: Forecast trading volume
  - **Sentiment Analysis**: Analyze market sentiment
- **Feature Engineering**:
  - **Technical Indicators**: Create ML features from technical indicators
  - **Market Microstructure**: Use order book data as features
  - **Economic Indicators**: Incorporate economic data
  - **News Sentiment**: Use news sentiment as features
- **Model Management**:
  - **Model Training**: Regular model retraining
  - **Model Validation**: Validate model performance
  - **Model Deployment**: Deploy models in production
  - **Model Monitoring**: Monitor model performance in real-time

### 11.7 Multi-Broker Support & Redundancy

- **Broker Integration**:
  - **Multiple Broker Support**: Support for multiple brokers
  - **Broker Comparison**: Compare execution quality across brokers
  - **Best Execution**: Route orders to best broker
  - **Broker Failover**: Switch brokers on failure
- **Redundancy Systems**:
  - **Data Redundancy**: Multiple data sources
  - **Execution Redundancy**: Multiple execution paths
  - **System Redundancy**: Backup systems
  - **Network Redundancy**: Multiple network connections

### 11.8 Advanced Risk Management

- **Real-time Risk Monitoring**:
  - **Position Risk**: Monitor individual position risks
  - **Portfolio Risk**: Monitor overall portfolio risk
  - **Market Risk**: Monitor market-wide risks
  - **Liquidity Risk**: Monitor liquidity risks
- **Stress Testing**:
  - **Historical Stress Tests**: Test against historical scenarios
  - **Hypothetical Stress Tests**: Test against hypothetical scenarios
  - **Monte Carlo Stress Tests**: Random scenario testing
  - **Regime-based Stress Tests**: Test across market regimes
- **Risk Limits & Controls**:
  - **Position Limits**: Set limits on individual positions
  - **Portfolio Limits**: Set limits on overall portfolio
  - **Loss Limits**: Set limits on losses
  - **Volatility Limits**: Set limits on portfolio volatility

### 11.9 Performance Analytics & Reporting

- **Advanced Analytics**:
  - **Attribution Analysis**: Analyze performance attribution
  - **Risk-adjusted Returns**: Calculate risk-adjusted metrics
  - **Benchmark Comparison**: Compare against benchmarks
  - **Peer Comparison**: Compare against similar strategies
- **Reporting Systems**:
  - **Real-time Dashboards**: Live performance dashboards
  - **Daily Reports**: Automated daily performance reports
  - **Monthly Reports**: Comprehensive monthly reports
  - **Custom Reports**: User-defined custom reports
- **Data Visualization**:
  - **Performance Charts**: Visualize performance metrics
  - **Risk Charts**: Visualize risk metrics
  - **Correlation Charts**: Visualize correlations
  - **Distribution Charts**: Visualize return distributions

### 11.10 System Architecture & Scalability

- **Microservices Architecture**:
  - **Data Service**: Dedicated data management service
  - **Strategy Service**: Dedicated strategy execution service
  - **Risk Service**: Dedicated risk management service
  - **Order Service**: Dedicated order management service
- **Scalability Considerations**:
  - **Horizontal Scaling**: Scale across multiple servers
  - **Load Balancing**: Distribute load across servers
  - **File Storage Optimization**: Use compression and efficient file I/O for high throughput
  - **Caching Strategy**: Implement efficient caching
- **High Availability**:
  - **Fault Tolerance**: Handle system failures gracefully
  - **Disaster Recovery**: Recover from disasters
  - **Backup Systems**: Maintain backup systems
  - **Monitoring**: Comprehensive system monitoring

### 11.11 Security & Access Control

- **Security Measures**:
  - **Encryption**: Encrypt sensitive data
  - **Access Control**: Implement role-based access control
  - **Audit Logging**: Log all system access
  - **Intrusion Detection**: Detect security breaches
- **API Security**:
  - **Rate Limiting**: Limit API access rates
  - **Authentication**: Secure API authentication
  - **Authorization**: Control API access permissions
  - **Input Validation**: Validate all API inputs
- **Data Security**:
  - **Data Encryption**: Encrypt data at rest and in transit
  - **Data Backup**: Secure data backup procedures
  - **Data Retention**: Implement data retention policies
  - **Data Privacy**: Protect user data privacy

### 11.12 Integration & Connectivity

- **External Data Sources**:
  - **Economic Data**: Integrate economic indicators
  - **News Feeds**: Integrate news sentiment data
  - **Social Media**: Integrate social media sentiment
  - **Alternative Data**: Integrate alternative data sources
- **Third-party Integrations**:
  - **Analytics Platforms**: Integrate with analytics platforms
  - **Reporting Tools**: Integrate with reporting tools
  - **Risk Management Systems**: Integrate with risk systems
  - **Compliance Systems**: Integrate with compliance systems
- **API Management**:
  - **API Gateway**: Centralized API management
  - **API Versioning**: Manage API versions
  - **API Documentation**: Comprehensive API documentation
  - **API Testing**: Automated API testing

## Implementation Checklist - Additional Critical Components

### Regulatory & Compliance

- [ ] SEBI compliance framework
- [ ] Tax calculation and reporting
- [ ] Legal documentation
- [ ] Audit trail maintenance
- [ ] Risk disclosure systems

### Advanced Analytics

- [ ] Greeks management system
- [ ] Advanced technical indicators
- [ ] Sentiment analysis
- [ ] Portfolio risk metrics
- [ ] Performance attribution

### Machine Learning & AI

- [ ] Predictive models
- [ ] Feature engineering
- [ ] Model management
- [ ] Real-time inference
- [ ] Model monitoring

### System Architecture

- [ ] Microservices architecture
- [ ] High availability design
- [ ] Scalability planning
- [ ] Security framework
- [ ] Monitoring and alerting

### Advanced Trading Features

- [ ] Smart order routing
- [ ] Advanced order types
- [ ] Execution algorithms
- [ ] Multi-broker support
- [ ] Redundancy systems

### Backtesting & Validation

- [ ] Comprehensive backtesting
- [ ] Strategy optimization
- [ ] Performance metrics
- [ ] Stress testing
- [ ] Walk-forward analysis

This enhanced version provides comprehensive guidance for implementation with intelligent decision-making logic and smart operations that adapt to market conditions while maintaining safety and preventing system failures.

---

## 12. Rust + Angel One API Implementation Gap Analysis

### 12.9 Rust Runtime Architecture

- **Crates**: `tokio` (runtime), `reqwest` (REST), `tokio-tungstenite` (WS), `serde/serde_json` (data), `thiserror` (errors), `tracing` (logging)
- **Broker Abstraction**: Define a `Broker` trait with methods for auth, quotes, order placement, positions, and WS subscriptions. Implement `AngelOneBroker` for SmartAPI.
- **Tasks (async services)**:
  - **Auth Service**: Manages session (`jwtToken`, `feedToken`), refresh, and secure storage
  - **Market Data Service**: REST polling + WS streaming, publishes normalized ticks and candles
  - **Strategy Service**: Consumes signals, performs MTF analysis, generates trade intents
  - **Risk Service**: Enforces circuit breakers, daily loss limits, kill-switch
  - **Order Service**: Places/modifies/cancels orders, idempotency, fill verification
  - **Persistence Service**: Writes JSON files, handles rotation and atomic writes
- **Communication**: Use `tokio::sync::mpsc` channels for events; define typed messages for safety
- **State & Time**: Keep clocks NTP-synced; store timestamps in UTC, include exchange ts when available
- **Testing**: Unit tests for indicators, integration tests with mocked SmartAPI, replay engine for WS data

### 12.1 API Integration

- **Angel One API Mapping**: Document SmartAPI endpoints for auth, quotes, orders, positions, and WebSocket; note Angel One’s endpoints, authentication, and data formats.
- **Rust SDK**: No official Rust SDK for Angel One. You must implement REST and WebSocket clients using crates like `reqwest` (HTTP) and `tokio-tungstenite` (WebSocket).
- **Authentication**: Angel One uses OAuth2 and session tokens. Update login/session logic accordingly.

### 12.2 Rust-Specific Architecture

- **Async Design**: Use Rust’s async ecosystem (`tokio`, `async-std`) for concurrent data fetching, order placement, and real-time monitoring.
- **Error Handling**: Refactor all error handling to use Rust’s `Result` and `Option` types, with robust error propagation and recovery.
- **Data Storage**: Use `serde_json` for JSON file management. Implement efficient file rotation and retention logic for tick/timeframe data.
- **Testing**: Leverage Rust’s built-in test framework for unit and integration tests, especially for trading logic and API wrappers.

### 12.3 Angel One API Limitations

- **Rate Limits**: Validate current SmartAPI limits; update request queuing, batching, and backoff strategies.
- **Order Types**: Confirm support for all required order types (market, limit, bracket, cover, etc.) and execution features.
- **WebSocket Features**: Ensure Angel One’s WebSocket provides all necessary real-time data (LTP, option chain, order status).

### 12.4 Regulatory & Compliance

- **SEBI Compliance**: Implement position limits, margin checks, and audit trails in Rust. Angel One may provide some compliance data, but reporting logic must be custom-built.
- **Tax & Legal**: Build P&L, STT, and tax reporting modules in Rust.

### 12.5 Advanced Features

- **Greeks & Analytics**: Rust ecosystem has limited options for options Greeks and advanced analytics. You may need to port Python logic or implement from scratch.
- **Machine Learning**: ML libraries in Rust are less mature. For predictive models, consider using Python microservices or FFI if needed.
- **Visualization**: For dashboards and charts, use external tools or web integrations.

### 12.6 Monitoring & Notification

- **Logging**: Use crates like `log` and `env_logger` for system and error logs.
- **Notification**: Implement email/SMS/push notifications using Rust crates or external services.

### 12.7 Multi-Broker & Redundancy

- **Broker Abstraction**: Design a trait-based abstraction for broker APIs to support multi-broker and failover logic.
- **Redundancy**: Implement backup data sources and execution paths.

### 12.8 Documentation & API Differences

- **Document Angel One API endpoints** and data models for future maintainability.
- **Update all code references** and examples to Angel One API.

---

**Summary:**  
This gap analysis highlights the key areas to address for a robust Rust + Angel One option trading bot.  
Prioritize API mapping, async architecture, error handling, compliance, and advanced analytics for production readiness.

## Terminology Consistency

- Use `Angel One SmartAPI` consistently throughout
- Use `jwtToken` (REST) and `feedToken` (WebSocket) for auth
- Refer to `SmartAPI` for endpoints, quotes, and order placement
- Rust-specific: use `tokio`, `reqwest`, `tokio-tungstenite`, `serde`, `tracing`

## 13. Go-Live Checklist & Production Readiness

### 13.1 Broker, Legal, and TOS

- Confirm SmartAPI automated trading is permitted for your account; sign API agreements
- Respect RMS constraints: freeze quantity, price bands, lot multiples, min tick
- Validate allowed order types (MIS/CNC/NRML), intraday vs carry-forward rules
- Keep instrument master in sync; handle symbol changes and contract rollovers

### 13.2 Execution Protections

- Price protection: max slippage fences; reject fills beyond thresholds
- Quote-depth-aware limit pricing; **ALWAYS use LIMIT orders** (never MARKET orders for safety); allow IOC/FOK when needed
- Per-second and per-minute order throttling; backoff on rejects
- Persist idempotency/intent hashes to avoid duplicates across restarts

### 13.3 Reconciliation & Charges

- EOD reconcile orders, trades, positions, and funds with broker reports
- Break-glass flow for mismatches: flag, halt trading, require manual ACK
- Full fees model: brokerage, STT, GST, exchange/SEBI fees, stamp duty, DP
- P&L parity: net of all charges; variance thresholds and alerts

### 13.4 Observability

- Metrics: API latency, error rate, WS reconnects, order rejects, slippage, P&L, drawdown
- Structured logs with trace IDs and correlation between order intent → order → trade
- Alerts: email/pager thresholds for token expiry, WS down, reject spikes, loss limits
- Dashboards: live positions, exposure, margin, risk states, health

### 13.5 Runbooks

- Incident response steps for common failures (auth, WS, rejects, data gaps)
- Manual kill‑switch and flatten procedure; who/when gets paged
- Market halt or extraordinary volatility procedure
- Restart/recovery steps and validation checklist

### 13.6 Resilience & Recovery

- WS heartbeat, jittered reconnects, exponential backoff; cap retries with circuit breaker
- REST retry policy with idempotency; fast‑fail on validation errors
- Persistent queues/backpressure to avoid data loss under load
- Crash‑safe state: persist open intents, orders, positions; rebuild on restart

### 13.7 Instrument & Calendar Upkeep

- Auto-refresh instrument master daily; verify tokens/symbols for new weekly series
- Validate lot sizes, tick sizes, trading status; purge expired contracts
- Sync NSE holiday calendar and any special trading sessions

### 13.8 Time & Clock Discipline

- NTP-sync host clock; detect clock skew vs exchange timestamps
- Store UTC in files/logs; display IST in UI
- **ALWAYS use exchange timestamps when available** (fallback to local time only if unavailable)

### 13.9 Risk Controls (Hard Caps)

- Per-trade, per-day, and portfolio loss limits; consecutive‑loss lockout
- Margin pre-checks (SPAN/Exposure) before order; pre‑trade notional caps
- Position concentration and correlated exposure limits
- VIX/volatility circuit breakers tied to position sizing and halts

### 13.10 Security & Compliance

- Secrets in OS vault; rotation policy; no secrets in logs
- Role-based access, least privilege, audit log immutability and retention
- Data retention policy for ticks/bars/logs per regulation and business need

### 13.11 Testing Strategy

- Replay/paper mode using recorded WS data; parity checks with live logic
- Integration tests against SmartAPI sandbox or throttled prod; mock external deps
- Soak tests and chaos tests (network blips, WS drops, partial outages)
- Backtest vs live: include liquidity, spread, and fees; slippage modeling

### 13.12 Operational Environment

- Reliable host: UPS/VPS, redundant internet paths
- Process supervision: service manager, watchdog, auto‑restart with backoff
- Configuration gating: clear LIVE vs PAPER modes, feature flags
- Versioned deploys with rollback and change logs

### 13.13 Paper Trading & Simulation Mode

#### 13.13.1 Paper Trading Configuration

- **Mode Selection**:

  - Environment variable: `TRADING_MODE=PAPER` or `TRADING_MODE=LIVE`
  - Configuration file flag: `mode = "paper"` or `mode = "live"`
  - Command-line argument: `--mode paper` or `--mode live`
  - Default to PAPER mode for safety (require explicit LIVE activation)

- **Paper Trading Behavior**:
  - Use same strategy logic and signal generation as live trading
  - Connect to real SmartAPI for market data (prices, option chain, VIX)
  - Simulate order placement without sending to broker
  - Track virtual positions, P&L, and account balance
  - Log all simulated trades with timestamps and prices

#### 13.13.2 Order Fill Simulation

- **Fill Price Simulation**:

  - **Market Orders**: Fill at current ask (buy) or bid (sell) price
  - **Limit Orders**: Fill only when market price crosses limit price
  - **Slippage Modeling**: Add realistic slippage (0.1-0.5% for options)
  - **Partial Fills**: Simulate partial fills for large orders (>10% of OI)

- **Fill Timing Simulation**:

  - **Immediate Fill**: High liquidity strikes (OI > 10,000)
  - **Delayed Fill**: 2-5 seconds for moderate liquidity (OI 1,000-10,000)
  - **Rejection**: Reject orders for low liquidity strikes (OI < 500)
  - **Market Impact**: Increase slippage for position sizes >100 lots

- **Realistic Constraints**:
  - Honor lot size multiples and tick size rules
  - Respect RMS freeze quantity limits
  - Simulate margin requirements and checks
  - Enforce price band restrictions (±20% circuit limits)

#### 13.13.3 Paper Trading Validation

- **Performance Comparison**:

  - Compare paper trading results vs backtest results
  - Acceptable variance: ±10% due to real-time execution differences
  - Track slippage, commissions, and fees in paper mode
  - Validate order logic, timing, and risk management

- **Data Validation**:

  - Ensure real-time data matches paper trading assumptions
  - Verify option chain data quality and completeness
  - Check for data gaps or delays in paper mode
  - Log any discrepancies between expected and actual data

- **Transition to Live**:
  - Minimum 2 weeks successful paper trading (positive P&L, no errors)
  - Paper trading Sharpe ratio > 1.5
  - Maximum drawdown < 10% in paper mode
  - No critical errors or system crashes during paper trading
  - Manual review and approval required before live trading

#### 13.13.4 Parallel Paper Trading (Alongside Live)

- **Dual Mode Operation**:

  - Run paper trading engine parallel to live trading
  - Use same market data and signals for both
  - Compare live vs paper performance daily
  - Detect divergence between live and paper results

- **Divergence Alerts**:
  - Alert if paper P&L differs from live by >20%
  - Investigate order execution, slippage, or data issues
  - Flag for manual review and strategy adjustment
  - Pause live trading if unexplained divergence >30%

### 13.14 Disaster Recovery & Emergency Procedures

#### 13.14.1 System Crash Recovery

- **Crash Detection**:

  - Process watchdog detects bot process termination
  - Health check endpoint stops responding (>30 seconds)
  - WebSocket disconnection without auto-reconnect
  - Log file shows fatal error or panic

- **Recovery Steps**:

  1. **Position Reconstruction**: Query broker API for current open positions
  2. **Order Reconciliation**: Fetch all orders placed today from broker
  3. **State Restoration**: Load last saved state from persistent storage
  4. **Balance Verification**: Verify account balance and margin availability
  5. **Data Sync**: Download missing tick data and reconstruct bars
  6. **Validation**: Ensure reconstructed state matches broker state
  7. **Resume**: Resume trading only after full validation passes

- **Orphan Order Detection**:
  - Compare system's order log with broker's order book
  - Identify orders placed by bot but missing from internal state
  - Identify positions held by broker but missing from bot state
  - **Action**: Cancel orphan pending orders, reconcile positions
  - **Alert**: Notify operator of any orphan orders/positions found

#### 13.14.2 Network Outage Recovery

- **Short Outage (<5 minutes)**:

  - WebSocket reconnects automatically with exponential backoff
  - Resume position monitoring and price updates
  - No position changes needed if positions are safe
  - Log outage duration and resume time

- **Medium Outage (5-30 minutes)**:

  - Assess market conditions after reconnection
  - Check VIX for any spikes during outage
  - Review open positions for adverse movements
  - Consider closing positions if significant movement against us
  - Skip new entries for 15 minutes after reconnection (market assessment)

- **Long Outage (>30 minutes)**:
  - Treat as disaster scenario
  - Close all open positions immediately after reconnection
  - Halt trading for remainder of day
  - Perform full system validation and reconciliation
  - Generate incident report for review

#### 13.14.3 JSON File Corruption Recovery

- **Corruption Detection**:

  - JSON parse errors when loading files
  - Missing or truncated data files
  - Checksum validation failures
  - Timestamp inconsistencies or gaps

- **Recovery Procedure**:

  - Restore from previous day's backup (created at EOD)
  - Download missing historical data from SmartAPI
  - Reconstruct missing bars from raw tick files if available
  - Validate restored data integrity before resuming
  - If unable to restore: halt trading, manual intervention required

- **Data Backup Strategy**:
  - Incremental backups every hour during trading
  - Full backup at end of day (3:45 PM)
  - Retain 7 days of backups locally
  - Upload daily backups to cloud/external storage
  - Test restore procedure weekly

#### 13.14.4 Wrong Order Execution (Fat Finger)

- **Prevention Measures**:

  - Pre-trade validation: maximum quantity per order (100 lots)
  - Price sanity check: reject orders >20% from current LTP
  - Maximum order value: ₹5,00,000 per single order
  - Duplicate order prevention: check for similar orders in last 60 seconds
  - Confirmation delays: 2-second delay for large orders (>50 lots)

- **Detection**:

  - Order placed with quantity >10x normal position size
  - Order price significantly different from market (>10%)
  - Multiple identical orders placed within seconds
  - Position size exceeds account risk limits

- **Emergency Response**:
  1. **Immediate Action**: Cancel order if still pending (within 5 seconds)
  2. **If Filled**: Assess damage, consider immediate exit or hedge
  3. **Risk Assessment**: Calculate potential loss and margin impact
  4. **Decision Tree**:
     - Loss <1% of account: Monitor and exit strategically
     - Loss 1-3% of account: Exit immediately at market
     - Loss >3% of account: Consider hedging with opposite position
  5. **Documentation**: Log incident with full details for review
  6. **Prevention**: Update validation rules to prevent recurrence

#### 13.14.5 Emergency Contact Tree

- **Level 1 Alerts** (automated, no human intervention needed):

  - Token expiry warnings (4 hours before)
  - Position P&L updates
  - Normal trading alerts
  - System health checks

- **Level 2 Alerts** (operator notification, non-urgent):

  - Daily loss approaching 2%
  - VIX elevated (25-30)
  - Single position loss >1%
  - Data quality issues
  - Order reject rate >10%

- **Level 3 Alerts** (immediate operator action required):

  - Daily loss limit reached (3%)
  - System crash or fatal error
  - Network outage >5 minutes
  - Token expired during trading hours
  - Wrong order execution detected
  - VIX >30 (extreme volatility)

- **Level 4 Alerts** (emergency, page on-call):

  - Account loss >5% in single day
  - Multiple system failures
  - Broker API completely down
  - Data corruption affecting positions
  - Regulatory violation detected

- **Contact Methods**:
  - Email: For Level 1-2 alerts
  - SMS: For Level 3 alerts
  - Phone call: For Level 4 emergencies
  - Slack/Telegram: Real-time operational updates

#### 13.14.6 Manual Intervention Procedures

- **Kill-Switch Activation**:
  - **Trigger Methods**:
    - Keyboard shortcut: `CTRL+C` (graceful shutdown)
    - CLI command: `bot kill-switch --confirm`
    - Emergency file: Create `EMERGENCY_STOP` file in bot directory
    - API endpoint: `POST /api/emergency-stop` with auth token
- **Kill-Switch Actions** (execute in order):

  1. Stop generating new signals and trade intents
  2. Cancel all pending orders immediately
  3. Close all open positions at market price (within 30 seconds)
  4. Disconnect WebSocket and stop data collection
  5. Save current state and generate incident log
  6. Send emergency notification to operator
  7. Halt trading engine completely

- **Resume After Kill-Switch**:
  1. Operator investigates root cause of kill-switch trigger
  2. Operator validates system is safe to resume
  3. Operator runs health check: `bot health-check --full`
  4. Operator explicitly confirms: `bot resume --confirm-safe --reason "issue resolved"`
  5. System performs full validation before accepting trades
  6. Resume trading in limited mode (50% position sizes for 1 hour)
  7. Gradually return to normal operations if no issues

### 13.15 Performance SLAs & System Requirements

#### 13.15.1 Latency Requirements

- **Order Placement Latency**:

  - **Target**: Order placement round-trip <300ms (p50)
  - **Maximum**: Order placement round-trip <500ms (p99)
  - **Components**:
    - Signal generation to order intent: <50ms
    - Order validation and safety checks: <50ms
    - Network to broker API: <150ms
    - Broker processing and confirmation: <100ms
    - Order status update: <50ms

- **Data Processing Latency**:

  - WebSocket tick to internal update: <10ms
  - 1-minute bar construction: <50ms after bar close
  - Indicator calculation refresh: <100ms
  - Position P&L update: <50ms

- **System Response Times**:
  - Health check endpoint: <100ms
  - Dashboard data refresh: <200ms
  - Manual command execution: <1 second
  - Emergency stop execution: <500ms

#### 13.15.2 Throughput Requirements

- **Market Data Processing**:

  - WebSocket tick processing: >5,000 ticks/second capacity
  - Simultaneous symbol subscriptions: 50-100 symbols
  - Option chain refresh rate: Full chain every 10 minutes
  - Top-of-book strikes refresh: Every 1-2 minutes

- **Order Processing**:
  - Maximum concurrent orders: 10 orders in-flight
  - Order queue capacity: 100 orders (should never exceed)
  - Order burst handling: 20 orders in 10 seconds
  - Order validation rate: >100 validations/second

#### 13.15.3 Resource Utilization Limits

- **CPU Usage**:

  - Normal operation: <25% of single core (leave headroom for spikes)
  - During market hours: <40% of single core
  - Indicator calculation: <10% of single core
  - **Alert**: If CPU >60% sustained for 5 minutes
  - **Action**: Reduce indicator calculations or symbol subscriptions

- **Memory Usage**:

  - Baseline (no data): <100 MB
  - With 2 days tick data: <500 MB
  - With 3 months historical bars: <1 GB
  - Maximum allowed: 2 GB
  - **Alert**: If memory >1.5 GB
  - **Action**: Clear old tick data, reduce retention

- **Network Bandwidth**:

  - WebSocket data: 100-500 KB/s during active trading
  - REST API calls: 10-50 KB/s average
  - Historical data download: Burst to 1 MB/s, throttle to stay within limits
  - **Alert**: If bandwidth >1 MB/s sustained

- **Disk I/O**:
  - Tick data writes: <10 MB/minute during trading
  - Log file writes: <5 MB/minute
  - Disk space required: 10 GB minimum, 50 GB recommended
  - **Alert**: If disk space <5 GB remaining
  - **Action**: Archive old data, clean temporary files

#### 13.15.4 Availability & Uptime

- **Target Uptime**: 99.5% during market hours (9:15 AM - 3:30 PM)

  - Allowed downtime: ~1.9 minutes per trading day
  - Maximum single outage: <60 seconds
  - Recovery time objective: <60 seconds

- **System Health Monitoring**:

  - Health check every 30 seconds
  - Process heartbeat every 10 seconds
  - WebSocket connection status: continuous monitoring
  - Data freshness: alert if no new data for 60 seconds

- **Auto-Recovery Mechanisms**:
  - Process watchdog restarts on crash (max 3 attempts)
  - WebSocket auto-reconnect with backoff
  - API retry with exponential backoff
  - Automatic state restoration from last checkpoint

#### 13.15.5 Data Quality SLAs

- **Price Data Accuracy**:

  - Tick data timestamp accuracy: ±1 second
  - Price value accuracy: Exact match with broker (no interpolation)
  - Missing data tolerance: <0.1% of expected ticks
  - **Alert**: If missing data >1% in any 5-minute window

- **Bar Construction Accuracy**:

  - 1-minute bars: OHLCV calculated from exact ticks
  - Higher timeframe bars: Aggregated from 1-minute bars
  - Bar timestamp: Aligned to exact bar boundaries
  - **Validation**: Random spot-check against broker historical data

- **Indicator Calculation Accuracy**:
  - ADX, RSI, EMA: Match reference implementations (TA-Lib)
  - Tolerance: ±0.01% for floating-point calculations
  - Validation: Backtest against known datasets

#### 13.15.6 Error Rate Limits

- **Order Rejection Rate**:

  - Target: <5% order rejection rate
  - Maximum: <10% order rejection rate
  - **Alert**: If rejection rate >10% in 15 minutes
  - **Action**: Pause trading, investigate order validation logic

- **API Error Rate**:

  - Target: <1% API call failure rate
  - Maximum: <5% API call failure rate
  - **Alert**: If error rate >5% in 5 minutes
  - **Action**: Check broker API status, network connectivity

- **Data Gap Rate**:
  - Target: <0.1% missing data points
  - Maximum: <1% missing data points
  - **Alert**: If gaps >1% in any 10-minute window
  - **Action**: Check WebSocket connection, request missing data

### 13.16 Deployment & Versioning

#### 13.16.1 Build & Release Process

- **Version Numbering**: Semantic versioning (MAJOR.MINOR.PATCH)

  - MAJOR: Breaking changes, major strategy overhaul
  - MINOR: New features, non-breaking strategy updates
  - PATCH: Bug fixes, performance improvements

- **Build Pipeline**:

  1. **Code Commit**: Developer commits to feature branch
  2. **Automated Tests**: Run unit tests, integration tests (must pass)
  3. **Code Review**: Peer review required before merge
  4. **Merge to Main**: Merge to main branch after approval
  5. **Build Binary**: Compile optimized release binary
  6. **Run Backtests**: Execute strategy backtest on historical data
  7. **Tag Release**: Tag version in git (e.g., v1.2.3)
  8. **Generate Changelog**: Auto-generate changelog from commits
  9. **Package**: Create deployment package with binary + config

- **Pre-Deployment Checklist**:
  - [ ] All tests pass (unit, integration, backtest)
  - [ ] Code review approved by 2+ reviewers
  - [ ] Configuration files updated for new version
  - [ ] JSON schema migrations prepared (if needed)
  - [ ] Rollback plan documented
  - [ ] Deployment scheduled during non-market hours
  - [ ] Backup of current production system created

#### 13.16.2 Deployment Strategy (Blue-Green)

- **Blue-Green Setup**:

  - **Blue Environment**: Current production system
  - **Green Environment**: New version staging environment
  - Both environments share same configuration and data sources

- **Deployment Steps**:

  1. **Deploy to Green**: Deploy new version to green environment
  2. **Validation**: Run full health check on green environment
  3. **Paper Trading**: Run green in paper mode for 1 hour (non-market hours)
  4. **Switch**: Update routing to point to green environment
  5. **Monitor**: Closely monitor green for 30 minutes
  6. **Promote**: If successful, green becomes new blue (production)
  7. **Retain Blue**: Keep old blue as backup for 24 hours

- **Rollback Procedure**:
  - **Trigger Conditions**:
    - Fatal errors or crashes in first 30 minutes
    - Order rejection rate >20%
    - Data processing errors
    - Performance degradation >50%
  - **Rollback Steps**:
    1. Switch routing back to old blue environment (takes <60 seconds)
    2. Verify blue is functioning correctly
    3. Investigate issues with green offline
    4. Notify team of rollback and reason

#### 13.16.3 Configuration Management

- **Environment-Specific Configs**:

  - `config.dev.toml`: Development settings (paper mode default)
  - `config.staging.toml`: Staging settings (paper mode forced)
  - `config.prod.toml`: Production settings (live mode allowed)

- **Configuration Versioning**:

  - Store configs in version control (git)
  - Tag config versions matching software versions
  - Track all config changes with commit messages
  - Require review for production config changes

- **Configuration Hot-Reload**:

  - Monitor config file for changes
  - Reload non-critical settings without restart:
    - Position sizing parameters
    - Risk thresholds (VIX limits, loss limits)
    - Strategy parameters (ADX threshold, RSI levels)
  - **Require restart** for critical settings:
    - Trading mode (PAPER/LIVE)
    - Broker credentials
    - System architecture changes

- **Configuration Drift Detection**:
  - Compare running config with expected config (from git)
  - Alert if production config differs from version control
  - Generate drift report daily
  - **Action**: Sync configs or document exceptions

#### 13.16.4 Schema Migrations

- **JSON Schema Versioning**:

  - Include schema version in each JSON file: `"schema_version": "1.2"`
  - Maintain backward compatibility for 2 versions
  - Support forward migration (old to new schema)

- **Migration Process**:

  1. Detect old schema version in existing files
  2. Run migration script to convert to new schema
  3. Validate converted data integrity
  4. Create backup of original files (pre-migration)
  5. Atomic swap: rename old files, write new files
  6. Delete backups after 7 days if no issues

- **Migration Testing**:
  - Test migrations on sample data before production
  - Verify round-trip conversion (old→new→old)
  - Ensure performance: migrations should complete in <5 minutes

#### 13.16.5 Monitoring Post-Deployment

- **First 30 Minutes** (critical monitoring period):

  - Watch logs in real-time for errors
  - Monitor order placement and execution
  - Track latency and throughput metrics
  - Verify position tracking accuracy
  - Check WebSocket connection stability

- **First 24 Hours** (enhanced monitoring):

  - Compare performance vs previous version
  - Monitor for memory leaks or resource growth
  - Track error rates and rejection rates
  - Validate P&L calculation accuracy
  - Check for any behavioral changes

- **First Week** (performance validation):
  - Compare strategy performance vs backtest expectations
  - Monitor for any edge cases or bugs
  - Gather user feedback (if applicable)
  - Assess overall system stability
  - Decide whether to keep or rollback

## 14. Operational Interface & Control System

### 14.1 Command-Line Interface (CLI)

#### 14.1.1 Bot Control Commands

- **Start Trading**:

  - `bot start --mode live`: Start in live trading mode
  - `bot start --mode paper`: Start in paper trading mode
  - `bot start --config prod.toml`: Start with specific config file
  - `bot start --dry-run`: Start but don't place any orders (validation only)

- **Stop Trading**:

  - `bot stop`: Graceful shutdown (close positions, save state)
  - `bot stop --immediate`: Stop without closing positions (emergency only)
  - `bot stop --no-exit`: Stop but leave positions open

- **Status & Monitoring**:

  - `bot status`: Show current status (running/stopped/error)
  - `bot positions`: Display all open positions with P&L
  - `bot orders`: Show recent orders and their status
  - `bot balance`: Display account balance, margin, available funds
  - `bot performance`: Show today's performance metrics

- **Emergency Controls**:
  - `bot kill-switch --confirm`: Emergency stop + close all positions
  - `bot pause`: Pause new entries, keep monitoring existing positions
  - `bot resume --confirm-safe`: Resume after pause/kill-switch

#### 14.1.2 Configuration Commands

- **Config Management**:

  - `bot config show`: Display current configuration
  - `bot config validate`: Validate config file syntax
  - `bot config reload`: Hot-reload configuration
  - `bot config edit`: Open config in editor

- **Parameter Adjustment**:
  - `bot set position-size 1.5`: Adjust base position size percentage
  - `bot set daily-loss-limit 2.5`: Adjust daily loss limit
  - `bot set max-positions 2`: Change maximum concurrent positions

#### 14.1.3 Data Management Commands

- **Data Operations**:

  - `bot data sync`: Download missing historical data
  - `bot data validate`: Validate data integrity
  - `bot data cleanup`: Remove old data beyond retention period
  - `bot data backup`: Create manual backup

- **Data Inspection**:
  - `bot data gaps`: Show data gap report
  - `bot data stats`: Display data statistics (size, coverage, quality)

#### 14.1.4 System Maintenance Commands

- **Health & Diagnostics**:

  - `bot health-check`: Run full system health check
  - `bot health-check --quick`: Quick health check (<5 seconds)
  - `bot diagnose`: Run diagnostic tests on all components
  - `bot logs --tail 100`: Show last 100 log lines
  - `bot logs --follow`: Follow logs in real-time
  - `bot logs --level error`: Show only error logs

- **Testing & Validation**:
  - `bot test connection`: Test broker API connection
  - `bot test websocket`: Test WebSocket connectivity
  - `bot test order --paper`: Test order placement in paper mode
  - `bot validate strategy`: Validate strategy logic

### 14.2 Emergency Controls & Kill-Switch

#### 14.2.1 Kill-Switch Trigger Mechanisms

- **Manual Triggers**:

  - **Keyboard**: `CTRL+C` sends graceful shutdown signal
  - **Keyboard**: `CTRL+C` (second press) sends emergency kill signal
  - **CLI**: `bot kill-switch --confirm` command
  - **File**: Create `EMERGENCY_STOP` file in bot directory
  - **API**: `POST /api/emergency/kill-switch` with authentication

- **Automated Triggers**:
  - Daily loss limit exceeded (3% default)
  - Account balance drops below critical threshold (80% of starting balance)
  - VIX spike >7 points in 5 minutes
  - Token/session expires during trading hours (unable to refresh)
  - System error rate >50% for 5 minutes
  - Margin utilization >95%
  - Wrong order detected (fat finger scenario)

#### 14.2.2 Kill-Switch Execution Sequence

1. **Immediate Actions** (within 1 second):

   - Set global flag: `EMERGENCY_MODE = true`
   - Stop signal generation engine
   - Stop accepting new trade intents

2. **Order Management** (within 5 seconds):

   - Cancel ALL pending orders (market + limit)
   - Log all cancelled orders with reason
   - Wait for cancellation confirmations

3. **Position Closure** (within 30 seconds):

   - Close all open positions at MARKET price
   - Log each position closure with exit price and P&L
   - Verify all positions closed with broker API
   - Retry up to 3 times if any positions remain open

4. **Data & State Management** (within 60 seconds):

   - Save current system state to disk
   - Generate emergency incident report
   - Backup all data files
   - Close all data connections

5. **Shutdown** (within 90 seconds):
   - Disconnect WebSocket
   - Close REST API clients
   - Send emergency notification to operator
   - Log shutdown completion
   - Exit process

#### 14.2.3 Post-Kill-Switch Procedures

- **Incident Investigation**:

  1. Review incident report and logs
  2. Identify root cause of kill-switch trigger
  3. Assess financial impact (P&L, slippage, fees)
  4. Check for any system errors or bugs
  5. Verify broker reconciliation (all positions closed correctly)

- **System Validation Before Resume**:

  1. Fix root cause issue (code bug, config error, external issue resolved)
  2. Run full health check: `bot health-check --full`
  3. Validate data integrity and freshness
  4. Test connectivity (REST + WebSocket)
  5. Verify account balance and margin
  6. Run paper trading for 15 minutes to validate system

- **Resume Procedure**:
  1. Operator approval required (cannot resume automatically)
  2. Document reason for resume and validation steps taken
  3. Execute: `bot resume --confirm-safe --reason "issue resolved, system validated"`
  4. Start in LIMITED MODE:
     - 50% of normal position sizes
     - Maximum 1 concurrent position
     - Extra conservative risk parameters
  5. Monitor closely for 1 hour
  6. If stable, gradually return to normal: `bot mode normal`

#### 14.2.4 Manual Order Placement Mode

- **Purpose**: Allow operator to manually place orders through bot interface

  - Useful when automated strategy is paused but manual trading needed
  - Provides consistent order interface with safety checks
  - Logs all manual orders for audit trail

- **Manual Mode Activation**:

  - `bot manual-mode enable`: Enable manual order placement
  - Automated signal generation stops
  - Operator can submit orders via CLI or API

- **Manual Order Commands**:

  - `bot order buy-ce NIFTY 23500 --qty 1 --price 150.50`: Buy call option
  - `bot order buy-pe NIFTY 23500 --qty 1 --price 145.25`: Buy put option
  - `bot order close <position-id>`: Close specific position
  - `bot order close-all`: Close all positions

- **Manual Mode Safety**:
  - All safety checks still apply (position limits, margin, price bands)
  - Log operator identity and reason for manual order
  - Require confirmation for large orders (>50 lots)
  - Daily loss limits still enforced

### 14.3 Monitoring Dashboard & Alerts

#### 14.3.1 Real-Time Dashboard Panels

- **System Health Panel**:

  - Bot status: Running/Stopped/Error
  - Uptime: Hours since start
  - Last heartbeat: Seconds ago
  - WebSocket: Connected/Disconnected
  - API status: Healthy/Degraded/Down
  - Data freshness: Last tick received

- **Trading Status Panel**:

  - Trading mode: LIVE/PAPER
  - VIX level: Current value + trend
  - Market hours: Open/Closed + time remaining
  - Signal generation: Active/Paused
  - Circuit breakers: Status of each breaker

- **Positions Panel**:

  - Open positions: Count + list
  - Total exposure: Notional value
  - Net delta: Portfolio delta
  - Unrealized P&L: Real-time per position + total
  - Margin used: Percentage of available margin

- **Performance Panel**:

  - Today's P&L: Absolute + percentage
  - Win rate: Wins / Total trades today
  - Average profit: Per winning trade
  - Average loss: Per losing trade
  - Largest winner: Best trade today
  - Largest loser: Worst trade today
  - Sharpe ratio: Intraday Sharpe

- **Order Flow Panel**:

  - Recent orders: Last 10 orders with status
  - Order success rate: Percentage filled
  - Average fill time: Seconds
  - Rejection count: Today
  - Pending orders: Count + list

- **Risk Metrics Panel**:
  - Daily loss: Current / Limit
  - Consecutive losses: Count
  - Position concentration: By underlying
  - Margin utilization: Percentage
  - Max drawdown: Today's maximum drawdown

#### 14.3.2 Alert Configuration

- **Critical Alerts** (email + SMS + dashboard):

  - Daily loss limit reached
  - Kill-switch triggered
  - System crash or fatal error
  - Token expiry during trading
  - Wrong order detected
  - Network outage >5 minutes

- **Warning Alerts** (email + dashboard):

  - Daily loss approaching limit (>2%)
  - VIX spike >5 points
  - Order rejection rate >10%
  - Margin utilization >70%
  - Data quality issues
  - Position stop-loss hit

- **Info Alerts** (dashboard only):
  - Position opened/closed
  - Target reached
  - Daily performance summary
  - System maintenance completed

### 14.4 Logging & Audit Trail

#### 14.4.1 Log Levels & Categories

- **Log Levels**:

  - ERROR: Fatal errors, crashes, critical failures
  - WARN: Non-fatal issues, degraded performance, risk warnings
  - INFO: Normal operations, trade executions, state changes
  - DEBUG: Detailed diagnostics, trace information (development only)

- **Log Categories**:
  - `AUTH`: Token generation, session management, authentication
  - `DATA`: Market data, WebSocket, tick processing, bar construction
  - `STRATEGY`: Signal generation, indicator calculations, entry/exit logic
  - `RISK`: Risk checks, circuit breakers, position sizing, limits
  - `ORDER`: Order placement, fills, cancellations, rejections
  - `POSITION`: Position tracking, P&L updates, margin
  - `SYSTEM`: Health checks, errors, performance metrics

#### 14.4.2 Audit Trail Requirements

- **Order Audit**:

  - Log every order intent with timestamp, symbol, direction, size, price
  - Log order placement with broker order ID
  - Log order fill with execution price, time, and slippage
  - Log order cancellations with reason
  - Log order rejections with broker reason code

- **Position Audit**:

  - Log position opens with entry price, size, stop-loss, target
  - Log position updates (stop-loss adjustments, partial exits)
  - Log position closes with exit price, reason, P&L
  - Log position duration and holding time

- **Risk Audit**:

  - Log all risk check results (pass/fail with values)
  - Log circuit breaker activations and deactivations
  - Log daily loss limit checks
  - Log margin utilization checks

- **System Audit**:
  - Log all configuration changes
  - Log all operator commands (CLI/API)
  - Log system start/stop with version information
  - Log all error conditions and recovery actions

#### 14.4.3 Log Retention & Archival

- **Active Logs**:

  - Current day logs: Keep in active log file
  - Log rotation: Daily at midnight
  - Log file naming: `bot-YYYY-MM-DD.log`

- **Archive Policy**:

  - Compress logs older than 7 days (gzip)
  - Retain compressed logs for 90 days locally
  - Upload to cloud storage for 1 year retention
  - Delete logs older than 1 year (unless required for audit)

- **Searchability**:
  - Structured logging format (JSON REQUIRED)
  - Indexed by timestamp, category, level
  - Include correlation IDs for order/position tracking
  - Support grep/search across archived logs

## 15. Configuration Reference & Examples

### 15.1 Complete Configuration File Structure

```toml
[system]
# System identification and mode
app_name = "rustro-option-bot"
version = "1.0.0"
environment = "production"  # development, staging, production
trading_mode = "paper"      # paper or live (default paper for safety)

[broker]
# Angel One SmartAPI credentials
name = "angelone"
api_base_url = "https://apiconnect.angelbroking.com"
ws_url = "wss://smartapisocket.angelbroking.com"
client_code = "${SMARTAPI_CLIENT_CODE}"  # From environment variable
# Never store password/TOTP in config - use OS credential manager

[market]
# Market hours and calendar
timezone = "Asia/Kolkata"
market_open = "09:15:00"
market_close = "15:30:00"
pre_market = "09:00:00"
post_market = "16:00:00"
trading_days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
holiday_calendar_url = "https://www.nseindia.com/api/holiday-master"

[data]
# Data management settings
storage_path = "./data"
retention_raw_ticks_days = 2
retention_1m_bars_months = 3
retention_1h_bars_months = 3
retention_daily_bars_years = 1
enable_data_validation = true
max_data_gap_seconds = 300

[websocket]
# WebSocket connection settings
reconnect_delay_ms = 1000
max_reconnect_delay_ms = 30000
max_reconnect_attempts = 10
heartbeat_interval_seconds = 30
subscription_batch_size = 50

[rest_api]
# REST API rate limiting
requests_per_second = 5
requests_per_minute = 100
timeout_seconds = 30
max_retries = 3
backoff_multiplier = 2.0

[strategy]
# Strategy parameters
name = "adx_trend_following"
enabled = true
min_adx_threshold = 25.0
max_adx_threshold = 100.0
rsi_oversold = 40.0
rsi_overbought = 60.0
ema_fast_period = 9
ema_slow_period = 20
volume_multiplier = 1.2

[risk]
# Risk management settings
base_position_size_pct = 2.0          # 2% of account per trade
max_positions = 3                      # Maximum concurrent positions
max_position_per_underlying = 1        # Maximum 1 position per underlying
daily_loss_limit_pct = 3.0            # Stop trading after 3% daily loss
consecutive_loss_limit = 3             # Halt after 3 consecutive losses
margin_utilization_max_pct = 70.0     # Maximum margin usage

[risk.vix_adjustments]
# VIX-based position sizing
vix_low_threshold = 15.0
vix_low_multiplier = 1.25
vix_normal_low = 15.0
vix_normal_high = 20.0
vix_normal_multiplier = 1.0
vix_elevated_threshold = 20.0
vix_elevated_multiplier = 0.75
vix_high_threshold = 25.0
vix_high_multiplier = 0.50
vix_extreme_threshold = 30.0
vix_extreme_multiplier = 0.25

[risk.circuit_breakers]
# Circuit breaker thresholds
enable_vix_breaker = true
vix_pause_threshold = 30.0
vix_spike_threshold = 5.0
vix_spike_window_minutes = 10
enable_flash_spike_breaker = true
flash_spike_pct = 2.0
flash_spike_window_minutes = 5

[entry]
# Entry rules
enable_time_filter = true
no_entry_before = "10:00:00"
no_entry_after = "14:30:00"
min_oi_threshold = 1000
require_volume_confirmation = true
require_trend_alignment = true

[exit]
# Exit rules
stop_loss_pct = 1.0               # 1% stop loss on underlying
target_pct = 3.0                  # 3% target on underlying
enable_trailing_stop = true
breakeven_at_rr = 1.0            # Move to breakeven at 1:1
trailing_start_at_rr = 2.0       # Start trailing at 1:2
trailing_pct = 1.5               # Trail 1.5% below high
exit_before_close_minutes = 30    # Exit all positions 30min before close
exit_before_expiry_days = 3       # Exit all positions 3 days before expiry
max_holding_time_minutes = 120    # Maximum hold time

[orders]
# Order management
default_order_type = "LIMIT"          # LIMIT or MARKET
default_product_type = "MIS"          # MIS, CNC, NRML
limit_price_offset_pct = 0.5          # 0.5% offset from LTP for limit orders
order_timeout_seconds = 60            # Cancel pending orders after 60s
max_slippage_pct = 2.0               # Reject fills with >2% slippage
enable_order_idempotency = true
max_order_value = 500000              # Maximum ₹5 lakh per order
max_order_quantity = 100              # Maximum 100 lots per order

[monitoring]
# Monitoring and alerts
enable_dashboard = true
dashboard_port = 8080
health_check_interval_seconds = 30
log_level = "INFO"                    # ERROR, WARN, INFO, DEBUG
log_format = "json"                   # MUST be json for structured logging
enable_email_alerts = true
enable_sms_alerts = false
alert_email = "operator@example.com"

[performance]
# Performance settings
enable_parallel_processing = true
worker_threads = 4
tick_processing_batch_size = 100
indicator_calculation_interval_ms = 1000
position_update_interval_ms = 500

[backup]
# Backup settings
enable_auto_backup = true
backup_interval_hours = 24
backup_path = "./backups"
backup_retention_days = 7
enable_cloud_backup = false
```

### 15.2 Environment-Specific Configurations

#### 15.2.1 Development Config (`config.dev.toml`)

```
[system]
environment = "development"
trading_mode = "paper"  # ALWAYS paper in dev

[data]
storage_path = "./data/dev"

[monitoring]
log_level = "DEBUG"
enable_email_alerts = false

[risk]
daily_loss_limit_pct = 1.0  # Lower limits for testing
max_positions = 1
```

#### 15.2.2 Production Config (`config.prod.toml`)

```
[system]
environment = "production"
trading_mode = "paper"  # Requires explicit override to "live"

[monitoring]
log_level = "INFO"
enable_email_alerts = true
enable_sms_alerts = true

[backup]
enable_cloud_backup = true
```

### 15.3 Secrets Management

- **Never store in config files**:

  - Broker passwords
  - TOTP secrets
  - API keys
  - Authentication tokens

- **Use OS credential managers**:

  - Windows: Windows Credential Manager
  - Linux: Secret Service API / gnome-keyring
  - macOS: Keychain

- **Environment variables** (secondary option):

  - `SMARTAPI_CLIENT_CODE`
  - `SMARTAPI_PASSWORD` (retrieve from secure vault)
  - `SMARTAPI_TOTP_SECRET` (retrieve from secure vault)

- **Configuration references** (use placeholders):
  - `${ENV_VAR_NAME}` - replaced at runtime
  - Never log actual secret values

### 15.4 Configuration Validation

- **Startup Validation**:

  - Verify all required fields present
  - Check value ranges (percentages 0-100, positive numbers, etc.)
  - Validate file paths exist and writable
  - Ensure trading_mode is valid (paper/live)
  - Confirm broker credentials accessible (without logging them)

- **Runtime Validation**:
  - Reject hot-reload if validation fails
  - Alert operator of invalid configuration
  - Continue with previous valid configuration

## 16. Testing Strategy & Scenarios

### 16.1 Unit Testing Requirements

- **Minimum Coverage**: 80% code coverage
- **Critical Path Coverage**: 100% coverage for:

  - Order placement logic
  - Position sizing calculations
  - Risk management checks
  - Stop-loss and target calculations
  - Entry/exit decision logic

- **Indicator Calculations**:
  - ADX: Test against TA-Lib reference implementation
  - RSI: Validate with known datasets
  - EMA: Cross-check with industry standard values
  - Volume calculations: Verify aggregation correctness

### 16.2 Integration Testing Scenarios

1. **Authentication Flow**:

   - Test successful login
   - Test invalid credentials
   - Test token expiry handling
   - Test token refresh mechanism

2. **Market Data Integration**:

   - Test WebSocket connection and subscription
   - Test tick processing and bar construction
   - Test handling of missing data
   - Test option chain parsing

3. **Order Execution Flow**:

   - Test order placement (paper mode)
   - Test order fill confirmation
   - Test order rejection handling
   - Test order cancellation

4. **Position Management**:
   - Test position opening
   - Test P&L calculation
   - Test stop-loss trigger
   - Test target trigger
   - Test position closure

### 16.3 Edge Case Test Scenarios

1. **Gap Scenarios**:

   - Gap-up >100 points at market open
   - Gap-down >100 points at market open
   - Verify ATM recalculation
   - Verify token pool refresh

2. **Volatility Scenarios**:

   - VIX spike from 15 to 32 in 5 minutes
   - Verify circuit breaker activation
   - Verify position size reduction
   - Verify position closure

3. **Network Issues**:

   - WebSocket disconnection during active trade
   - API timeout during order placement
   - Network outage for 10 minutes
   - Verify auto-reconnection and recovery

4. **Order Scenarios**:

   - Order rejected by broker
   - Partial order fill
   - Order fill with high slippage
   - Multiple simultaneous order placements

5. **Time Scenarios**:

   - Market close approaching (3:25 PM)
   - Expiry approaching (3 days before)
   - Position held beyond max holding time
   - Trading hours vs non-trading hours

6. **Data Scenarios**:

   - Missing 1-minute bar data
   - Corrupt JSON file
   - Timestamp out of sequence
   - Price outlier detection

7. **Risk Scenarios**:
   - Daily loss limit reached
   - 3 consecutive losses
   - Margin utilization >90%
   - Maximum positions exceeded

### 16.4 Load & Stress Testing

- **Tick Processing Load**:

  - Simulate 10,000 ticks/second
  - Verify no data loss
  - Monitor CPU and memory usage
  - Ensure latency stays <10ms per tick

- **Order Burst**:

  - Submit 50 orders in 10 seconds
  - Verify all orders queued correctly
  - Check rate limiting enforcement
  - Validate no duplicate orders

- **Memory Stress**:
  - Run for 6 hours continuous trading
  - Monitor memory growth
  - Verify no memory leaks
  - Check garbage collection performance

### 16.5 Chaos Engineering Tests

- **Failure Injection**:

  - Kill WebSocket connection mid-trade
  - Simulate broker API timeout
  - Corrupt active data file
  - Force system crash with open positions

- **Recovery Validation**:
  - Verify auto-recovery mechanisms
  - Check position reconstruction accuracy
  - Validate data integrity after recovery
  - Ensure no orphan orders

### 16.6 Backtest vs Live Parity

- **Backtest Setup**:

  - Use 3 months of historical data
  - Include realistic slippage (0.2%)
  - Include all fees and charges
  - Model liquidity constraints

- **Parity Checks**:

  - Signal generation: Same signals on same data
  - Position sizing: Same lot calculations
  - Risk management: Same circuit breaker triggers
  - Performance: Paper trading within ±15% of backtest

- **Acceptable Variances**:
  - P&L difference: ±10% (due to real-time execution)
  - Win rate difference: ±5% (due to slippage)
  - Drawdown difference: ±3% (due to timing)

### 16.7 Pre-Live Trading Validation

**Week 1: Paper Trading**

- Run full system in paper mode
- Monitor for errors and crashes
- Validate signal generation
- Check order logic and timing
- Target: Zero critical errors

**Week 2: Limited Live**

- Start with 1 lot only
- Trade only 2 hours per day (11am-1pm)
- Maximum 1 position at a time
- Daily review of all trades
- Target: Positive P&L, no errors

**Week 3-4: Gradual Scale**

- Increase to 2 lots
- Trade full market hours
- Maximum 2 positions
- Weekly performance review
- Target: Consistent with paper trading

**Month 2+: Full Production**

- Full position sizing
- All features enabled
- Continuous monitoring
- Regular optimization

## 17. Gradual Rollout & Production Ramp

### 17.1 Phase 1: Paper Trading (Week 1-2)

- **Objective**: Validate system functionality without real money
- **Configuration**:
  - `trading_mode = "paper"`
  - Use real market data
  - Simulate order execution
  - Track performance metrics
- **Success Criteria**:
  - Zero system crashes
  - Zero critical errors
  - Positive paper P&L
  - All features working correctly
  - Performance matches backtest expectations (±15%)

### 17.2 Phase 2: Minimal Live (Week 3-4)

- **Objective**: Test with minimal real capital
- **Configuration**:
  - `trading_mode = "live"`
  - `max_positions = 1`
  - `base_position_size_pct = 0.5` (0.5% instead of 2%)
  - Trading hours: 11:00 AM - 1:00 PM only
  - Maximum 3 trades per day
- **Monitoring**:
  - Operator present during all trading hours
  - Real-time monitoring of all orders
  - Daily reconciliation with broker
- **Success Criteria**:
  - Order execution working correctly
  - Position tracking accurate
  - No wrong orders or fat fingers
  - Broker reconciliation 100% match

### 17.3 Phase 3: Limited Live (Month 2)

- **Objective**: Scale up gradually
- **Configuration**:
  - `max_positions = 2`
  - `base_position_size_pct = 1.0` (1% instead of 2%)
  - Full market hours (9:15 AM - 3:30 PM)
  - All strategy features enabled
- **Monitoring**:
  - Daily performance review
  - Weekly reconciliation
  - Error tracking and analysis
- **Success Criteria**:
  - Consistent profitability
  - No critical errors for 2+ weeks
  - Sharpe ratio > 1.0
  - Maximum drawdown < 10%

### 17.4 Phase 4: Full Production (Month 3+)

- **Objective**: Full-scale automated trading
- **Configuration**:
  - `max_positions = 3`
  - `base_position_size_pct = 2.0` (full position sizing)
  - All features enabled
  - Automated decision making
- **Ongoing Monitoring**:
  - Real-time dashboard monitoring
  - Daily performance reports
  - Weekly strategy review
  - Monthly system audit

### 17.5 Rollback Triggers

- **Immediate Rollback** (back to previous phase):

  - Single day loss >5%
  - System crashes more than once per week
  - Multiple wrong orders or execution errors
  - Broker reconciliation mismatches

- **Phase Suspension** (pause current phase):
  - Three consecutive losing days
  - Sharpe ratio drops below 0.5
  - Win rate drops below 40%
  - Unexplained system behavior

## 18. Production Readiness Final Checklist

### Core Trading System

- [ ] Market hours and holiday calendar validation
- [ ] Token management and auto-refresh
- [ ] WebSocket connection and auto-reconnect
- [ ] Real-time data processing and bar construction
- [ ] ADX-based categorization logic
- [ ] Multi-timeframe analysis implementation
- [ ] Entry signal generation
- [ ] Exit signal generation
- [ ] Position sizing calculations
- [ ] Risk management and circuit breakers

### Order Management

- [ ] Order placement with idempotency
- [ ] Order fill verification
- [ ] Order cancellation logic
- [ ] Order retry with exponential backoff
- [ ] Slippage validation
- [ ] Fat finger prevention

### Risk Controls

- [ ] Daily loss limits
- [ ] Position limits
- [ ] Margin checks
- [ ] VIX-based circuit breakers
- [ ] Stop-loss enforcement
- [ ] Target management
- [ ] Trailing stop logic

### Data Management

- [ ] JSON file storage structure
- [ ] Data retention and cleanup
- [ ] Data validation and gap detection
- [ ] Backup and recovery procedures
- [ ] Historical data download

### Monitoring & Alerts

- [ ] Health check endpoints
- [ ] Real-time dashboard
- [ ] Email/SMS alerts configured
- [ ] Logging and audit trail
- [ ] Performance metrics tracking

### Operational Controls

- [ ] CLI commands implemented
- [ ] Kill-switch mechanism
- [ ] Manual trading mode
- [ ] Configuration hot-reload
- [ ] Graceful shutdown

### Testing

- [ ] Unit tests (80%+ coverage)
- [ ] Integration tests
- [ ] Edge case tests (20+ scenarios)
- [ ] Load tests
- [ ] Chaos tests
- [ ] Backtest parity validation

### Deployment

- [ ] Blue-green deployment setup
- [ ] Rollback procedures documented
- [ ] Configuration management
- [ ] Schema migrations tested
- [ ] Post-deployment monitoring plan

### Documentation

- [ ] System architecture documented
- [ ] Operational runbooks created
- [ ] Emergency procedures documented
- [ ] Configuration reference complete
- [ ] API documentation

### Compliance & Security

- [ ] Broker TOS compliance verified
- [ ] Secrets management implemented
- [ ] Audit trail complete
- [ ] Data retention policy defined
- [ ] Access control implemented

### Pre-Live Validation

- [ ] 2+ weeks successful paper trading
- [ ] All tests passing
- [ ] Performance matches expectations
- [ ] Operator training complete
- [ ] Emergency contacts configured

## 19. Technical Indicator Calculation Specifications

### 19.1 ADX (Average Directional Index) Calculation

#### 19.1.1 ADX Overview & Interpretation

**ADX Purpose**: Measures trend strength on a 0-100 scale

- **ADX < 20**: Weak trend, ranging market → Avoid trading
- **ADX 20-25**: Emerging trend → Cautious entry
- **ADX 25-50**: Strong trend → Ideal for trading
- **ADX > 50**: Very strong trend → May be overextended, monitor closely

**Usage in Strategy**: Categorize stocks for CE vs PE selection

- Category 1 (Buy CE): ADX > 25 AND +DI > -DI
- Category 2 (Buy PE): ADX > 25 AND -DI > +DI
- Category 3 (No Trade): ADX < 20 OR directional indicators unclear

#### 19.1.2 ADX Calculation Algorithm (14-period standard)

**Step 1: Calculate True Range (TR) for each bar**

- True Range = Maximum of:
  - Current High - Current Low
  - Absolute value of (Current High - Previous Close)
  - Absolute value of (Current Low - Previous Close)
- TR captures the full range of price movement including gaps

**Step 2: Calculate Directional Movement (+DM and -DM)**

- **+DM (Positive Directional Movement)**:
  - If (Current High - Previous High) > (Previous Low - Current Low) AND (Current High - Previous High) > 0:
    - +DM = Current High - Previous High
  - Otherwise: +DM = 0
- **-DM (Negative Directional Movement)**:
  - If (Previous Low - Current Low) > (Current High - Previous High) AND (Previous Low - Current Low) > 0:
    - -DM = Previous Low - Current Low
  - Otherwise: -DM = 0

**Step 3: Apply Wilder's Smoothing (14-period)**

- **First Smoothed Value** = Sum of first 14 periods
- **Subsequent Smoothed Values** = Previous Smoothed Value - (Previous Smoothed Value / 14) + Current Value
- Apply smoothing to: TR, +DM, -DM

**Step 4: Calculate Directional Indicators (+DI and -DI)**

- **+DI** = (Smoothed +DM / Smoothed TR) × 100
- **-DI** = (Smoothed -DM / Smoothed TR) × 100
- These indicate the strength of upward vs downward movement

**Step 5: Calculate Directional Index (DX)**

- **DX** = (Absolute value of (+DI - -DI) / (+DI + -DI)) × 100
- DX measures the difference between directional movements

**Step 6: Calculate ADX (Average of DX)**

- **First ADX** = Sum of first 14 DX values / 14
- **Subsequent ADX** = ((Previous ADX × 13) + Current DX) / 14
- ADX is a smoothed average of DX values

#### 19.1.3 ADX Implementation Notes

- **Minimum Data Required**: At least 28-30 bars for reliable ADX (14 for smoothing + 14 for ADX average)
- **Update Frequency**: Recalculate daily after market close for categorization
- **Storage**: Store ADX, +DI, -DI values for each symbol in daily timeframe data
- **Validation**: Cross-check initial calculations against known reference implementations

### 19.2 RSI (Relative Strength Index) Calculation

#### 19.2.1 RSI Overview & Interpretation

**RSI Purpose**: Measures momentum on 0-100 scale

- **RSI < 30**: Oversold condition (potential reversal upward)
- **RSI 30-40**: Pullback in uptrend → Entry zone for CE options
- **RSI 40-60**: Neutral zone → No clear signal
- **RSI 60-70**: Pullback in downtrend → Entry zone for PE options
- **RSI > 70**: Overbought condition (potential reversal downward)

**Usage in Strategy**: Fine-tune entry timing on 5-minute timeframe

- Wait for RSI to reach entry zone before placing order
- Confirm pullback is complete before entering

#### 19.2.2 RSI Calculation Algorithm (14-period standard)

**Step 1: Calculate Price Changes**

- For each bar: Change = Current Close - Previous Close
- If Change > 0: Gain = Change, Loss = 0
- If Change < 0: Gain = 0, Loss = Absolute value of Change
- If Change = 0: Gain = 0, Loss = 0

**Step 2: Calculate Initial Average Gain and Loss**

- **First Average Gain** = Sum of first 14 gains / 14
- **First Average Loss** = Sum of first 14 losses / 14
- Requires 14 bars of historical data minimum

**Step 3: Apply Wilder's Smoothing for Subsequent Values**

- **Average Gain** = ((Previous Average Gain × 13) + Current Gain) / 14
- **Average Loss** = ((Previous Average Loss × 13) + Current Loss) / 14
- This smoothing gives more weight to recent values

**Step 4: Calculate Relative Strength (RS)**

- **RS** = Average Gain / Average Loss
- Special case: If Average Loss = 0, set RS to infinity (RSI = 100)

**Step 5: Calculate RSI**

- **RSI** = 100 - (100 / (1 + RS))
- Result is a value between 0 and 100

#### 19.2.3 RSI Implementation Notes

- **Minimum Data Required**: 15 bars (14 for averaging + 1 for current)
- **Update Frequency**: Recalculate on every new bar close (1-minute, 5-minute)
- **Storage**: Maintain running state (average gain, average loss, previous close)
- **Edge Cases**: Handle zero division when Average Loss = 0 (return RSI = 100)

### 19.3 EMA (Exponential Moving Average) Calculation

#### 19.3.1 EMA Overview & Purpose

**EMA Purpose**: Trend-following indicator with more weight on recent prices

- **Price above EMA**: Uptrend signal
- **Price below EMA**: Downtrend signal
- **EMA crossovers**: Trend change signals

**Usage in Strategy**: Support/resistance validation

- 15-minute low staying above EMA-20 confirms support (for CE entry)
- 15-minute high staying below EMA-20 confirms resistance (for PE entry)
- 5-minute price bouncing off EMA-9 provides entry trigger

#### 19.3.2 EMA Calculation Algorithm

**Step 1: Calculate Smoothing Multiplier**

- **Multiplier** = 2 / (Period + 1)
- Examples:
  - 9-period EMA: Multiplier = 2 / 10 = 0.20
  - 20-period EMA: Multiplier = 2 / 21 = 0.095

**Step 2: Calculate First EMA**

- **First EMA** = Simple Moving Average of first N periods
- SMA = Sum of first N closing prices / N
- This initializes the EMA calculation

**Step 3: Calculate Subsequent EMA Values**

- **EMA** = (Current Price × Multiplier) + (Previous EMA × (1 - Multiplier))
- Alternative formula: EMA = Previous EMA + (Multiplier × (Current Price - Previous EMA))
- This gives exponentially decreasing weight to older prices

#### 19.3.3 EMA Implementation Notes

- **Minimum Data Required**: N periods for initialization (9 or 20 bars)
- **Update Frequency**: Recalculate on every new bar close
- **Storage**: Maintain only the last EMA value (no need to store all history)
- **Multiple EMAs**: Calculate EMA-9 and EMA-20 separately for each timeframe

### 19.4 Volume Average Calculation

#### 19.4.1 Volume Average Overview

**Purpose**: Identify volume surges indicating strong moves

- **Volume > 1.2× Average**: Strong move, valid signal
- **Volume < 0.8× Average**: Weak move, questionable signal
- **Volume > 3× Average**: Flash spike, potential reversal

**Usage in Strategy**: Confirm entry signals

- Entry requires: Current volume > 120% of average volume
- Validates that price move has participation

#### 19.4.2 Simple Volume Average (SMA) Calculation

**Algorithm**:

1. Maintain a rolling window of N volume values (e.g., 20 periods)
2. On each new bar, add current volume to window
3. If window size exceeds N, remove oldest volume value
4. **Volume Average** = Sum of all volumes in window / Number of values in window

**Comparison Logic**:

- Calculate current volume as percentage of average: (Current Volume / Average Volume) × 100
- If percentage > threshold (120%), volume condition is met

#### 19.4.3 Volume Implementation Notes

- **Period**: Use 20-period average for daily/hourly, 14-period for intraday
- **Update Frequency**: Recalculate on every bar close
- **Storage**: Maintain rolling array of last N volume values
- **Startup**: Need minimum N bars before volume average is valid

### 19.5 Indicator Library Requirements

**MANDATORY: Use Existing Technical Analysis Libraries**

- **MUST use established TA libraries** (TA-Lib, ta-rs for Rust, or equivalent)
- **DO NOT implement indicators manually** - error-prone and time-consuming
- Advantages: Battle-tested implementations, optimized performance, edge cases handled
- Required validation:
  - Validate calculations against known test cases
  - Cross-check ADX/RSI/EMA values with TradingView or other platforms
  - Implement error handling for edge cases (insufficient data, zero divisions)

**For Rust Implementation**:

- Use `ta` crate (most popular) or `yata` crate (high performance)
- Never write custom indicator logic
- Trust established implementations

## 20. Transaction Cost Model & P&L Calculation

### 20.1 Complete Fee Structure (India - Options Trading)

#### 20.1.1 All Transaction Cost Components

**1. Brokerage**

- **Angel One Rate**: ₹20 per order (flat fee for intraday options)
- **Applied on**: Each order (both entry and exit)
- **Note**: Verify current broker rates as they may change

**2. Securities Transaction Tax (STT)**

- **Buy Side**: No STT on options purchase
- **Sell Side**: 0.0625% of premium (₹0.0625 per ₹100 premium)
- **Applied on**: Premium amount only, not notional value
- **Calculation**: Turnover × 0.0625 / 100

**3. Exchange Transaction Charges (NSE)**

- **Rate**: 0.053% of premium (₹0.053 per ₹100 premium)
- **Applied on**: Both buy and sell sides
- **Calculation**: Turnover × 0.053 / 100

**4. SEBI Turnover Charges**

- **Rate**: ₹10 per crore of turnover
- **Applied on**: Premium turnover
- **Calculation**: (Turnover / ₹1,00,00,000) × 10

**5. GST (Goods and Services Tax)**

- **Rate**: 18% on (Brokerage + Exchange Charges)
- **Applied on**: Service charges only (not on STT/SEBI charges)
- **Calculation**: (Brokerage + Exchange Charges) × 0.18

**6. Stamp Duty**

- **Rate**: 0.003% of turnover (varies by state)
- **Applied on**: Buy side only
- **Maharashtra/Delhi**: 0.003%
- **Calculation**: Turnover × 0.003 / 100 (buy side only)

### 20.2 Complete Cost Calculation Algorithm

#### 20.2.1 Per-Order Cost Calculation

**Input Parameters:**

- Premium (price per unit)
- Lot size (units per lot)
- Number of lots
- Order side (Buy or Sell)
- Broker rate per order (₹20 for Angel One)

**Calculation Steps:**

1. **Calculate Turnover**:

   - Turnover = Premium × Lot Size × Number of Lots

2. **Calculate Brokerage**:

   - Brokerage = Flat rate per order (₹20)

3. **Calculate STT**:

   - If Buy order: STT = 0
   - If Sell order: STT = Turnover × 0.0625 / 100

4. **Calculate Exchange Charges**:

   - Exchange Charges = Turnover × 0.053 / 100

5. **Calculate SEBI Charges**:

   - SEBI Charges = (Turnover / 1,00,00,000) × 10

6. **Calculate GST**:

   - GST = (Brokerage + Exchange Charges) × 0.18

7. **Calculate Stamp Duty**:

   - If Buy order: Stamp Duty = Turnover × 0.003 / 100
   - If Sell order: Stamp Duty = 0

8. **Calculate Total Cost**:
   - Total = Brokerage + STT + Exchange Charges + SEBI Charges + GST + Stamp Duty

### 20.3 Complete P&L Calculation Process

#### 20.3.1 Round-Trip P&L Algorithm

**Input Parameters:**

- Entry premium (buy price)
- Exit premium (sell price)
- Lot size
- Number of lots

**Calculation Steps:**

1. **Calculate Total Quantity**:

   - Total Quantity = Lot Size × Number of Lots

2. **Calculate Entry Costs** (using buy-side cost algorithm):

   - Entry Costs = Calculate transaction costs for buy order

3. **Calculate Exit Costs** (using sell-side cost algorithm):

   - Exit Costs = Calculate transaction costs for sell order

4. **Calculate Gross P&L**:

   - Gross P&L = (Exit Premium - Entry Premium) × Total Quantity

5. **Calculate Net P&L**:

   - Total Costs = Entry Costs + Exit Costs
   - Net P&L = Gross P&L - Total Costs

6. **Calculate Cost Percentage**:
   - Cost % = (Total Costs / Gross P&L) × 100

### 20.4 Worked Example - Full Trade Calculation

**Trade Scenario:**

- Symbol: NIFTY 23,500 CE
- Entry: Buy at ₹150 premium
- Exit: Sell at ₹180 premium
- Lot size: 50 units
- Quantity: 1 lot

**Entry Costs Calculation (Buy Side):**

1. Turnover = 150 × 50 = ₹7,500
2. Brokerage = ₹20.00 (flat)
3. STT = ₹0.00 (no STT on buy)
4. Exchange = 7,500 × 0.053% = ₹3.98
5. SEBI = (7,500 / 1,00,00,000) × 10 = ₹0.01
6. GST = (20.00 + 3.98) × 18% = ₹4.32
7. Stamp Duty = 7,500 × 0.003% = ₹0.23
8. **Total Entry Costs = ₹28.54**

**Exit Costs Calculation (Sell Side):**

1. Turnover = 180 × 50 = ₹9,000
2. Brokerage = ₹20.00 (flat)
3. STT = 9,000 × 0.0625% = ₹5.63
4. Exchange = 9,000 × 0.053% = ₹4.77
5. SEBI = (9,000 / 1,00,00,000) × 10 = ₹0.01
6. GST = (20.00 + 4.77) × 18% = ₹4.46
7. Stamp Duty = ₹0.00 (no stamp duty on sell)
8. **Total Exit Costs = ₹34.87**

**P&L Summary:**

- Gross P&L = (180 - 150) × 50 = ₹1,500.00
- Total Costs = 28.54 + 34.87 = ₹63.41
- **Net P&L = ₹1,436.59**
- Cost as % of Gross P&L = 4.23%

### 20.5 Break-Even Calculation

#### 20.5.1 Break-Even Premium Formula

**Objective**: Find exit premium where Net P&L = 0

**Approximate Method**:

1. Calculate entry costs (known)
2. Assume exit costs ≈ entry costs (approximation)
3. Total estimated costs = Entry costs × 2
4. Break-even premium = Entry premium + (Total costs / Total quantity)

**Example**:

- Entry premium = ₹150
- Entry costs = ₹28.54
- Estimated total costs = ₹28.54 × 2 = ₹57.08
- Break-even premium = 150 + (57.08 / 50) = 150 + 1.14 = ₹151.14

**Note**: This is approximate because exit costs depend on exit premium

#### 20.5.2 Precise Break-Even Calculation

For exact break-even, solve iteratively:

1. Start with approximate break-even premium
2. Calculate actual exit costs at that premium
3. Recalculate break-even = Entry premium + ((Entry costs + Exit costs) / Quantity)
4. Repeat until convergence (usually 2-3 iterations)

### 20.6 Minimum Profit Target Guidelines

#### 20.6.1 Cost-Based Profit Targets

**Given typical transaction costs of 4% of gross P&L:**

- **Conservative Target**: 2× transaction costs

  - Minimum gross move: 8-10% in premium
  - Ensures healthy profit margin

- **Realistic Target**: 1.5× transaction costs

  - Minimum gross move: 6-8% in premium
  - Balanced risk-reward

- **Minimum Viable**: 1.2× transaction costs
  - Minimum gross move: 5% in premium
  - Tight margins, use sparingly

#### 20.6.2 Strategy Profit Target Validation

**Our Strategy Targets** (1% stop, 3% target on underlying):

- Underlying move: 1-3% (NIFTY/BANKNIFTY)
- Option premium move: Typically 10-30% (depending on delta)
- Transaction costs: ~4% of premium move
- **Cost ratio**: 4% / 20% average = 20% of profit
- **Verdict**: Acceptable cost ratio (<25%)

#### 20.6.3 Cost Impact by Position Size

**Small Position (1 lot)**:

- Fixed costs dominate (brokerage ₹20 + ₹20 = ₹40)
- Cost percentage: Higher (~5-6% of turnover)
- Need larger % moves to be profitable

**Medium Position (2-3 lots)**:

- Variable costs increase proportionally
- Cost percentage: Moderate (~4% of turnover)
- Optimal for most strategies

**Large Position (5+ lots)**:

- Variable costs dominate (STT, exchange charges)
- Cost percentage: Lower (~3-3.5% of turnover)
- Better efficiency but higher risk

**Recommendation**: Trade 2-3 lots minimum for cost efficiency

## 21. SEBI Compliance & Position Limits

### 21.1 SEBI Position Limit Rules

#### 21.1.1 Market-Wide Position Limit (MWPL)

**MWPL Definition**: Maximum position allowed market-wide for each underlying

**Calculation Formula**:

- MWPL = **HIGHER OF**:
  1. 15% of total open interest (in number of underlying units)
  2. ₹500 crore worth of gross open positions

**Example Calculation for NIFTY Options**:

- Scenario: Total NIFTY OI = 50 lakh units (50,00,000)

Method 1 - Based on OI:

- MWPL = 15% × 50,00,000 = 7,50,000 units

Method 2 - Based on notional value:

- If NIFTY = 23,500
- ₹500 crore / 23,500 = 2,12,766 units

Final MWPL = MAX(7,50,000, 2,12,766) = **7,50,000 units**

#### 21.1.2 Client-Level Position Limit

**Client Limit Definition**: Maximum position allowed per individual client

**Calculation Formula**:

- Client Limit = **HIGHER OF**:
  1. 1% of MWPL
  2. ₹5 crore worth of gross open positions

**Example Calculation**:

- Given: MWPL = 7,50,000 units

Method 1 - Based on MWPL:

- Client Limit = 1% × 7,50,000 = 7,500 units

Method 2 - Based on notional value:

- If NIFTY = 23,500
- ₹5 crore / 23,500 = 21,277 units

Final Client Limit = MAX(7,500, 21,277) = **21,277 units**

#### 21.1.3 Position Limit Application Scope

**Combined Limit Across**:

- All strike prices (ATM, ITM, OTM)
- All expiry dates (weekly + monthly + quarterly)
- Both CE and PE positions combined
- All contract months

**Important**: Bot must track total position across all option contracts for each underlying

#### 21.1.4 Position Limit Tracking System

**System Requirements**:

1. **Initialize Limits**:

   - Fetch current MWPL data from NSE website/API
   - Calculate client limits based on MWPL
   - Store limits per underlying (NIFTY, BANKNIFTY, FINNIFTY, etc.)

2. **Track Current Positions**:

   - Maintain running total of positions per underlying
   - Update on every order fill (not on order placement)
   - Include all strikes and expiries in total

3. **Pre-Order Validation**:

   - Before placing order, calculate: New Total = Current Position + Proposed Order Size
   - If New Total > Client Limit: Reject order
   - If New Total ≤ Client Limit: Allow order

4. **Daily Sync**:
   - Fetch latest MWPL data daily (OI changes daily)
   - Recalculate client limits
   - Update position tracking from broker API

**Data Sources for MWPL**:

- NSE URL: `https://www.nseindia.com/api/underlying-market-watch-all`
- Update frequency: Daily after market close
- Backup: Maintain conservative estimates if API unavailable

### 21.2 SEBI Reporting Requirements

#### 21.2.1 Algorithmic Trading Registration

**Registration Process**:

1. Inform broker that trading is algorithmic/automated
2. Provide strategy details and risk management framework to broker
3. Broker reviews and assigns unique Algo ID
4. Exchange assigns algo identifier for order tagging
5. All orders must be tagged with Algo ID

**Required Documentation**:

- Strategy description (ADX-based trend following)
- Risk management controls (circuit breakers, loss limits, kill-switch)
- Testing results (backtest, paper trading)
- Operator details and escalation procedures

#### 21.2.2 Audit Trail Requirements

**Mandatory Records (5-year retention)**:

**1. Order Audit Log** (every order):

- Timestamp (millisecond precision)
- Symbol and contract details
- Order type (LIMIT/MARKET), side (BUY/SELL), product type (MIS/CNC/NRML)
- Price and quantity
- Unique order ID (internal + broker)
- Order status (PENDING/COMPLETE/REJECTED/CANCELLED)
- Rejection reason (if rejected)

**2. Trade Execution Log** (every fill):

- Fill timestamp
- Fill price and quantity
- Execution ID from broker
- Slippage (difference from intended price)
- Transaction costs breakdown

**3. Signal Generation Log** (every signal):

- Timestamp of signal generation
- Symbol and recommended action (BUY_CE/BUY_PE/NO_TRADE)
- Indicator values (ADX, +DI, -DI, RSI, EMA, Volume)
- Reason for signal
- Whether signal was acted upon (Yes/No) and reason if not

**4. Risk Management Log** (all risk events):

- Circuit breaker activations/deactivations
- Position limit checks (pass/fail)
- Daily loss limit checks
- Margin utilization checks
- Kill-switch triggers

**5. System Event Log** (all system events):

- System start/stop with version information
- Configuration changes
- Token expiry and refresh events
- WebSocket disconnections and reconnections
- API errors and retry attempts
- Crash logs and stack traces

**Log Format Requirements**:

- Structured format (JSON)
- Searchable and filterable
- Immutable (append-only, no deletion/modification)
- Include correlation IDs to track order lifecycle

#### 21.2.3 Risk Management System (RMS) Requirements

**SEBI-Mandated Controls**:

**1. Order Price Validation**:

- Maximum deviation from LTP (typically ±20%)
- Reject orders outside price bands

**2. Order Quantity Validation**:

- Maximum order size per order
- Lot size multiple validation
- Freeze quantity checks

**3. Position Limit Enforcement**:

- SEBI client-level limits
- Custom lower limits if desired
- Real-time position tracking

**4. Loss Limit Controls**:

- Daily loss limit (halt trading after breach)
- Weekly/monthly loss tracking
- Consecutive loss limits

**5. Kill-Switch Mechanism**:

- Manual trigger (keyboard/CLI/API)
- Automated triggers (loss limits, errors)
- Emergency position closure
- Trading halt capability

### 21.3 Broker RMS (Risk Management System) Rules

#### 21.3.1 Freeze Quantity Limits

**Definition**: Maximum quantity per single order before order freezes

**Current Limits** (verify periodically as these change):

- **NIFTY Options**: 36,000 units (720 lots of 50)
- **BANKNIFTY Options**: 14,400 units (960 lots of 15)
- **FINNIFTY Options**: 40,000 units (1,000 lots of 40)

**Freeze Behavior**:

- Orders exceeding freeze quantity require manual approval
- Automated systems cannot place orders above freeze quantity
- Split large orders into multiple smaller orders if needed

**Validation Algorithm**:

1. Calculate total order quantity = Lot Size × Number of Lots
2. Compare against freeze quantity for that underlying
3. If quantity > freeze quantity: Reject order
4. If quantity ≤ freeze quantity: Proceed

#### 21.3.2 Price Band Restrictions

**Definition**: Maximum price deviation from LTP allowed for orders

**Standard Band**: ±20% from Last Traded Price (LTP)

**Validation Algorithm**:

1. Get current LTP for the option
2. Calculate lower band = LTP × (1 - 0.20) = LTP × 0.80
3. Calculate upper band = LTP × (1 + 0.20) = LTP × 1.20
4. Check if order price is within [lower band, upper band]
5. If outside band: Reject order
6. If inside band: Proceed

**Purpose**: Prevents erroneous orders (fat finger errors)

#### 21.3.3 Margin Requirements

**For Options Buying** (our strategy):

- **Margin Required** = Premium × Lot Size × Number of Lots
- No SPAN margin needed (unlike option selling)
- Simply need funds to pay for the premium

**Pre-Order Margin Check**:

1. Calculate required margin = Order Value
2. Query broker API for available margin
3. If available margin < required margin: Reject order (insufficient funds)
4. If available margin ≥ required margin: Proceed

**Margin Monitoring**:

- Check margin before each order
- Alert if margin utilization > 70%
- Pause trading if margin utilization > 80%

#### 21.3.4 Lot Size and Tick Size Validation

**Lot Size** (units per contract):

- **NIFTY**: 50 units (verify current)
- **BANKNIFTY**: 15 units (verify current)
- **FINNIFTY**: 40 units (verify current)
- Updates periodically (usually quarterly)

**Tick Size** (minimum price increment):

- **All Options**: ₹0.05 (5 paise)
- Orders must be in multiples of tick size

**Validation Algorithms**:

**Lot Size Validation**:

1. Total quantity must be multiple of lot size
2. If (Quantity % Lot Size) ≠ 0: Reject order
3. Valid example: 100 units for NIFTY (100 / 50 = 2 lots) ✓
4. Invalid example: 75 units for NIFTY (75 / 50 = 1.5 lots) ✗

**Tick Size Validation**:

1. Order price must be multiple of ₹0.05
2. Valid prices: 150.00, 150.05, 150.10, 150.15 ✓
3. Invalid prices: 150.03, 150.07, 150.12 ✗

### 21.4 Pre-Order Validation Checklist

**Complete Validation Sequence** (execute before every order):

**1. Position Limit Validation**:

- Current position + proposed order ≤ Client limit
- If exceeded: Error "Position limit breach"

**2. Freeze Quantity Validation**:

- Order quantity ≤ Freeze quantity
- If exceeded: Error "Exceeds freeze quantity"

**3. Price Band Validation**:

- Order price within ±20% of LTP
- If outside: Error "Price outside band"

**4. Lot Size Validation**:

- Quantity is multiple of lot size
- If not: Error "Quantity must be multiple of lot size"

**5. Tick Size Validation**:

- Price is multiple of ₹0.05
- If not: Error "Price must be multiple of tick size"

**6. Margin Validation**:

- Available margin ≥ Required margin
- If insufficient: Error "Insufficient margin"

**7. Contract Validation**:

- Symbol exists and is active
- Expiry date is future (not expired)
- Strike price is valid

**8. Market Hours Validation**:

- Market is open
- Not in pre-market or post-market
- Not a holiday

**Result**: Only if ALL validations pass, submit order to broker

## 22. NSE Holiday Calendar & Market Hours

### 22.1 NSE Holiday Calendar Integration

#### 22.1.1 Holiday Calendar Data Source

**NSE Official API**:

- **URL**: `https://www.nseindia.com/api/holiday-master?type=trading`
- **Method**: GET
- **Headers Required**:
  - User-Agent: Mozilla/5.0 (standard browser user agent)
  - Accept: application/json
- **Update Frequency**: Fetch annually at start of year, refresh monthly

**Response Format** (JSON):

- Field `CBM` contains array of holidays
- Each holiday has:
  - `tradingDate`: Date in format "DD-MMM-YYYY" (e.g., "26-Jan-2024")
  - `weekDay`: Day of week (e.g., "Friday")
  - `description`: Holiday name (e.g., "Republic Day")
  - `Sr_no`: Serial number

**Typical Indian Market Holidays** (15-20 per year):

- Republic Day (January 26)
- Holi
- Good Friday
- Dr. Ambedkar Jayanti
- Mahavir Jayanti
- Independence Day (August 15)
- Ganesh Chaturthi
- Dussehra
- Diwali (Laxmi Pujan)
- Diwali (Balipratipada)
- Guru Nanak Jayanti
- Christmas (December 25)

#### 22.1.2 Holiday Calendar Management Process

**1. Initialization (Start of Year)**:

- Fetch holiday list from NSE API
- Parse JSON response
- Extract all trading dates
- Convert dates from "DD-MMM-YYYY" format to standard date format
- Store in local file: `data/holidays_2024.json` (year-specific)

**2. Local Storage Format**:

- Store as JSON array of ISO date strings
- Example: `["2024-01-26", "2024-03-25", "2024-08-15", ...]`
- Allows offline access without API dependency

**3. Trading Day Validation Algorithm**:

**Check if date is trading day**:

1. If day is Saturday or Sunday → Not a trading day
2. If date is in holiday list → Not a trading day
3. Otherwise → Trading day

**Implementation Logic**:

- Input: Date to check
- Step 1: Extract day of week
- Step 2: If weekend (Sat/Sun), return False
- Step 3: Convert date to standard format (YYYY-MM-DD)
- Step 4: Check if date exists in holiday set
- Step 5: If in holiday set, return False
- Step 6: Return True (is a trading day)

**4. Daily Check**:

- At system startup, check if today is a trading day
- If not a trading day, enter data management mode (no live trading)
- If trading day, verify market hours and proceed

**5. Fallback Mechanism**:

- If API fetch fails, use locally cached file from previous update
- If cache is stale (>30 days old), use conservative approach:
  - Mark weekends as non-trading days
  - Allow all weekdays (may miss some holidays, but safe)
  - Alert operator about stale calendar data

#### 22.1.3 Holiday Calendar Update Schedule

**Annual Update**:

- Fetch new year's calendar in December (before year starts)
- Validate against previous year's calendar for sanity
- NSE typically publishes next year's calendar by November

**Monthly Refresh**:

- Re-fetch calendar data monthly to catch any changes
- NSE may occasionally add/remove holidays due to special circumstances

**On-Demand Update**:

- Manual refresh command: `bot calendar refresh`
- After major announcements or emergency market closures

### 22.2 Market Hours Definition & Management

#### 22.2.1 Standard Market Hours (Monday-Friday)

**Market Timings (IST)**:

- **Pre-Market Session**: 09:00 AM - 09:15 AM

  - Purpose: Pre-open session, indicative price discovery
  - No options trading in pre-market
  - Bot in preparation mode (data sync, token refresh)

- **Regular Trading Hours**: 09:15 AM - 03:30 PM (3:30 PM)

  - Primary trading session
  - Bot actively trades during these hours
  - Full strategy execution enabled

- **Post-Market Session**: 03:30 PM - 04:00 PM
  - Purpose: Closing session
  - No new positions
  - Data collection and reconciliation

#### 22.2.2 Market Hours Validation Logic

**1. Current Session Status Detection**:

- Get current system time (IST timezone)
- Compare with defined session boundaries
- Return session status: HOLIDAY | PRE_MARKET | OPEN | POST_MARKET | CLOSED

**Status Determination Logic**:

```
IF today is not a trading day (weekend or holiday):
    Status = HOLIDAY
ELSE IF current time >= 09:00 AND < 09:15:
    Status = PRE_MARKET
ELSE IF current time >= 09:15 AND < 15:30:
    Status = OPEN
ELSE IF current time >= 15:30 AND < 16:00:
    Status = POST_MARKET
ELSE:
    Status = CLOSED
```

**2. Trading Permission Logic**:

**Can Place New Trades**:

- Market status = OPEN
- AND current time < 14:30 (2:30 PM)
- Reason: Stop new entries 30 minutes before market close

**Must Close All Positions**:

- Market status = OPEN
- AND current time >= 15:20 (3:20 PM)
- Reason: Exit all positions 10 minutes before market close

**3. Time-Based Actions**:

**09:00 AM** (Pre-Market):

- Load today's trading plan
- Refresh token if needed
- Sync historical data
- Verify system health
- Subscribe to WebSocket feeds

**09:15 AM** (Market Open):

- Activate signal generation
- Begin monitoring for entry opportunities
- Enable order placement

**14:30 PM** (2:30 PM):

- Stop looking for new entry signals
- Continue monitoring existing positions
- Update stop-loss and targets

**15:20 PM** (3:20 PM):

- Begin systematic position closure
- Close positions at market price if needed
- Ensure all positions closed by 3:25 PM

**15:30 PM** (Market Close):

- Verify all positions closed
- Generate end-of-day reports
- Backup data files
- Reconcile with broker

**16:00 PM** (After Market Close):

- Complete data download (historical data sync)
- Calculate daily performance metrics
- Update ADX categorization for tomorrow
- Prepare next day's trading plan

#### 22.2.3 Time Calculation Utilities

**Minutes Until Market Close**:

- Current time in minutes = (Hour × 60) + Minutes
- Market close in minutes = (15 × 60) + 30 = 930
- Minutes remaining = 930 - Current time in minutes

**Example**:

- Current time: 14:45 (2:45 PM)
- Current in minutes: (14 × 60) + 45 = 885
- Minutes remaining: 930 - 885 = 45 minutes

**Time-Based Trading Rules**:

- Stop new entries when minutes remaining < 30
- Start closing positions when minutes remaining ≤ 10
- Emergency close if minutes remaining ≤ 5

### 22.3 Trading Session State Management

#### 22.3.1 Session State Transitions

**State Flow**:

```
CLOSED (before 9:00 AM)
    ↓
PRE_MARKET (9:00 AM - 9:15 AM)
    ↓
OPEN (9:15 AM - 3:30 PM)
    ↓
POST_MARKET (3:30 PM - 4:00 PM)
    ↓
CLOSED (after 4:00 PM)

OR at any time:
    ↓
HOLIDAY (weekends, holidays)
```

#### 22.3.2 Bot Behavior by Session State

**HOLIDAY State**:

- No trading activities
- Data management mode: Download historical data, fill gaps, run backtests
- Maintenance tasks: Update holiday calendar, clean old data, optimize storage
- System health checks and updates

**PRE_MARKET State**:

- Preparation mode
- Token validation and refresh
- WebSocket connection establishment
- Historical data sync for today
- Load configuration and trading plan
- No order placement

**OPEN State**:

- Full trading mode enabled
- Signal generation active
- Order placement allowed (until 2:30 PM)
- Position monitoring active
- All risk controls active
- Real-time P&L tracking

**POST_MARKET State**:

- Trading halted
- Final position verification (should be zero)
- Data collection and backup
- Generate performance reports
- Calculate metrics for today
- Prepare next day's plan

**CLOSED State**:

- System in standby mode
- Can be shut down safely
- Background tasks only (if any)
- Ready for next day's pre-market session

### 22.4 Special Trading Sessions

#### 22.4.1 Muhurat Trading (Diwali Special Session)

**Timing**: Usually 18:15 - 19:15 (6:15 PM - 7:15 PM)
**Duration**: 1 hour
**Date**: Diwali evening (varies each year, usually October/November)

**Bot Behavior**:

- This is a special evening session
- NSE publishes exact timings in advance
- Bot must be configured with special session parameters
- Consider whether to trade this session (usually low volume, symbolic trading)

**Configuration**:

- Add special session override for Diwali date
- Override market hours: 18:15 - 19:15
- Consider disabling automated trading (low liquidity, symbolic nature)

#### 22.4.2 Early Market Closures

**Scenarios**:

- Special occasions (rare)
- Market disruptions (technical issues)
- Emergency situations

**Bot Handling**:

- Monitor for exchange announcements
- If early closure announced:
  - Stop new entries immediately
  - Close positions at market price
  - Don't wait for normal closure times
- Manual intervention may be required

### 22.5 Timezone & Time Synchronization

#### 22.5.1 Timezone Management

**Standard Timezone**: Asia/Kolkata (IST - Indian Standard Time)

- UTC +5:30
- No daylight saving time
- All market hours are in IST

**System Time Requirements**:

- Ensure system clock is set to IST
- Or convert all times to IST for comparison
- NSE timestamps are always in IST

#### 22.5.2 Time Synchronization

**NTP Sync Requirement**:

- System clock must be NTP-synchronized
- Maximum acceptable drift: ±1 second
- Critical for accurate timestamp logging (audit trail)

**Time Accuracy Verification**:

- Compare system time with NTP server daily
- Alert if drift > 1 second
- Critical for order timestamps and audit compliance

**Time Sources**:

- Exchange timestamps (when provided by broker API)
- System clock (NTP-synced)
- Store both exchange time and system time in logs

**Time Handling**:

- **Storage**: Store all timestamps in UTC
- **Display**: Show in IST for operator
- **Comparison**: Convert to IST for market hours validation

## Appendix A. Angel One SmartAPI Endpoints and Rust Examples

### A.1 Cargo Dependencies

```toml
[package]
name = "smartapi-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json", "gzip", "brotli", "deflate", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time", "signal"] }
tokio-tungstenite = "0.21"
url = "2"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# windows credential manager example (optional)
keyring = "2"
```

### A.2 Auth: Session Generation (REST)

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct LoginRequest<'a> {
    clientcode: &'a str,
    password: &'a str,
    totp: &'a str,
}

#[derive(Deserialize)]
struct LoginResponse {
    data: LoginData,
}

#[derive(Deserialize)]
struct LoginData {
    jwtToken: String,
    feedToken: String,
}

pub async fn smartapi_login(client: &Client, url: &str, clientcode: &str, password: &str, totp: &str) -> anyhow::Result<(String, String)> {
    let req = LoginRequest { clientcode, password, totp };
    let resp = client.post(format!("{}/login", url)).json(&req).send().await?;
    let body: LoginResponse = resp.error_for_status()?.json().await?;
    Ok((body.data.jwtToken, body.data.feedToken))
}
```

### A.3 Quotes/LTP (REST)

```rust
pub async fn ltp(client: &Client, base: &str, jwt: &str, symbol: &str) -> anyhow::Result<f64> {
    let resp = client
        .get(format!("{}/ltp?symbol={}", base, symbol))
        .bearer_auth(jwt)
        .send()
        .await?;
    let v = resp.error_for_status()?.json::<serde_json::Value>().await?;
    Ok(v["data"]["ltp"].as_f64().unwrap_or(0.0))
}
```

### A.4 WebSocket Ticker

```rust
use tokio_tungstenite::connect_async;
use url::Url;

pub async fn connect_ws(ws_url: &str, feed_token: &str, clientcode: &str) -> anyhow::Result<()> {
    let url = Url::parse_with_params(ws_url, &[("feedToken", feed_token), ("clientcode", clientcode)])?;
    let (ws, _) = connect_async(url).await?;
    // subscribe message format depends on SmartAPI spec; send after connect
    // ws.send(Message::text("{\"action\":\"subscribe\",...}"));
    drop(ws);
    Ok(())
}
```

### A.5 Place Order (REST)

```rust
#[derive(Serialize)]
struct OrderRequest<'a> {
    symbol: &'a str,
    transactiontype: &'a str, // BUY/SELL
    ordertype: &'a str,       // LIMIT/MARKET
    producttype: &'a str,     // MIS/CNC/NRML
    quantity: i32,
    price: f64,
}

pub async fn place_order(client: &Client, base: &str, jwt: &str, req: &OrderRequest<'_>) -> anyhow::Result<String> {
    let resp = client
        .post(format!("{}/orders", base))
        .bearer_auth(jwt)
        .json(req)
        .send()
        .await?;
    let v = resp.error_for_status()?.json::<serde_json::Value>().await?;
    Ok(v["data"]["orderid"].as_str().unwrap_or("").to_string())
}
```

### A.6 Historical Candles

```rust
pub async fn candles(client: &Client, base: &str, jwt: &str, symbol: &str, interval: &str, from: &str, to: &str) -> anyhow::Result<Vec<[f64; 6]>> {
    let resp = client
        .get(format!("{}/historical?symbol={}&interval={}&from={}&to={}", base, symbol, interval, from, to))
        .bearer_auth(jwt)
        .send()
        .await?;
    let v = resp.error_for_status()?.json::<serde_json::Value>().await?;
    // adapt shape per SmartAPI response
    Ok(vec![])
}
```

### A.7 Config and Secrets

- Use env vars for `SMARTAPI_BASE_URL`, `SMARTAPI_WS_URL`, `SMARTAPI_CLIENT_CODE`, and store `password/TOTP` in OS vault (e.g., Windows Credential Manager via `keyring`).
- Never log secrets. Rotate credentials periodically.

### A.8 Retry, Backoff, and Idempotency

- Wrap REST calls with exponential backoff; treat HTTP 4xx validation errors as non-retry
- Persist a hash of order intent to dedupe retries and restarts
- For WS, implement heartbeats and jittered reconnects
