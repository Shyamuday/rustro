# Option Trading Bot Flow - Final Version

## 1. System Initialization

- Initialize broker API connection
- Load configuration and credentials
- Check system health and connectivity
- **Market Hours & Holiday Management:**
  - Trading Hours: 9:15 AM - 3:30 PM (Monday to Friday)
  - Weekend Check: Skip trading on Saturday & Sunday
  - Holiday Calendar: Load NSE holiday list for current year
  - Session Validation: Check if market is open before trading

## 2. Data Management & Synchronization

- **Data Source Strategy:**
  - **Historical API**: 3 months futures/options, 1-2 years underlying
  - **Market Data API**: Periodic updates, validation queries
  - **WebSocket**: Real-time trading decisions
- **Underlying-Options Sync:**
  - Use underlying (NIFTY, BANKNIFTY) for trend analysis (1-2 years)
  - Use options data for current trading (3 months)
  - Map underlying price to option strikes
  - Sync trend signals with options strategy
- **Data Quality Control:**
  - No interpolation for options (discrete jumps)
  - Interpolation allowed for underlying data only
  - Discard incomplete bars, validate data completeness
- **Rate Limit Management:**
  - 3 requests/second for Historical/Market APIs
  - Request queuing and exponential backoff
  - Batch requests using kite.quote()

## 3. Token Management & ATM Selection

- **Token Discovery:**
  - Download CSV from broker at month-end expiry
  - Parse futures and options tokens
  - Create master token mapping database
  - Validate token status and liquidity
- **Dynamic ATM Management:**
  - Calculate ATM: Current price ± 50 points
  - Monitor price changes every 5-10 second
    s
  - Update trigger: >50 points movement
  - Maintain 5-10 CE/PE tokens around ATM
- **Gap-Up/Gap-Down Handling:**
  - Detect gaps >100 points at 9:15 AM
  - Immediate token refresh for gap scenarios
  - Emergency token pool: 10-15 strikes around new ATM
  - Wider strike range (±100 points) for gap recovery

## 4. Volatility Management & Risk Control

- **Volatility Detection:**
  - VIX monitoring via kite.ltp("NSE:INDIA VIX")
  - VIX spike detection: >5 points in 10 minutes
  - Price movement: >1% in 5 minutes
  - Volume spikes: >300% of average
- **Circuit Breaker Logic:**
  - Level 1 (VIX 18-25): Reduce positions by 50%
  - Level 2 (VIX 25-30): Reduce positions by 75%
  - Level 3 (VIX >30): Pause trading, close positions
  - Flash spike: Immediate trading halt
- **High Volatility Response:**
  - Switch to lower timeframes (15min → 5min)
  - Focus on ATM options only
  - Tighten stops to 0.5-1%
  - Reduce position duration to 15-30 minutes

## 5. Strategy Analysis & Signal Generation

- **Timeframe Selection:**
  - Choose appropriate timeframe (1min, 5min, 15min, 1hr, daily)
  - Match timeframe to strategy type
  - High volatility: Switch to lower timeframes
- **Multi-Timeframe Analysis:**
  - Trend Confirmation: Daily, 1-hour, 15-minute
  - Entry/Exit: 5-minute, 1-minute
  - Use underlying data for trend detection
- **CE vs PE Selection:**
  - Buy CE: Bullish trend, support holding, volume up, RSI oversold
  - Buy PE: Bearish trend, resistance holding, volume up, RSI overbought
  - Avoid: Sideways market, low volatility, no clear direction

## 6. Order Management & Safety

- **Order Safety Measures:**
  - Idempotent order IDs (UUID-based)
  - Order verification via order book
  - Auto-cancel after 30-60 seconds
  - Cancel on VIX spikes >5 points
- **Order Execution:**
  - Place orders through Kite API
  - Verify fill via order book
  - Update positions only after confirmed fill
  - Retry with exponential backoff (max 3 attempts)

## 7. Position Monitoring & Risk Management

- Track open positions in real-time
- Monitor P&L changes
- Update stop-loss levels dynamically
- Close stock options before expiry (physical settlement)
- Execute stop-loss orders
- Hedge delta exposure if needed

## 8. Performance Tracking & System Management

- Log all trades and outcomes
- Calculate daily/weekly P&L
- Track win rate and average returns
- Generate performance reports
- **Error Handling:**
  - Monitor API failures
  - Handle network disconnections
  - Manage order rejections
  - Implement circuit breakers
- **Market Off-Time Actions:**
  - Token refresh and re-login
  - Data backup and system maintenance
  - Strategy analysis and parameter updates
  - Next day preparation

## 9. System Shutdown

- Close all open positions
- Save trading logs
- Update performance metrics
- Prepare for next trading session

---

## Key Implementation Notes

### Data Flow Priority

1. **Underlying Data** → Trend Analysis (1-2 years)
2. **Options Data** → Current Trading (3 months)
3. **Real-time Data** → Execution Decisions

### Critical Safety Features

- No interpolation for options data
- Idempotent order management
- Circuit breakers for volatility spikes
- Gap-up/gap-down handling
- Auto-cancel stale orders

### Rate Limit Compliance

- Batch requests using kite.quote()
- Queue requests with 333ms delay
- Use WebSocket for real-time data
- Cache frequently accessed data

### Volatility Management

- VIX-based circuit breakers
- Dynamic position sizing
- Timeframe switching
- Flash spike detection

This streamlined version maintains all critical functionality while being more organized and easier to follow.
