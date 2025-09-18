# Strategy Playbook (Step-by-Step): ADX+VWAP ATM Momentum with Dynamic Targets

## 1) Scope

- Objective: Intraday capture of index momentum using ATM options with dynamic target sizing and trailing, aligned with VIX tiers, ADX bias, VWAP gating, and gap safety.
- Instruments: NIFTY, BANKNIFTY weekly options (CE/PE); ATM only.

## 2) Daily Pre-Open Checklist (09:00–09:15 IST)

- Trading day validation: Non-holiday, Mon–Fri.
- Session and token: Valid token until market close; WebSocket capable.
- Clock sync: Drift ≤ 1s.
- Data health: 1m/5m underlying series available; VIX feed live.
- Event calendar: If major event within ±15 min, enforce event buffer.
- Config load: Risk, VIX tiers, time windows, liquidity thresholds.
- Safety arming: Circuit breakers, daily loss limit, max positions.

## 3) Market Open Handling (Gap-Safe)

- At 09:15: Detect open gap vs previous close.
- If |gap| > 2%: wait 2–3 minutes before any entry; widen ATM subscription range to ±100 points during stabilization.
- After stabilization: shrink subscription to ±50 points.

## 4) Regime and Risk Tiers (VIX)

- Low: VIX < 12 → more conservative entries; larger RR target (example 1.8x risk).
- Normal: 12–18 → base RR (example 1.5x risk).
- Reduced: 18–25 → reduce position size by 50%; slightly lower RR (example 1.2x risk).
- High: 25–30 → reduce size by 75%; tight stops; lowest RR (example 1.0x risk).
- Halt: VIX > 30 → no new entries; manage or close open risk.

## 5) Bias Determination (Daily)

- Compute ADX(14), +DI, −DI on daily underlying.
- Bullish bias: ADX > 25 and +DI > −DI.
- Bearish bias: ADX > 25 and −DI > +DI.
- No trade: Otherwise.

## 6) Liquidity & Tradability Filters (Per Signal)

- Selected ATM option must have OI ≥ 1000.
- Bid–ask spread ≤ 0.5% of premium (or tighter if configured).
- Tradingsymbol active and allowed; margin available.

## 7) Entry Window and Filters

- Time window: 09:20–15:00 for entries; no new entries after 15:00.
- Event buffer: No entries within ±15 minutes of scheduled major events.
- VWAP gate: For longs, underlying above VWAP; for shorts, below VWAP.
- Breakout structure (5m): Close beyond prior swing (lookback ~20 bars) in bias direction.
- Volume confirm (5m): Volume ≥ 1.2 × 20-bar average.
- 1m confirmation: Break candle closes beyond swing; next 1m does not instantly reject beyond mid of entry candle.

## 8) ATM Strike Selection & Subscription

- Compute ATM strike nearest to current underlying price.
- Subscribe to strikes within ±50 points (±100 points during gap stabilization).
- If underlying drifts > 50 points from current ATM, update ATM for new entries; do not disturb existing positions solely due to drift.

## 9) Position Sizing Rules

- Base size: percent of account equity (example 3%) divided by option premium; round to lot size.
- Adjustments:
  - VIX tiers as per Section 4.
  - Near expiry (last 2 days): size × 0.5.
  - Concurrent positions: each position size × 0.8.
- Enforce daily loss limit and max concurrent positions before entry.

## 10) Dynamic Targets and Trailing (Underlying-Referenced)

- Define risk unit: underlying-based stop distance (example 0.6% adverse move from entry reference).
- Compute target multipliers by VIX tier:
  - Low VIX: higher RR (example 1.8x risk) for Target 1; Target 2 at Target 1 + 0.5x risk.
  - Normal VIX: base RR (example 1.5x) for Target 1; Target 2 at Target 1 + 0.5x.
  - Reduced/High VIX: lower RR (example 1.2x/1.0x) to book sooner under volatility.
- Break-even move: after +0.4% favorable underlying move, move stop to entry reference (breakeven).
- Trailing method after Target 1:
  - ATR-based trail: stop follows underlying at N × ATR (example 2.5 × ATR(14)).
  - Alternative swing-based trail: stop below last 5m swing low (long) or above swing high (short).
- Scale-out at Target 1: close 50% of quantity, keep remainder to trail toward Target 2.
- Map underlying triggers to option exits using marketable limit orders with small cushion; never rely on interpolating option prices.

## 11) Order Placement & Safety

- Entry order: limit buy at best ask plus minimal cushion; verify within timeout; retry with backoff; abort if price drifts > 1% from signal.
- Idempotency: deterministic client order id to prevent duplicates.
- Auto-cancel pending orders near end-of-day cutoff (example 15:25) or on volatility spike trigger.

## 12) Position Management Loop (Operational Steps)

- On each tick batch:
  - Update underlying reference, VIX tier, ATR (if used), and VWAP status.
  - If not hit Target 1 and BE threshold crossed, move stop to breakeven.
  - If Target 1 reached, scale out 50% and switch to trailing mode.
  - Update trailing stop (ATR or swing). Ensure stops tighten, never loosen.
  - Check stop or Target 2 conditions; execute exits accordingly.
  - Enforce time stop (example 30 minutes) if neither stop nor target hits.

## 13) Exit Rules

- Stop-loss: immediate exit at stop trigger.
- Target 1: scale-out 50%, continue with trailing for remainder.
- Target 2: exit remainder.
- Time stop: exit entirely after configured minutes without progress.
- Circuit breaker: on VIX spike > configured threshold (example >5 points in 10 minutes), halt new entries; optionally flatten current positions if policy requires.
- End-of-day: force exit of all positions by 15:15.

## 14) Exceptional Scenario Handling

- Token expiry mid-session: pause trading, re-authenticate, resubscribe feeds, reconcile positions, resume only after state verified.
- Data/WebSocket outage: mark data stale; block new signals; on recovery, resubscribe and reconcile orders/positions before resuming signals.
- Order rejection loop: apply exponential backoff, reduce size, slightly adjust price within risk limits; halt symbol after repeated rejects.
- Wide spreads or low liquidity: skip new entries; if already in position, manage exits with conservative limits to reduce slippage.

## 15) Parameter Defaults (Recommended Starting Points)

- Entry window: 09:20–15:00; flatten at 15:15.
- ADX threshold: 25 on daily timeframe.
- Swing lookback: 20 bars (5m).
- Volume confirm: 1.2 × 20-bar average.
- Stop (underlying): 0.6% adverse move.
- Move to breakeven: +0.4% favorable move.
- Time stop: 30 minutes.
- VIX tiers: Low <12; Normal 12–18; Reduced 18–25; High 25–30; Halt >30.
- Liquidity: OI ≥ 1000; spread ≤ 0.5% premium.
- Subscription range: ±50 points (±100 during gap stabilization).

## 16) Rollout & Acceptance

- Backtest: 2–3 years including various regimes; validate PF, Sharpe, max drawdown, hit rate, slippage.
- Paper trade: 2–4 weeks; monitor rejects, latencies, circuit-breaker activations.
- Live gradual: 25% size → 50% → 100% if KPIs hold.

## 17) Operator Quick Checklists

- Before entry: time window ok, event buffer ok, VIX tier ok, ADX bias ok, VWAP gate ok, swing + volume ok, liquidity ok, daily loss ok, positions < cap.
- After entry: stop placed, Target 1 and BE thresholds noted, trailing mode armed, timers running.
- During trade: monitor VIX spike, drift vs ATM, data health, partial fills, trailing updates.
- Before close: flatten all, rotate data files, generate P&L and reports.
