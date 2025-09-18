# Option Trading Bot Flow - Comprehensive Unified Spec (v2)

## 0. Scope, Goals, and Non-Goals

- **Goal**: Provide an end-to-end, implementation-ready specification for an India options trading bot (indices + select stocks), covering data, signals, execution, risk, compliance, recovery, and operations.
- **Audience**: Developers, quants, and operators building and running the bot.
- **Non-Goals**: This document does not prescribe a single strategy; it defines frameworks, guardrails, and concrete interfaces to plug strategies into.

## 1. Operating Modes & Environment

- **Live Trading Mode**: Places real orders via broker API.
- **Paper Trading Mode**: Simulates orders, full logic enabled, no live order placement.
- **Data-Only Mode**: Market closed/weekends/holidays; download/repair data, run backtests, generate reports.
- **Environment**:
  - **Timezone**: Asia/Kolkata (IST). No DST adjustments.
  - **Clock Sync**: Enforce NTP sync; reject if drift > 1s.
  - **Secrets**: API keys/tokens stored in OS secret manager or encrypted file (AES-256-GCM) with KMS-protected key.

## 2. Configuration: Schema and Example

- **Format**: JSON or YAML, loaded at startup; supports hot-reload (signal-triggered) for non-critical parameters.
- **Schema (subset)**:

```json
{
  "app": {
    "mode": "live|paper|data_only",
    "logging_level": "INFO|DEBUG|WARN|ERROR",
    "run_id": "2025-09-18"
  },
  "broker": {
    "name": "zerodha",
    "api_key": "***",
    "api_secret": "***",
    "redirect_uri": "https://example.com/redirect",
    "user_id": "AB1234",
    "totp": {
      "enabled": true,
      "issuer": "Zerodha",
      "secret_env": "Z_TOTP_SECRET"
    }
  },
  "trading": {
    "underlyings": ["NIFTY", "BANKNIFTY"],
    "stock_whitelist": ["RELIANCE", "TCS"],
    "max_concurrent_positions": 3,
    "risk": {
      "daily_loss_limit_pct": 3.0,
      "per_trade_risk_pct": 1.0,
      "vix_thresholds": { "normal": [12, 18], "elevated": [18, 25], "high": [25, 30], "extreme": [30, 99] }
    },
    "position_sizing": {
      "base_account_pct": 0.03,
      "reduce_near_expiry_days": 7,
      "reduce_near_expiry_factor": 0.5
    },
    "expiry_rules": {
      "close_stock_opts_before_days": 2
    },
    "order": {
      "type": "LIMIT",
      "timeout_sec": 45,
      "retry": { "max": 3, "backoff_sec": [1, 2, 4] },
      "cancel_cutoff_time": "15:25:00"
    }
  },
  "data": {
    "root": "./data",
    "retention": { "ticks_days": 2, "min_1m_months": 3, "daily_years": 1 },
    "rate_limits": { "req_per_sec": 3 }
  },
  "alerts": {
    "email": "ops@example.com",
    "slack_webhook": "***",
    "critical": ["token_expiry", "large_loss", "network_down"]
  }
}
```

## 3. Startup & Session Control

- **Trading Day Validation**: Mon-Fri, exclude NSE holidays. Trading 09:15-15:30, pre-market 09:00-09:15, post 15:30-16:00.
- **Token Validation**:
  - Check stored access token via profile call.
  - Ensure validity through 15:30; if not, perform auto-login (browser automation). Prompt only for TOTP if required.
  - Refresh monitor every 5 minutes; pause trading and alert if token expires.
- **System Checklist**: Clock sync, connectivity, config presence, disk space, dependency health, WebSocket capability.
- **Mode Gate**: If non-trading time → Data-Only Mode tasks; else → full trading.

## 4. Data Architecture & Schemas

### 4.1 Directory Layout

- `raw/[SYMBOL]_today.json`
- `raw/[SYMBOL]_yesterday.json`
- `timeframes/[SYMBOL]/1m.json`
- `timeframes/[SYMBOL]/5m.json`
- `timeframes/[SYMBOL]/15m.json`
- `timeframes/[SYMBOL]/1h.json`
- `timeframes/[SYMBOL]/daily.json`
- `tokens/master.json` (instrument metadata)

### 4.2 Raw Tick Schema (Append-only)

```json
{
  "symbol": "NIFTY",
  "ts": "2025-09-18T09:15:02.345+05:30",
  "ltp": 24512.35,
  "bid": 24512.2,
  "ask": 24512.45,
  "bid_qty": 50,
  "ask_qty": 75,
  "oi": 0,
  "v": 120
}
```

