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

#### 13.14.3 Database/File Corruption Recovery

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
  - [ ] Database/JSON schema migrations prepared (if needed)
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
  - Structured logging format (JSON preferred)
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
log_format = "json"                   # json or text
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
