# Option Trading Bot Flow - Detailed Implementation Guide

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
  - Test token validity with simple API call (kite.profile())
  - **Session Duration Check**: Verify token will remain valid until market close (3:30 PM)
  - **If token invalid or expires before market close**: Proceed to auto-login
  - **If token valid for entire trading day**: Continue with system initialization
- **Step 3: Automated Login Process**
  - **Auto-Open Browser**: Automatically open Kite login page
  - **Pre-filled URL**: Include API key in login URL
  - **User Interaction**: Prompt user to enter TOTP only
  - **Token Extraction**: Automatically capture request_token from redirect URL
  - **Token Exchange**: Convert request_token to access_token
  - **Token Storage**: Save new access_token to configuration
- **Step 4: Final Validation**
  - Test new token with API call
  - Verify all required permissions
  - Initialize KiteConnect with valid token
  - Confirm system ready for trading

### 1.2 Configuration Loading

- Load trading parameters from config file
- Set position sizing rules
- Configure risk management limits
- Load strategy parameters
- Set up logging and monitoring

### 1.3 Token Management & Authentication

- **Token Validation Process**:
  - Load stored access token from secure storage
  - Test token validity with kite.profile() API call
  - **Critical: Session Duration Check**: Verify token will remain valid until market close (3:30 PM)
  - **If token invalid or expires before market close**: Proceed to auto-login
  - **If token valid for entire trading day**: Continue with system initialization
- **Automated Login Workflow**:
  - **Browser Automation**: Use Selenium WebDriver for browser control
  - **Auto-Open Browser**: Automatically open Kite login page with API key
  - **User Interaction**: Prompt user to enter TOTP only (username/password pre-filled)
  - **Token Capture**: Monitor redirect URL for request_token extraction
  - **Token Exchange**: Convert request_token to access_token via API
  - **Token Storage**: Save new access_token to configuration
- **Critical: Daily Token Expiry Handling**:
  - **Token Expiry**: Access tokens expire daily after market close (~4:00 PM)
  - **Pre-Market Validation**: Check token validity before 9:15 AM
  - **Session Duration Check**: Verify token will remain valid until 3:30 PM
  - **During Trading**: Monitor token validity every 5 minutes
  - **Emergency Response**: Pause trading if token expires during market hours
  - **User Notification**: Alert user when token needs refresh
- **Continuous Session Monitoring**:
  - **Pre-Trading Check**: Ensure token valid for entire trading day
  - **Mid-Session Check**: Monitor token status every 5 minutes
  - **Proactive Re-login**: Re-login if token expires before market close
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
  - **When to use**: Every 5-10 minutes, on-demand queries, validation

- **WebSocket**:

  - Live price updates for active trading
  - Real-time option chain changes
  - Instant order book updates
  - Live P&L and position monitoring
  - **When to use**: During active trading hours, real-time decisions

- **Raw Tick Data Storage**:
  - **Duration**: 2 days only (for gap detection and 1m bar construction)
  - **Storage**: `raw/[symbol]_today.json`, `raw/[symbol]_yesterday.json`
  - **Purpose**: Build 1-minute bars, detect gap openings
  - **Daily Rotation Process**:
    - **End of Day (3:30 PM)**:
      - Process all ticks from `today.json` into timeframes
      - Rename `today.json` → `yesterday.json`
      - Delete old `yesterday.json` (now 2 days old)
    - **New Day (9:15 AM)**:
      - Create new `today.json` for current day
      - Start collecting new tick data
    - **Gap Detection**:
      - Compare `today.json` open with `yesterday.json` close
      - Use `yesterday.json` for gap calculation

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

### 2.5 Kite API Rate Limits & Management

- **Rate Limits**:
  - **Historical API**: 3 requests per second
  - **Market Data API**: 3 requests per second
  - **WebSocket**: No rate limit (but connection limits apply)
  - **Order API**: 3 requests per second
- **Rate Limit Handling**:
  - **Request Queuing**: Queue requests to stay within limits
  - **Exponential Backoff**: Retry with increasing delays on 429 errors
  - **Request Batching**: Combine multiple requests where possible
  - **Priority System**: Critical requests (orders) get priority
- **Optimization Strategies**:
  - **Cache Data**: Store frequently accessed data locally
  - **Batch Requests**: Use kite.quote() for multiple instruments
  - **WebSocket Priority**: Use WebSocket for real-time data
  - **Scheduled Updates**: Update non-critical data every 5-10 minutes

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
  - Monthly expiry only
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
  - **ATM Calculation**: Current underlying price ± 50 points (adjustable)
  - **Price Monitoring**: Track underlying price changes every 5-10 seconds
  - **ATM Update Trigger**: When price moves >50 points from current ATM
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
  - Get instrument token for "INDIA VIX" from instruments list
  - Fetch live VIX value using kite.ltp("NSE:INDIA VIX")
  - Subscribe to VIX via WebSocket for real-time updates
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
  - **Place Order**: Submit order via Kite API
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
  - **Duplicate Prevention**: Check existing orders before retry

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
- **Session Breaks**: Handle lunch breaks if applicable
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

This detailed version provides comprehensive guidance for implementation while maintaining clarity and preventing confusion.
