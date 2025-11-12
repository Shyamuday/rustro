# 🚀 Quick Reference Card

## Commands

```bash
# Extract index tokens (NIFTY, BANKNIFTY, FINNIFTY)
cargo run --bin extract_tokens --release

# Extract F&O stocks (popular stocks)
cargo run --bin extract_fno_stocks --release

# Extract ALL F&O stocks (takes 5-10 min)
cargo run --bin extract_all_fno_stocks --release

# Sync historical data (indices)
cargo run --bin sync_multi_asset --release

# Build release version
cargo build --release

# Run main trading bot
cargo run --release
```

## File Locations

```
data/
├── bars/
│   ├── nifty_daily.jsonl          # NIFTY daily bars
│   ├── nifty_hourly.jsonl         # NIFTY hourly bars
│   ├── banknifty_daily.jsonl      # BANKNIFTY daily bars
│   ├── banknifty_hourly.jsonl     # BANKNIFTY hourly bars
│   ├── finnifty_daily.jsonl       # FINNIFTY daily bars
│   ├── finnifty_hourly.jsonl      # FINNIFTY hourly bars
│   └── multi_asset_sync_report_*.json  # Sync reports
├── extracted_tokens.json          # Index token mapping
├── all_fno_stocks.json            # List of all F&O stocks
├── popular_fno_stocks_tokens.json # Popular stocks tokens
├── all_fno_stocks_complete.json   # ALL stocks (large file)
└── all_fno_stocks_summary.json    # Summary with counts
```

## Code Usage

### Extract Tokens (Indices)
```rust
use rustro::broker::TokenExtractor;

let extractor = TokenExtractor::new(instruments);
let tokens = extractor.extract_asset_tokens("NIFTY");
```

### Extract F&O Stocks
```rust
// Get all F&O stocks
let all_stocks = extractor.get_all_fno_stocks();

// Get popular stocks
let popular = extractor.get_popular_fno_stocks();

// Extract tokens for a stock
let reliance = extractor.extract_asset_tokens("RELIANCE");
```

### Get ATM Options
```rust
// For indices
let nifty_atm = extractor.get_atm_options(
    "NIFTY",    // underlying
    23500.0,    // current price
    50,         // strike increment
    5           // ±5 strikes
);

// For stocks
let reliance_atm = extractor.get_atm_options(
    "RELIANCE", // stock
    2500.0,     // current price
    50,         // strike increment
    5           // ±5 strikes
);
```

### Sync Historical Data
```rust
use rustro::data::{MultiAssetHistoricalSync, FilterConfig};

let syncer = MultiAssetHistoricalSync::new(broker, cache, config);
let report = syncer.sync_all_assets().await?;
```

## Configuration (config.toml)

```toml
strike_increment = 50           # NIFTY/FINNIFTY: 50, BANKNIFTY: 100
initial_strike_range = 200      # ±200 points from ATM
strike_subscription_count = 9   # Max 9 strikes per side

[lot_size]
nifty = 50
banknifty = 15
finnifty = 40
```

## Filter Configuration

```rust
let filter_config = FilterConfig {
    include_spot: true,
    include_futures: false,
    include_options: true,
    strike_range: 200,
    max_strikes_per_side: 9,
    expiry_filter: ExpiryFilter::NearestWeekly,
};
```

## Strike Calculation

```
Current Price: 23,500
Strike Increment: 50
Range: ±200

ATM = round(23,500 / 50) × 50 = 23,500
Min = 23,500 - 200 = 23,300
Max = 23,500 + 200 = 23,700

Strikes: 23,300, 23,350, ..., 23,700 (9 strikes)
Options: 9 × 2 (CE + PE) = 18 options
```

## Documentation

| File | Purpose |
|------|---------|
| `AUTOMATIC_TOKEN_EXTRACTION.md` | Detailed docs |
| `MULTI_ASSET_QUICK_START.md` | Quick start |
| `TOKEN_FILTER_SUMMARY.md` | Complete summary |
| `IMPLEMENTATION_SUMMARY.md` | Implementation |
| `QUICK_REFERENCE.md` | This file |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Token not found | Run `extract_tokens` first |
| Rate limit exceeded | Increase delays in code |
| Login failed | Check credentials in `config.toml` |
| Some options failed | Normal for new options |

## Daily Workflow

```bash
# 1. Sync data (morning)
cargo run --bin sync_multi_asset --release

# 2. Check report
cat data/bars/multi_asset_sync_report_*.json

# 3. Start bot
cargo run --release
```

## Key Features

✅ Automatic token extraction  
✅ Multi-asset support (NIFTY, BANKNIFTY, FINNIFTY)  
✅ Intelligent strike filtering  
✅ Expiry filtering  
✅ Rate limiting  
✅ Error resilience  
✅ Comprehensive reporting  

**No manual token lookup needed!** 🎉