### 4.3 Candle Schema (Generic)

```json
{
  "t": "2025-09-18T09:15:00+05:30",
  "o": 24500.0,
  "h": 24520.0,
  "l": 24490.0,
  "c": 24512.35,
  "v": 12000,
  "oi": 0
}
```

### 4.4 Rotation & Gap Logic

- At 15:30: process `*_today.json` → update timeframes; rename today → yesterday; delete old yesterday.
- At 09:15: create fresh today files.
- Gap = (Open_today − Close_prev) / Close_prev × 100. If |gap| > 2% → widen strike range and rebuild ATM token pool.

### 4.5 Rate Limits and Throttling

- Historical/Market/Order API: ≤ 3 req/sec. Use queues, batching, and exponential backoff on 429.

## 5. Token Master, Option Chain, and ATM Management

### 5.1 Token Master Creation

- Monthly at series roll: download instrument CSV or fetch via API.
- Filter FUT/OPT, group by underlying, parse expiries/strikes/lot_sizes/tick_sizes.
- Persist to `tokens/master.json` with checksums.

### 5.2 Token Validation

- Exclude suspended/expired; validate trading status, lot sizes, tick sizes, price bands.
- Maintain whitelist for liquid symbols; enforce min OI/volume thresholds.

### 5.3 ADX-Based Categorization

- Category 1 (Bullish CE): ADX > 25 and +DI > −DI and rising volume.
- Category 2 (Bearish PE): ADX > 25 and −DI > +DI and rising volume.
- Category 3 (No Trade): ADX < 20 or crossover indecisive or low volume.
- Recompute daily after close; update pools gradually over 1–2 minutes without interrupting live positions.

### 5.4 ATM Selection & Dynamic Strike Pool

- ATM = nearest strike to current underlying price; default ±50 points range; widen to ±100 on gap days.
- Monitor underlying every 5–10s; if drift > 50 points from ATM, recalc ATM and rotate subscriptions.

### 5.5 Margin & Delivery Safety

- Before any stock option trade, ensure D−2 rule: auto-close before `close_stock_opts_before_days`.
- Real-time margin check via broker endpoint; enforce 25% buffer in high VIX.

## 6. Indicator Computation (with concrete formulas)

### 6.1 ADX (14) Definitions

- True Range: TR_t = max(H−L, |H−C_prev|, |L−C_prev|)
- +DM_t = max(H−H_prev, 0) if H−H_prev > L_prev−L else 0
- −DM_t = max(L_prev−L, 0) if L_prev−L > H−H_prev else 0
- Smooth via Wilder’s EMA over 14 periods → ATR, +DI, −DI, DX, ADX.

### 6.2 Pseudocode

```python
for t in bars:
  tr = max(h-l, abs(h-c_prev), abs(l-c_prev))
  plus_dm = (h-h_prev) if (h-h_prev) > (l_prev-l) and (h-h_prev) > 0 else 0
  minus_dm = (l_prev-l) if (l_prev-l) > (h-h_prev) and (l_prev-l) > 0 else 0
  atr = wilder_smooth(atr, tr, 14)
  plus_di = 100 * wilder_smooth(plus_dm, plus_dm, 14) / atr
  minus_di = 100 * wilder_smooth(minus_dm, minus_dm, 14) / atr
  dx = 100 * abs(plus_di - minus_di) / (plus_di + minus_di)
  adx = wilder_smooth(adx, dx, 14)
```

## 7. Signal Framework & Trade Selection

- Multi-timeframe: daily/1h for trend; 15m for context; 5m/1m for entries.
- CE bias: higher timeframe uptrend, supports holding, volume increasing, RSI turns up on 5m.
- PE bias: mirror conditions on the downside.
- Avoid trades in sideways/low VIX regimes.

## 8. Volatility & Position Sizing

### 8.1 VIX-Based Regimes

- Normal 12–18: normal sizing.
- Elevated 18–25: −50% size.
- High 25–30: −75% size, tighter stops.
- Extreme >30: halt new trades, evaluate closing open.

### 8.2 Position Size Formula

- Base size = account_equity × base_account_pct.
- Adjust: near expiry (−50% if < reduce_near_expiry_days), low liquidity (−25%), multiple concurrent positions (−20% each additional).

## 9. Order Safety & Execution Engine

### 9.1 Pre-Trade Checks

- Market hours, token tradability, OI > threshold, margin available, drift since signal < 1%.

