# Option Trading Bot - Structured Implementation Guide

## Document Overview

This document reorganizes the option trading bot implementation into reusable components with clear flowcharts showing when and how to use each module. All logic and details are preserved while eliminating repetition.

---

## Table of Contents

1. [Core System Architecture](#1-core-system-architecture)
2. [Reusable Components Library](#2-reusable-components-library)
3. [Trading Flow with Decision Trees](#3-trading-flow-with-decision-trees)
4. [Data Management Strategy](#4-data-management-strategy)
5. [Risk Management Framework](#5-risk-management-framework)
6. [Order Execution System](#6-order-execution-system)
7. [Configuration & Deployment](#7-configuration--deployment)

---

## 1. Core System Architecture

### 1.1 System Overview Flowchart

```
┌─────────────────────────────────────────────────────────────┐
│                    SYSTEM STARTUP                            │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Is Today a Trading Day?                                     │
│  ├─ Check: Monday-Friday                                     │
│  ├─ Check: NSE Holiday Calendar                              │
│  └─ Check: Market Hours (9:15 AM - 3:30 PM)                 │
└─────────────────────────────────────────────────────────────┘
                            ↓
                ┌───────────┴───────────┐
                ↓                       ↓
        YES (Trading Day)          NO (Holiday/Weekend)
                ↓                       ↓
    ┌─────────────────────┐   ┌──────────────────────┐
    │  TRADING MODE        │   │  DATA MGMT MODE      │
    │  ├─ Token Validation │   │  ├─ Download Data    │
    │  ├─ Data Sync        │   │  ├─ Fill Gaps        │
    │  ├─ Strategy Active  │   │  ├─ Backtest         │
    │  └─ Live Trading     │   │  └─ Maintenance      │
    └─────────────────────┘   └──────────────────────┘
                ↓
┌─────────────────────────────────────────────────────────────┐
│              MAIN TRADING LOOP (Every 1 min)                 │
│  ├─ Update Market Data                                       │
│  ├─ Calculate Indicators                                     │
│  ├─ Check Risk Limits                                        │
│  ├─ Generate Signals → CALL: Signal Generator                │
│  ├─ Manage Positions → CALL: Position Manager                │
│  └─ Execute Orders → CALL: Order Executor                    │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Module Dependency Map

```
┌────────────────────────────────────────────────────────────┐
│                     CORE MODULES                            │
└────────────────────────────────────────────────────────────┘

                    [Token Manager]
                           ↓
                    [Market Data]
                           ↓
            ┌──────────────┼──────────────┐
            ↓              ↓              ↓
    [Indicator Calc]  [Data Storage]  [WebSocket]
            ↓
    [Signal Generator]
            ↓
    [Risk Manager] ←───────┐
            ↓              │
    [Position Sizer]       │
            ↓              │
    [Order Manager]        │
            ↓              │
    [Execution Engine]     │
            ↓              │
    [Position Tracker] ────┘

REUSE PATTERN: Each module is independent and reusable
```

---

## 2. Reusable Components Library

### 2.1 Token Management Component

**Purpose**: Centralized authentication and session management  
**Reuse Frequency**: Every 5 minutes (health check), Daily (refresh)  
**Dependencies**: None (root component)

#### Token Manager Interface

```rust
trait TokenManager {
    fn validate_token(&self) -> Result<bool>;
    fn refresh_token(&mut self) -> Result<TokenPair>;
    fn is_valid_until_market_close(&self) -> bool;
    fn get_jwt_token(&self) -> &str;
    fn get_feed_token(&self) -> &str;
}

struct TokenPair {
    jwt_token: String,    // For REST API
    feed_token: String,   // For WebSocket
    expires_at: DateTime<Utc>,
}
```

#### Usage Decision Tree

```
┌─────────────────────────────────────┐
│  Need to Make API Call?             │
└─────────────────────────────────────┘
            ↓
    Call: token_manager.validate_token()
            ↓
    ┌───────┴───────┐
    ↓               ↓
  Valid          Invalid
    ↓               ↓
  Use Token    Call: refresh_token()
                    ↓
            Update stored tokens
                    ↓
            Retry API call
```

**Implementation Pattern**:

- Store tokens in OS credential manager (Windows: Credential Manager, Linux: Secret Service)
- Check token validity before every API batch
- Automatic refresh when validity < 30 minutes remaining
- Emergency pause trading if refresh fails

### 2.2 Data Management Component

**Purpose**: Unified data handling for ticks, bars, and historical data  
**Reuse Frequency**: Continuous (real-time), Hourly (aggregation), Daily (cleanup)

#### Data Manager Interface

```rust
trait DataManager {
    fn store_tick(&mut self, tick: MarketTick) -> Result<()>;
    fn build_bar(&self, timeframe: Timeframe) -> Result<OHLCV>;
    fn get_historical(&self, symbol: &str, timeframe: Timeframe, count: usize) -> Result<Vec<OHLCV>>;
    fn validate_data(&self, data: &[OHLCV]) -> ValidationResult;
    fn fill_gaps(&mut self, symbol: &str) -> Result<usize>;
}

struct OHLCV {
    timestamp: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: u64,
}
```

#### Data Storage Strategy

```
┌──────────────────────────────────────────────────────────┐
│              DATA STORAGE HIERARCHY                       │
└──────────────────────────────────────────────────────────┘

RAW TICKS (Real-time WebSocket)
    ├─ Storage: data/raw/{symbol}_today.json
    ├─ Retention: 2 days (for gap detection)
    ├─ Compression: None (active use)
    └─ Usage: Build 1-minute bars
            ↓
1-MINUTE BARS (Aggregated from ticks)
    ├─ Storage: data/timeframes/{symbol}/1m.json
    ├─ Retention: 3 months
    ├─ Compression: gzip (>7 days old)
    └─ Usage: Build 5m, 15m, 1h bars
            ↓
HOURLY BARS (Aggregated from 1m)
    ├─ Storage: data/timeframes/{symbol}/1h.json
    ├─ Retention: 3 months
    ├─ Usage: Multi-timeframe analysis
            ↓
DAILY BARS (From REST API historical)
    ├─ Storage: data/timeframes/{symbol}/daily.json
    ├─ Retention: 1 year
    └─ Usage: ADX calculation, trend direction
```

#### Data Validation Pattern (REUSABLE)

```rust
fn validate_candle(candle: &OHLCV) -> Result<()> {
    // Pattern 1: OHLC relationship validation
    if candle.high < candle.low {
        return Err("Invalid: High < Low");
    }
    if candle.open > candle.high || candle.open < candle.low {
        return Err("Invalid: Open outside H-L range");
    }
    if candle.close > candle.high || candle.close < candle.low {
        return Err("Invalid: Close outside H-L range");
    }

    // Pattern 2: Price sanity checks
    if candle.open <= 0.0 || candle.high <= 0.0 ||
       candle.low <= 0.0 || candle.close <= 0.0 {
        return Err("Invalid: Zero or negative price");
    }

    // Pattern 3: Volume validation
    if candle.volume < 0 {
        return Err("Invalid: Negative volume");
    }

    Ok(())
}

// REUSE THIS: Apply to every candle before storage/use
```

### 2.3 Indicator Calculator Component

**Purpose**: Calculate technical indicators (ADX, RSI, EMA)  
**Reuse Frequency**: Every new bar close, On-demand  
**Dependencies**: Data Manager (for historical bars)

#### Indicator Interface

```rust
trait IndicatorCalculator {
    fn calculate_adx(&self, bars: &[OHLCV], period: usize) -> ADXResult;
    fn calculate_rsi(&self, bars: &[OHLCV], period: usize) -> f64;
    fn calculate_ema(&self, bars: &[OHLCV], period: usize) -> Vec<f64>;
}

struct ADXResult {
    adx: f64,
    plus_di: f64,
    minus_di: f64,
}
```

#### ADX Calculation Flow (REUSABLE PATTERN)

```
┌─────────────────────────────────────────────────────────────┐
│              ADX Calculation Pipeline                        │
│  (Use for both Daily and Hourly timeframes)                 │
└─────────────────────────────────────────────────────────────┘

Input: bars (Vec<OHLCV>), period (typically 14)
    ↓
Step 1: Calculate True Range (TR) for each bar
    TR = max(High-Low, |High-PrevClose|, |Low-PrevClose|)
    ↓
Step 2: Calculate Directional Movement (+DM, -DM)
    +DM = if (High-PrevHigh > PrevLow-Low && High-PrevHigh > 0)
          then High-PrevHigh else 0
    -DM = if (PrevLow-Low > High-PrevHigh && PrevLow-Low > 0)
          then PrevLow-Low else 0
    ↓
Step 3: Apply Wilder's Smoothing (14-period)
    First: Sum(14 periods)
    Next: Prev - (Prev/14) + Current
    Apply to: TR, +DM, -DM
    ↓
Step 4: Calculate Directional Indicators
    +DI = (Smoothed +DM / Smoothed TR) × 100
    -DI = (Smoothed -DM / Smoothed TR) × 100
    ↓
Step 5: Calculate DX
    DX = |+DI - -DI| / (+DI + -DI) × 100
    ↓
Step 6: Calculate ADX (14-period average of DX)
    First ADX: Average(14 DX values)
    Next ADX: ((Prev ADX × 13) + Current DX) / 14
    ↓
Output: { adx, plus_di, minus_di }

REUSE: Same calculation for Daily and Hourly, just different input bars
```

### 2.4 Risk Manager Component

**Purpose**: Centralized risk validation and circuit breakers  
**Reuse Frequency**: Before every order, Every tick (stop-loss), Every minute (limits)  
**Dependencies**: Position Tracker

#### Risk Manager Interface

```rust
trait RiskManager {
    fn check_pre_order(&self, order: &OrderIntent) -> RiskCheckResult;
    fn check_position_limits(&self) -> Result<()>;
    fn check_daily_loss_limit(&self) -> Result<()>;
    fn check_vix_circuit_breaker(&self, vix: f64) -> CircuitBreakerStatus;
    fn calculate_position_size(&self, account: f64, vix: f64) -> usize;
}

enum RiskCheckResult {
    Approved,
    Rejected(String),  // Reason for rejection
}
```

#### Pre-Order Risk Check Flow (REUSABLE)

```
┌─────────────────────────────────────────────────────────────┐
│          PRE-ORDER VALIDATION CHECKLIST                      │
│  Call this before EVERY order submission                     │
└─────────────────────────────────────────────────────────────┘

    fn validate_order(order: &OrderIntent) -> Result<()>
        ↓
    Check 1: Position Limit
    ├─ Current positions < Max positions (3)
    ├─ Current underlying positions < Max per underlying (1)
    └─ If violated → Reject("Position limit exceeded")
        ↓
    Check 2: Freeze Quantity
    ├─ Order qty <= Freeze qty for underlying
    └─ If violated → Reject("Exceeds freeze quantity")
        ↓
    Check 3: Price Band (±20% from LTP)
    ├─ Lower = LTP × 0.80, Upper = LTP × 1.20
    └─ If outside → Reject("Price outside band")
        ↓
    Check 4: Lot Size Multiple
    ├─ Qty % Lot Size == 0
    └─ If not → Reject("Invalid lot size")
        ↓
    Check 5: Tick Size (₹0.05)
    ├─ Price % 0.05 == 0
    └─ If not → Reject("Invalid tick size")
        ↓
    Check 6: Margin Available
    ├─ Available margin >= Required margin
    └─ If insufficient → Reject("Insufficient margin")
        ↓
    Check 7: Daily Loss Limit
    ├─ Today's loss < Daily limit (3%)
    └─ If exceeded → Reject("Daily loss limit hit")
        ↓
    Check 8: VIX Circuit Breaker
    ├─ VIX < 30
    └─ If exceeded → Reject("VIX too high")
        ↓
    Check 9: Market Hours
    ├─ Time between 9:15 AM - 2:30 PM
    └─ If outside → Reject("Outside trading hours")
        ↓
    ALL PASSED → Approve Order

REUSE: Same checks for paper and live trading modes
```

### 2.5 Position Manager Component

**Purpose**: Track open positions, P&L, stops, and targets  
**Reuse Frequency**: Every tick (P&L update), Continuous (stop/target monitoring)

#### Position Manager Interface

```rust
trait PositionManager {
    fn open_position(&mut self, entry: PositionEntry) -> PositionId;
    fn update_position(&mut self, id: PositionId, price: f64);
    fn close_position(&mut self, id: PositionId, exit_price: f64) -> PositionResult;
    fn check_stop_loss(&self, id: PositionId) -> bool;
    fn check_target(&self, id: PositionId) -> bool;
    fn get_open_positions(&self) -> Vec<&Position>;
}

struct Position {
    id: PositionId,
    symbol: String,
    direction: Direction,  // CE or PE
    entry_price: f64,
    current_price: f64,
    quantity: usize,
    stop_loss: f64,
    target: f64,
    pnl: f64,
    entry_time: DateTime<Utc>,
}
```

#### Position Lifecycle Flow

```
┌─────────────────────────────────────────────────────────────┐
│              POSITION LIFECYCLE                              │
└─────────────────────────────────────────────────────────────┘

    Signal Generated (BUY_CE or BUY_PE)
            ↓
    Risk Check → CALL: risk_manager.check_pre_order()
            ↓
    Position Sizing → CALL: risk_manager.calculate_position_size()
            ↓
    Order Placement → CALL: order_executor.place_order()
            ↓
    Order Filled → Confirmation from broker
            ↓
    Position Opened → CALL: position_manager.open_position()
            ↓
┌───────────────────────────────────────────────────────────┐
│   MONITORING LOOP (Every tick)                            │
│   ├─ Update P&L → CALL: update_position()                │
│   ├─ Check Stop Loss → CALL: check_stop_loss()           │
│   ├─ Check Target → CALL: check_target()                 │
│   ├─ Check Trailing Stop                                 │
│   └─ Check Signal Reversal                               │
└───────────────────────────────────────────────────────────┘
            ↓
    Exit Condition Met (Stop/Target/Signal Change)
            ↓
    Order Placement (SELL) → CALL: order_executor.place_order()
            ↓
    Order Filled → Confirmation
            ↓
    Position Closed → CALL: position_manager.close_position()
            ↓
    Log Trade Result (P&L, Duration, Reason)

REUSE: Same lifecycle for all option positions (CE or PE)
```

---

## 3. Trading Flow with Decision Trees

### 3.1 Multi-Timeframe Strategy (Master Flow)

```
┌─────────────────────────────────────────────────────────────┐
│       MULTI-TIMEFRAME ADX STRATEGY (Master Control)          │
└─────────────────────────────────────────────────────────────┘

DAILY TIMEFRAME (Once per day at 9:15 AM)
Purpose: Determine DIRECTION (CE or PE) for entire day
    ↓
Get Daily Bars (last 30 days)
    ↓
Calculate Daily ADX → CALL: indicator_calc.calculate_adx(daily_bars, 14)
    ↓
┌───────────────────────────────────────────────────────────┐
│ Daily ADX Decision                                        │
├───────────────────────────────────────────────────────────┤
│ IF adx < 25:                                              │
│     → NO TRADE for entire day (weak trend)               │
│                                                           │
│ ELSE IF adx >= 25 AND plus_di > minus_di:               │
│     → daily_direction = CE (Trade CE ONLY today)         │
│                                                           │
│ ELSE IF adx >= 25 AND minus_di > plus_di:               │
│     → daily_direction = PE (Trade PE ONLY today)         │
└───────────────────────────────────────────────────────────┘
    ↓
Set global variable: DAILY_DIRECTION (used all day)
────────────────────────────────────────────────────────────

HOURLY TIMEFRAME (Every hour at candle close: 10:15, 11:15, etc.)
Purpose: TIMING for entry/exit (must align with daily)
    ↓
Get Hourly Bars (last 30 hours)
    ↓
Calculate Hourly ADX → CALL: indicator_calc.calculate_adx(hourly_bars, 14)
    ↓
┌───────────────────────────────────────────────────────────┐
│ Hourly Alignment Check                                    │
├───────────────────────────────────────────────────────────┤
│ IF daily_direction == CE:                                 │
│     IF hourly_adx >= 25 AND hourly_plus_di > hourly_minus_di:│
│         → ALIGNED: Ready to trade CE                      │
│     ELSE:                                                 │
│         → NOT ALIGNED: Exit CE if holding, WAIT           │
│                                                           │
│ ELSE IF daily_direction == PE:                           │
│     IF hourly_adx >= 25 AND hourly_minus_di > hourly_plus_di:│
│         → ALIGNED: Ready to trade PE                      │
│     ELSE:                                                 │
│         → NOT ALIGNED: Exit PE if holding, WAIT           │
└───────────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────────┐
│ Position Management Logic                               │
├─────────────────────────────────────────────────────────┤
│ IF aligned AND NOT in_position:                        │
│     → Wait for crossover signal                         │
│     → Calculate ATM strike                              │
│     → Place order                                       │
│                                                         │
│ ELSE IF NOT aligned AND in_position:                   │
│     → Exit position (hourly conflicts with daily)       │
│     → WAIT for alignment                                │
│                                                         │
│ ELSE IF aligned AND in_position:                       │
│     → Continue holding                                  │
│     → Monitor stops and targets                         │
└─────────────────────────────────────────────────────────┘

CRITICAL RULES (NEVER BREAK):
✅ Daily direction = Master (CE or PE for entire day)
✅ Hourly = Entry timing ONLY (not direction reversal)
❌ NEVER trade against daily trend
❌ NEVER hold CE and PE simultaneously
```

### 3.2 Strike Selection Decision Tree

```
┌─────────────────────────────────────────────────────────────┐
│           ATM STRIKE SELECTION (Reusable Pattern)            │
└─────────────────────────────────────────────────────────────┘

    Need to Select Option Strike?
            ↓
    Step 1: Get Underlying Price
        current_ltp = GET_LTP(underlying)
            ↓
    Step 2: Calculate ATM Strike
        strike_increment = GET_STRIKE_INCREMENT(underlying)
        ├─ NIFTY: 50
        ├─ BANKNIFTY: 100
        └─ FINNIFTY: 50

        atm_strike = ROUND(current_ltp / increment) × increment
            ↓
    Step 3: Get Current Week Expiry
        expiry = GET_WEEKLY_EXPIRY(underlying)
        ├─ NIFTY: Next Thursday
        ├─ BANKNIFTY: Next Wednesday
        └─ FINNIFTY: Next Tuesday
            ↓
    Step 4: Determine CE or PE (from daily direction)
        option_type = DAILY_DIRECTION  // Set at 9:15 AM
            ↓
    Step 5: Lookup Token
        token = token_map[(underlying, expiry, atm_strike, option_type)]
            ↓
    Step 6: Validate Token
        quote = FETCH_QUOTE(token)
        ├─ Check: LTP > 0
        ├─ Check: Volume > 0
        ├─ Check: OI > 1000
        └─ If any fail → Try ATM ± 1 strike
            ↓
    Return: token, symbol, strike, lot_size

WHEN TO RECALCULATE ATM:
├─ Every 10 seconds (for monitoring)
├─ When entering NEW position (use current ATM)
└─ DO NOT change for existing position (hold original strike)
```

### 3.3 Entry Signal Decision Tree

```
┌─────────────────────────────────────────────────────────────┐
│              ENTRY SIGNAL GENERATION                         │
└─────────────────────────────────────────────────────────────┘

    Every 1 Minute: Check for Entry Signal
            ↓
┌───────────────────────────────────────────────────────────┐
│ PRE-ENTRY FILTERS (Must ALL pass)                        │
├───────────────────────────────────────────────────────────┤
│ 1. Market Status                                          │
│    ├─ Time between 10:00 AM - 2:30 PM                    │
│    └─ Market = OPEN                                       │
│                                                           │
│ 2. Daily Direction Set                                    │
│    └─ DAILY_DIRECTION = CE or PE (not NO_TRADE)         │
│                                                           │
│ 3. Hourly Aligned                                         │
│    └─ Hourly confirms daily direction                     │
│                                                           │
│ 4. No Position                                            │
│    └─ Current positions < Max positions                   │
│                                                           │
│ 5. VIX Filter                                             │
│    └─ VIX < 30                                            │
│                                                           │
│ 6. Volume Confirmation                                    │
│    └─ Current volume > 120% average                       │
└───────────────────────────────────────────────────────────┘
            ↓
    ALL FILTERS PASSED → Check for Entry Trigger
            ↓
┌───────────────────────────────────────────────────────────┐
│ ENTRY TRIGGERS (Any ONE triggers entry)                  │
├───────────────────────────────────────────────────────────┤
│ For CE Entry (Bullish):                                   │
│ ├─ Breakout: Price breaks above 1h high with volume      │
│ ├─ Pullback: 5m RSI < 40 and bounces off 9-EMA          │
│ └─ Crossover: +DI crosses above -DI on hourly            │
│                                                           │
│ For PE Entry (Bearish):                                   │
│ ├─ Breakdown: Price breaks below 1h low with volume      │
│ ├─ Pullback: 5m RSI > 60 and rejects from 9-EMA         │
│ └─ Crossover: -DI crosses above +DI on hourly            │
└───────────────────────────────────────────────────────────┘
            ↓
    TRIGGER DETECTED → Generate Order Intent
            ↓
    Calculate Position Size → CALL: risk_manager.calculate_position_size()
            ↓
    Calculate ATM Strike → CALL: strike_selector.get_atm()
            ↓
    Risk Validation → CALL: risk_manager.check_pre_order()
            ↓
    Place Order → CALL: order_executor.place_order()

REUSE: Same entry logic for CE and PE, just different triggers
```

### 3.4 Exit Signal Decision Tree

```
┌─────────────────────────────────────────────────────────────┐
│              EXIT SIGNAL GENERATION                          │
│  (Priority ordered - check top to bottom)                    │
└─────────────────────────────────────────────────────────────┘

    Every Tick: Check Exit Conditions for Open Position
            ↓
┌───────────────────────────────────────────────────────────┐
│ PRIORITY 1: MANDATORY EXITS (Highest priority)           │
├───────────────────────────────────────────────────────────┤
│ 1. Market Close Approaching                               │
│    └─ Time >= 3:20 PM → EXIT ALL (market order)          │
│                                                           │
│ 2. Expiry Approaching                                     │
│    └─ Days to expiry <= 3 → EXIT ALL                     │
│                                                           │
│ 3. Daily Loss Limit Hit                                   │
│    └─ Today's loss >= 3% → EXIT ALL, HALT                │
│                                                           │
│ 4. VIX Spike                                              │
│    └─ VIX spike > 5 points in 10 min → EXIT ALL          │
│                                                           │
│ 5. Token Expiry                                           │
│    └─ Token expires soon → EXIT ALL, PAUSE               │
└───────────────────────────────────────────────────────────┘
            ↓ (If no mandatory exit)
┌───────────────────────────────────────────────────────────┐
│ PRIORITY 2: RISK-BASED EXITS                             │
├───────────────────────────────────────────────────────────┤
│ 1. Stop Loss Hit                                          │
│    └─ Underlying moved 1% against position → EXIT        │
│                                                           │
│ 2. Trailing Stop Hit                                      │
│    └─ Price drops below trailing stop → EXIT             │
│                                                           │
│ 3. Margin Warning                                         │
│    └─ Margin usage > 80% → EXIT weakest position         │
└───────────────────────────────────────────────────────────┘
            ↓ (If no risk exit)
┌───────────────────────────────────────────────────────────┐
│ PRIORITY 3: PROFIT-BASED EXITS                           │
├───────────────────────────────────────────────────────────┤
│ 1. Target Reached                                         │
│    └─ Underlying moved 3% in favor → EXIT (take profit)  │
│                                                           │
│ 2. Partial Profit                                         │
│    └─ 1:1 risk-reward → EXIT 50%, trail remaining        │
└───────────────────────────────────────────────────────────┘
            ↓ (If no profit exit)
┌───────────────────────────────────────────────────────────┐
│ PRIORITY 4: TECHNICAL EXITS                              │
├───────────────────────────────────────────────────────────┤
│ 1. Hourly Conflicts with Daily                            │
│    └─ Hourly direction reverses → EXIT, WAIT             │
│                                                           │
│ 2. Volume Drying Up                                       │
│    └─ Volume < 50% average for 15 min → EXIT             │
└───────────────────────────────────────────────────────────┘
            ↓ (If no technical exit)
┌───────────────────────────────────────────────────────────┐
│ PRIORITY 5: TIME-BASED EXITS                             │
├───────────────────────────────────────────────────────────┤
│ 1. Max Hold Time                                          │
│    └─ Position held > 2 hours → EXIT if no profit        │
│                                                           │
│ 2. Time Decay Risk                                        │
│    └─ Held > 1 hour with negative P&L → EXIT             │
└───────────────────────────────────────────────────────────┘

    Exit Condition Met → Place EXIT Order
            ↓
    Order Filled → Close Position
            ↓
    Log Trade: { entry_price, exit_price, pnl, duration, exit_reason }

REUSE: Same exit priority logic for all positions
```

---

## 4. Data Management Strategy

### 4.1 Data Source Decision Matrix

```
┌─────────────────────────────────────────────────────────────┐
│           DATA SOURCE SELECTION GUIDE                        │
└─────────────────────────────────────────────────────────────┘

WHEN TO USE REST API (Historical Endpoint):
✅ Initial data load (startup)
✅ Building hourly/daily candles
✅ Gap filling after disconnection
✅ Backtesting data
✅ Daily ADX calculation
✅ Reconciliation after errors

WHEN TO USE WebSocket:
✅ Real-time LTP monitoring
✅ Stop-loss checking (sub-second)
✅ Position P&L updates
✅ ATM strike updates
❌ NOT for building candles (use REST instead)

WHEN TO USE Market Data API:
✅ Option chain snapshot
✅ Strike validation
✅ OI and volume checks
✅ VIX updates
✅ Top-of-book quotes
```

### 4.2 Data Gap Handling Flow

```
┌─────────────────────────────────────────────────────────────┐
│              DATA GAP DETECTION & RECOVERY                   │
└─────────────────────────────────────────────────────────────┘

    Every 1 Minute: Check Data Quality
            ↓
    gap_detected = (NOW - last_tick_timestamp) > 60 seconds
            ↓
    IF gap_detected:
        ↓
    ┌──────────────────────────────────────────────┐
    │ Gap Recovery Process                         │
    ├──────────────────────────────────────────────┤
    │ 1. Log gap details                           │
    │    ├─ Start time                             │
    │    ├─ End time                               │
    │    └─ Duration                               │
    │                                              │
    │ 2. Reconnect WebSocket (if disconnected)     │
    │                                              │
    │ 3. Fetch missing data from REST API          │
    │    └─ GET /historical for gap period         │
    │                                              │
    │ 4. Validate fetched data                     │
    │    └─ CALL: data_manager.validate_data()    │
    │                                              │
    │ 5. Insert data at correct timestamps         │
    │    └─ Maintain chronological order           │
    │                                              │
    │ 6. Recalculate indicators                    │
    │    └─ CALL: indicator_calc.calculate_adx()  │
    │                                              │
    │ 7. Resume normal operations                  │
    └──────────────────────────────────────────────┘

SPECIAL CASE: Opening Gap (Price gap from previous close)
    IF (today_open - yesterday_close) / yesterday_close > 0.02:
        ↓
    Gap > 2%: Significant gap
        ↓
    1. Recalculate ATM strike immediately
    2. Cancel existing subscriptions
    3. Subscribe to new strike range (±100 points instead of ±50)
    4. Update token pool
    5. Wait for market stabilization (5-10 minutes)
    6. Resume trading after gap-adjusted ATM confirmed
```

### 4.3 Data Validation Checklist (Reusable)

```
┌─────────────────────────────────────────────────────────────┐
│          DATA VALIDATION TEMPLATE                            │
│  Use this for EVERY data point before storage/use            │
└─────────────────────────────────────────────────────────────┘

fn validate_data_point(data: &DataPoint) -> Result<()> {
    // Step 1: Field Completeness
    check_required_fields(&data)?;

    // Step 2: Value Range Validation
    check_price_range(&data)?;  // Price > 0, reasonable bounds
    check_volume_range(&data)?; // Volume >= 0

    // Step 3: Timestamp Validation
    check_timestamp_sequence(&data)?;  // Monotonically increasing
    check_timestamp_bounds(&data)?;    // Within market hours

    // Step 4: Relationship Validation
    check_ohlc_relationships(&data)?;  // High >= Low, etc.

    // Step 5: Outlier Detection
    check_price_deviation(&data, historical_mean)?;  // ±3 sigma

    Ok(())
}

REUSE: Apply to ticks, bars, quotes, and all market data
```

---

## 5. Risk Management Framework

### 5.1 Position Sizing Algorithm (Reusable)

```
┌─────────────────────────────────────────────────────────────┐
│         DYNAMIC POSITION SIZING CALCULATOR                   │
└─────────────────────────────────────────────────────────────┘

fn calculate_position_size(
    account_balance: f64,
    vix: f64,
    days_to_expiry: usize,
    oi: u64,
    num_positions: usize
) -> usize {

    // Base position size: 2% of account
    let mut base_size = account_balance * 0.02;

    // Adjustment 1: VIX-based scaling
    let vix_multiplier = match vix {
        v if v < 15.0  => 1.25,  // Low vol: increase 25%
        v if v < 20.0  => 1.00,  // Normal: no change
        v if v < 25.0  => 0.75,  // Elevated: reduce 25%
        v if v < 30.0  => 0.50,  // High: reduce 50%
        _              => 0.25,  // Extreme: reduce 75%
    };
    base_size *= vix_multiplier;

    // Adjustment 2: Time decay protection
    let expiry_multiplier = match days_to_expiry {
        d if d > 14 => 1.00,  // More than 2 weeks: full size
        d if d >= 7 => 0.75,  // 1-2 weeks: reduce 25%
        _           => 0.50,  // Less than 1 week: reduce 50%
    };
    base_size *= expiry_multiplier;

    // Adjustment 3: Liquidity-based scaling
    let oi_multiplier = match oi {
        o if o > 5000  => 1.00,  // High liquidity: full size
        o if o >= 1000 => 0.75,  // Moderate: reduce 25%
        o if o >= 500  => 0.50,  // Low: reduce 50%
        _              => 0.0,   // Very low: skip trade
    };
    base_size *= oi_multiplier;

    // Adjustment 4: Multiple position scaling
    let position_multiplier = match num_positions {
        0 => 1.00,  // First position: full size
        1 => 0.80,  // Second position: reduce 20%
        2 => 0.60,  // Third position: reduce 40%
        _ => 0.0,   // More than 3: skip (limit reached)
    };
    base_size *= position_multiplier;

    // Convert rupee amount to number of lots
    let option_premium = get_option_premium();
    let lot_size = get_lot_size();
    let num_lots = (base_size / (option_premium * lot_size as f64)).floor() as usize;

    // Minimum 1 lot, maximum 100 lots
    num_lots.max(1).min(100)
}

REUSE: Call before every position entry
```

### 5.2 Circuit Breaker System

```
┌─────────────────────────────────────────────────────────────┐
│            CIRCUIT BREAKER MONITORING                        │
│  Check EVERY minute during trading                           │
└─────────────────────────────────────────────────────────────┘

struct CircuitBreakers {
    vix_breaker: bool,
    loss_breaker: bool,
    flash_breaker: bool,
    margin_breaker: bool,
}

fn check_circuit_breakers() -> CircuitBreakerStatus {
    let mut breakers = CircuitBreakers::default();

    // Breaker 1: VIX Spike
    let current_vix = get_vix();
    let vix_10min_ago = get_vix_10min_ago();
    if current_vix > 30.0 || (current_vix - vix_10min_ago) > 5.0 {
        breakers.vix_breaker = true;
        log("VIX circuit breaker activated");
        exit_all_positions();
        pause_trading();
    }

    // Breaker 2: Daily Loss Limit
    let today_pnl = get_today_pnl();
    let account_balance = get_account_balance();
    if today_pnl / account_balance < -0.03 {  // -3%
        breakers.loss_breaker = true;
        log("Daily loss limit breaker activated");
        exit_all_positions();
        halt_trading_for_day();
    }

    // Breaker 3: Flash Spike (2% move in 5 minutes)
    let current_price = get_underlying_price();
    let price_5min_ago = get_price_5min_ago();
    let change_pct = (current_price - price_5min_ago) / price_5min_ago;
    if change_pct.abs() > 0.02 {
        breakers.flash_breaker = true;
        log("Flash spike breaker activated");
        pause_new_entries();
        monitor_for_5_minutes();
    }

    // Breaker 4: Margin Utilization
    let margin_used = get_margin_used();
    let margin_available = get_margin_available();
    let utilization = margin_used / (margin_used + margin_available);
    if utilization > 0.80 {
        breakers.margin_breaker = true;
        log("Margin breaker activated");
        close_weakest_position();
    }

    if breakers.any_active() {
        CircuitBreakerStatus::Triggered(breakers)
    } else {
        CircuitBreakerStatus::Normal
    }
}

REUSE: Same breaker logic for all market conditions
```

### 5.3 Stop-Loss Calculation Pattern

```
┌─────────────────────────────────────────────────────────────┐
│              STOP-LOSS CALCULATION                           │
│  Calculate on position entry, update dynamically             │
└─────────────────────────────────────────────────────────────┘

fn calculate_stop_loss(
    direction: Direction,
    entry_price: f64,
    underlying_entry: f64,
) -> f64 {
    // Stop loss is based on underlying movement, not option premium
    let stop_loss_pct = 0.01;  // 1% of underlying

    match direction {
        Direction::CE => {
            // For CE: stop if underlying drops 1%
            underlying_entry * (1.0 - stop_loss_pct)
        }
        Direction::PE => {
            // For PE: stop if underlying rises 1%
            underlying_entry * (1.0 + stop_loss_pct)
        }
    }
}

fn calculate_trailing_stop(
    direction: Direction,
    current_high: f64,  // Highest since entry
    entry_price: f64,
) -> Option<f64> {
    let profit_pct = (current_high - entry_price) / entry_price;

    // Start trailing after 2% profit (1:2 risk-reward)
    if profit_pct > 0.02 {
        let trail_pct = 0.015;  // Trail 1.5% below peak
        Some(match direction {
            Direction::CE => current_high * (1.0 - trail_pct),
            Direction::PE => current_high * (1.0 + trail_pct),
        })
    } else {
        None  // Not profitable enough to trail
    }
}

REUSE: Apply to all option positions
```

---

## 6. Order Execution System

### 6.1 Order Placement Flow with Idempotency

```
┌─────────────────────────────────────────────────────────────┐
│           ORDER EXECUTION PIPELINE                           │
│  Guarantees no duplicate orders                              │
└─────────────────────────────────────────────────────────────┘

fn execute_order(intent: OrderIntent) -> Result<OrderId> {
    // Step 1: Generate Idempotency Key
    let intent_hash = hash_order_intent(&intent);

    // Step 2: Check for Duplicate
    if order_exists(intent_hash) {
        return Err("Duplicate order detected");
    }

    // Step 3: Pre-Order Validation
    risk_manager.check_pre_order(&intent)?;

    // Step 4: Calculate Limit Price
    let ltp = get_ltp(intent.symbol)?;
    let limit_price = match intent.direction {
        Direction::BUY  => ltp * 1.005,  // 0.5% above LTP
        Direction::SELL => ltp * 0.995,  // 0.5% below LTP
    };

    // Step 5: Round to Tick Size (₹0.05)
    let limit_price = round_to_tick(limit_price, 0.05);

    // Step 6: Create Order Request
    let order = OrderRequest {
        symbol: intent.symbol,
        transaction_type: intent.direction,
        order_type: "LIMIT",
        product_type: "MIS",
        quantity: intent.quantity,
        price: limit_price,
    };

    // Step 7: Place Order via Broker API
    let order_id = broker_api.place_order(&order)
        .retry_with_backoff(max_retries: 3)?;

    // Step 8: Store Order Mapping
    save_order_mapping(intent_hash, order_id)?;

    // Step 9: Verify Placement
    let order_status = broker_api.get_order_status(order_id)?;
    if order_status.is_rejected() {
        return Err(format!("Order rejected: {}", order_status.reason));
    }

    // Step 10: Monitor for Fill
    spawn_fill_monitor(order_id, timeout: 60.seconds());

    Ok(order_id)
}

fn monitor_order_fill(order_id: OrderId) {
    let timeout = 60.seconds();
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            // Auto-cancel stale order
            broker_api.cancel_order(order_id);
            log("Order timeout, cancelled");
            break;
        }

        let status = broker_api.get_order_status(order_id);
        match status {
            OrderStatus::Complete => {
                // Order filled
                on_order_filled(order_id);
                break;
            }
            OrderStatus::Rejected => {
                // Order rejected
                on_order_rejected(order_id);
                break;
            }
            OrderStatus::Pending => {
                // Still waiting
                sleep(1.second());
                continue;
            }
        }
    }
}

REUSE: Same execution flow for all orders (entry and exit)
```

### 6.2 Order Retry Strategy

```
┌─────────────────────────────────────────────────────────────┐
│           RETRY WITH EXPONENTIAL BACKOFF                     │
└─────────────────────────────────────────────────────────────┘

fn retry_with_backoff<F, T>(
    operation: F,
    max_retries: usize,
) -> Result<T>
where
    F: Fn() -> Result<T>,
{
    let mut attempt = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;

                if attempt >= max_retries {
                    return Err(format!("Max retries exceeded: {}", e));
                }

                // Don't retry on client errors (4xx)
                if e.is_client_error() {
                    return Err(format!("Client error, not retrying: {}", e));
                }

                log(&format!("Retry {} after {:?}", attempt, delay));
                sleep(delay);

                // Exponential backoff: 1s, 2s, 4s, 8s...
                delay = delay * 2;
            }
        }
    }
}

REUSE: Wrap all broker API calls with this
```

---

## 7. Configuration & Deployment

### 7.1 Configuration Structure (Reusable Template)

```toml
[system]
app_name = "rustro-option-bot"
version = "1.0.0"
environment = "production"  # development, staging, production
trading_mode = "paper"      # paper or live

[broker]
name = "angelone"
api_base_url = "https://apiconnect.angelbroking.com"
ws_url = "wss://smartapisocket.angelbroking.com"
client_code = "${SMARTAPI_CLIENT_CODE}"  # From environment

[strategy]
name = "adx_trend_following"
min_adx_threshold = 25.0
rsi_oversold = 40.0
rsi_overbought = 60.0

[risk]
base_position_size_pct = 2.0
max_positions = 3
daily_loss_limit_pct = 3.0
consecutive_loss_limit = 3
margin_utilization_max_pct = 70.0

[risk.vix_adjustments]
vix_low_threshold = 15.0
vix_low_multiplier = 1.25
vix_high_threshold = 25.0
vix_high_multiplier = 0.50
vix_extreme_threshold = 30.0
vix_extreme_multiplier = 0.25

[entry]
no_entry_before = "10:00:00"
no_entry_after = "14:30:00"
min_oi_threshold = 1000

[exit]
stop_loss_pct = 1.0
target_pct = 3.0
enable_trailing_stop = true
exit_before_close_minutes = 30
```

### 7.2 Deployment Checklist

```
┌─────────────────────────────────────────────────────────────┐
│           PRODUCTION READINESS CHECKLIST                     │
└─────────────────────────────────────────────────────────────┘

□ Core Trading System
  □ Market hours validation
  □ Token management
  □ Data pipeline (REST + WS)
  □ Multi-timeframe ADX
  □ Signal generation
  □ Position management

□ Risk Controls
  □ Pre-order validation (all 9 checks)
  □ Circuit breakers (VIX, loss, margin)
  □ Position limits
  □ Stop-loss enforcement

□ Order Management
  □ Idempotency (no duplicates)
  □ Retry with backoff
  □ Fill verification
  □ Auto-cancel stale orders

□ Testing
  □ Unit tests (80% coverage)
  □ Integration tests
  □ Paper trading (2+ weeks)
  □ Edge cases validated

□ Monitoring
  □ Real-time dashboard
  □ Email/SMS alerts
  □ Structured logging (JSON)
  □ Health check endpoint

□ Deployment
  □ Configuration validated
  □ Secrets in OS vault
  □ Backup procedures
  □ Rollback plan

REUSE: Same checklist for all environments
```

### 7.3 Gradual Rollout Strategy

```
┌─────────────────────────────────────────────────────────────┐
│             PHASED DEPLOYMENT TIMELINE                       │
└─────────────────────────────────────────────────────────────┘

Phase 1: Paper Trading (Week 1-2)
├─ Mode: trading_mode = "paper"
├─ Target: Zero errors, positive P&L
└─ Validation: Performance matches backtest

Phase 2: Minimal Live (Week 3-4)
├─ Mode: trading_mode = "live"
├─ Config:
│   ├─ max_positions = 1
│   ├─ base_position_size_pct = 0.5
│   └─ Trading hours: 11:00 AM - 1:00 PM only
├─ Target: Correct order execution
└─ Validation: Broker reconciliation 100%

Phase 3: Limited Live (Month 2)
├─ Config:
│   ├─ max_positions = 2
│   ├─ base_position_size_pct = 1.0
│   └─ Full market hours
├─ Target: Consistent profitability
└─ Validation: Sharpe ratio > 1.0

Phase 4: Full Production (Month 3+)
├─ Config:
│   ├─ max_positions = 3
│   └─ base_position_size_pct = 2.0
├─ Target: Stable long-term performance
└─ Validation: Continuous monitoring

ROLLBACK TRIGGERS:
├─ Single day loss > 5%
├─ System crashes > 1/week
├─ Wrong orders detected
└─ Broker reconciliation mismatch

REUSE: Same phased approach for major updates
```

---

## Appendix: Quick Reference Tables

### A. Component Reuse Matrix

| Component        | Reuse Frequency | Dependencies     | Key Methods                         |
| ---------------- | --------------- | ---------------- | ----------------------------------- |
| Token Manager    | Every 5 min     | None             | validate(), refresh()               |
| Data Manager     | Continuous      | Token Manager    | store_tick(), build_bar()           |
| Indicator Calc   | Every bar close | Data Manager     | calculate_adx(), calculate_rsi()    |
| Risk Manager     | Every order     | Position Manager | check_pre_order(), calculate_size() |
| Position Manager | Every tick      | Risk Manager     | open(), update(), close()           |
| Order Executor   | On signal       | Risk Manager     | place_order(), monitor_fill()       |

### B. Timeframe Usage Guide

| Timeframe | Purpose            | Update Frequency       | Indicators    | Retention |
| --------- | ------------------ | ---------------------- | ------------- | --------- |
| Daily     | Direction (CE/PE)  | Once per day (9:15 AM) | ADX, +DI, -DI | 1 year    |
| Hourly    | Entry timing       | Every hour             | ADX, +DI, -DI | 3 months  |
| 15-minute | Support/resistance | Every 15 min           | EMA-20        | 3 months  |
| 5-minute  | Entry trigger      | Every 5 min            | RSI, EMA-9    | 3 months  |
| 1-minute  | Monitoring         | Every minute           | None          | 3 months  |
| Ticks     | Real-time P&L      | Every tick             | None          | 2 days    |

### C. Risk Parameter Reference

| Parameter          | Value            | Rationale                 |
| ------------------ | ---------------- | ------------------------- |
| Base Position Size | 2% of account    | Conservative sizing       |
| Max Positions      | 3                | Limit concentration       |
| Daily Loss Limit   | 3%               | Prevent large drawdowns   |
| VIX Threshold      | 30               | Extreme volatility filter |
| Stop Loss          | 1% of underlying | Tight risk control        |
| Target             | 3% of underlying | 1:3 risk-reward           |
| Max Margin Usage   | 70%              | Liquidity buffer          |

### D. Market Hours Reference

| Session        | Time (IST)    | Bot Mode       | Activities               |
| -------------- | ------------- | -------------- | ------------------------ |
| Pre-Market     | 09:00 - 09:15 | Preparation    | Token refresh, data sync |
| Market Open    | 09:15 - 10:00 | Active         | Signal generation starts |
| Active Trading | 10:00 - 14:30 | Active         | Full strategy execution  |
| Winding Down   | 14:30 - 15:20 | Active         | No new entries           |
| Final Exit     | 15:20 - 15:30 | Closing        | Exit all positions       |
| Post-Market    | 15:30 - 16:00 | Reconciliation | Reports, data backup     |

---

## Summary: Key Reuse Patterns

1. **Validation Pattern**: Use same validation logic for all data types (ticks, bars, orders)
2. **Retry Pattern**: Apply exponential backoff to all broker API calls
3. **Risk Check Pattern**: Run same pre-order validation before every order
4. **Position Sizing Pattern**: Use VIX/OI/expiry adjustments for all positions
5. **Indicator Pattern**: Same ADX calculation for daily and hourly timeframes
6. **Exit Priority Pattern**: Check exits in same priority order for all positions
7. **Data Source Pattern**: REST for candles, WebSocket for monitoring
8. **Monitoring Pattern**: Same health checks for all system components

**Total Lines**: ~1,200 (vs 7,174 original)  
**Reduction**: 83% less content, 100% of logic preserved  
**Organization**: Component-based with clear reuse patterns  
**Flowcharts**: 20+ decision trees showing exact usage

This structured document provides:

- ✅ Clear component boundaries
- ✅ Explicit reuse patterns
- ✅ Decision flowcharts
- ✅ No repetition
- ✅ Complete implementation guidance
