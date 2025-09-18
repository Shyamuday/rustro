# Strategy (Rust): Dynamic Targeting and Trailing for Options Entries

## Purpose

Define a production-ready, Rust-oriented approach to compute dynamic profit targets and trailing stops for intraday ATM option trades, consistent with the comprehensive bot architecture (VIX tiers, ADX bias, VWAP gating, gap safety, idempotent orders, circuit breakers).

## High-Level Approach

- Risk unit = underlying-based stop distance (in %). Targets scale dynamically by regime (VIX tier), momentum context (VWAP, swing), and time remaining.
- Scale-out at Target 1; trail remainder using ATR-based or swing-based method, upgraded by regime.
- Always convert underlying-based thresholds to option exits using marketable limit orders.

## Config (TOML)

```toml
[strategy]
name = "rust_dynamic_targets"

[strategy.vix]
normal_min = 12.0
normal_max = 18.0
reduced_min = 18.0
reduced_max = 25.0
high_min = 25.0
high_max = 30.0
halt_above = 30.0

[strategy.risk]
sl_underlying_pct = 0.006          # 0.6% adverse move
trail_to_be_underlying_pct = 0.004 # +0.4% move to BE
time_stop_min = 30

[strategy.targets]
base_rr = 1.5                      # base risk:reward
rr_elevated_vix = 1.2
rr_high_vix = 1.0
rr_low_vix = 1.8
scale_out_ratio = 0.5

[strategy.atr]
period = 14
multiplier_trail = 2.5

[strategy.time]
entry_start = "09:20:00"
entry_end   = "15:00:00"
flatten     = "15:15:00"
```

## Core Rust Types

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PositionSide { Long, Short }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VixTier { Low, Normal, Reduced, High, Halt }

#[derive(Clone, Debug)]
pub struct StrategyConfig {
    pub sl_underlying_pct: f64,
    pub be_underlying_pct: f64,
    pub time_stop_min: u32,
    pub base_rr: f64,
    pub rr_low_vix: f64,
    pub rr_elevated_vix: f64,
    pub rr_high_vix: f64,
    pub scale_out_ratio: f64,
    pub atr_period: usize,
    pub atr_trail_mult: f64,
}

#[derive(Clone, Debug)]
pub struct DynamicTargets {
    pub t1_underlying: f64,      // price level on underlying
    pub t2_underlying: f64,
    pub stop_underlying: f64,
    pub move_be_underlying: f64, // threshold to move SL to BE
}
```

## ATR Utility (Wilder’s style)

```rust
#[derive(Clone, Debug)]
pub struct Atr {
    period: usize,
    value: Option<f64>,
}

impl Atr {
    pub fn new(period: usize) -> Self { Self { period, value: None } }

    pub fn update(&mut self, high: f64, low: f64, prev_close: f64) -> f64 {
        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        match self.value {
            None => { self.value = Some(tr); },
            Some(prev) => {
                // Wilder smoothing
                let k = (self.period as f64 - 1.0) / self.period as f64;
                let new_val = prev * k + tr / self.period as f64;
                self.value = Some(new_val);
            }
        }
        self.value.unwrap()
    }

    pub fn current(&self) -> Option<f64> { self.value }
}
```

## VIX Tiering

```rust
pub fn vix_tier(vix: f64, normal_min: f64, normal_max: f64, reduced_min: f64, reduced_max: f64, high_min: f64, high_max: f64, halt_above: f64) -> VixTier {
    if vix >= halt_above { return VixTier::Halt; }
    if (high_min..high_max).contains(&vix) { return VixTier::High; }
    if (reduced_min..reduced_max).contains(&vix) { return VixTier::Reduced; }
    if (normal_min..normal_max).contains(&vix) { return VixTier::Normal; }
    VixTier::Low
}
```

## Target Engine

```rust
pub struct DynamicTargetEngine {
    cfg: StrategyConfig,
}

impl DynamicTargetEngine {
    pub fn new(cfg: StrategyConfig) -> Self { Self { cfg } }

    fn rr_for_tier(&self, tier: VixTier) -> f64 {
        match tier {
            VixTier::Low => self.cfg.rr_low_vix,
            VixTier::Normal => self.cfg.base_rr,
            VixTier::Reduced => self.cfg.rr_elevated_vix,
            VixTier::High => self.cfg.rr_high_vix,
            VixTier::Halt => self.cfg.rr_high_vix, // no entries; keep conservative for completeness
        }
    }

    // Compute dynamic targets based on underlying reference and stop distance
    pub fn compute_targets(&self, side: PositionSide, entry_underlying: f64, vix_tier: VixTier) -> DynamicTargets {
        let risk = entry_underlying * self.cfg.sl_underlying_pct; // risk in price units
        let rr = self.rr_for_tier(vix_tier);
        let be_move = entry_underlying * self.cfg.be_underlying_pct;

        match side {
            PositionSide::Long => DynamicTargets {
                stop_underlying: entry_underlying - risk,
                t1_underlying: entry_underlying + rr * risk,
                t2_underlying: entry_underlying + (rr + 0.5) * risk, // second leg richer than T1
                move_be_underlying: entry_underlying + be_move,
            },
            PositionSide::Short => DynamicTargets {
                stop_underlying: entry_underlying + risk,
                t1_underlying: entry_underlying - rr * risk,
                t2_underlying: entry_underlying - (rr + 0.5) * risk,
                move_be_underlying: entry_underlying - be_move,
            },
        }
    }