### 9.2 Idempotency & Retries

- Deterministic `client_order_id = hash(signal_id, symbol, ts_bucket)`.
- On retry, check existing order state; never duplicate.
- Exponential backoff [1,2,4] seconds; max 3 attempts.

### 9.3 Pricing & Slippage Control

- Default LIMIT at best price ± tick(s); abandon if mid moves > 1% during timeout.
- Auto-cancel pending > timeout; cut off all new orders after 15:25:00.

### 9.4 Partial Fills & Reconciliation

- Track cumulative filled qty; if partial at timeout, either cancel remainder or adjust stops/targets proportionally.
- Reconcile positions vs broker at fixed intervals (e.g., every 30s) and on each order update.

## 10. Position Lifecycle Management

- Real-time P&L track; trail stop to breakeven after +0.5% on underlying.
- Dynamic targets: take 50% at 1:1, trail remainder; time-stop: exit if no progress in 30m.
- EOD Safety: flatten 30m before close; stock options auto-close per delivery rule.

## 11. Connectivity, WebSocket & Subscriptions

- Heartbeat every 3–5s; detect liveness within 10s.
- Reconnect strategy: jittered backoff 1s → 2s → 4s → 8s (cap 30s). Resubscribe tokens post-reconnect.
- Subscription budgets: limit to active strikes around ATM; expand temporarily on gaps; shrink after stabilization.

## 12. Error Handling, Recovery & Journaling

- Classify: Critical (token expired, network down), Warning (data gaps, low liquidity), Info (minor delays).
- Critical: pause trading, alert, attempt automated recovery (login/reconnect), verify state before resuming.
- Persistent journal: append-only JSONL of orders, positions, decisions; aids crash recovery.
- Crash recovery: reload latest positions from broker, rebuild internal state from journal and broker reality.

## 13. Monitoring, Metrics & Alerting

- Metrics: API latency, error rate, WebSocket reconnects, order reject ratio, slippage, P&L, drawdown, CPU/mem usage.
- Dashboards: real-time status, risk limits, positions, performance.
- Alerts: token expiry, large loss (> X%), repeated rejects, missed heartbeats, data gaps.

## 14. Testing & Validation Plan

- Unit tests: indicators, sizing, ATM logic, order idempotency.
- Integration: broker sandbox/paper mode; simulated WebSocket streams.
- Backtests: walk-forward, Monte Carlo on shuffles; regime-separated metrics.
- Pre-market dry-run checklist executed daily at 09:00.

## 15. Compliance, Audit & Reporting

- SEBI position limits and margins enforced; maintain complete audit trails.
- Tax: compute STT, P&L lot-wise; export CSV for filing.
- Daily/weekly/monthly performance reports; attribution and benchmark comparisons (optional).

## 16. Security & Access Control

- Principle of least privilege; rotate tokens; encrypt at rest and in transit.
- RBAC for ops actions (pause, resume, kill-switch).

## 17. Deployment & Operations Runbook

- Process manager (systemd/PM2/supervisor) with autorestart and exponential backoff.
- Healthcheck endpoint and CLI:
  - `bot status`, `bot pause`, `bot resume`, `bot kill`, `bot snapshot`.
- Backups: daily configs, journals, token master, timeframes.
- Disaster recovery: restore data, refresh tokens, reconcile positions before resume.

## 18. Implementation Checklists (Actionable)

### 18.1 Critical Safety

- [ ] No interpolation for options data
- [ ] Idempotent client order IDs
- [ ] Circuit breakers for VIX/flash spikes
- [ ] Gap-up/down handling and ATM rebuild
- [ ] Auto-cancel stale orders and 15:25 cutoff
- [ ] Position verification post-fill

### 18.2 Tokens & Data

- [ ] Daily token validity checks; pre-market validation
- [ ] Token master build/refresh at roll
- [ ] Underlying 1–2y daily; options 3m; 1m/5m/15m/1h/daily series
- [ ] Raw tick rotation and gap detection
- [ ] Rate-limit queues and batching

### 18.3 Strategy & Risk

- [ ] ADX pipeline with DI and volume gates
- [ ] Multi-timeframe gating and entry triggers
- [ ] Dynamic sizing rules (VIX, expiry, liquidity)
- [ ] Daily loss limit enforcement

### 18.4 Execution & Ops

- [ ] Retry/backoff; partial fill handling
- [ ] WebSocket heartbeat and reconnect
- [ ] Journaling and crash recovery
- [ ] Metrics, dashboards, and alerts
- [ ] Compliance/audit exports

