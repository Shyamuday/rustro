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
  - **System Maintenance**: Run database cleanup and optimization
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
  - Optimize database performance
  - Update instrument lists
  - Refresh configuration files
  - Test system components
- **Monitoring & Alerts**:
  - Check system health status
  - Monitor data quality metrics
  - Send alerts for critical issues
  - Log all operations for audit trail

### 1.6 System Health Check

- Verify database connectivity
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
  - **File Structure** (Hybrid Organization):
    - `raw/[symbol]_today.json` (current day)
    - `raw/[symbol]_yesterday.json` (previous day, delete after 2 days)
    - `timeframes/[symbol]/1m.json` (3 months)
    - `timeframes/[symbol]/1h.json` (3 months)
    - `timeframes/[symbol]/daily.json` (1 year)
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

### 3.1 Token Discovery & Database Creation

- **CSV Download & Parsing**:
  - Download CSV from broker at month-end expiry
  - If CSV not available, use broker API to fetch token list
  - Parse CSV/API response to extract future and option tokens
- **First Time Setup**:
  - Filter by instrument type (FUT/OPT)
  - Group by underlying symbol (NIFTY, BANKNIFTY, etc.)
  - Separate futures (FUT) and options (OPT) tokens
  - Identify expiry dates and strike prices
  - Create master token mapping database
- **Token Validation**:
  - Validate token status (active/suspended/expired)
  - Check lot sizes and tick sizes for each token
  - Verify trading hours and market timings
  - Filter by minimum price (avoid penny stocks)
  - Check corporate actions (splits, bonuses, dividends)

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

- **Historical Data Requirement**: Download 1-2 years of historical data for underlying stocks/indices
- **Daily ADX Calculation**: Calculate Average Directional Index on daily timeframe for each stock
- **Stock Categorization Process**:
  - **Category 1 - Buy CE**: ADX > 25, +DI > -DI, volume increasing
  - **Category 2 - Buy PE**: ADX > 25, -DI > +DI, volume increasing
  - **Category 3 - No Trade**: ADX < 20 or +DI ≈ -DI or low volume
- **Category 1 - Buy CE Stocks**:
  - **Criteria**: ADX > 25, +DI > -DI, volume > average
  - **Option Selection**: Buy corresponding CE (Call) options
  - **ATM Selection**: Select CE strikes closest to current stock price
  - **Strike Range**: ±50 points around current price for liquidity
- **Category 2 - Buy PE Stocks**:
  - **Criteria**: ADX > 25, -DI > +DI, volume > average
  - **Option Selection**: Buy corresponding PE (Put) options
  - **ATM Selection**: Select PE strikes closest to current stock price
  - **Strike Range**: ±50 points around current price for liquidity
- **Category 3 - No Trade Stocks**:
  - **Criteria**: ADX < 20 or +DI ≈ -DI or volume < average
  - **Action**: Skip trading for these stocks
  - **Reason**: Weak trend or sideways movement
- **ATM Strike Selection Process**:
  - **Current Price**: Get real-time LTP of underlying stock
  - **Strike Calculation**: Find nearest available strike price
  - **CE Selection**: For Category 1 stocks, select CE at ATM
  - **PE Selection**: For Category 2 stocks, select PE at ATM
  - **Liquidity Check**: Ensure selected strikes have sufficient OI/volume
- **Daily Rebalancing**:
  - **Update Categories**: Recalculate ADX daily after market close
  - **Category Changes**: Move stocks between categories based on new ADX values
  - **Option Updates**: Update CE/PE selections based on new categories
  - **ATM Adjustments**: Adjust strike selections based on price movements
- **ADX Implementation Details**:
  - **Period**: 14-day ADX calculation (standard)
  - **Data Source**: Daily OHLCV data for 1-2 years
  - **Calculation**: True Range, +DI, -DI, and ADX values
  - **Validation**: Ensure sufficient historical data for reliable ADX
  - **Update Frequency**: Recalculate ADX daily after market close
  - **Multi-Timeframe**: Use daily ADX for categorization, 5min/1min for entry timing

## 6. Order Management & Safety

### 6.1 Order Generation

- Generate option orders (buy/sell calls/puts)
- Set appropriate limit prices
- Create unique order IDs for idempotency
- Validate order parameters before submission

### 6.2 Order Safety Measures

- **Idempotent Order IDs**: Use unique order IDs to prevent duplicates
  - Generate UUID-based order IDs
  - Store order ID mapping in database
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
  - If token not found in CSV/API, use cached database
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
  - **Database Scaling**: Scale database for high throughput
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
- Quote-depth-aware limit pricing; prefer limits over markets; allow IOC/FOK when needed
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
- Prefer exchange timestamps when available over local time

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
