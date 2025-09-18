# Intraday ATM Momentum Options Strategy (NIFTY/BANKNIFTY)

## Objective

- Capture intraday momentum moves on indices via buying ATM options with strict risk controls and execution rules tailored to Indian markets.

## Instruments & Session

- Underlyings: `NIFTY`, `BANKNIFTY`
- Options: Weekly expiry ATM `CE`/`PE`
- Session: 09:20–15:00 (avoid first 5 minutes and last 30 minutes)

## Dependencies

- Data: 1m candles for underlying and option, INDIA VIX, OI/volume
- Indicators: ADX(14) on daily for trend bias; RSI(14) on 5m; VWAP intraday

## Entry Logic

- Regime gates:
  - Market open and token valid
  - VIX in [12, 30]; if >25 apply reduced sizing; if >30 no new trades
  - Daily loss limit not breached; max concurrent positions not exceeded
- Bias selection:
  - If Daily ADX > 25 and +DI > −DI → bullish bias (prefer CE)
  - If Daily ADX > 25 and −DI > +DI → bearish bias (prefer PE)
  - Else no trade
- Setup on 5m timeframe:
  - Bullish: price above VWAP and breaks above prior 5m swing high with volume > 120% of 20-bar avg; RSI(14) rising from 45–65 zone
  - Bearish: mirror below VWAP and breaks prior swing low; RSI(14) falling from 55–35 zone
- Confirmation on 1m:
  - Break candle close beyond swing with no immediate rejection (>50% wick against)

## Option Selection & Sizing

- Choose ATM strike nearest to underlying LTP at signal time
- Liquidity filter: OI ≥ 1000, bid-ask spread ≤ 0.5% of premium
- Base size: account_equity × 3% / option_price, rounded to lot size
- Adjustments:
  - VIX 18–25: size × 0.5; VIX 25–30: size × 0.25
  - Last 2 trading days of expiry: size × 0.5
  - If another open position exists: each position size × 0.8

## Orders & Execution

- Order type: LIMIT at best ask (buy) with small cushion (1–2 ticks)
- Retry: up to 3 with backoff [1s, 2s, 4s]; cancel if price drifts >1% from signal
- Cutoff: no new entries after 15:00; auto-cancel all pending at 15:25

## Risk Management

- Initial stop: underlying-based, 0.6% adverse move from entry reference (underlying)
- Trail: move stop to breakeven after +0.4% favorable move (underlying)
- Time stop: exit if neither SL/target hits in 30 minutes
- Daily loss limit: stop trading after −3% of account equity

## Targets & Management

- Scale-out: take 50% at 1:1 RR; trail remainder using 5m swing lows/highs
- Hard exit: close all positions by 15:15; stock options not used (avoid delivery)

## Avoid/Filter

- Major event windows (RBI policy, budget, CPI) ±15 minutes: no new entries
- Sideways regime: Daily ADX < 20 or VWAP whipsaw (≥3 crosses in 30 minutes)
- Wide spreads: skip if spread > 0.5%-1.0% of premium

## Config Snippet

```json
{
  "strategy": {
    "name": "atm_momentum_intraday",
    "symbols": ["NIFTY", "BANKNIFTY"],
    "session": { "start": "09:20:00", "end": "15:00:00" },
    "risk": {
      "per_trade_underlying_sl_pct": 0.006,
      "trail_to_be_at_pct": 0.004,
      "time_stop_min": 30,
      "daily_loss_limit_pct": 3.0
    },
    "vix": { "min": 12, "halt": 30, "reduced": 25 },
    "liquidity": { "min_oi": 1000, "max_spread_pct": 0.005 },
    "order": { "type": "LIMIT", "retries": [1, 2, 4], "max_drift_pct": 0.01, "entry_cutoff": "15:00:00" }
  }
}
```

## Parameters Table

- entry_time_window: 09:20–15:00
- daily_adx_threshold: 25
- rsi_window_5m: 14
- rsi_entry_zone_long: 45–65; short: 55–35
- vwap_bias_required: true
- swing_lookback_5m: 20 bars
- volume_confirm_mult: 1.2 × 20-bar avg
- sl_underlying_pct: 0.6%
- trail_to_be_pct: 0.4%
- time_stop: 30 minutes
- scale_out_ratio: 50%

## Pseudocode (Core)

```python
if not is_market_open() or not is_token_valid():
  return

if vix > 30 or exceeded_daily_loss():
  halt_trading()
  return

for sym in ["NIFTY", "BANKNIFTY"]:
  bias = compute_daily_adx_bias(sym)  # +1, -1, 0
  if bias == 0:
    continue

  if not within_time("09:20:00", "15:00:00"):
    continue

  setup = get_5m_setup(sym, bias)
  if not setup.ok:
    continue

  if not confirm_1m_break(sym, setup):
    continue

  strike = pick_atm_option(sym)
  if not has_liquidity(strike, min_oi=1000, max_spread_pct=0.005):
    continue

  qty = position_size(account_equity, vix, near_expiry, num_open_positions)
  price = best_ask_plus_ticks(strike, ticks=1)
  order_id = place_limit_buy(strike, qty, price)
  if not confirm_order(order_id):
    continue

  sl_level_underlying = entry_underlying_price * (1 - 0.006 if bias>0 else 1 + 0.006)
  schedule_stop_loss(strike, sl_level_underlying)

  manage_position(strike, bias, trail_to_be_pct=0.004, time_stop_min=30, scale_out_at_rr=1.0)
```

## Backtest Plan

- Data: 1m underlying and option, 2+ years, weekly expiries
- Slippage model: 1–2 ticks adverse on entry, 1 tick on exit
- Costs: STT, brokerage, exchange fees per Indian market norms
- Walk-forward across market regimes; report: CAGR, Sharpe, PF, Max DD, hit rate, avg win/loss, average slippage, exposure

## Notes

- This strategy is modular: plug into the bot’s signal runner, order engine, and risk layer defined in the comprehensive spec.
- Start with paper trading; graduate to live with small size; monitor reject/latency metrics.