## 19. Appendices

### 19.1 Option Order Object (Internal)

```json
{
  "id": "uuid-or-client_order_id",
  "symbol": "NIFTY24SEP24500CE",
  "side": "BUY",
  "qty": 50,
  "price": 120.5,
  "order_type": "LIMIT",
  "time_in_force": "DAY",
  "ts_created": "2025-09-18T10:05:01+05:30",
  "status": "PENDING|COMPLETE|REJECTED|CANCELLED|PARTIAL",
  "retries": 0,
  "parent_signal_id": "sig-20250918-1005-1"
}
```

### 19.2 Position Snapshot

```json
{
  "symbol": "NIFTY24SEP24500CE",
  "net_qty": 50,
  "avg_price": 118.3,
  "mtm": 4200.0,
  "stop_price": 110.0,
  "targets": [130.0, 140.0],
  "ts": "2025-09-18T10:20:00+05:30"
}
```

### 19.3 Minimal CLI Examples

- `bot status` → prints health, mode, positions
- `bot pause --reason "Ops maintenance"`
- `bot resume`
- `bot kill --now` (emergency kill-switch)

---

This comprehensive spec consolidates previous docs, adds concrete schemas, algorithms, execution and recovery details, and an operations runbook to make the system production-ready.

## 20. Enhanced Smart Decision Making & Operations (Full Detail)

### 20.1 Intelligent Market Condition Assessment

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

### 20.2 Dynamic Position Sizing Logic

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

### 20.3 Intelligent Entry Timing

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

### 20.4 Advanced Exit Management

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

### 20.5 Intelligent Risk Management

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

### 20.6 Adaptive Strategy Selection

- **Strategy Selection Matrix**:
  - **High VIX + Trending**: Momentum strategies
  - **High VIX + Ranging**: Volatility strategies
  - **Low VIX + Trending**: Trend-following strategies
  - **Low VIX + Ranging**: Mean reversion strategies
  - **Gap Market**: Gap-fill strategies
- **Strategy Performance Tracking**:
  - **Win Rate**, **Average Return**, **Max Drawdown**, **Sharpe**
  - **Strategy Rotation**: Switch based on rolling metrics

### 20.7 Smart Data Management

- **Intelligent Data Prioritization**:
  - **Critical**: Real-time prices for active positions
  - **Important**: Option chain for ATM calculation
  - **Background**: Historical data for analysis
  - **Maintenance**: Token lists and instrument data
- **Smart Data Refresh Logic**:
  - **Real-time Data**: Update every 1-5 seconds during trading
  - **Option Chain**: Update every 5-10 minutes
  - **Historical Data**: Update daily after market close
  - **Token Data**: Update monthly or on new contracts
- **Data Quality Validation**:
  - Price/Volume validation, gap and outlier detection

### 20.8 Intelligent Error Recovery

- **Error Classification**: Critical, Warning, Info
- **Smart Error Response**:
  - **Critical**: Pause trading, notify, attempt recovery
  - **Warning**: Reduce sizes, increase monitoring
  - **Info**: Log and continue
- **Recovery Strategies**: Token re-login, network backoff, data recovery, position reconciliation

### 20.9 Performance Optimization

- **Resource Management**: Optimize CPU/memory/network/storage usage
- **Intelligent Caching**: Price, token, indicator, backtest results
- **Performance Monitoring**: Response times, throughput, error rates

### 20.10 Smart Notification System

- **Prioritization**: Critical, Important, Info
- **Delivery**: Real-time (popup/email), scheduled, on-demand, contextual
- **Content**: Action required, context, impact, timeline

## 21. Critical Missing Components & Additional Considerations (Full Detail)

### 21.1 Regulatory Compliance & Legal Framework

- **SEBI Compliance Requirements**:
  - Position limits, margin requirements, regulatory reporting, audit trail, mandated risk systems
- **Tax Compliance**:
  - Trade-wise taxable P&L, STT tracking, short/long-term classification, tax reports
- **Legal Documentation**:
  - Terms of service, risk disclosures, user agreements, data privacy

### 21.2 Advanced Technical Indicators & Analysis

- **Greeks Management**: Delta, Gamma, Theta, Vega, Rho monitoring and actions
- **Advanced TA**: Volume/Market Profile, order flow, dynamic S/R, Fibonacci
- **Sentiment**: PCR, OI analysis, IV vs HV, skew analysis

### 21.3 Portfolio Management & Diversification

