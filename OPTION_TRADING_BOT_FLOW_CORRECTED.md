# Option Trading Bot Flow (Corrected Order)

## 1. System Startup

- Initialize broker API connection
- Load configuration and credentials
- Check system health and connectivity
- **Market Hours & Holiday Management:**
  - **Trading Hours**: 9:15 AM - 3:30 PM (Monday to Friday)
  - **Weekend Check**: Skip trading on Saturday & Sunday
  - **Holiday Calendar**: Load NSE holiday list for current year
  - **Session Validation**: Check if market is open before trading

## 2. Data Download & Management

- **Historical Data Download:**
  - Download 3 months of historical data for futures and options
  - Download 1-2 years of historical data for underlying stocks/indices
  - Store OHLCV data for multiple timeframes (1min, 5min, 15min, 1hr, daily)
  - Download option chain historical data (3 months max)
  - Store volatility data (IV, HV) for each strike
- **Real-time Data Collection:**
  - **WebSocket Connection**: Establish websocket connection to broker
  - Subscribe to live market data feeds
  - Stream real-time price updates
  - Auto-reconnect on connection loss
- **Timeframe Construction from Tick Data:**
  - **Market Time Boundaries**: Use exact market time (9:15 AM - 3:30 PM)
  - **1-minute bars**: 9:15-9:16, 9:16-9:17, 9:17-9:18... 3:29-3:30
  - **5-minute bars**: 9:15-9:20, 9:20-9:25, 9:25-9:30... 3:25-3:30
  - **15-minute bars**: 9:15-9:30, 9:30-9:45, 9:45-10:00... 3:15-3:30
  - **1-hour bars**: Dynamic based on historical data analysis with day carryover
  - **Daily bars**: 9:15 AM - 3:30 PM (single bar for entire session)

## 3. Token Management & Classification

- **CSV Download & Parsing:**
  - Download CSV from broker at month-end expiry
  - If CSV not available, use broker API to fetch token list
  - Parse CSV/API response to extract future and option tokens
- **First Time Setup:**
  - Filter by instrument type (FUT/OPT)
  - Group by underlying symbol (NIFTY, BANKNIFTY, etc.)
  - Separate futures (FUT) and options (OPT) tokens
  - Identify expiry dates and strike prices
  - Create master token mapping database
- **Underlying Classification:**
  - **Index Options (NIFTY, BANKNIFTY):** Cash settled, lower margins, monthly + weekly
  - **Stock Options (RELIANCE, TCS, etc.):** Physical settlement, higher margins, monthly only
- **Token Validation:**
  - Validate token status (active/suspended/expired)
  - Check lot sizes and tick sizes for each token
  - Verify trading hours and market timings
  - Filter by minimum price (avoid penny stocks)
  - Check corporate actions (splits, bonuses, dividends)

## 4. Dynamic ATM Management & Token Switching

- **ATM Calculation & Monitoring:**
  - **ATM Calculation**: Current underlying price ± 50 points (adjustable)
  - **Price Monitoring**: Track underlying price changes every 5-10 seconds
  - **ATM Update Trigger**: When price moves >50 points from current ATM
  - **Strike Range**: Monitor 5-10 strikes around current ATM
- **Token Pool Management:**
  - **Active Token Pool**: Maintain 5-10 CE/PE tokens around ATM
  - **Buffer Tokens**: Keep 2-3 strikes on each side for smooth transitions
  - **Liquidity Check**: Verify new tokens have sufficient OI/volume
  - **Token Validation**: Ensure tokens are active and tradeable
- **Dynamic Token Switching:**
  - **Add New Tokens**: When price moves, add new ATM tokens
  - **Remove Old Tokens**: Remove tokens >100 points away from ATM
  - **Gradual Replacement**: Update token list over 1-2 minutes
  - **No Trade Interruption**: Switch tokens without stopping ongoing trades

## 5. Timeframe Selection & Strategy Analysis

- **Timeframe Selection:**
  - Choose appropriate timeframe (1min, 5min, 15min, 1hr, daily)
  - Match timeframe to strategy type (scalping, swing, positional)
  - Consider option expiry timeline vs strategy duration
  - **High Volatility Actions:**
    - Switch to lower timeframes (15min → 5min, 1hr → 15min)
    - Reduce position sizes by 50%
    - **Dynamic Risk-Reward:**
      - Tight stop-loss levels (0.5-1% of underlying)
      - Big dynamic targets (2-5x risk or more)
      - Trail stop-loss as price moves favorably
      - Scale out positions at multiple target levels
- **Strategy Analysis:**
  - Run technical indicators on underlying
  - Calculate option pricing models
  - Identify high-probability setups
  - **Timeframe Hierarchy:**
    - **Trend Confirmation (Higher Timeframe):** Daily, 1-hour, 15-minute
    - **Entry/Exit (Lower Timeframe):** 5-minute, 1-minute
  - **CE vs PE Selection:**
    - **Buy CE when:** Bullish trend, support holding, volume up, RSI oversold
    - **Buy PE when:** Bearish trend, resistance holding, volume up, RSI overbought
    - **Avoid both when:** Sideways market, low volatility, no clear direction
  - Generate buy/sell signals

## 6. Risk Assessment

- Calculate position sizing
- Check maximum drawdown limits
- Verify available margin
- Assess portfolio exposure
- Monitor delivery margins for stock options

## 7. Order Placement

- Generate option orders (buy/sell calls/puts)
- Set appropriate limit prices
- Place orders through broker API
- Confirm order execution

## 8. Position Monitoring

- Track open positions in real-time
- Monitor P&L changes
- Update stop-loss levels
- Check for early exit conditions
- Close stock options before expiry to avoid delivery

## 9. Risk Management

- Execute stop-loss orders
- Adjust position sizes
- Hedge delta exposure if needed
- Close positions at expiry
- Handle settlement differences (cash vs physical)

## 10. Performance Tracking

- Log all trades and outcomes
- Calculate daily/weekly P&L
- Track win rate and average returns
- Generate performance reports

## 11. Error Handling & System Management

- Monitor for API failures
- Handle network disconnections
- Manage order rejections
- Implement circuit breakers
- **Missing Data Handling:**
  - If token not found in CSV/API, use cached database
  - If cached data is stale (>7 days), skip trading that token
  - If no data available, use broker's instrument list API
  - Fallback to manual token list if all else fails
- **Market Closure Handling:**
  - **Weekend Mode**: System maintenance and data backup
  - **Holiday Mode**: Skip trading, update holiday calendar
  - **Position Management**: Close positions before market close
- **Market Off-Time Actions:**
  - **Token Refresh**: Re-login and get fresh tokens
  - **Data Backup**: Backup all trading data and logs
  - **System Maintenance**: Update software, clear temp files
  - **Strategy Analysis**: Review performance, update parameters
- **Market On-Time Actions:**
  - **Session Validation**: Check if market is open
  - **Token Validation**: Verify tokens are still valid
  - **Auto Re-login**: If token expired, re-login immediately
  - **Data Sync**: Sync with latest market data
  - **Position Check**: Verify all positions are active
  - **Strategy Activation**: Start trading based on signals

## 12. System Shutdown

- Close all open positions
- Save trading logs
- Update performance metrics
- Prepare for next trading session

