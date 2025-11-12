# 🚀 Multi-Asset Historical Sync - Quick Start

## What This Does

Automatically downloads historical data for:
- ✅ **NIFTY** (spot + options)
- ✅ **BANKNIFTY** (spot + options)
- ✅ **FINNIFTY** (spot + options)
- ✅ **Futures** (optional)

**No manual token lookup needed!** The system automatically identifies all tokens from the instrument master.

## Quick Start (3 Steps)

### Step 1: Extract Tokens (First Time Only)

```bash
cargo run --bin extract_tokens --release
```

This will:
- Download instrument master from Angel One
- Automatically identify all tokens
- Export token mapping to `data/extracted_tokens.json`

**Output:**
```
✅ NIFTY spot token: 99926000
✅ BANKNIFTY spot token: 99926009
✅ FINNIFTY spot token: 99926037
💾 Token mapping exported to: data/extracted_tokens.json
```

### Step 2: Sync Historical Data

```bash
cargo run --bin sync_multi_asset --release
```

This will:
- Download 365 days of daily data for each index
- Download 30 days of hourly data for each index
- Download option data for relevant strikes (±200 points from ATM)
- Save comprehensive sync report

**Output:**
```
🚀 Starting MULTI-ASSET historical data synchronization
📊 [1/3] Processing NIFTY...
✅ NIFTY sync complete: 19 instruments, 8,234 bars
📊 [2/3] Processing BANKNIFTY...
✅ BANKNIFTY sync complete: 19 instruments, 8,156 bars
📊 [3/3] Processing FINNIFTY...
✅ FINNIFTY sync complete: 19 instruments, 8,098 bars

✅ Multi-asset sync complete!
   Duration: 245s
   Total instruments: 57
   Total bars: 24,488
   Success rate: 98.2%
```

### Step 3: Use in Your Trading Bot

```rust
use rustro::data::{MultiAssetHistoricalSync, FilterConfig, ExpiryFilter};

// Create syncer
let syncer = MultiAssetHistoricalSync::new(
    broker.clone(),
    instrument_cache.clone(),
    config.clone()
);

// Sync all assets
let report = syncer.sync_all_assets().await?;

println!("Downloaded {} bars for {} instruments", 
         report.total_bars_downloaded,
         report.total_instruments);
```

## Configuration

### Basic Configuration (config.toml)

```toml
# Strike filtering
strike_increment = 50           # NIFTY/FINNIFTY: 50, BANKNIFTY: 100
initial_strike_range = 200      # ±200 points from ATM
strike_subscription_count = 9   # Max 9 strikes per side

# Lot sizes
[lot_size]
nifty = 50
banknifty = 15
finnifty = 40
```

### Advanced Filter Configuration (in code)

```rust
use rustro::data::{FilterConfig, ExpiryFilter};

let filter_config = FilterConfig {
    include_spot: true,              // Download underlying index
    include_futures: false,          // Skip futures
    include_options: true,           // Download options
    strike_range: 200,               // ±200 from ATM
    max_strikes_per_side: 9,         // Max 9 strikes
    expiry_filter: ExpiryFilter::NearestWeekly,  // Only nearest expiry
};

let syncer = syncer.with_filter_config(filter_config);
```

## What Gets Downloaded?

### For Each Asset (NIFTY, BANKNIFTY, FINNIFTY):

#### 1. **Underlying Index**
- 365 days of daily bars
- 30 days of hourly bars
- Stored in: `data/bars/{asset}_daily.jsonl` and `data/bars/{asset}_hourly.jsonl`

#### 2. **Option Contracts**
- Strikes within ±200 points of ATM
- Only nearest weekly expiry (configurable)
- Both CE and PE for each strike
- 30 days of daily bars per option
- 7 days of hourly bars per option

**Example for NIFTY at 23,500:**
- Strikes: 23,300, 23,350, 23,400, 23,450, 23,500, 23,550, 23,600, 23,650, 23,700
- Total: 9 strikes × 2 (CE + PE) = **18 options**

#### 3. **Futures** (if enabled)
- All active futures contracts
- 60 days of daily bars per future
- 14 days of hourly bars per future

## Understanding the Filter Logic

### Strike Range Calculation

```
Current Price: 23,500 (NIFTY)
Strike Increment: 50
Strike Range: ±200

ATM Strike = round(23,500 / 50) × 50 = 23,500

Min Strike = 23,500 - 200 = 23,300
Max Strike = 23,500 + 200 = 23,700

Strikes: 23,300, 23,350, 23,400, 23,450, 23,500, 23,550, 23,600, 23,650, 23,700
Total: 9 strikes
```

### Expiry Filtering

```rust
// Only nearest weekly expiry
ExpiryFilter::NearestWeekly

// Only nearest monthly expiry
ExpiryFilter::NearestMonthly

// All active expiries
ExpiryFilter::AllActive

// Specific expiry date
ExpiryFilter::Specific(NaiveDate::from_ymd(2023, 12, 28))
```

## File Structure

After running the sync, you'll have:

```
data/
├── bars/
│   ├── nifty_daily.jsonl          # NIFTY daily bars
│   ├── nifty_hourly.jsonl         # NIFTY hourly bars
│   ├── banknifty_daily.jsonl      # BANKNIFTY daily bars
│   ├── banknifty_hourly.jsonl     # BANKNIFTY hourly bars
│   ├── finnifty_daily.jsonl       # FINNIFTY daily bars
│   ├── finnifty_hourly.jsonl      # FINNIFTY hourly bars
│   └── multi_asset_sync_report_*.json  # Sync report
└── extracted_tokens.json          # Token mapping reference
```