- **Construction**: Asset/sector/expiry diversification; correlation control
- **Risk Metrics**: VaR, Max DD, Sharpe, Beta
- **Rebalancing**: Time-, threshold-, volatility-, performance-based

### 21.4 Advanced Order Types & Execution

- **Routing**: Iceberg, TWAP, VWAP, implementation shortfall
- **Order Types**: Bracket, cover, AMO, GTD
- **Execution Algos**: Slippage reduction, impact control, timing, liquidity seeking

### 21.5 Backtesting & Strategy Validation

- **Framework**: Data validation, parameter optimization, walk-forward, Monte Carlo
- **Metrics**: Total/annualized return, vol, max DD, win rate, profit factor
- **Validation**: Out-of-sample, cross-validation, stress and regime tests

### 21.6 Machine Learning & AI Integration

- **Models**: Direction, volatility, volume, sentiment
- **Features**: Indicators, microstructure, economics, news
- **Ops**: Training, validation, deployment, real-time monitoring

### 21.7 Multi-Broker Support & Redundancy

- **Brokers**: Multi-broker, best execution, failover
- **Redundancy**: Data, execution, system, network

### 21.8 Advanced Risk Management

- **Real-time Risk**: Position, portfolio, market, liquidity
- **Stress Tests**: Historical, hypothetical, Monte Carlo, regime-based
- **Limits**: Position, portfolio, loss, volatility

### 21.9 Performance Analytics & Reporting

- **Analytics**: Attribution, risk-adjusted returns, benchmark/peer comparisons
- **Reporting**: Real-time dashboards, daily/monthly reports, custom reports
- **Visualization**: Performance, risk, correlation, distributions

### 21.10 System Architecture & Scalability

- **Microservices**: Data/Strategy/Risk/Order services
- **Scalability**: Horizontal scaling, load balancing, DB scaling, caching
- **HA**: Fault tolerance, DR, backups, monitoring

### 21.11 Security & Access Control

- **Security**: Encryption, RBAC, audit logging, intrusion detection
- **API Security**: Rate limiting, authN/Z, input validation
- **Data Security**: At rest/in transit, backups, retention, privacy

### 21.12 Integration & Connectivity

- **External Data**: Economics, news, social, alternative
- **Third-party**: Analytics, reporting, risk, compliance
- **API Management**: Gateway, versioning, docs, automated testing

### 21.A Implementation Checklist - Additional Critical Components

- Regulatory & Compliance: SEBI, tax, legal, audit, risk disclosures
- Advanced Analytics: Greeks, advanced TA, sentiment, portfolio metrics, attribution
- ML & AI: Predictive models, features, model ops
- System Architecture: Microservices, HA, scale, security, monitoring
- Advanced Trading: Routing, order types, algos, multi-broker, redundancy
- Backtesting & Validation: Comprehensive tests and analyses

## 22. State Machines & Workflow Diagrams (Text)

### 22.1 Token Lifecycle State Machine

- **States**: NoToken → LoginPending → TokenValid → TokenExpiring → TokenExpired → RecoveryFailed
- **Transitions**:
  - NoToken → LoginPending: startup without valid token
  - LoginPending → TokenValid: login success and token exchange
  - TokenValid → TokenExpiring: T−30min to cutoff or TTL threshold
  - TokenExpiring → TokenValid: proactive refresh success
  - TokenExpiring → TokenExpired: TTL elapsed or auth error
  - TokenExpired → LoginPending: auto re-login
  - Any → RecoveryFailed: repeated failures > N, trigger circuit breaker and alert
- **Actions**: pause trading on Expired, resume after Valid; notify user on transitions

### 22.2 Order Lifecycle State Machine

- **States**: Created → Submitted → PartiallyFilled → Filled | CancelPending → Cancelled | Rejected
- **Transitions**:
  - Created → Submitted: send to broker (idempotent client_order_id)
  - Submitted → PartiallyFilled/Filled/Rejected: based on broker events
  - PartiallyFilled → Filled or CancelPending (on timeout)
  - CancelPending → Cancelled (confirm) or PartiallyFilled (race)
- **Guards**: timeouts, price drift > threshold, risk gates, daily cut-off
- **Reconciliation**: poll orderbook every 30s and on events; rebuild from broker truth on restart

### 22.3 WebSocket Connectivity State Machine

- **States**: Disconnected → Connecting → Subscribed → Degraded → Reconnecting
- **Heartbeats**: expect tick ≤ 5s; if > 10s → Degraded; > 20s → Reconnecting
- **Backoff**: 1s → 2s → 4s → 8s (cap 30s with jitter); resubscribe tokens after connect

