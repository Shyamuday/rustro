# Quick Start Guide

Get up and running in 5 minutes!

## Prerequisites

- **Rust installed**: https://rustup.rs
- **Angel One account** with API enabled
- **Angel One credentials**:
  - Client Code
  - Password
  - TOTP Secret (base32 format)

## Step 1: Setup (1 minute)

```bash
cd rustro
```

## Step 2: Configure (2 minutes)

Edit `config.toml`:

```toml
# REQUIRED: Your Angel One credentials
angel_one_client_code = "A12345"           # Your client code
angel_one_password = "YourPassword"        # Your password
angel_one_totp_secret = "JBSWY3DPEHPK3P"   # TOTP secret (base32)

# Risk settings (adjust as needed)
option_stop_loss_pct = 0.20               # 20% stop loss
daily_loss_limit_pct = 2.0                # 2% daily loss limit
max_positions = 3                         # Max 3 positions
```

## Step 3: Build (1 minute)

```bash
cargo build --release
```

## Step 4: Run (1 minute)

```bash
cargo run --release
```

## Expected Output

```
[INFO] Starting trading bot...
[INFO] Configuration loaded
[INFO] Login successful, tokens expire at: 2025-10-16T03:30:00Z
[INFO] Session initialized successfully
[INFO] Market closed - waiting for market open
```

## What Happens Next?

### If Market is Closed
- Bot waits for next market open (9:15 AM IST)
- You can press Ctrl+C to stop

### If Market is Open (9:15 AM - 3:30 PM IST)

**9:15 AM** - Market opens
- Bot authenticates
- Downloads instrument master
- Subscribes to data feeds

**9:30 AM** - Daily direction analysis
- Calculates daily ADX
- Determines direction: CE (bullish) / PE (bearish) / NO_TRADE
- You'll see: `[INFO] Daily direction determined: CE`

**10:00 AM** - Entry window opens
- Bot can now place trades
- Waits for hourly alignment

**10:15 AM** - First hourly analysis
- Checks if hourly aligns with daily
- Evaluates entry filters (RSI, EMA, VIX)
- If all pass → Places order

**During Market Hours**
- Monitors open positions
- Updates stop loss / trailing stop
- Watches VIX for circuit breaker
- Tracks daily P&L

**3:20 PM** - End of day
- Closes all open positions
- Saves trade history
- Resets counters

**3:30 PM** - Market close
- Bot enters standby mode
- Waits for next day

## First Trade Walkthrough

When bot places your first trade, you'll see:

```
[INFO] Daily direction determined: CE (ADX: 28.5)
[INFO] Hourly alignment confirmed (ADX: 24.2)
[INFO] Entry signal generated: CE @ strike 19500
[INFO] Order placed successfully: order_uuid
[INFO] Order executed @ 125.50
[INFO] Position opened: NIFTY24OCT19500CE x 50 @ 125.50
[INFO] Position updated: Current price: 128.30, PNL: +140.00 (+2.2%)
[INFO] Trailing stop activated @ 126.37
[INFO] Trailing stop updated @ 129.15
[INFO] Position closed: Exit @ 145.20, PNL: +985.00 (+15.7%) - TRAILING_STOP
```

## Monitoring

### Console
- Real-time logs appear in terminal
- Shows all important events
- Press Ctrl+C for graceful shutdown

### Event Log
- Check `data/events.jsonl` for full audit trail
- One event per line (JSON format)
- Useful for debugging

### Trade History
- Check `data/trades_YYYYMMDD.json` at end of day
- Contains all completed trades
- Includes entry/exit prices, P&L, reasons

## Stopping the Bot

Press **Ctrl+C** to stop:

```
[INFO] Ctrl+C received - initiating graceful shutdown
[WARN] Closing 1 open positions
[INFO] Position closed: Exit @ 132.40, Reason: Shutdown
[INFO] Completed 3 trades today
[INFO] Shutdown sequence completed in 2s
```

Bot will:
1. Close all open positions
2. Save trade history
3. Log final events
4. Exit cleanly

## Verification Checklist

Before live trading, verify:

- [ ] Bot starts without errors
- [ ] Authentication successful (check logs)
- [ ] Market hours detected correctly
- [ ] Daily direction analysis runs (9:30 AM)
- [ ] Hourly analysis runs every hour
- [ ] Stop loss works (test in paper mode)
- [ ] VIX circuit breaker activates (simulate)
- [ ] EOD exit happens at 3:20 PM
- [ ] Ctrl+C shutdown works cleanly

## Common Issues

### "Authentication failed"
- **Fix**: Check TOTP secret is base32 format
- **Verify**: Use TOTP app to confirm 6-digit codes match

### "Market closed"
- **Fix**: Normal behavior outside 9:15 AM - 3:30 PM IST
- **Action**: Bot will auto-start when market opens

### "No trades happening"
- **Reason 1**: Daily ADX below threshold (weak trend)
- **Reason 2**: Hourly not aligned with daily
- **Reason 3**: VIX too high
- **Action**: Check `data/events.jsonl` for `DAILY_DIRECTION_DETERMINED` event

### "Permission denied" on data/
- **Fix**: Create data directory: `mkdir -p data`
- **Fix**: Check write permissions: `chmod 755 data`

## Next Steps

### 1. Run in Paper Mode (Recommended)
Edit `config.toml`:
```toml
enable_paper_trading = true
```
Run for 1-2 weeks to verify strategy.

### 2. Adjust Risk Settings
Start conservative:
```toml
max_positions = 1                    # One position at a time
daily_loss_limit_pct = 1.5          # Stricter limit
base_position_size_pct = 5.0        # Smaller size
```

### 3. Monitor Daily
- Check trade history each evening
- Review event logs for errors
- Track win rate and average P&L
- Adjust configuration as needed

### 4. Scale Gradually
- Week 1: 1 lot, 1 position
- Week 2-4: 2 lots, 2 positions
- Month 2+: Scale to comfort level

## Full Documentation

- **Complete Guide**: [USAGE.md](USAGE.md)
- **Build Instructions**: [BUILD.md](BUILD.md)
- **Project Overview**: [README.md](README.md)
- **Status & Roadmap**: [PROJECT_STATUS.md](PROJECT_STATUS.md)
- **Event Specification**: [OPTION_TRADING_BOT_EVENT_SPEC.md](OPTION_TRADING_BOT_EVENT_SPEC.md)

## Support

Having issues? Check:
1. Event log: `data/events.jsonl`
2. Configuration: `config.toml`
3. Documentation: `USAGE.md`
4. GitHub Issues

## Safety Reminder

⚠️ **Before live trading:**
- Test in paper mode for 2+ weeks
- Start with 1-2 lots only
- Use only risk capital
- Monitor actively for first week
- Have a manual intervention plan

**Trading involves substantial risk. Use at your own risk.**

---

**Happy Trading! 🚀**

Remember: Start small, test thoroughly, scale gradually.

