# Strategy: ADX+VWAP ATM Momentum (Gap-Safe, VIX-Tiered, Production)

## 0. Purpose & Fit

- Built for the existing bot: token/session guards, JSON storage, ATM token pool, idempotent orders, circuit breakers, journaling, recovery.
- Intraday buyer of liquidity in the trend direction with ADX bias, VWAP gating, and strict VIX-tier sizing; designed to survive gaps and spikes.

## 1. Instruments & Timing

- Underlyings: NIFTY, BANKNIFTY
- Options: Weekly ATM CE/PE only (avoid delivery risk)
- Trading window: 09:20–15:00. No entries 15:00+. Flatten by 15:15.

## 2. Regime, Bias, and Filters

- Regime (VIX tiers):
  - 12–18: normal; 18–25: −50% size; 25–30: −75% size; >30: halt entries.
- Daily ADX bias:
  - ADX(14) > 25 and +DI > −DI → bullish; ADX(14) > 25 and −DI > +DI → bearish; else none.
- Gaps: If open gap magnitude > 2% vs previous close, wait 2–3 minutes for stabilize; use widened ATM range (±100) then shrink to ±50 after stabilization.
- Liquidity: OI ≥ 1000; spread ≤ 0.5% premium; skip if violated.

## 3. Entry Conditions (5m primary, 1m confirm)

- Long (CE):
  - Price above VWAP; 5m close breaks prior swing high (lookback 20 bars).
  - Volume filter: 5m volume ≥ 1.2 × 20-bar average.
  - RSI(14, 5m) rising; no long upper wick rejection (>50%).
- Short (PE): mirror below VWAP and swing low break.
- 1m confirmation: break candle closes beyond swing; next 1m does not instantly reverse beyond entry candle mid.

## 4. ATM Selection & Switching

- Compute ATM from underlying LTP at signal; subscribe to strikes in ±50 range (±100 on gap mornings).
- If underlying drifts > 50 from current ATM, rotate ATM selection for new entries; do not flip existing positions unless stop/exit triggers.

## 5. Sizing, Orders, and Stops

- Position size: account_equity × base_account_pct (default 3%) / option_price; round to lot.
- Adjust size for: VIX tiers; near-expiry (last 2 days ×0.5); concurrent positions (each ×0.8).
- Entry: LIMIT buy at best ask + 1 tick; retry [1,2,4] seconds; abort if mid drifts > 1%.
- Initial stop (underlying-referenced): 0.6% adverse move; map to option exit by marketable limit.
- Trail to breakeven on +0.4% underlying move; thereafter trail below 5m swing lows (for long) or above swing highs (for short).
- Time stop: 30 minutes if neither SL/target hits.

## 6. Targets & Management

- Scale out: 50% at 1:1 RR (underlying-based). Remainder trails per swings.
- Hard exits: 15:15; circuit-breaker exits if VIX spike > 5 points in 10 minutes.

## 7. Event/News Safeguards

- No new entries in ±15 minutes around major scheduled events (RBI, budget, CPI); maintain via calendar file. Manage open positions with tighter stops.

## 8. Integration Points with Bot

- Data layer: uses 1m/5m underlying series, option LTP, VIX; JSON storage per `timeframes/*`.
- Token pool: `tokens/master.json`, ATM subscription management (±50/±100 policy).
- Execution: idempotent `client_order_id = hash(signal_id, symbol, ts_bucket)`; retries with backoff; auto-cancel at timeout.
- Risk: VIX circuit breakers; daily loss stop; max concurrent positions.
- Journaling: append decisions, orders, fills, P&L; required for crash recovery.

## 9. Config (merge into bot config)

```json
{
  "strategy": {
    "name": "prod_adx_vwap_atm_gapsafe",
    "symbols": ["NIFTY", "BANKNIFTY"],
    "time": { "entry_start": "09:20:00", "entry_end": "15:00:00", "flatten": "15:15:00" },
    "bias": { "adx": 25 },
    "vix": { "normal": [12, 18], "reduced": [18, 25], "high": [25, 30], "halt": 30 },
    "gap": { "threshold_pct": 2.0, "stabilize_min": 3, "widen_points": 100, "normal_points": 50 },
    "liquidity": { "min_oi": 1000, "max_spread_pct": 0.005 },
    "sizing": { "base_equity_pct": 0.03, "near_expiry_days": 2, "near_expiry_factor": 0.5, "concurrent_factor": 0.8 },
    "order": { "type": "LIMIT", "ticks_cushion": 1, "retries": [1, 2, 4], "max_drift_pct": 0.01 },
    "risk": { "sl_underlying_pct": 0.006, "trail_to_be_pct": 0.004, "time_stop_min": 30, "daily_loss_limit_pct": 3.0 },
    "filters": { "no_trade_events": true, "event_buffer_min": 15 },
    "entries": { "swing_lookback": 20, "volume_mult": 1.2, "rsi_window": 14 }
  }
}
```

