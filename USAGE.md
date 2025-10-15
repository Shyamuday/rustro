# Usage Guide

## First Time Setup

### 1. Install Prerequisites

- Install Rust: https://rustup.rs
- Create Angel One account
- Enable API access in Angel One

### 2. Get Angel One Credentials

#### Client Code & Password
- Your Angel One login credentials

#### TOTP Secret
1. Open Angel One mobile app
2. Go to Settings → API
3. Enable TOTP
4. Copy the **base32 secret** (not the 6-digit code)
   - Should look like: `JBSWY3DPEHPK3PXP`

### 3. Configure

Edit `config.toml`:

```toml
angel_one_client_code = "A12345"
angel_one_password = "YourPassword"
angel_one_totp_secret = "JBSWY3DPEHPK3PXP"
```

### 4. Build & Run

```bash
# Build (first time only)
cargo build --release

# Run
cargo run --release
```

## Understanding the Bot

### Strategy Overview

The bot uses a **Multi-Timeframe ADX Strategy**:

1. **Daily Analysis (9:30 AM)**
   - Calculates Daily ADX, +DI, -DI
   - Determines direction: CE (Call), PE (Put), or NO_TRADE
   - Only trades if ADX > threshold (strong trend)

2. **Hourly Analysis (Every hour)**
   - Calculates Hourly ADX, +DI, -DI
   - Checks alignment with daily direction
   - Waits for hourly confirmation

3. **Entry Filters (10:00 AM - 3:00 PM)**
   - RSI not overbought/oversold
   - Price above/below EMA
   - VIX below threshold
   - Risk checks passed

4. **Position Management**
   - 20% stop loss on option premium
   - Trailing stop activates at 2% profit
   - Trails 1.5% below highs
   - Auto-exits at 3:20 PM

### Event Flow

```
Market Open (9:15)
    ↓
Daily Direction Analysis (9:30)
    ↓
Entry Window Opens (10:00)
    ↓
Hourly Analysis (10:15, 11:15, 12:15, etc.)
    ↓
Entry Signal (if all filters pass)
    ↓
Position Opened
    ↓
Continuous Monitoring
    ↓
Exit (Stop Loss / Target / EOD)
    ↓
Position Closed
```

## Daily Operations

### Morning Routine

**9:00 AM - Pre-Market**
- Bot starts automatically
- Authenticates with Angel One
- Downloads instrument master
- Loads historical data

**9:15 AM - Market Open**
- Subscribes to NIFTY, VIX data
- Starts collecting ticks
- Aggregates bars (1min, 5min, 1hour)

**9:30 AM - Daily Direction**
- Bot analyzes daily timeframe
- Determines trading direction
- Emits `DAILY_DIRECTION_DETERMINED` event

### During Market Hours

**10:00 AM - Entry Window Opens**
- Bot can now place new trades
- Waits for hourly alignment
- Evaluates entry filters

**Every Hour (10:15, 11:15, 12:15, etc.)**
- Hourly bar completes
- Bot runs hourly analysis
- Checks for entry signals

**Continuous**
- Updates positions with current prices
- Monitors stop loss / trailing stop
- Checks VIX for circuit breaker
- Tracks daily P&L

### End of Day

**3:20 PM - Mandatory Exit**
- Bot closes all open positions
- No questions asked
- Uses market orders if needed

**3:30 PM - Market Close**
- Bot saves daily trades to JSON
- Resets daily counters
- Prepares for next day

## Monitoring

### Console Output

```
2025-10-15T15:30:00Z [INFO] Starting trading bot...
2025-10-15T15:30:01Z [INFO] Configuration loaded
2025-10-15T15:30:02Z [INFO] Login successful, tokens expire at: 2025-10-16T03:30:00Z
2025-10-15T15:30:03Z [INFO] Session initialized successfully
2025-10-15T15:30:04Z [INFO] Market closed - waiting for market open
```

### Event Log

All events are logged to `data/events.jsonl`:

```json
{"event_type":"DAILY_DIRECTION_DETERMINED","timestamp":"2025-10-15T04:00:00Z","payload":{"direction":"CE","daily_adx":28.5}}
{"event_type":"HOURLY_ALIGNMENT_CONFIRMED","timestamp":"2025-10-15T04:45:00Z","payload":{"hourly_adx":24.2}}
{"event_type":"SIGNAL_GENERATED","timestamp":"2025-10-15T04:45:10Z","payload":{"strike":19500,"option_type":"CE"}}
```

### Trade History

Daily trades saved to `data/trades_YYYYMMDD.json`:

```json
[
  {
    "trade_id": "uuid...",
    "symbol": "NIFTY24OCT19500CE",
    "entry_time": "2025-10-15T04:45:15Z",
    "entry_price": 125.50,
    "exit_time": "2025-10-15T09:50:00Z",
    "exit_price": 148.25,
    "pnl_net": 1087.50,
    "exit_reason": "TRAILING_STOP"
  }
]
```

## Risk Management

### VIX Circuit Breaker

**Scenario 1: VIX Spike During Trading**
```
VIX crosses 30 → Circuit breaker ACTIVE
    ↓
All open positions closed immediately
    ↓
No new entries until VIX < 22
```

**Scenario 2: High VIX at Market Open**
```
Market opens, VIX = 32
    ↓
Bot in NO_TRADE mode
    ↓
Monitors VIX throughout the day
    ↓
When VIX < 22, resumes normal operation
```

### Daily Loss Limit

```
Daily P&L reaches -2% of capital
    ↓
All positions closed
    ↓
Bot stops taking new trades
    ↓
Resets next day
```