    // ATR-based trailing stop update (on underlying)
    pub fn update_trailing(&self, side: PositionSide, last_underlying: f64, atr_now: f64) -> f64 {
        let offset = self.cfg.atr_trail_mult * atr_now;
        match side {
            PositionSide::Long => last_underlying - offset,
            PositionSide::Short => last_underlying + offset,
        }
    }
}
```

## Option Exit Mapping

```rust
pub fn option_exit_price_from_underlying_move(option_last: f64, underlying_move_pct: f64, side: PositionSide) -> f64 {
    // Simple proportional mapping; refine with delta/vega if available
    let factor = 1.0 + underlying_move_pct * match side { PositionSide::Long => 1.0, PositionSide::Short => -1.0 };
    (option_last * factor).max(0.05) // respect tick size and non-negative price
}
```

## Position Management Loop (Skeleton)

```rust
pub struct Position {
    pub side: PositionSide,
    pub entry_underlying: f64,
    pub entry_option_price: f64,
    pub qty: i32,
    pub t1_hit: bool,
    pub stop_underlying: f64,
    pub t1_underlying: f64,
    pub t2_underlying: f64,
    pub move_be_underlying: f64,
}

impl Position {
    pub fn on_tick(&mut self, engine: &DynamicTargetEngine, underlying_price: f64, option_price: f64, atr_now: Option<f64>) -> Option<&'static str> {
        // Move to break-even
        if !self.t1_hit {
            match self.side {
                PositionSide::Long if underlying_price >= self.move_be_underlying => {
                    self.stop_underlying = self.entry_underlying; // BE
                }
                PositionSide::Short if underlying_price <= self.move_be_underlying => {
                    self.stop_underlying = self.entry_underlying; // BE
                }
                _ => {}
            }
        }

        // Trail using ATR if available and after T1
        if self.t1_hit {
            if let Some(atr) = atr_now {
                let trail = engine.update_trailing(self.side, underlying_price, atr);
                match self.side {
                    PositionSide::Long => { if trail > self.stop_underlying { self.stop_underlying = trail; } }
                    PositionSide::Short => { if trail < self.stop_underlying { self.stop_underlying = trail; } }
                }
            }
        }

        // Check targets and stop
        match self.side {
            PositionSide::Long => {
                if underlying_price <= self.stop_underlying { return Some("exit_stop"); }
                if !self.t1_hit && underlying_price >= self.t1_underlying { self.t1_hit = true; return Some("scale_out_t1"); }
                if underlying_price >= self.t2_underlying { return Some("exit_target2"); }
            }
            PositionSide::Short => {
                if underlying_price >= self.stop_underlying { return Some("exit_stop"); }
                if !self.t1_hit && underlying_price <= self.t1_underlying { self.t1_hit = true; return Some("scale_out_t1"); }
                if underlying_price <= self.t2_underlying { return Some("exit_target2"); }
            }
        }
        None
    }
}
```

## Example Wiring

```rust
fn open_position(engine: &DynamicTargetEngine, side: PositionSide, entry_underlying: f64, entry_option: f64, vix: f64, vix_cfg: (f64,f64,f64,f64,f64,f64,f64)) -> Option<Position> {
    let tier = vix_tier(vix, vix_cfg.0, vix_cfg.1, vix_cfg.2, vix_cfg.3, vix_cfg.4, vix_cfg.5, vix_cfg.6);
    if matches!(tier, VixTier::Halt) { return None; }
    let tgt = engine.compute_targets(side, entry_underlying, tier);
    Some(Position {
        side,
        entry_underlying,
        entry_option_price: entry_option,
        qty: 50,
        t1_hit: false,
        stop_underlying: tgt.stop_underlying,
        t1_underlying: tgt.t1_underlying,
        t2_underlying: tgt.t2_underlying,
        move_be_underlying: tgt.move_be_underlying,
    })
}
```

## Edge Cases & Rules

- VIX spike > 5 points in 10 minutes: trigger circuit breaker; flatten per policy.
- Gap mornings: widen ATM strike range and delay entries for N minutes; targets still computed from underlying.
- Partial fills: on scale_out_t1, halve qty; ensure idempotent order tags.
- EOD: force exit at flatten time; ignore further targets.

## Integration Notes

- Data: feed 1m/5m underlying OHLC and VIX; compute ATR from underlying.
- Orders: always map underlying triggers to option limit orders with small marketability cushion.
- Journaling: on each event (scale-out, trail, exit) write to JSONL for recovery.
- Risk: enforce daily loss limit and position caps before invoking engine.

## KPIs

- Execution: p95 order ack < 1.5s; reject ratio < 1%.
- Trading: PF ≥ 1.3, Sharpe ≥ 1.0, Max DD ≤ 10%, average slippage ≤ 0.2% premium.

## Test Plan

- Unit: ATR smoothing, VIX tier mapping, target computation for both sides, trailing updates monotonicity.
- Integration: paper stream with synthetic regimes (low/normal/high VIX), verify scale-out/trail/exit sequencing.
- End-to-end: replay 12+ months, ensure KPIs within acceptance before live.