### 22.4 Gap Handling Workflow

- 09:15 detect gap vs previous close; if |gap| > 2%: cancel existing subs, compute new ATM, subscribe ±100 range; wait 2–3m stabilize; shrink to ±50; resume normal ops

### 22.5 Circuit Breaker Workflow

- Levels by VIX: 18–25 (−50% size), 25–30 (−75% size + tighter stops), >30 (halt new trades; evaluate exits)
- Flash spike: immediate halt, flatten; recovery after VIX < threshold − 2 for N minutes

## 23. Broker API Contracts (Abstracted)

### 23.1 Instruments

- Request: GET instruments
- Response (shape):

```json
[
  {
    "instrument_token": 12345,
    "tradingsymbol": "NIFTY24SEP24500CE",
    "name": "NIFTY",
    "segment": "NFO-OPT",
    "exchange": "NFO",
    "tick_size": 0.05,
    "lot_size": 50,
    "expiry": "2024-09-26",
    "strike": 24500,
    "instrument_type": "CE"
  }
]
```

### 23.2 LTP/Quote

- Request: GET ltp/quote?instruments=[...]
- Response:

```json
{
  "NFO:NIFTY24SEP24500CE": {
    "instrument_token": 12345,
    "last_price": 120.5,
    "depth": { "buy": [{ "price": 120.45, "quantity": 50 }], "sell": [{ "price": 120.55, "quantity": 50 }] }
  }
}
```

### 23.3 Order Place

- Request:

```json
{
  "exchange": "NFO",
  "tradingsymbol": "NIFTY24SEP24500CE",
  "transaction_type": "BUY",
  "order_type": "LIMIT",
  "product": "MIS",
  "quantity": 50,
  "price": 120.5,
  "validity": "DAY",
  "tag": "client_order_id"
}
```

- Response:

```json
{ "order_id": "22090100012345" }
```

### 23.4 Orderbook/Trades Stream (Event Shape)

```json
{
  "type": "order_update",
  "order_id": "22090100012345",
  "status": "COMPLETE|PARTIAL|REJECTED|CANCELLED|OPEN",
  "filled_quantity": 50,
  "average_price": 120.45,
  "timestamp": "2025-09-18T10:05:15+05:30"
}
```

## 24. Operational Runbooks

### 24.1 Pre-Open (09:00–09:15 IST)

1. Sync clock; 2) Validate token; 3) Load configs; 4) Build/refresh token master if needed; 5) Download missing data; 6) Start services; 7) Dry-run signals without orders; 8) Arm circuit breakers

### 24.2 Intraday Anomaly: Token Expiry Mid-Session

- Pause trading, notify; attempt auto-login; on success, re-init session, resubscribe; verify positions; resume; on repeated failure → stay paused and alert

### 24.3 Network Down/Degraded

- Enter Degraded; backoff reconnect; block new orders; on restore, reconcile orders/positions and resume

### 24.4 Order Rejection Loop

- Escalate: reduce size, widen price bands within risk, switch to marketable limit if policy allows, or halt symbol; log RCA

### 24.5 EOD Wrap-Up (15:30–16:00)

- Flatten positions (if policy); rotate raw ticks; persist journals; compute P&L; generate reports; backups; schedule next-day tasks

## 25. Data Quality & Validation Rules

- **Options (no interpolation)**: discard incomplete bars; require valid ticks for all tracked strikes in bar window
- **Underlying**: allow linear interpolation < 5m; discard gaps > 10m
- **Outliers**: reject price jumps > X sigma; log as Warning
- **Completeness**: 1m series should have exact number of bars per session

## 26. Config Reference (Extended)

- **Risk**: daily_loss_limit_pct, consecutive_loss_limit, vix_thresholds
- **Sizing**: base_account_pct, expiry adjustments, liquidity factor
- **Execution**: timeouts, retries, cutoffs, slippage thresholds
- **Data**: retention windows, directories, rate limits
- **Monitoring**: metric sampling, alert sinks

## 27. KPIs & Targets

- Technical: API p95 latency, reconnect count/day, error rate < 1%
- Trading: win rate, profit factor, Sharpe, max drawdown, average slippage

## 28. Glossary

- **ATM**: At-The-Money; **CE/PE**: Call/Put; **ADX/DI**: Average Directional Index/Directional Indicators; **VIX**: Volatility Index; **OI**: Open Interest; **MIS**: Intraday product; **TTL**: Time-to-live; **RCA**: Root Cause Analysis
