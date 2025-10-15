# Option Trading Bot - Complete Event-Driven Implementation Guide

**A fully event-driven architecture showing EXACTLY when to do what**

---

## Document Structure

This document organizes the entire system as **events** and **handlers**. Each event has:

- **Trigger**: What causes this event
- **Preconditions**: What must be true before handling
- **Handler Steps**: Detailed actions to take (labeled, not numbered)
- **Postconditions**: What's guaranteed after handling
- **Next Events**: What events this triggers

**Total Event Categories**: 12  
**Total Handler Steps**: 147 (all labeled for easy insertion)

---

## Table of Contents

1. [System Lifecycle Events](#1-system-lifecycle-events)
2. [Authentication Events](#2-authentication-events)
3. [Market Data Events](#3-market-data-events)
4. [Strategy Events](#4-strategy-events)
5. [Signal Events](#5-signal-events)
6. [Order Events](#6-order-events)
7. [Position Events](#7-position-events)
8. [Risk Events](#8-risk-events)
9. [Time-Based Events](#9-time-based-events)
10. [Error Events](#10-error-events)
11. [Data Management Events](#11-data-management-events)
12. [Monitoring Events](#12-monitoring-events)

---

## 1. System Lifecycle Events

### EVENT_SYSTEM_STARTUP

**Trigger**: Application process starts  
**Preconditions**: None (entry point)

**Handler Steps**:

**STEP_INIT_LOGGER**: Initialize logging system

- Load log configuration from `config.toml`
- Set log level based on environment (DEBUG/INFO/WARN/ERROR)
- Create log directory: `logs/`
- Initialize JSON structured logging
- Set log rotation: daily, keep 30 days
- **Trigger Next**: EVENT_LOG_INITIALIZED

**STEP_LOAD_CONFIG**: Load application configuration

- Read `config.toml` file
- Validate all required sections present
- Parse broker settings (URL, client code)
- Parse strategy parameters (ADX threshold, etc.)
- Parse risk parameters (position size, limits)
- Store config in global state
- **Trigger Next**: EVENT_CONFIG_LOADED

**STEP_INIT_STORAGE**: Initialize file storage

- Create directory structure:
  - `data/raw/`
  - `data/timeframes/{symbol}/`
  - `data/tokens/`
  - `state/`
  - `backups/`
- Verify write permissions
- Check disk space (minimum 1GB free)
- **Trigger Next**: EVENT_STORAGE_READY

**STEP_CHECK_TRADING_DAY**: Determine if today is a trading day

- Get current date and time (IST timezone)
- **Trigger Next**: EVENT_TRADING_DAY_CHECK

**Postconditions**:

- Logging active and working
- Configuration loaded and validated
- File system ready
- Trading day status determined

**Next Events**:

- If trading day → EVENT_TRADING_DAY_DETECTED
- If holiday/weekend → EVENT_NON_TRADING_DAY_DETECTED

---

### EVENT_TRADING_DAY_DETECTED

**Trigger**: STEP_CHECK_TRADING_DAY determines today is trading day  
**Preconditions**: System initialized, storage ready

**Handler Steps**:

**STEP_LOAD_HOLIDAY_CALENDAR**: Load NSE holiday calendar

- Check cache: `data/calendar/holidays_2024.json`
- If cache older than 30 days → fetch from NSE API
- Parse holiday dates
- Verify today NOT in holiday list
- **Trigger Next**: EVENT_CALENDAR_VALIDATED

**STEP_CHECK_MARKET_HOURS**: Determine current market session

- Get current time (IST)
- Determine session:
  - 00:00-09:00 → PRE_STARTUP (wait until 09:00)
  - 09:00-09:15 → PRE_MARKET
  - 09:15-15:30 → MARKET_OPEN
  - 15:30-16:00 → POST_MARKET
  - 16:00-23:59 → MARKET_CLOSED
- **Trigger Next**: EVENT_MARKET_SESSION_DETERMINED

**STEP_INIT_BROKER_CLIENT**: Initialize broker API client

- Load broker configuration (Angel One SmartAPI)
- Set base URLs (REST, WebSocket)
- Initialize HTTP client with timeout settings
- Prepare authentication headers
- **Trigger Next**: EVENT_BROKER_CLIENT_READY

**Postconditions**:

- Holiday calendar loaded and validated
- Current market session known
- Broker client initialized

**Next Events**:

- → EVENT_AUTH_TOKEN_CHECK_REQUIRED

---

### EVENT_NON_TRADING_DAY_DETECTED

**Trigger**: STEP_CHECK_TRADING_DAY determines today is NOT trading day  
**Preconditions**: System initialized

**Handler Steps**:

**STEP_LOG_NON_TRADING**: Log non-trading day

- Log reason (weekend or holiday name)
- Log next trading day

**STEP_ENTER_DATA_MGMT_MODE**: Enter data management mode

- Set mode: DATA_MANAGEMENT
- Disable live trading
- Enable background tasks
- **Trigger Next**: EVENT_DATA_MGMT_MODE_ACTIVE

**Postconditions**:

- System in data management mode
- Live trading disabled

**Next Events**:

- → EVENT_HISTORICAL_DATA_SYNC_REQUIRED
- → EVENT_DATA_GAP_DETECTION_REQUIRED
- → EVENT_BACKUP_REQUIRED

---

### EVENT_SYSTEM_SHUTDOWN

**Trigger**: User interrupt (Ctrl+C) or fatal error  
**Preconditions**: System running

**Handler Steps**:

**STEP_STOP_NEW_ORDERS**: Halt new order placement

- Set flag: ACCEPTING_ORDERS = false
- Cancel all pending order intents
- Log shutdown initiated

**STEP_CLOSE_OPEN_POSITIONS**: Close all open positions

- Get list of open positions
- For each position:
  - Create exit order (MARKET order)
  - Place order via broker API
  - Wait for fill confirmation (max 30 seconds)
- Log all closures with P&L
- **Trigger Next**: EVENT_POSITIONS_CLOSED

**STEP_DISCONNECT_WEBSOCKET**: Close WebSocket connections

- Unsubscribe all symbols
- Send close frame
- Wait for graceful close (max 5 seconds)
- Force close if timeout

**STEP_SAVE_STATE**: Save system state to disk

- Save current positions (should be empty)
- Save today's P&L
- Save indicator state
- Save last known prices
- Write to: `state/system_state_YYYYMMDD_HHMMSS.json`

**STEP_BACKUP_DATA**: Backup critical data

- Copy today's data to backup directory
- Compress if needed
- Verify backup integrity

**STEP_FINAL_LOG**: Write final log entry

- Log total runtime
- Log today's P&L
- Log total trades executed
- Log shutdown reason
- Close log file

**Postconditions**:

- All positions closed
- State saved to disk
- Data backed up
- Clean shutdown completed

**Next Events**: None (system terminates)

---

## 2. Authentication Events

### EVENT_AUTH_TOKEN_CHECK_REQUIRED

**Trigger**: Broker client initialized on trading day  
**Preconditions**: Broker client ready

**Handler Steps**:

**STEP_LOAD_STORED_TOKEN**: Attempt to load stored token

- Check OS credential store (Windows Credential Manager)
- Look for key: `angelone_jwt_token`
- If found → parse token and expiry time
- **Trigger Next**: EVENT_TOKEN_LOADED or EVENT_TOKEN_NOT_FOUND

**Postconditions**:

- Token loaded or not found determined

**Next Events**:

- If token found → EVENT_TOKEN_VALIDATION_REQUIRED
- If not found → EVENT_AUTH_LOGIN_REQUIRED

---

### EVENT_TOKEN_VALIDATION_REQUIRED

**Trigger**: Stored token loaded  
**Preconditions**: Token exists in memory

**Handler Steps**:

**STEP_CHECK_TOKEN_EXPIRY**: Check token expiration

- Get current time
- Get token expiry time
- Calculate time until expiry
- If expires before 3:30 PM today → invalid
- If already expired → invalid
- **Trigger Next**: EVENT_TOKEN_EXPIRY_CHECKED

**STEP_VALIDATE_WITH_BROKER**: Test token with broker API

- Make test API call: GET /profile
- Include token in Authorization header
- Check response status
- If 200 OK → token valid
- If 401 Unauthorized → token invalid
- If other error → retry once
- **Trigger Next**: EVENT_TOKEN_BROKER_VALIDATED

**Postconditions**:

- Token validity confirmed or rejected

**Next Events**:

- If valid → EVENT_AUTH_SUCCESS
- If invalid → EVENT_AUTH_LOGIN_REQUIRED

---

### EVENT_AUTH_LOGIN_REQUIRED

**Trigger**: No token or invalid token  
**Preconditions**: Broker client initialized

**Handler Steps**:

**STEP_LOAD_CREDENTIALS**: Load broker credentials

- Read client code from config
- Read password from OS credential store
- Read TOTP secret from OS credential store
- Validate all credentials present
- **Trigger Next**: EVENT_CREDENTIALS_LOADED

**STEP_GENERATE_TOTP**: Generate TOTP code

- Use TOTP secret
- Generate 6-digit code
- Validate code format

**STEP_CALL_LOGIN_API**: Call broker login API

- Endpoint: POST /login
- Body: { clientcode, password, totp }
- Send request
- Parse response
- Extract: jwtToken, feedToken, refreshToken
- **Trigger Next**: EVENT_LOGIN_API_CALLED

**STEP_STORE_TOKENS**: Store tokens securely

- Save jwtToken to OS credential store
- Save feedToken to OS credential store
- Save expiry time (from response or 24 hours)
- Log successful authentication (no sensitive data)
- **Trigger Next**: EVENT_TOKENS_STORED

**Postconditions**:

- Valid tokens obtained and stored
- Authentication complete

**Next Events**:

- → EVENT_AUTH_SUCCESS

---

### EVENT_AUTH_SUCCESS

**Trigger**: Token validated or login succeeded  
**Preconditions**: Valid jwtToken and feedToken available

**Handler Steps**:

**STEP_SET_AUTH_STATUS**: Update authentication status

- Set global flag: AUTHENTICATED = true
- Store token expiry time
- Log authentication success

**STEP_INIT_RATE_LIMITER**: Initialize API rate limiter

- Create rate limiter for orders: 10 req/sec
- Create rate limiter for market data: 3 req/sec
- Create rate limiter for historical: 3 req/sec

**STEP_INIT_TOKEN_MONITOR**: Start token monitoring

- Schedule token check every 5 minutes
- Alert if token expires in <30 minutes
- **Trigger Next**: EVENT_TOKEN_MONITOR_ACTIVE

**Postconditions**:

- System authenticated with broker
- Rate limiters active
- Token monitoring active

**Next Events**:

- → EVENT_INSTRUMENT_MASTER_LOAD_REQUIRED

---

### EVENT_AUTH_TOKEN_EXPIRY_WARNING

**Trigger**: Token monitor detects expiry <30 minutes  
**Preconditions**: System running, token expiring soon

**Handler Steps**:

**STEP_LOG_TOKEN_WARNING**: Log token expiry warning

- Log time until expiry
- Log that trading will pause if not refreshed

**STEP_ATTEMPT_TOKEN_REFRESH**: Try to refresh token

- If refresh API available → call refresh endpoint
- If no refresh API → **Trigger Next**: EVENT_AUTH_LOGIN_REQUIRED

**STEP_NOTIFY_USER**: Send notification to user

- Email/SMS: "Token expiring in X minutes"
- Include action required

**Postconditions**:

- User notified
- Refresh attempted if possible

**Next Events**:

- If refresh success → EVENT_AUTH_SUCCESS
- If refresh failed → EVENT_AUTH_LOGIN_REQUIRED

---

## 3. Market Data Events

### EVENT_INSTRUMENT_MASTER_LOAD_REQUIRED

**Trigger**: Authentication successful  
**Preconditions**: Authenticated with broker

**Handler Steps**:

**STEP_CHECK_CACHED_MASTER**: Check for cached instrument master

- Look for: `data/tokens/token_map_YYYYMMDD.json`
- Check if file date = today
- If today's file exists and valid → use cached
- Else → download new file
- **Trigger Next**: EVENT_CACHE_CHECK_COMPLETE

**STEP_DOWNLOAD_INSTRUMENT_MASTER**: Download from broker

- URL: `https://margincalculator.angelbroking.com/OpenAPI_File/files/OpenAPIScripMaster.json`
- Download CSV/JSON file
- Validate file size > 0
- Validate JSON/CSV format
- **Trigger Next**: EVENT_MASTER_DOWNLOADED

**STEP_PARSE_INSTRUMENT_MASTER**: Parse instrument data

- Read all rows/records
- Filter: exch_seg == "NFO"
- Filter: instrumenttype IN ["OPTIDX", "OPTSTK"]
- Filter: name IN ["NIFTY", "BANKNIFTY", "FINNIFTY"]
- For each row:
  - Extract: token, symbol, name, expiry, strike, lot_size, tick_size
  - Identify option type (CE or PE) from symbol suffix
  - Skip if neither CE nor PE
- **Trigger Next**: EVENT_MASTER_PARSED

**STEP_BUILD_TOKEN_MAP**: Build token lookup map

- Create map: (underlying, expiry, strike, type) → TokenInfo
- For each parsed instrument:
  - key = (name, expiry_date, strike, "CE" or "PE")
  - value = { token, symbol, lot_size, tick_size }
- **Trigger Next**: EVENT_TOKEN_MAP_BUILT

**STEP_VALIDATE_TOKEN_MAP**: Validate token map completeness

- For each underlying (NIFTY, BANKNIFTY, FINNIFTY):
  - Check minimum strikes available (>100)
  - Check both CE and PE exist for each strike
  - Check current ATM range covered
- If validation fails → log warning
- **Trigger Next**: EVENT_TOKEN_MAP_VALIDATED

**STEP_SAVE_TOKEN_MAP**: Save token map to file

- File: `data/tokens/token_map_YYYYMMDD.json`
- Format: JSON
- Compress old files (>7 days) to .gz
- Delete files older than 30 days

**Postconditions**:

- Token map loaded and ready
- Strike → Token lookup available

**Next Events**:

- → EVENT_WEBSOCKET_CONNECT_REQUIRED

---

### EVENT_WEBSOCKET_CONNECT_REQUIRED

**Trigger**: Token map loaded  
**Preconditions**: Authenticated, token map ready

**Handler Steps**:

**STEP_INIT_WEBSOCKET_CLIENT**: Initialize WebSocket client

- URL: `wss://smartapisocket.angelbroking.com`
- Set connection timeout: 10 seconds
- Set ping interval: 30 seconds
- Prepare authentication payload

**STEP_CONNECT_WEBSOCKET**: Connect to broker WebSocket

- Open WebSocket connection
- Wait for connection established
- Send authentication message with feedToken
- Wait for auth confirmation
- **Trigger Next**: EVENT_WEBSOCKET_CONNECTED

**STEP_SETUP_WEBSOCKET_HANDLERS**: Register event handlers

- On message → **Trigger**: EVENT_WEBSOCKET_TICK_RECEIVED
- On error → **Trigger**: EVENT_WEBSOCKET_ERROR
- On close → **Trigger**: EVENT_WEBSOCKET_DISCONNECTED
- On ping → send pong

**STEP_START_HEARTBEAT**: Start WebSocket heartbeat

- Send ping every 30 seconds
- Track last pong received
- If no pong for 90 seconds → reconnect

**Postconditions**:

- WebSocket connected to broker
- Heartbeat monitoring active
- Ready to subscribe to symbols

**Next Events**:

- → EVENT_MARKET_SESSION_DETERMINED (re-evaluate)

---

### EVENT_WEBSOCKET_TICK_RECEIVED

**Trigger**: WebSocket receives market data message  
**Preconditions**: WebSocket connected

**Handler Steps**:

**STEP_PARSE_TICK_MESSAGE**: Parse incoming tick

- Decode message format (JSON or binary)
- Extract: symbol/token, ltp, bid, ask, volume, timestamp
- Validate required fields present

**STEP_UPDATE_LAST_TICK_TIME**: Update gap detector

- Record current time for this symbol
- **Trigger Next**: EVENT_TICK_TIMESTAMP_UPDATED

**STEP_UPDATE_PRICE_CACHE**: Update in-memory price cache

- Store latest price for symbol
- Update bid/ask spread
- Store volume

**STEP_CHECK_POSITION_UPDATES**: Check if affects open positions

- If symbol is an open position → **Trigger**: EVENT_POSITION_PRICE_UPDATE
- Calculate new P&L
- Check stop-loss and target

**STEP_STORE_TICK_TO_BUFFER**: Add tick to aggregation buffer

- Add to buffer for 1-minute bar construction
- If buffer full or time boundary → **Trigger**: EVENT_BAR_READY

**Postconditions**:

- Tick processed and stored
- Price cache updated
- Positions updated if applicable

**Next Events**:

- If position affected → EVENT_POSITION_PRICE_UPDATE
- If bar ready → EVENT_BAR_READY

---

### EVENT_WEBSOCKET_DISCONNECTED

**Trigger**: WebSocket connection lost  
**Preconditions**: Was previously connected

**Handler Steps**:

**STEP_LOG_DISCONNECT**: Log disconnection

- Log timestamp
- Log last successful message received
- Log duration of connection

**STEP_MARK_DATA_GAP**: Mark potential data gap

- Record disconnection time
- Prepare to fill gap when reconnected

**STEP_SCHEDULE_RECONNECT**: Schedule reconnection attempt

- Wait: 1 second (first attempt)
- Exponential backoff: 2s, 4s, 8s, 16s, 30s (max)
- Max attempts: 10 per minute
- **Trigger Next**: EVENT_WEBSOCKET_RECONNECT_SCHEDULED

**Postconditions**:

- Disconnection logged
- Reconnection scheduled

**Next Events**:

- After delay → EVENT_WEBSOCKET_CONNECT_REQUIRED
- If reconnect fails → EVENT_CRITICAL_ERROR

---

### EVENT_BAR_READY

**Trigger**: Enough ticks accumulated to form a bar  
**Preconditions**: Tick buffer has data

**Handler Steps**:

**STEP_CONSTRUCT_BAR**: Build OHLCV bar from ticks

- Timeframe: 1-minute
- Open: first tick LTP
- High: max tick LTP
- Low: min tick LTP
- Close: last tick LTP
- Volume: sum of volumes
- Timestamp: bar end time

**STEP_VALIDATE_BAR**: Validate bar data

- Check: high >= low
- Check: open within [low, high]
- Check: close within [low, high]
- Check: all values > 0
- Check: volume >= 0
- If validation fails → discard bar and log warning

**STEP_STORE_BAR**: Store bar to file

- File: `data/timeframes/{symbol}/1m.json`
- Append bar to array
- Keep only last 3 months of 1m bars
- **Trigger Next**: EVENT_BAR_STORED

**STEP_TRIGGER_HIGHER_TIMEFRAME**: Check if higher timeframe ready

- If 5 minutes passed → aggregate to 5m bar
- If 15 minutes passed → aggregate to 15m bar
- If 60 minutes passed → aggregate to 1h bar
- **Trigger Next**: EVENT_HIGHER_TIMEFRAME_BAR_READY

**Postconditions**:

- 1-minute bar constructed and stored
- Higher timeframes triggered if needed

**Next Events**:

- Every 1m → EVENT_INDICATOR_CALCULATION_REQUIRED
- Every 1h → EVENT_HOURLY_ANALYSIS_REQUIRED

---

## 4. Strategy Events

### EVENT_DAILY_ANALYSIS_REQUIRED

**Trigger**: Market opens (9:15 AM) on trading day  
**Preconditions**: Authenticated, data available

**Handler Steps**:

**STEP_LOAD_DAILY_BARS**: Load historical daily data

- Source: `data/timeframes/NIFTY/daily.json`
- Load last 30 daily bars
- Validate: all bars complete, no gaps
- **Trigger Next**: EVENT_DAILY_DATA_LOADED

**STEP_CALCULATE_DAILY_ADX**: Calculate daily ADX

- Period: 14
- Calculate: True Range (TR)
- Calculate: +DM, -DM
- Apply Wilder's smoothing
- Calculate: +DI, -DI
- Calculate: DX
- Calculate: ADX (14-period average of DX)
- **Trigger Next**: EVENT_DAILY_ADX_CALCULATED

**STEP_DETERMINE_DAILY_DIRECTION**: Determine trend direction

- If daily_adx < 25 → direction = NO_TRADE
- Else if daily_plus_di > daily_minus_di → direction = CE
- Else if daily_minus_di > daily_plus_di → direction = PE
- Else → direction = NO_TRADE
- **Trigger Next**: EVENT_DAILY_DIRECTION_SET

**STEP_STORE_DAILY_DIRECTION**: Store direction for day

- Set global: DAILY_DIRECTION = CE/PE/NO_TRADE
- Log daily direction and reason
- Save to state file
- **Trigger Next**: EVENT_DAILY_DIRECTION_STORED

**STEP_NOTIFY_DAILY_DIRECTION**: Send notification

- Email/SMS: "Today's direction: CE/PE/NO_TRADE"
- Include ADX value and DI values
- Include reasoning

**Postconditions**:

- Daily direction determined
- DAILY_DIRECTION set for entire day
- User notified

**Next Events**:

- If NO_TRADE → no further action (wait for next day)
- If CE or PE → EVENT_HOURLY_ANALYSIS_REQUIRED (continuous)

---

### EVENT_HOURLY_ANALYSIS_REQUIRED

**Trigger**: Hourly bar closes (10:15, 11:15, 12:15, etc.)  
**Preconditions**: Daily direction set, hourly data available

**Handler Steps**:

**STEP_LOAD_HOURLY_BARS**: Load hourly bar data

- Source: `data/timeframes/NIFTY/1h.json`
- Load last 30 hourly bars
- Validate: all bars complete
- **Trigger Next**: EVENT_HOURLY_DATA_LOADED

**STEP_CALCULATE_HOURLY_ADX**: Calculate hourly ADX

- Same calculation as daily, but on hourly bars
- Period: 14
- Calculate: ADX, +DI, -DI
- **Trigger Next**: EVENT_HOURLY_ADX_CALCULATED

**STEP_CHECK_HOURLY_ALIGNMENT**: Check alignment with daily

- Get: daily_direction (from STEP_STORE_DAILY_DIRECTION)
- If daily_direction == CE:
  - Check: hourly_adx >= 25 AND hourly_plus_di > hourly_minus_di
  - If yes → aligned = true
  - Else → aligned = false
- If daily_direction == PE:
  - Check: hourly_adx >= 25 AND hourly_minus_di > hourly_plus_di
  - If yes → aligned = true
  - Else → aligned = false
- **Trigger Next**: EVENT_HOURLY_ALIGNMENT_CHECKED

**STEP_HANDLE_ALIGNMENT_CHANGE**: Handle alignment status

- If aligned AND was_not_aligned_before:
  - Log: "Hourly now aligned with daily"
  - **Trigger Next**: EVENT_ENTRY_SIGNAL_SEARCH_REQUIRED
- If NOT aligned AND was_aligned_before:
  - Log: "Hourly conflict with daily"
  - **Trigger Next**: EVENT_EXIT_REQUIRED (if in position)
- If NOT aligned AND NOT in_position:
  - Log: "Waiting for alignment"
  - No action (wait)

**Postconditions**:

- Hourly alignment status determined
- Position actions triggered if needed

**Next Events**:

- If aligned + no position → EVENT_ENTRY_SIGNAL_SEARCH_REQUIRED
- If not aligned + in position → EVENT_EXIT_SIGNAL_GENERATED
- If not aligned + no position → wait (no events)

---

## 5. Signal Events

### EVENT_ENTRY_SIGNAL_SEARCH_REQUIRED

**Trigger**: Hourly aligned with daily, no position open  
**Preconditions**: Daily direction set, hourly aligned, position count < max

**Handler Steps**:

**STEP_CHECK_ENTRY_FILTERS**: Validate all pre-entry filters

- Filter 1: Time between 10:00 AM - 2:30 PM
- Filter 2: DAILY_DIRECTION != NO_TRADE
- Filter 3: Hourly aligned (already checked)
- Filter 4: Position count < max_positions (3)
- Filter 5: VIX < 30
- Filter 6: Current volume > 120% average
- If ALL pass → continue
- If ANY fail → log reason and skip entry
- **Trigger Next**: EVENT_ENTRY_FILTERS_PASSED or EVENT_ENTRY_FILTERS_FAILED

**STEP_CHECK_ENTRY_TRIGGERS**: Check for entry trigger

- For CE (bullish):
  - Trigger A: Price breaks above 1h high with volume
  - Trigger B: 5m RSI < 40 and bounces off 9-EMA
  - Trigger C: +DI crosses above -DI on hourly
- For PE (bearish):
  - Trigger A: Price breaks below 1h low with volume
  - Trigger B: 5m RSI > 60 and rejects from 9-EMA
  - Trigger C: -DI crosses above +DI on hourly
- If ANY trigger detected → **Trigger Next**: EVENT_ENTRY_TRIGGER_DETECTED
- If NO trigger → wait (check again next minute)

**Postconditions**:

- Entry opportunity identified or not found

**Next Events**:

- If trigger detected → EVENT_ENTRY_SIGNAL_GENERATED
- If no trigger → wait, recheck every minute

---

### EVENT_ENTRY_SIGNAL_GENERATED

**Trigger**: Entry trigger detected and filters passed  
**Preconditions**: Entry conditions met

**Handler Steps**:

**STEP_LOG_ENTRY_SIGNAL**: Log signal details

- Log: direction (CE/PE), trigger type, price, time
- Log: ADX values (daily and hourly)
- Log: all filter values

**STEP_CALCULATE_ATM_STRIKE**: Calculate ATM strike

- Get current underlying price (NIFTY LTP)
- Strike increment: 50 (NIFTY)
- ATM = ROUND(LTP / 50) \* 50
- Example: LTP 23,456 → ATM 23,450
- **Trigger Next**: EVENT_ATM_STRIKE_CALCULATED

**STEP_LOOKUP_OPTION_TOKEN**: Get option token from map

- Underlying: "NIFTY"
- Strike: ATM strike (from previous step)
- Option type: CE or PE (from DAILY_DIRECTION)
- Expiry: current week expiry
- Lookup in token map
- Get: token, symbol, lot_size, tick_size
- **Trigger Next**: EVENT_OPTION_TOKEN_FOUND

**STEP_VALIDATE_OPTION_LIQUIDITY**: Validate option is tradeable

- Fetch quote from broker API
- Check: LTP > 0
- Check: Volume > 0
- Check: OI > 1000
- If ALL pass → continue
- If ANY fail → try ATM ± 1 strike
- **Trigger Next**: EVENT_OPTION_VALIDATED

**STEP_CALCULATE_POSITION_SIZE**: Calculate position size

- Get account balance
- Get current VIX
- Get days to expiry
- Get OI
- Get current position count
- Call position sizing algorithm:
  - Base: 2% of account
  - Adjust for VIX (multiplier 0.25 to 1.25)
  - Adjust for expiry (multiplier 0.50 to 1.00)
  - Adjust for OI (multiplier 0.0 to 1.00)
  - Adjust for existing positions (multiplier 0.60 to 1.00)
- Convert to number of lots
- **Trigger Next**: EVENT_POSITION_SIZE_CALCULATED

**Postconditions**:

- Entry signal fully validated
- Strike and token identified
- Position size calculated

**Next Events**:

- → EVENT_ORDER_PLACEMENT_REQUIRED

---

### EVENT_EXIT_SIGNAL_GENERATED

**Trigger**: Exit condition detected  
**Preconditions**: Position open

**Handler Steps**:

**STEP_DETERMINE_EXIT_REASON**: Identify exit reason (priority order)

- Priority 1: Mandatory exits
  - Time >= 3:20 PM → reason = "MARKET_CLOSE"
  - Days to expiry <= 3 → reason = "EXPIRY_NEAR"
  - Daily loss >= 3% → reason = "DAILY_LOSS_LIMIT"
  - VIX spike > 5 points in 10 min → reason = "VIX_SPIKE"
  - Token expiry soon → reason = "TOKEN_EXPIRY"
- Priority 2: Risk exits
  - Stop loss hit → reason = "STOP_LOSS"
  - Trailing stop hit → reason = "TRAILING_STOP"
  - Margin > 80% → reason = "MARGIN_WARNING"
- Priority 3: Profit exits
  - Target reached → reason = "TARGET_REACHED"
  - Partial profit (1:1) → reason = "PARTIAL_PROFIT"
- Priority 4: Technical exits
  - Hourly conflicts daily → reason = "HOURLY_CONFLICT"
  - Volume < 50% avg for 15min → reason = "LOW_VOLUME"
- Priority 5: Time exits
  - Held > 2 hours, no profit → reason = "MAX_HOLD_TIME"
  - Held > 1 hour, negative P&L → reason = "TIME_DECAY"
- **Trigger Next**: EVENT_EXIT_REASON_DETERMINED

**STEP_LOG_EXIT_SIGNAL**: Log exit details

- Log: reason, position details, current P&L
- Log: entry time, exit time, duration held

**STEP_CALCULATE_EXIT_PRICE**: Determine exit order type

- If reason in [MARKET_CLOSE, DAILY_LOSS_LIMIT, VIX_SPIKE, TOKEN_EXPIRY]:
  - Order type: MARKET (exit immediately)
- Else:
  - Order type: LIMIT (0.5% from LTP)

**Postconditions**:

- Exit reason determined
- Exit order type decided

**Next Events**:

- → EVENT_ORDER_PLACEMENT_REQUIRED (for exit)

---

## 6. Order Events

### EVENT_ORDER_PLACEMENT_REQUIRED

**Trigger**: Entry signal generated OR exit signal generated  
**Preconditions**: Order parameters ready

**Handler Steps**:

**STEP_GENERATE_ORDER_INTENT**: Create order intent object

- Intent ID: UUID
- Symbol: option symbol
- Direction: BUY (entry) or SELL (exit)
- Quantity: number of lots × lot_size
- Order type: LIMIT or MARKET
- Limit price: calculated from LTP
- Timestamp: current time
- Reason: entry trigger or exit reason

**STEP_CALCULATE_IDEMPOTENCY_KEY**: Generate idempotency key

- Hash: SHA256(symbol + direction + quantity + timestamp + reason)
- Check if this hash already exists in order log
- If exists → duplicate order, reject
- If not exists → continue

**STEP_RUN_PRE_ORDER_VALIDATION**: Validate order against all checks

- Check 1: Position limit (< max_positions)
- Check 2: Freeze quantity (quantity <= freeze_qty)
- Check 3: Price band (price within ±20% of LTP)
- Check 4: Lot size multiple (quantity % lot_size == 0)
- Check 5: Tick size (price % 0.05 == 0)
- Check 6: Margin available (margin >= required)
- Check 7: Daily loss limit (loss < 3%)
- Check 8: VIX circuit breaker (VIX < 30)
- Check 9: Market hours (time between 9:15 AM - 2:30 PM for entry)
- If ALL pass → **Trigger Next**: EVENT_PRE_ORDER_VALIDATION_PASSED
- If ANY fail → **Trigger Next**: EVENT_PRE_ORDER_VALIDATION_FAILED

**Postconditions**:

- Order validated or rejected
- Idempotency ensured

**Next Events**:

- If validation passed → EVENT_ORDER_SUBMISSION_REQUIRED
- If validation failed → EVENT_ORDER_REJECTED

---

### EVENT_ORDER_SUBMISSION_REQUIRED

**Trigger**: Pre-order validation passed  
**Preconditions**: Order intent validated

**Handler Steps**:

**STEP_ACQUIRE_RATE_LIMIT**: Wait for rate limit availability

- Call: rate_limiter.acquire()
- If available → proceed
- If limit exceeded → wait and retry

**STEP_GET_CURRENT_LTP**: Fetch current LTP

- Call broker API: GET /quotes
- Get latest price for symbol
- Use for limit price calculation

**STEP_CALCULATE_LIMIT_PRICE**: Calculate limit order price

- For BUY: limit_price = LTP × 1.005 (0.5% above)
- For SELL: limit_price = LTP × 0.995 (0.5% below)
- Round to tick size: round(price / 0.05) × 0.05

**STEP_CREATE_BROKER_ORDER**: Build order request

- Order object:
  - symbol: option symbol
  - transactiontype: BUY or SELL
  - ordertype: LIMIT (or MARKET for emergency exits)
  - producttype: MIS (intraday)
  - quantity: total quantity
  - price: limit_price
- **Trigger Next**: EVENT_ORDER_REQUEST_CREATED

**STEP_SUBMIT_ORDER_TO_BROKER**: Send order to broker

- API: POST /orders
- Include: jwtToken in Authorization header
- Send order request
- Handle response
- Extract: broker_order_id
- **Trigger Next**: EVENT_ORDER_SUBMITTED_TO_BROKER

**STEP_STORE_ORDER_MAPPING**: Store order tracking

- Save: idempotency_hash → broker_order_id
- Save: order_intent with broker_order_id
- Write to: `state/orders_YYYYMMDD.json`
- **Trigger Next**: EVENT_ORDER_STORED

**STEP_SCHEDULE_FILL_MONITOR**: Start monitoring for fill

- Spawn async task to monitor order status
- Check every 1 second
- Timeout: 60 seconds
- **Trigger Next**: EVENT_FILL_MONITOR_ACTIVE

**Postconditions**:

- Order submitted to broker
- Order ID received
- Fill monitoring started

**Next Events**:

- After 0-60 seconds → EVENT_ORDER_FILLED or EVENT_ORDER_TIMEOUT

---

### EVENT_ORDER_FILLED

**Trigger**: Broker confirms order filled  
**Preconditions**: Order was submitted

**Handler Steps**:

**STEP_FETCH_FILL_DETAILS**: Get fill information from broker

- API: GET /orders/{order_id}
- Extract: fill_price, fill_quantity, fill_time
- Validate: fill_quantity == order_quantity

**STEP_CALCULATE_SLIPPAGE**: Calculate execution slippage

- Expected price: limit_price (from order)
- Actual price: fill_price
- Slippage: (fill_price - expected_price) / expected_price
- If slippage > 2% → log warning
- **Trigger Next**: EVENT_SLIPPAGE_CALCULATED

**STEP_UPDATE_POSITION**: Update position tracking

- If BUY (entry):
  - Create new position
  - **Trigger Next**: EVENT_POSITION_OPENED
- If SELL (exit):
  - Close existing position
  - **Trigger Next**: EVENT_POSITION_CLOSED

**STEP_LOG_FILL**: Log order fill

- Log: order_id, symbol, direction, quantity, price
- Log: slippage, fill_time
- Log: reason (entry trigger or exit reason)

**Postconditions**:

- Order confirmed filled
- Position updated
- Fill logged

**Next Events**:

- If entry → EVENT_POSITION_OPENED
- If exit → EVENT_POSITION_CLOSED

---

### EVENT_ORDER_TIMEOUT

**Trigger**: Order not filled within 60 seconds  
**Preconditions**: Order submitted but not filled

**Handler Steps**:

**STEP_LOG_ORDER_TIMEOUT**: Log timeout

- Log: order_id, symbol, time elapsed

**STEP_CANCEL_ORDER**: Cancel pending order

- API: DELETE /orders/{order_id}
- Confirm cancellation
- **Trigger Next**: EVENT_ORDER_CANCELLED

**STEP_DECIDE_RETRY**: Decide whether to retry

- If MARKET order → don't retry (emergency exit failed)
- If LIMIT entry order → retry with adjusted price
- If LIMIT exit order → retry with market order

**Postconditions**:

- Order cancelled
- Retry decision made

**Next Events**:

- If retry → EVENT_ORDER_PLACEMENT_REQUIRED (with adjusted params)
- If no retry → EVENT_ORDER_REJECTED

---

### EVENT_ORDER_REJECTED

**Trigger**: Order validation failed or broker rejected  
**Preconditions**: Order attempted

**Handler Steps**:

**STEP_LOG_REJECTION**: Log rejection details

- Log: rejection reason
- Log: order details
- Log: validation failures

**STEP_NOTIFY_REJECTION**: Send notification

- Alert: "Order rejected: [reason]"
- Include: symbol, direction, reason

**STEP_CLEANUP_ORDER_STATE**: Clean up order tracking

- Remove order intent from pending list
- Mark order as rejected in log

**Postconditions**:

- Rejection logged
- User notified
- State cleaned up

**Next Events**: None (order flow ends)

---

## 7. Position Events

### EVENT_POSITION_OPENED

**Trigger**: Entry order filled  
**Preconditions**: BUY order confirmed filled

**Handler Steps**:

**STEP_CREATE_POSITION_RECORD**: Create position object

- Position ID: UUID
- Symbol: option symbol
- Direction: CE or PE
- Entry price: fill_price
- Entry time: fill_time
- Quantity: filled_quantity
- Underlying entry price: current NIFTY LTP
- **Trigger Next**: EVENT_POSITION_RECORD_CREATED

**STEP_CALCULATE_STOP_LOSS**: Calculate stop loss

- For CE: stop_loss = underlying_entry \* (1 - 0.01) [1% below]
- For PE: stop_loss = underlying_entry \* (1 + 0.01) [1% above]
- Store stop loss price

**STEP_CALCULATE_TARGET**: Calculate profit target

- For CE: target = underlying_entry \* (1 + 0.03) [3% above]
- For PE: target = underlying_entry \* (1 - 0.03) [3% below]
- Store target price

**STEP_INITIALIZE_TRACKING**: Initialize position tracking

- P&L: 0.0
- Current price: entry_price
- High since entry: entry_price
- Low since entry: entry_price
- Time held: 0

**STEP_SAVE_POSITION**: Save position to state

- Write to: `state/positions.json`
- Add to open_positions list

**STEP_LOG_POSITION_OPEN**: Log position details

- Log: all position details
- Log: stop loss and target levels
- Log: entry reason

**STEP_NOTIFY_POSITION_OPEN**: Send notification

- Alert: "Position opened: [symbol] @ [price]"
- Include: direction, stop loss, target

**Postconditions**:

- Position created and tracked
- Stop loss and target set
- User notified

**Next Events**:

- Every tick → EVENT_POSITION_PRICE_UPDATE
- Continuous → EVENT_POSITION_MONITORING_REQUIRED

---

### EVENT_POSITION_PRICE_UPDATE

**Trigger**: Tick received for symbol with open position  
**Preconditions**: Position open, new price available

**Handler Steps**:

**STEP_UPDATE_POSITION_PRICE**: Update current price

- Set: position.current_price = new_ltp
- Update: position.high = max(high, new_ltp)
- Update: position.low = min(low, new_ltp)

**STEP_CALCULATE_CURRENT_PNL**: Calculate P&L

- For long: pnl = (current_price - entry_price) × quantity
- Update: position.pnl = pnl
- Update: position.pnl_pct = pnl / (entry_price × quantity)

**STEP_UPDATE_TRAILING_STOP**: Update trailing stop if applicable

- If pnl_pct > 2% (1:2 risk-reward):
  - trailing_stop = current_price × 0.985 (1.5% below)
  - Update: position.trailing_stop = trailing_stop

**Postconditions**:

- Position price and P&L updated
- Trailing stop adjusted

**Next Events**:

- → EVENT_POSITION_MONITORING_REQUIRED

---

### EVENT_POSITION_MONITORING_REQUIRED

**Trigger**: Position price updated  
**Preconditions**: Position open, prices current

**Handler Steps**:

**STEP_CHECK_STOP_LOSS**: Check if stop loss hit

- Get current underlying price (NIFTY LTP)
- For CE: if underlying < stop_loss → **Trigger**: EVENT_EXIT_SIGNAL_GENERATED
- For PE: if underlying > stop_loss → **Trigger**: EVENT_EXIT_SIGNAL_GENERATED

**STEP_CHECK_TARGET**: Check if target reached

- Get current underlying price
- For CE: if underlying >= target → **Trigger**: EVENT_EXIT_SIGNAL_GENERATED
- For PE: if underlying <= target → **Trigger**: EVENT_EXIT_SIGNAL_GENERATED

**STEP_CHECK_TRAILING_STOP**: Check trailing stop if active

- If trailing_stop set:
  - If current_price < trailing_stop → **Trigger**: EVENT_EXIT_SIGNAL_GENERATED

**STEP_CHECK_TIME_LIMITS**: Check time-based exits

- Time held: current_time - entry_time
- If held > 2 hours AND pnl <= 0 → **Trigger**: EVENT_EXIT_SIGNAL_GENERATED
- If held > 1 hour AND pnl < 0 → **Trigger**: EVENT_EXIT_SIGNAL_GENERATED

**Postconditions**:

- All exit conditions checked
- Exit triggered if condition met

**Next Events**:

- If exit condition met → EVENT_EXIT_SIGNAL_GENERATED
- Else → continue monitoring (wait for next price update)

---

### EVENT_POSITION_CLOSED

**Trigger**: Exit order filled  
**Preconditions**: SELL order confirmed filled, position exists

**Handler Steps**:

**STEP_FETCH_POSITION_RECORD**: Get position details

- Lookup position by symbol
- Get all entry details

**STEP_CALCULATE_FINAL_PNL**: Calculate final P&L

- Final P&L: (exit_price - entry_price) × quantity
- P&L %: pnl / (entry_price × quantity) × 100
- Duration: exit_time - entry_time

**STEP_UPDATE_DAILY_PNL**: Update today's total P&L

- Add position P&L to today's total
- Update: daily_pnl_total

**STEP_LOG_TRADE_RESULT**: Log complete trade details

- Log: entry_time, entry_price
- Log: exit_time, exit_price, exit_reason
- Log: pnl, pnl_pct, duration
- Log: high, low during hold
- Write to: `logs/trades_YYYYMMDD.json`

**STEP_REMOVE_FROM_OPEN_POSITIONS**: Clean up position tracking

- Remove from open_positions list
- Archive to closed_positions list
- Update positions.json

**STEP_NOTIFY_POSITION_CLOSED**: Send notification

- Alert: "Position closed: [symbol]"
- Include: P&L, reason, duration

**STEP_CHECK_DAILY_LIMITS**: Check if daily limits hit

- If daily_pnl < -3% of account → **Trigger**: EVENT_DAILY_LOSS_LIMIT_HIT
- If consecutive_losses >= 3 → **Trigger**: EVENT_CONSECUTIVE_LOSS_LIMIT_HIT

**Postconditions**:

- Position closed and archived
- P&L calculated and logged
- Daily limits checked
- User notified

**Next Events**:

- If limits hit → EVENT_TRADING_HALT_REQUIRED
- Else → ready for new entry (if aligned)

---

## 8. Risk Events

### EVENT_CIRCUIT_BREAKER_CHECK_REQUIRED

**Trigger**: Every 1 minute during trading  
**Preconditions**: Market open, system trading

**Handler Steps**:

**STEP_CHECK_VIX_BREAKER**: Check VIX circuit breaker

- Get current VIX
- Get VIX from 10 minutes ago
- If VIX > 30 → **Trigger**: EVENT_VIX_CIRCUIT_BREAKER_TRIGGERED
- If VIX_change > 5 in 10 minutes → **Trigger**: EVENT_VIX_CIRCUIT_BREAKER_TRIGGERED

**STEP_CHECK_FLASH_SPIKE_BREAKER**: Check flash spike breaker

- Get underlying price now
- Get underlying price from 5 minutes ago
- Change %: (now - 5min_ago) / 5min_ago
- If |change| > 2% → **Trigger**: EVENT_FLASH_SPIKE_BREAKER_TRIGGERED

**STEP_CHECK_MARGIN_BREAKER**: Check margin utilization

- Get margin used
- Get margin available
- Utilization: used / (used + available)
- If utilization > 80% → **Trigger**: EVENT_MARGIN_BREAKER_TRIGGERED

**Postconditions**:

- All circuit breakers checked
- Breakers triggered if thresholds exceeded

**Next Events**:

- If any breaker triggered → corresponding EVENT\_\*\_BREAKER_TRIGGERED
- Else → no action

---

### EVENT_VIX_CIRCUIT_BREAKER_TRIGGERED

**Trigger**: VIX exceeds threshold  
**Preconditions**: VIX > 30 or spike > 5 points

**Handler Steps**:

**STEP_LOG_VIX_BREACH**: Log VIX breach

- Log: current VIX, threshold, spike amount

**STEP_PAUSE_NEW_ENTRIES**: Stop new order placement

- Set flag: ACCEPTING_NEW_ENTRIES = false
- Log: "New entries paused due to VIX"

**STEP_EXIT_ALL_POSITIONS**: Close all open positions

- For each open position:
  - Generate exit signal with reason "VIX_SPIKE"
  - **Trigger**: EVENT_EXIT_SIGNAL_GENERATED

**STEP_NOTIFY_VIX_BREACH**: Send alert

- Alert: "VIX circuit breaker: [current_vix]"
- Include: action taken (positions closed)

**STEP_SCHEDULE_VIX_RECHECK**: Schedule check for resumption

- Recheck VIX every 5 minutes
- If VIX < 28 for 10 minutes → resume trading

**Postconditions**:

- New entries paused
- Existing positions closed
- User alerted

**Next Events**:

- After 10 minutes of VIX < 28 → EVENT_TRADING_RESUME_ALLOWED

---

### EVENT_DAILY_LOSS_LIMIT_HIT

**Trigger**: Daily loss reaches -3% of account  
**Preconditions**: Position closed, daily P&L calculated

**Handler Steps**:

**STEP_LOG_LOSS_LIMIT**: Log daily loss limit hit

- Log: daily P&L, limit threshold
- Log: time of breach

**STEP_HALT_TRADING**: Halt all trading for day

- Set flag: TRADING_HALTED = true
- Cancel all pending orders
- Close all open positions (market orders)

**STEP_NOTIFY_LOSS_LIMIT**: Send critical alert

- Alert: "CRITICAL: Daily loss limit hit"
- Include: total loss, trades today
- Require acknowledgment

**STEP_GENERATE_LOSS_REPORT**: Create detailed report

- List all trades today
- Calculate: win rate, avg win, avg loss
- Identify: what went wrong
- Save to: `reports/loss_report_YYYYMMDD.json`

**STEP_LOCK_SYSTEM**: Lock system for day

- Set: TRADING_ENABLED = false
- Require manual unlock to resume tomorrow
- Log: system locked

**Postconditions**:

- Trading halted for day
- System locked
- User notified
- Report generated

**Next Events**: None (wait for next day or manual intervention)

---

### EVENT_CONSECUTIVE_LOSS_LIMIT_HIT

**Trigger**: 3 consecutive losing trades  
**Preconditions**: Position closed with loss, loss count = 3

**Handler Steps**:

**STEP_LOG_CONSECUTIVE_LOSSES**: Log consecutive loss event

- Log: loss count, trades details

**STEP_REDUCE_POSITION_SIZE**: Reduce next position size

- Set: position_size_multiplier = 0.5 (50% of normal)
- Log: position sizing reduced

**STEP_PAUSE_TRADING**: Pause trading temporarily

- Set flag: TRADING_PAUSED = true
- Wait: 30 minutes cooling period

**STEP_NOTIFY_CONSECUTIVE_LOSSES**: Send alert

- Alert: "3 consecutive losses, position sizing reduced"

**STEP_SCHEDULE_RESUME**: Schedule trading resume

- After 30 minutes → **Trigger**: EVENT_TRADING_RESUME_ALLOWED

**Postconditions**:

- Trading paused for 30 minutes
- Position sizing reduced
- User notified

**Next Events**:

- After 30 minutes → EVENT_TRADING_RESUME_ALLOWED

---

## 9. Time-Based Events

### EVENT_MARKET_OPEN

**Trigger**: Time reaches 9:15 AM on trading day  
**Preconditions**: System initialized, authenticated

**Handler Steps**:

**STEP_LOG_MARKET_OPEN**: Log market open

- Log: date, time, market session start

**STEP_CHECK_OPENING_GAP**: Check for gap-up or gap-down

- Get yesterday's close
- Get today's open (first tick)
- Calculate gap: (open - close) / close
- If |gap| > 2% → **Trigger**: EVENT_OPENING_GAP_DETECTED

**STEP_RUN_DAILY_ANALYSIS**: Run daily ADX analysis

- **Trigger**: EVENT_DAILY_ANALYSIS_REQUIRED

**STEP_SUBSCRIBE_INITIAL_STRIKES**: Subscribe to option strikes

- Calculate current ATM
- Subscribe to strikes: ATM -200 to ATM +200 (9 strikes each side)
- Subscribe to underlying (NIFTY)

**STEP_ENABLE_TRADING**: Enable trading mode

- Set flag: TRADING_ENABLED = true
- Set flag: ACCEPTING_NEW_ENTRIES = true (after 10:00 AM)
- Log: system ready for trading

**Postconditions**:

- Market open acknowledged
- Daily analysis complete
- Strikes subscribed
- Trading enabled

**Next Events**:

- → EVENT_DAILY_ANALYSIS_REQUIRED
- If gap > 2% → EVENT_OPENING_GAP_DETECTED

---

### EVENT_NO_NEW_ENTRIES_TIME

**Trigger**: Time reaches 2:30 PM  
**Preconditions**: Market still open

**Handler Steps**:

**STEP_LOG_CUTOFF_TIME**: Log new entry cutoff

- Log: time, reason (insufficient time for trade)

**STEP_DISABLE_NEW_ENTRIES**: Stop accepting new entries

- Set flag: ACCEPTING_NEW_ENTRIES = false
- Log: new entries disabled

**STEP_CONTINUE_MONITORING**: Continue monitoring existing positions

- Positions still being monitored
- Stop loss and target still active
- Trailing stops still active

**STEP_NOTIFY_CUTOFF**: Send notification

- Alert: "No new entries after 2:30 PM"
- Include: open positions count

**Postconditions**:

- New entries disabled
- Existing positions continue normally

**Next Events**:

- Positions continue → EVENT_POSITION_MONITORING_REQUIRED
- → EVENT_EXIT_TIME_APPROACHING

---

### EVENT_EXIT_TIME_APPROACHING

**Trigger**: Time reaches 3:20 PM  
**Preconditions**: Market still open, 10 minutes to close

**Handler Steps**:

**STEP_LOG_EXIT_TIME**: Log market close approaching

- Log: time, minutes to close

**STEP_GET_OPEN_POSITIONS**: Get list of open positions

- Count: number of open positions
- If count = 0 → no action needed

**STEP_INITIATE_POSITION_CLOSURE**: Close all positions

- For each open position:
  - Generate exit signal with reason "MARKET_CLOSE"
  - Use MARKET orders (immediate execution)
  - **Trigger**: EVENT_EXIT_SIGNAL_GENERATED

**STEP_WAIT_FOR_CLOSURES**: Wait for all positions to close

- Monitor all exit orders
- Wait up to 5 minutes (until 3:25 PM)
- If any position still open at 3:25 PM → force exit

**STEP_NOTIFY_CLOSURE**: Send notification

- Alert: "All positions closed for market close"
- Include: count closed, final P&L for day

**Postconditions**:

- All positions closed or closing
- System ready for market close

**Next Events**:

- → EVENT_MARKET_CLOSE

---

### EVENT_MARKET_CLOSE

**Trigger**: Time reaches 3:30 PM  
**Preconditions**: Market was open

**Handler Steps**:

**STEP_LOG_MARKET_CLOSE**: Log market close

- Log: date, time, market session end

**STEP_VERIFY_POSITIONS_CLOSED**: Verify no open positions

- Check: open_positions list is empty
- If not empty → log critical error
- Alert user of any stuck positions

**STEP_CALCULATE_DAILY_PNL**: Calculate final daily P&L

- Sum all trade P&Ls for today
- Calculate: win rate, total trades
- Calculate: largest win, largest loss

**STEP_UNSUBSCRIBE_WEBSOCKET**: Unsubscribe from all symbols

- Unsubscribe all option strikes
- Unsubscribe underlying
- Keep connection alive (don't disconnect)

**STEP_GENERATE_EOD_REPORT**: Generate end-of-day report

- Report includes:
  - Total P&L for day
  - Number of trades
  - Win rate
  - Largest win/loss
  - Time in market
  - Trading direction (CE/PE/both)
- Save to: `reports/daily_report_YYYYMMDD.json`

**STEP_BACKUP_DATA**: Backup today's data

- Copy all data files to backup directory
- Compress if needed
- Verify backup integrity

**STEP_NOTIFY_EOD**: Send end-of-day notification

- Email/SMS: "EOD Report"
- Include: P&L, trades, win rate

**STEP_ENTER_POST_MARKET_MODE**: Enter post-market operations

- Set flag: MARKET_SESSION = POST_MARKET
- Enable data management tasks

**Postconditions**:

- Market close acknowledged
- Daily P&L calculated
- EOD report generated
- Data backed up
- User notified

**Next Events**:

- → EVENT_POST_MARKET_OPERATIONS_REQUIRED

---

### EVENT_POST_MARKET_OPERATIONS_REQUIRED

**Trigger**: Market closes, enters post-market phase  
**Preconditions**: Market closed, EOD report generated

**Handler Steps**:

**STEP_DOWNLOAD_HISTORICAL_DATA**: Download today's data

- Fetch completed candles for today
- Download: 1m, 5m, 15m, 1h, daily bars
- Validate all data complete

**STEP_RECONCILE_WITH_BROKER**: Reconcile orders and positions

- Fetch all orders from broker for today
- Compare with internal order log
- Check for any discrepancies
- If mismatch → log critical error and alert

**STEP_UPDATE_INSTRUMENT_MASTER**: Refresh instrument master

- Download latest instrument master
- Check for new contracts (weekly expiry)
- Update token map
- Archive old token map

**STEP_CLEANUP_OLD_DATA**: Clean up old data files

- Delete raw ticks older than 2 days
- Compress bars older than 7 days
- Delete compressed files older than retention period

**STEP_RUN_PERFORMANCE_ANALYSIS**: Analyze today's performance

- Compare with historical performance
- Identify patterns (winning/losing trades)
- Update strategy metrics

**Postconditions**:

- Data downloaded and validated
- Reconciliation complete
- Instrument master updated
- Old data cleaned up

**Next Events**:

- At 4:00 PM → EVENT_DAILY_SHUTDOWN_ALLOWED

---

### EVENT_DAILY_SHUTDOWN_ALLOWED

**Trigger**: Post-market operations complete OR time reaches 4:00 PM  
**Preconditions**: EOD operations done

**Handler Steps**:

**STEP_SAVE_FINAL_STATE**: Save system state

- Save all in-memory state to disk
- Save today's results
- Save strategy state

**STEP_LOG_SHUTDOWN_READY**: Log ready for shutdown

- Log: all operations complete
- Log: can safely shutdown

**STEP_NOTIFY_SHUTDOWN_READY**: Notify operator

- Alert: "System ready for shutdown"
- Include: option to keep running or shutdown

**STEP_WAIT_OR_SHUTDOWN**: Decision point

- If user wants shutdown → **Trigger**: EVENT_SYSTEM_SHUTDOWN
- Else → keep running, wait for next day

**Postconditions**:

- System ready for shutdown or overnight idle

**Next Events**:

- If shutdown → EVENT_SYSTEM_SHUTDOWN
- Else → wait until next day's EVENT_SYSTEM_STARTUP

---

## 10. Error Events

### EVENT_CRITICAL_ERROR

**Trigger**: Unrecoverable error occurs  
**Preconditions**: System in any state

**Handler Steps**:

**STEP_LOG_ERROR**: Log error details

- Log: error type, message, stack trace
- Log: system state at time of error
- Log: timestamp

**STEP_CAPTURE_STATE**: Capture system state

- Snapshot all variables
- Snapshot positions
- Snapshot pending orders
- Save to: `errors/error_state_TIMESTAMP.json`

**STEP_ATTEMPT_SAFE_SHUTDOWN**: Try to shutdown safely

- **Trigger**: EVENT_SYSTEM_SHUTDOWN
- If shutdown fails → force terminate

**STEP_NOTIFY_CRITICAL_ERROR**: Alert operator

- Email/SMS: "CRITICAL ERROR"
- Include: error details
- Mark as high priority

**Postconditions**:

- Error logged
- State captured
- System shutdown (safe or forced)
- Operator alerted

**Next Events**:

- → EVENT_SYSTEM_SHUTDOWN

---

### EVENT_WEBSOCKET_ERROR

**Trigger**: WebSocket error occurs  
**Preconditions**: WebSocket was connected or connecting

**Handler Steps**:

**STEP_LOG_WS_ERROR**: Log WebSocket error

- Log: error type, message
- Log: connection state

**STEP_CHECK_ERROR_SEVERITY**: Determine severity

- Fatal errors: authentication failure, protocol error
- Recoverable errors: network timeout, connection reset

**STEP_HANDLE_ERROR**: Handle based on severity

- If fatal → **Trigger**: EVENT_CRITICAL_ERROR
- If recoverable → **Trigger**: EVENT_WEBSOCKET_DISCONNECTED (reconnect)

**Postconditions**:

- Error logged
- Recovery action initiated

**Next Events**:

- If fatal → EVENT_CRITICAL_ERROR
- If recoverable → EVENT_WEBSOCKET_DISCONNECTED

---

### EVENT_ORDER_ERROR

**Trigger**: Error during order placement or monitoring  
**Preconditions**: Order operation attempted

**Handler Steps**:

**STEP_LOG_ORDER_ERROR**: Log order error

- Log: error type, message
- Log: order details
- Log: broker response if available

**STEP_CLASSIFY_ERROR**: Classify error type

- Client error (4xx): validation failed, wrong parameters
- Server error (5xx): broker system issue
- Network error: timeout, connection lost

**STEP_DECIDE_RECOVERY**: Decide recovery action

- Client error: don't retry, fix and resubmit
- Server error: retry with backoff
- Network error: retry with backoff

**STEP_HANDLE_RECOVERY**: Execute recovery

- If retry → schedule retry with backoff
- If no retry → mark order as failed

**STEP_NOTIFY_ERROR**: Notify if critical

- If position entry failed → alert operator
- If position exit failed → critical alert

**Postconditions**:

- Error logged and classified
- Recovery attempted if applicable
- User notified if critical

**Next Events**:

- If retry → EVENT_ORDER_PLACEMENT_REQUIRED (after delay)
- If failed → EVENT_ORDER_REJECTED

---

### EVENT_DATA_QUALITY_ERROR

**Trigger**: Data validation fails  
**Preconditions**: Data received or loaded

**Handler Steps**:

**STEP_LOG_DATA_ERROR**: Log data quality issue

- Log: data type (tick, bar, etc.)
- Log: validation failure reason
- Log: data values

**STEP_QUARANTINE_BAD_DATA**: Isolate bad data

- Move bad data to quarantine directory
- Don't use for trading decisions
- Mark timestamp as data gap

**STEP_ATTEMPT_RECOVERY**: Try to recover good data

- If tick data bad → fetch from REST API
- If bar data bad → rebuild from ticks
- If historical data bad → re-download

**STEP_NOTIFY_DATA_ISSUE**: Alert if significant

- If multiple consecutive failures → alert
- If affects trading → alert

**Postconditions**:

- Bad data isolated
- Recovery attempted
- User notified if significant

**Next Events**:

- If recovered → continue normal operation
- If not recovered → continue with data gap logged

---

## 11. Data Management Events

### EVENT_HISTORICAL_DATA_SYNC_REQUIRED

**Trigger**: Non-trading day OR post-market operations  
**Preconditions**: System in data management mode

**Handler Steps**:

**STEP_IDENTIFY_MISSING_DATA**: Identify data gaps

- Check expected vs actual data points
- Identify missing dates
- Identify missing timeframes
- Create list of gaps

**STEP_PRIORITIZE_GAPS**: Prioritize data to fetch

- Priority 1: Today's data (if missing)
- Priority 2: Recent data (last 7 days)
- Priority 3: Older data for backtesting

**STEP_DOWNLOAD_MISSING_DATA**: Download data from broker

- For each gap:
  - Call historical API
  - Respect rate limits
  - Validate downloaded data
  - Store to appropriate file

**STEP_VERIFY_DATA_INTEGRITY**: Verify all data complete

- Check: no gaps remain
- Check: all timestamps sequential
- Check: OHLC relationships valid

**STEP_UPDATE_DATA_LOG**: Update data status log

- Record: what was downloaded
- Record: timestamp of sync
- Record: any remaining gaps

**Postconditions**:

- Missing data downloaded
- Data integrity verified
- Data log updated

**Next Events**:

- If gaps remain → schedule retry later
- If complete → mark sync done

---

### EVENT_DATA_GAP_DETECTION_REQUIRED

**Trigger**: Every 1 minute during trading  
**Preconditions**: WebSocket active

**Handler Steps**:

**STEP_CHECK_TICK_RECENCY**: Check last tick time

- For each subscribed symbol:
  - Get last tick timestamp
  - Calculate: time since last tick
  - If > 60 seconds → gap detected

**STEP_TRIGGER_GAP_RECOVERY**: Recover gap if detected

- For each symbol with gap:
  - **Trigger**: EVENT_DATA_GAP_RECOVERY_REQUIRED

**Postconditions**:

- All subscribed symbols checked
- Gaps identified

**Next Events**:

- If gap detected → EVENT_DATA_GAP_RECOVERY_REQUIRED

---

### EVENT_DATA_GAP_RECOVERY_REQUIRED

**Trigger**: Data gap detected  
**Preconditions**: Gap identified

**Handler Steps**:

**STEP_LOG_DATA_GAP**: Log gap details

- Log: symbol, start time, end time, duration

**STEP_FETCH_MISSING_DATA**: Fetch data via REST API

- Start time: last_tick_timestamp
- End time: current_time
- Interval: 1 minute
- Download bars from historical API

**STEP_VALIDATE_FETCHED_DATA**: Validate recovered data

- Check: all bars complete
- Check: timestamps correct
- Check: OHLC valid

**STEP_INSERT_DATA**: Insert data into timeline

- Merge with existing data
- Maintain chronological order
- Update tick buffer if needed

**STEP_RECALCULATE_INDICATORS**: Recalculate affected indicators

- If hourly bars affected → recalculate hourly ADX
- Update all dependent calculations

**Postconditions**:

- Gap filled with data from REST API
- Data timeline complete
- Indicators updated

**Next Events**:

- Resume normal operation

---

### EVENT_OPENING_GAP_DETECTED

**Trigger**: Market opens with >2% gap  
**Preconditions**: Market just opened, gap calculated

**Handler Steps**:

**STEP_LOG_OPENING_GAP**: Log gap details

- Log: gap size, direction (up or down)
- Log: yesterday close, today open

**STEP_CANCEL_OLD_SUBSCRIPTIONS**: Unsubscribe old strikes

- Unsubscribe all current option strikes
- Keep underlying subscription

**STEP_RECALCULATE_ATM**: Calculate new ATM based on gap

- New ATM: ROUND(today_open / 50) \* 50

**STEP_SUBSCRIBE_WIDER_RANGE**: Subscribe to wider strike range

- If gap up: subscribe ATM-100 to ATM+200
- If gap down: subscribe ATM-200 to ATM+100
- Total: ~15 strikes each side (30 strikes total)

**STEP_WAIT_FOR_STABILIZATION**: Wait for market to stabilize

- Wait: 5 minutes (300 seconds)
- Monitor volatility during wait
- Don't place any orders during wait

**STEP_NOTIFY_GAP**: Send notification

- Alert: "Opening gap detected: [size]%"
- Include: adjusted ATM, wider range

**Postconditions**:

- Wider strike range subscribed
- ATM adjusted for gap
- System waiting for stabilization
- User notified

**Next Events**:

- After 5 minutes → resume normal trading

---

### EVENT_BACKUP_REQUIRED

**Trigger**: Non-trading day OR scheduled time  
**Preconditions**: Data exists to backup

**Handler Steps**:

**STEP_CREATE_BACKUP_MANIFEST**: List files to backup

- List all data files
- List all state files
- List all log files
- Prioritize critical files

**STEP_COPY_FILES_TO_BACKUP**: Copy to backup directory

- Destination: `backups/backup_YYYYMMDD/`
- Copy all listed files
- Preserve directory structure

**STEP_COMPRESS_BACKUP**: Compress backup

- Format: tar.gz or zip
- Name: `backup_YYYYMMDD.tar.gz`
- Verify compression successful

**STEP_VERIFY_BACKUP**: Verify backup integrity

- Check: compressed file can be extracted
- Check: file count matches manifest
- Check: sample files can be read

**STEP_CLEANUP_OLD_BACKUPS**: Delete old backups

- Keep: last 7 daily backups
- Keep: last 4 weekly backups
- Keep: last 12 monthly backups
- Delete others

**STEP_LOG_BACKUP_COMPLETE**: Log backup completion

- Log: timestamp, file count, size
- Log: backup location

**Postconditions**:

- Backup created and verified
- Old backups cleaned up
- Backup logged

**Next Events**: None (backup complete)

---

## 12. Monitoring Events

### EVENT_HEALTH_CHECK_REQUIRED

**Trigger**: Every 30 seconds during operation  
**Preconditions**: System running

**Handler Steps**:

**STEP_CHECK_AUTHENTICATION**: Check auth status

- Verify: token still valid
- Check: time to expiry
- Status: OK or WARNING or CRITICAL

**STEP_CHECK_WEBSOCKET**: Check WebSocket status

- Verify: connection active
- Check: last message received time
- Check: subscription count
- Status: OK or WARNING or CRITICAL

**STEP_CHECK_DATA_FLOW**: Check data quality

- Verify: ticks being received
- Check: tick rate (ticks/minute)
- Check: data gaps
- Status: OK or WARNING or CRITICAL

**STEP_CHECK_POSITIONS**: Check position health

- Verify: position count <= max
- Check: P&L status
- Check: time in positions
- Status: OK or WARNING or CRITICAL

**STEP_CHECK_SYSTEM_RESOURCES**: Check system resources

- Check: disk space available
- Check: memory usage
- Check: CPU usage
- Status: OK or WARNING or CRITICAL

**STEP_UPDATE_HEALTH_STATUS**: Update overall health

- Aggregate all component statuses
- Overall: HEALTHY or DEGRADED or CRITICAL
- Update health dashboard

**STEP_ALERT_IF_UNHEALTHY**: Alert on issues

- If CRITICAL → immediate alert
- If DEGRADED → warning notification
- If HEALTHY → no alert

**Postconditions**:

- Health status updated
- Issues identified
- Alerts sent if needed

**Next Events**:

- If CRITICAL → may trigger EVENT_CRITICAL_ERROR
- Else → continue monitoring

---

### EVENT_PERFORMANCE_METRICS_UPDATE

**Trigger**: Every 1 minute during trading  
**Preconditions**: System trading

**Handler Steps**:

**STEP_CALCULATE_CURRENT_METRICS**: Calculate metrics

- Win rate: wins / total_trades
- Average profit: sum_profits / winning_trades
- Average loss: sum_losses / losing_trades
- Profit factor: total_profit / total_loss
- Sharpe ratio: (avg_return - risk_free) / std_dev

**STEP_UPDATE_METRICS_DASHBOARD**: Update display

- Update real-time metrics
- Update charts/graphs
- Update position status

**STEP_CHECK_PERFORMANCE_THRESHOLDS**: Check thresholds

- If win_rate < 30% → alert
- If profit_factor < 1.0 → warning
- If Sharpe ratio < 0.5 → warning

**STEP_STORE_METRICS**: Save metrics

- Append to: `metrics/performance_YYYYMMDD.json`
- Keep time series for analysis

**Postconditions**:

- Metrics calculated and updated
- Dashboard refreshed
- Thresholds checked

**Next Events**: None (continuous monitoring)

---

### EVENT_NOTIFICATION_SEND_REQUIRED

**Trigger**: Various events requiring user notification  
**Preconditions**: Notification payload ready

**Handler Steps**:

**STEP_CLASSIFY_NOTIFICATION**: Classify notification priority

- CRITICAL: errors, loss limits, system issues
- HIGH: position opened/closed, daily report
- MEDIUM: warnings, alignment changes
- LOW: informational updates

**STEP_FORMAT_MESSAGE**: Format notification message

- Create subject line
- Create body content
- Include relevant data
- Add timestamp

**STEP_SEND_EMAIL**: Send email notification (if enabled)

- To: configured email address
- Subject: formatted subject
- Body: formatted body
- Track: delivery status

**STEP_SEND_SMS**: Send SMS notification (if enabled for priority)

- Only for CRITICAL and HIGH priority
- To: configured phone number
- Message: abbreviated message (160 char limit)
- Track: delivery status

**STEP_LOG_NOTIFICATION**: Log notification sent

- Log: timestamp, type, priority
- Log: delivery status

**Postconditions**:

- Notification sent via configured channels
- Delivery tracked
- User informed

**Next Events**: None (notification complete)

---

## Event Flow Summary

### Total Event Categories: 12

1. System Lifecycle: 4 events
2. Authentication: 4 events
3. Market Data: 8 events
4. Strategy: 2 events
5. Signal: 2 events
6. Order: 5 events
7. Position: 4 events
8. Risk: 4 events
9. Time-Based: 7 events
10. Error: 4 events
11. Data Management: 5 events
12. Monitoring: 3 events

### Total Handler Steps: 147 (all labeled)

---

## Event Dependency Graph

```
START
  ↓
EVENT_SYSTEM_STARTUP
  ↓
EVENT_TRADING_DAY_DETECTED or EVENT_NON_TRADING_DAY_DETECTED
  ↓
If Trading Day:
  EVENT_AUTH_TOKEN_CHECK_REQUIRED
    ↓
  EVENT_TOKEN_VALIDATION_REQUIRED or EVENT_AUTH_LOGIN_REQUIRED
    ↓
  EVENT_AUTH_SUCCESS
    ↓
  EVENT_INSTRUMENT_MASTER_LOAD_REQUIRED
    ↓
  EVENT_WEBSOCKET_CONNECT_REQUIRED
    ↓
  EVENT_MARKET_OPEN
    ↓
  EVENT_DAILY_ANALYSIS_REQUIRED
    ↓
  (If direction set) EVENT_HOURLY_ANALYSIS_REQUIRED
    ↓
  (If aligned) EVENT_ENTRY_SIGNAL_SEARCH_REQUIRED
    ↓
  (If signal) EVENT_ENTRY_SIGNAL_GENERATED
    ↓
  EVENT_ORDER_PLACEMENT_REQUIRED
    ↓
  EVENT_ORDER_SUBMISSION_REQUIRED
    ↓
  EVENT_ORDER_FILLED
    ↓
  EVENT_POSITION_OPENED
    ↓
  (Continuous) EVENT_POSITION_MONITORING_REQUIRED
    ↓
  (When exit) EVENT_EXIT_SIGNAL_GENERATED
    ↓
  EVENT_ORDER_PLACEMENT_REQUIRED
    ↓
  EVENT_POSITION_CLOSED
    ↓
  (3:20 PM) EVENT_EXIT_TIME_APPROACHING
    ↓
  (3:30 PM) EVENT_MARKET_CLOSE
    ↓
  EVENT_POST_MARKET_OPERATIONS_REQUIRED
    ↓
  (4:00 PM) EVENT_DAILY_SHUTDOWN_ALLOWED
    ↓
  EVENT_SYSTEM_SHUTDOWN
    ↓
END
```

---

## Usage Guide

### How to Insert New Steps

All steps are labeled (not numbered), so you can insert anywhere:

**Example: Insert new step between STEP_LOAD_CONFIG and STEP_INIT_STORAGE**

```
STEP_LOAD_CONFIG: Load application configuration
...

STEP_VALIDATE_CONFIG_VALUES: NEW STEP - Validate config ranges
- Check: adx_threshold between 20-30
- Check: position_size between 1-5%
- If invalid → log error and use defaults
- **Trigger Next**: EVENT_CONFIG_VALIDATED

STEP_INIT_STORAGE: Initialize file storage
...
```

The flow continues naturally with no renumbering needed!

### How to Find When an Event Occurs

Use the "Trigger" field at the top of each event:

- **Trigger**: Tells you exactly when this event happens
- **Preconditions**: Tells you what must be true first
- **Next Events**: Tells you what happens after

### How to Update Logic

1. Find the event (use table of contents)
2. Locate the specific step (all labeled)
3. Update step logic
4. Update Next Events if flow changes
5. No renumbering needed!

---

**This event-driven document provides complete implementation guidance with 147 labeled steps across 52 events, all easily updatable without renumbering!**