## Sync Report

After sync completes, check the report:

```bash
cat data/bars/multi_asset_sync_report_*.json
```

**Sample Report:**
```json
{
  "timestamp": "2023-12-15T10:30:45Z",
  "duration_sec": 245,
  "assets_synced": [
    {
      "asset": "NIFTY",
      "underlying_token": "99926000",
      "underlying_bars": 395,
      "futures_synced": 0,
      "options_synced": 18,
      "total_daily_bars": 935,
      "total_hourly_bars": 846,
      "strikes_covered": [23300, 23350, 23400, 23450, 23500, 23550, 23600, 23650, 23700],
      "errors": []
    },
    {
      "asset": "BANKNIFTY",
      "underlying_token": "99926009",
      "underlying_bars": 395,
      "futures_synced": 0,
      "options_synced": 18,
      "total_daily_bars": 935,
      "total_hourly_bars": 846,
      "strikes_covered": [48500, 48600, 48700, 48800, 48900, 49000, 49100, 49200, 49300],
      "errors": []
    },
    {
      "asset": "FINNIFTY",
      "underlying_token": "99926037",
      "underlying_bars": 395,
      "futures_synced": 0,
      "options_synced": 18,
      "total_daily_bars": 935,
      "total_hourly_bars": 846,
      "strikes_covered": [21700, 21750, 21800, 21850, 21900, 21950, 22000, 22050, 22100],
      "errors": []
    }
  ],
  "total_instruments": 57,
  "total_bars_downloaded": 24488,
  "total_errors": 0,
  "success_rate": 100.0
}
```

## Customization Examples

### Example 1: Wider Strike Range

```rust
let filter_config = FilterConfig {
    strike_range: 500,  // ±500 instead of ±200
    max_strikes_per_side: 15,  // More strikes
    ..Default::default()
};
```

### Example 2: Include Futures

```rust
let filter_config = FilterConfig {
    include_futures: true,  // Enable futures
    ..Default::default()
};
```

### Example 3: All Expiries

```rust
let filter_config = FilterConfig {
    expiry_filter: ExpiryFilter::AllActive,  // All expiries
    ..Default::default()
};
```

### Example 4: Sync Single Asset

```rust
use rustro::data::UnderlyingAsset;

// Sync only NIFTY
let report = syncer.sync_single_asset(UnderlyingAsset::Nifty).await?;

// Sync only BANKNIFTY
let report = syncer.sync_single_asset(UnderlyingAsset::BankNifty).await?;
```

## Troubleshooting

### Issue: "Token not found"

**Solution:** Run the token extractor first:
```bash
cargo run --bin extract_tokens --release
```

### Issue: "Rate limit exceeded"

**Solution:** The sync includes automatic rate limiting. If you still hit limits, increase delays in `historical_sync_multi.rs`:
```rust
sleep(tokio::time::Duration::from_secs(2)).await;  // Increase from 2 to 5
```

### Issue: "Some options failed to sync"

**Reason:** Options that haven't existed for the full lookback period will fail (normal).

**Solution:** Check the sync report for details. Errors are logged but don't stop the sync.

### Issue: "Login failed"

**Solution:** Check your credentials in `config.toml`:
```toml
angel_one_client_code = "YOUR_CLIENT_CODE"
angel_one_password = "YOUR_PASSWORD"
angel_one_totp_secret = "YOUR_TOTP_SECRET"
angel_one_api_key = "YOUR_API_KEY"
```

## Performance

### Expected Sync Times

| Assets | Instruments | Bars | Time |
|--------|-------------|------|------|
| NIFTY only | ~19 | ~8,000 | ~80s |
| NIFTY + BANKNIFTY | ~38 | ~16,000 | ~160s |
| All 3 indices | ~57 | ~24,000 | ~240s |

**Note:** Times vary based on API rate limits and network speed.

### Rate Limiting

The system includes automatic rate limiting:
- 500ms delay between options
- 2s delay between assets
- Respects Angel One API limits

## Next Steps

1. ✅ **Extract tokens** (first time only)
2. ✅ **Sync historical data** (run daily or as needed)
3. ✅ **Use in backtesting** or **live trading**

## Integration with Main Bot

The main bot (`src/main.rs`) can use this for initial data load:

```rust
// At startup, sync historical data
let syncer = MultiAssetHistoricalSync::new(
    broker.clone(),
    instrument_cache.clone(),
    config.clone()
);

info!("📥 Syncing historical data...");
let report = syncer.sync_all_assets().await?;
info!("✅ Synced {} bars for {} instruments", 
      report.total_bars_downloaded,
      report.total_instruments);

// Then start live trading
start_live_trading().await?;
```

## Summary

✅ **Automatic Token Discovery**: No manual lookup needed  
✅ **Multi-Asset Support**: NIFTY, BANKNIFTY, FINNIFTY  
✅ **Intelligent Filtering**: ATM strikes, nearest expiry  
✅ **Comprehensive Reports**: Detailed sync statistics  
✅ **Rate Limit Handling**: Automatic delays  
✅ **Error Resilient**: Continues on individual failures  

**Ready to go!** 🚀