## 10. Algorithm (Detailed Pseudocode)

```python
if not is_trading_session() or not is_token_valid():
  return

if vix > cfg.vix.halt or exceeded_daily_loss():
  pause_trading("VIX or daily loss")
  return

for sym in cfg.symbols:
  # Gap handling at open
  if is_open_window() and abs(gap_pct(sym)) > cfg.gap.threshold_pct:
    if minutes_since_open() < cfg.gap.stabilize_min:
      continue
    strike_range = cfg.gap.widen_points
  else:
    strike_range = cfg.gap.normal_points

  bias = daily_adx_bias(sym, threshold=cfg.bias.adx)  # +1, -1, 0
  if bias == 0:
    continue

  if not within_time(cfg.time.entry_start, cfg.time.entry_end):
    continue

  if has_scheduled_event_within(cfg.filters.event_buffer_min):
    continue

  setup = detect_5m_breakout_setup(sym, bias,
                                   swing_lookback=cfg.entries.swing_lookback,
                                   volume_mult=cfg.entries.volume_mult,
                                   rsi_window=cfg.entries.rsi_window)
  if not setup.ok:
    continue

  if not confirm_1m_break(sym, setup):
    continue

  atm = compute_atm_strike(sym)
  subscribe_strikes(sym, center=atm, points=strike_range)

  opt = choose_option_symbol(sym, atm, bias)  # CE for +1, PE for -1
  if not liquidity_ok(opt, cfg.liquidity.min_oi, cfg.liquidity.max_spread_pct):
    continue

  qty = compute_position_size(account_equity, opt,
                              base_pct=cfg.sizing.base_equity_pct,
                              vix=vix, near_expiry_days=cfg.sizing.near_expiry_days,
                              concurrent_positions=num_open_positions())
  if qty == 0:
    continue

  price = best_ask(opt) + ticks(cfg.order.ticks_cushion)
  oid = place_limit_buy(opt, qty, price, client_order_id=derive_client_id(setup))
  if not verify_order(oid, timeout_s=45):
    continue

  entry_ref = current_underlying_price(sym)
  sl_ref = entry_ref * (1 - cfg.risk.sl_underlying_pct) if bias>0 else entry_ref * (1 + cfg.risk.sl_underlying_pct)
  attach_underlying_stop(opt, sl_ref)

  manage_position(opt, sym, bias,
                  trail_to_be_pct=cfg.risk.trail_to_be_pct,
                  time_stop_min=cfg.risk.time_stop_min,
                  scale_out_rr=1.0,
                  flatten_time=cfg.time.flatten)
```

## 11. Edge Cases & Handling

- Token expiry mid-session: pause, auto-login, resubscribe, reconcile positions before resume.
- WebSocket gaps: mark data stale; block new signals until heartbeats restore; reconcile on recovery.
- Partial fills: track cumulative; cancel remainder at timeout; adjust stop/targets.
- ATM drift: do not rebase existing positions; only affect new entries.
- Rapid VIX spike (>5 points / 10m): immediate circuit-breaker; flatten if policy set.
- Order rejection loop: backoff, reduce size, widen price within limits; halt symbol after N rejects.

## 12. KPIs & Acceptance Criteria

- Technical: order reject ratio < 1%; reconnects/day < 20; p95 order ack < 1.5s.
- Trading (backtest, then paper, then live):
  - Profit factor ≥ 1.3, Sharpe ≥ 1.0, Max DD ≤ 10% of deployed capital.
  - Hit rate 40–55%; average win ≥ 1.3 × average loss; slippage ≤ 0.2% premium.

## 13. Rollout Plan

- Phase 1: Backtest across 2–3 years, regime-sliced; tune parameters.
- Phase 2: Paper trade 2–4 weeks; track KPIs and rejects; fix issues.
- Phase 3: Live with 25% size for 2 weeks; then 50%; then 100% if KPIs hold.

## 14. Notes

- Strategy is stateless between trades except for trailing; journal all state.
- Prefer indices to avoid delivery; if extended to stocks, enforce D−2 exit rule.
