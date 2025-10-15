# Rustro - Event-Driven Option Trading Bot

A high-performance, event-driven algorithmic trading bot for Indian options market (NSE) using Angel One SmartAPI.

## 🎯 Features

- **Event-Driven Architecture**: Data-driven, not time-driven - everything happens when data is ready
- **Multi-Timeframe ADX Strategy**: Daily direction + hourly alignment
- **Hybrid Bar Storage**: Ring buffer (memory) + JSONL (disk) for O(1) operations
- **Comprehensive Risk Management**: 
  - VIX circuit breaker
  - Daily loss limits
  - Trailing stop loss
  - Position sizing based on VIX and DTE
- **Order Management**: Retry logic with price adjustment and idempotency
- **Graceful Shutdown**: Closes all positions safely on Ctrl+C

## 📋 Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Angel One trading account
- Angel One API credentials (client code, password, TOTP secret)

## 🚀 Quick Start

### 1. Clone and Build

```bash
git clone <your-repo-url>
cd rustro
cargo build --release
```

### 2. Configure

Edit `config.toml` with your credentials:

```toml
angel_one_client_code = "YOUR_CLIENT_CODE"
angel_one_password = "YOUR_PASSWORD"
angel_one_totp_secret = "YOUR_TOTP_SECRET_BASE32"
```

**⚠️ SECURITY**: In production, use environment variables:

```bash
export ANGEL_CLIENT_CODE="..."
export ANGEL_PASSWORD="..."
export ANGEL_TOTP_SECRET="..."
```

### 3. Create Data Directory

```bash
mkdir -p data
```

### 4. Run

```bash
cargo run --release
```

Or with custom config:

```bash
CONFIG_PATH=config.toml cargo run --release
```

## 📚 Architecture

### Event Flow

```
TICK → BAR_READY → HOURLY_ANALYSIS → SIGNAL_GENERATED → ORDER_EXECUTED
```

### Key Components

- **Event Bus**: Pub/sub system for all events with idempotency
- **Hybrid Bar Store**: Memory (500 bars) + Disk (JSONL) for efficient storage
- **Angel One Client**: REST API with TOTP authentication
- **Strategy Engine**: ADX/RSI/EMA-based multi-timeframe strategy
- **Order Manager**: Retry logic with exponential backoff
- **Position Manager**: Real-time P&L tracking with stop loss
- **Risk Manager**: VIX monitoring and circuit breakers

### Directory Structure

```
rustro/
├── src/
│   ├── broker/         # Angel One API client
│   ├── config/         # Configuration loading
│   ├── data/           # Bar storage & tick buffer
│   ├── events/         # Event bus & types
│   ├── orders/         # Order management
│   ├── positions/      # Position tracking
│   ├── risk/           # Risk management
│   ├── strategy/       # Indicators & strategy
│   ├── utils/          # Utilities
│   ├── types.rs        # Core types
│   ├── error.rs        # Error types
│   ├── lib.rs          # Library exports
│   └── main.rs         # Application entry
├── config.toml         # Configuration
├── Cargo.toml          # Dependencies
└── README.md
```

## 🎛️ Configuration

Key configuration sections:

### Risk Parameters

```toml
option_stop_loss_pct = 0.20          # 20% stop loss on option premium
trail_activate_pnl_pct = 0.02        # Activate trailing at 2% profit
trail_gap_pct = 0.015                # Trail 1.5% below highs
max_positions = 3                    # Max concurrent positions
daily_loss_limit_pct = 2.0           # Daily loss limit: 2%
```

### VIX Circuit Breaker

```toml
vix_threshold = 25.0           # No new entries above this
vix_spike_threshold = 30.0     # Exit all positions above this
vix_resume_threshold = 22.0    # Resume trading below this
```

### Strategy Parameters

```toml
daily_adx_period = 14
daily_adx_threshold = 25.0
hourly_adx_period = 14
hourly_adx_threshold = 20.0
rsi_period = 14
ema_period = 20
```

## 📊 Data Management

### Bar Storage (Hybrid Approach)

- **Memory**: Last 500 bars (ring buffer) for fast access
- **Disk**: Complete history (JSONL format) for durability
- **Benefits**:
  - O(1) append operations
  - O(1) recent reads (from memory)
  - Crash-safe (immediate disk sync)
  - Memory-efficient (~50KB per symbol)

### Event Logging

All events logged to `data/events.jsonl` with:
- Idempotency keys (prevents duplicates)
- Full audit trail
- Recovery replay capability

## 🔒 Security Best Practices

1. **Never commit credentials** to Git
2. Use environment variables in production
3. Rotate TOTP secret periodically
4. Use separate API keys for dev/prod
5. Enable 2FA on Angel One account

## ⚠️ Risk Disclaimer

**THIS SOFTWARE IS FOR EDUCATIONAL PURPOSES ONLY.**

- Trading involves substantial risk of loss
- Past performance does not guarantee future results
- Test thoroughly in paper trading mode first
- Use at your own risk
- The authors are not responsible for any financial losses

## 🛠️ Development

### Run Tests

```bash
cargo test
```

### Check Lints

```bash
cargo clippy --all-targets --all-features
```

### Format Code

```bash
cargo fmt
```

### Build Documentation

```bash
cargo doc --open
```

## 📈 Monitoring

The bot logs all events to:
- Console (stdout) - Real-time logs
- `data/events.jsonl` - Event audit trail
- `data/trades_YYYYMMDD.json` - Daily trade history

## 🐛 Troubleshooting

### Authentication Failed

- Verify TOTP secret is correct (base32 encoded)
- Check system time is synchronized
- Ensure Angel One account is active

### Data Gaps

- Bot automatically recovers missing bars from REST API
- Pauses new entries during recovery
- Check `data/events.jsonl` for gap events

### Order Rejections

- Check margin availability
- Verify freeze quantity limits
- Check price bands (±20% circuit limits)

## 🤝 Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Write tests for new features
4. Submit a pull request

## 📄 License

MIT License - See LICENSE file

## 📞 Support

For issues and questions:
- GitHub Issues: [Create Issue]
- Documentation: [OPTION_TRADING_BOT_EVENT_SPEC.md]

## 🙏 Acknowledgments

- Angel One for SmartAPI
- Rust community for excellent async ecosystem
- Event-driven architecture inspired by production trading systems

---

**Built with ❤️ and Rust** 🦀