### Position Limits

- **Max positions**: 3 concurrent
- **Max quantity**: Respects freeze quantity
- **Consecutive losses**: Stops after 3 losses

## Common Scenarios

### Scenario 1: Clean Entry & Target Hit

```
09:30 → Daily: CE direction (ADX=28)
10:15 → Hourly: Aligned (ADX=24)
10:15 → Entry signal generated
10:15 → Order placed: NIFTY 19500 CE @ 125
10:16 → Order filled
10:45 → Trailing stop activated (PNL = 2.1%)
11:30 → Target hit → Position closed
Result: +8.5% profit
```

### Scenario 2: Stop Loss Hit

```
09:30 → Daily: PE direction
10:15 → Entry: NIFTY 19500 PE @ 110
10:30 → Price moves against us
10:45 → Stop loss triggered @ 88 (-20%)
Result: -20% loss (as expected)
```

### Scenario 3: VIX Spike

```
10:15 → Position opened
11:30 → VIX spikes to 32
11:30 → Circuit breaker activated
11:30 → Position closed immediately @ market price
11:45 → VIX = 31 (still high)
12:30 → VIX drops to 21
12:30 → Circuit breaker deactivated
12:45 → Bot resumes trading (if other conditions met)
```

### Scenario 4: Daily Loss Limit

```
Trade 1: -1.2% loss
Trade 2: -0.9% loss
Total: -2.1% (exceeds -2% limit)
    ↓
All positions closed
No new trades for rest of day
```

### Scenario 5: Alignment Lost

```
10:15 → Entry: CE position opened
12:15 → Hourly bar: Alignment lost (ADX weak)
12:15 → Exit signal generated
12:15 → Position closed
Reason: "ALIGNMENT_LOST"
```

## Configuration Tuning

### Conservative Settings

For lower risk:

```toml
option_stop_loss_pct = 0.15          # Tighter stop
trail_activate_pnl_pct = 0.015       # Activate earlier
max_positions = 2                    # Fewer positions
daily_loss_limit_pct = 1.5           # Stricter limit
vix_threshold = 22.0                 # Lower VIX threshold
daily_adx_threshold = 28.0           # Higher ADX (stronger trends)
```

### Aggressive Settings

For higher risk/reward:

```toml
option_stop_loss_pct = 0.25          # Wider stop
trail_activate_pnl_pct = 0.03        # Activate later
max_positions = 5                    # More positions
daily_loss_limit_pct = 3.0           # Higher limit
vix_threshold = 28.0                 # Higher VIX tolerance
daily_adx_threshold = 22.0           # Lower ADX (more trades)
```

## Troubleshooting

### Bot Won't Start

**Error: "Authentication failed"**
- Check client code/password
- Verify TOTP secret is base32 (not the 6-digit code)
- Ensure system time is correct

**Error: "Token expired"**
- Bot will auto-refresh
- If fails, delete `data/tokens.json` and restart

**Error: "Market closed"**
- Normal behavior outside market hours
- Bot will wait for market open

### No Trades Happening

**1. Check Daily Direction**
- Look for `DAILY_DIRECTION_DETERMINED` event
- If `NO_TRADE`, daily ADX is too low (weak trend)

**2. Check Hourly Alignment**
- Look for `HOURLY_ALIGNMENT_CONFIRMED` event
- If missing, hourly and daily not aligned

**3. Check Entry Filters**
- Look for `ENTRY_FILTERS_EVALUATED` event
- See which filter failed (RSI, EMA, VIX, etc.)

**4. Check Risk Limits**
- VIX too high?
- Already hit daily loss limit?
- Max positions reached?

### Unexpected Exits

**Check exit reason in `data/trades_YYYYMMDD.json`:**

- `STOP_LOSS`: Hit 20% stop
- `TRAILING_STOP`: Trailed back from profit
- `TARGET`: Hit profit target
- `VIX_SPIKE`: Circuit breaker
- `DAILY_LOSS_LIMIT`: Risk limit
- `ALIGNMENT_LOST`: Strategy invalidated
- `EOD_MANDATORY_EXIT`: End of day

## Safety Features

### Graceful Shutdown (Ctrl+C)

```
User presses Ctrl+C
    ↓
Bot receives signal
    ↓
Closes all open positions
    ↓
Saves trade history
    ↓
Logs final events
    ↓
Exits cleanly
```

### Crash Recovery

If bot crashes unexpectedly:

1. All events are already logged (durable)
2. Restart bot
3. Bot will reconcile positions with broker
4. Resume normal operation

### Data Integrity

- **Atomic writes**: No partial data
- **Idempotency**: No duplicate trades
- **Event sourcing**: Full audit trail
- **Crash-safe**: Immediate disk sync

## Best Practices

### 1. Paper Trading First

- Set `enable_paper_trading = true` in config
- Run for 1-2 weeks
- Verify strategy performance
- Check all exit scenarios

### 2. Start Small

- Use 1-2 lots initially
- Gradually increase size
- Monitor P&L closely
- Adjust risk parameters

### 3. Monitor Regularly

- Check console output
- Review `data/events.jsonl` daily
- Analyze `data/trades_*.json`
- Track win rate, average P&L

### 4. Update Configuration

- Adjust based on market conditions
- Lower ADX threshold in ranging markets
- Tighten VIX limits during volatility
- Review settings weekly

### 5. Backup Data

```bash
# Daily backup
tar -czf backup_$(date +%Y%m%d).tar.gz data/
```

---

**Happy Trading! 🚀**

Remember: Past performance ≠ Future results. Trade responsibly.

