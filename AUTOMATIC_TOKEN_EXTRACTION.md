# 🔍 Automatic Token Extraction System

## Overview

The system **automatically extracts and identifies tokens** from the Angel One instrument master. No manual token lookup required!

## How It Works

### 1. **Download Instrument Master**
```rust
let instrument_cache = Arc::new(InstrumentCache::new(broker.clone()));
instrument_cache.refresh().await?;
```

The system downloads ~150,000+ instruments from Angel One, including:
- **Indices**: NIFTY, BANKNIFTY, FINNIFTY, etc.
- **Futures**: All active futures contracts
- **Options**: All active option contracts (CE/PE)
- **Stocks**: Individual stocks and their derivatives

### 2. **Intelligent Token Identification**

The `TokenExtractor` automatically identifies tokens using multiple strategies:

#### **Strategy 1: Exact Name Match**
```rust
// Looks for: name == "NIFTY" && instrument_type == "INDEX"
```

#### **Strategy 2: OPTIDX Pattern**
```rust
// Looks for: name == "NIFTY" && instrument_type == "OPTIDX" && exch_seg == "NSE"
```

#### **Strategy 3: Symbol Prefix**
```rust
// Looks for: symbol.starts_with("NIFTY") && instrument_type == "INDEX"
```

#### **Strategy 4: Special Patterns**
```rust
// NIFTY: "NIFTY 50", "Nifty 50", "NIFTY50"
// BANKNIFTY: "NIFTY BANK", "Nifty Bank"
// FINNIFTY: "NIFTY FIN SERVICE", "Nifty Fin Service"
```

### 3. **Automatic Categorization**

The extractor automatically categorizes instruments:

```rust
pub struct AssetTokens {
    pub underlying_name: String,
    pub spot_token: Option<String>,      // Index token
    pub spot_symbol: Option<String>,     // Index symbol
    pub futures: Vec<FutureToken>,       // All futures contracts
    pub options: Vec<OptionToken>,       // All option contracts
}
```

## Usage Examples

### Example 1: Extract All Tokens for NIFTY

```rust
use rustro::broker::TokenExtractor;

let instruments = instrument_cache.get_all_instruments().await;
let extractor = TokenExtractor::new(instruments);

let nifty_tokens = extractor.extract_asset_tokens("NIFTY");

// Access spot token
if let Some(token) = nifty_tokens.spot_token {
    println!("NIFTY spot token: {}", token);
}

// Access futures
for future in &nifty_tokens.futures {
    println!("Future: {} (token: {}, expiry: {})", 
             future.symbol, future.token, future.expiry);
}

// Access options
for option in &nifty_tokens.options {
    println!("Option: {} (token: {}, strike: {}, type: {})",
             option.symbol, option.token, option.strike, option.option_type);
}
```

### Example 2: Get ATM Options Automatically

```rust
// Automatically calculate ATM strike and get surrounding options
let current_price = 23500.0;
let strike_increment = 50;  // NIFTY uses 50-point strikes
let range_strikes = 5;      // ±5 strikes from ATM

let atm_options = extractor.get_atm_options(
    "NIFTY",
    current_price,
    strike_increment,
    range_strikes
);

// Returns options from 23250 to 23750 (23500 ± 5*50)
for opt in atm_options {
    println!("{} - Strike: {}", opt.symbol, opt.strike);
}
```

### Example 3: Get Nearest Expiry Options

```rust
// Automatically find and filter to nearest expiry
let nearest_options = extractor.get_nearest_expiry_options("NIFTY");

println!("Nearest expiry: {}", nearest_options[0].expiry);
println!("Total options: {}", nearest_options.len());
```

### Example 4: Get Options in Specific Range

```rust
// Get options between specific strikes
let options = extractor.get_options_in_range(
    "BANKNIFTY",
    48500,  // min strike
    49500,  // max strike
    None    // any expiry
);

println!("Found {} BANKNIFTY options between 48500-49500", options.len());
```

### Example 5: Extract All Major Indices

```rust
// Extract tokens for NIFTY, BANKNIFTY, FINNIFTY at once
let all_tokens = extractor.extract_all_indices();

for (index_name, tokens) in all_tokens {
    println!("{}: {} futures, {} options",
             index_name,
             tokens.futures.len(),
             tokens.options.len());
}
```

## Command-Line Utilities

### 1. Extract and Display Tokens

```bash
cargo run --bin extract_tokens --release
```

**Output:**
```
🔍 Automatic Token Extraction Utility
=====================================

📥 Downloading instrument master from Angel One...
✅ Downloaded 156,234 instruments

📊 Analyzing instrument master...
   Total instruments: 156,234
   NSE: 45,678, NFO: 98,456, BSE: 12,100

🎯 Extracting tokens for major indices...

📈 NIFTY:
   ✅ Spot Token: 99926000
      Symbol: NIFTY 50
   
   📊 Futures Contracts: 3
      [1] NIFTY23DEC23FUT (token: 12345, expiry: 28DEC2023, lot: 50)
      [2] NIFTY24JAN24FUT (token: 12346, expiry: 25JAN2024, lot: 50)
      [3] NIFTY24FEB24FUT (token: 12347, expiry: 29FEB2024, lot: 50)
   
   🎯 Option Contracts: 2,456
      CE: 1,228, PE: 1,228
      Strike range: 19000 to 28000
      Sample options:
         [1] NIFTY23DEC2350000CE (token: 54321, strike: 23500, expiry: 28DEC2023)
         [2] NIFTY23DEC2350000PE (token: 54322, strike: 23500, expiry: 28DEC2023)
         ... and 2,454 more

💾 Token mapping exported to: data/extracted_tokens.json
```

### 2. Sync Multi-Asset Historical Data

```bash
cargo run --bin sync_multi_asset --release
```

This uses the automatic token extractor internally to:
1. Find NIFTY, BANKNIFTY, FINNIFTY tokens automatically
2. Download historical data for underlying indices
3. Identify and download relevant option strikes
4. Export comprehensive sync report

## Integration with Historical Sync

The `MultiAssetHistoricalSync` module automatically uses `TokenExtractor`:

```rust
// In historical_sync_multi.rs
async fn find_underlying_token(&self, asset: UnderlyingAsset) -> Result<String> {
    let instruments = self.instrument_cache.get_all_instruments().await;
    
    // Automatic token extraction
    let extractor = TokenExtractor::new(instruments);
    let asset_tokens = extractor.extract_asset_tokens(asset.as_str());
    
    asset_tokens.spot_token
        .ok_or_else(|| TradingError::InstrumentNotFound(
            format!("{} underlying token not found", asset.as_str())
        ))
}
```

## Filter Logic Explained

### What Gets Filtered?

#### **1. Underlying Indices**
```rust
// Filters for:
- name == "NIFTY" (or BANKNIFTY, FINNIFTY)
- instrument_type == "INDEX" or "OPTIDX"
- exch_seg == "NSE"
```

#### **2. Futures Contracts**
```rust
// Filters for:
- name == underlying
- exch_seg == "NFO"
- instrument_type == "FUTIDX"
```

#### **3. Option Contracts**
```rust
// Filters for:
- name == underlying
- exch_seg == "NFO"
- instrument_type == "OPTIDX"
- symbol.ends_with("CE") or symbol.ends_with("PE")
- strike within configured range
```

### Strike Range Calculation

```rust
// Example for NIFTY at 23,500
let strike_increment = 50;  // From config
let current_price = 23500.0;

// Calculate ATM
let atm_strike = round(23500.0 / 50) * 50 = 23500

// Calculate range (±200 from config)
let min_strike = 23500 - 200 = 23300
let max_strike = 23500 + 200 = 23700

// Strikes included: 23300, 23350, 23400, 23450, 23500, 23550, 23600, 23650, 23700
// Total: 9 strikes × 2 (CE + PE) = 18 options
```

## Configuration

### Strike Configuration (config.toml)

```toml
# Strike filtering
strike_increment = 50           # NIFTY: 50, BANKNIFTY: 100
initial_strike_range = 200      # ±200 points from ATM
strike_subscription_count = 9   # Max 9 strikes per side

# Asset-specific lot sizes
[lot_size]
nifty = 50
banknifty = 15
finnifty = 40
```

### Filter Configuration (in code)

```rust
let filter_config = FilterConfig {
    include_spot: true,              // Download underlying index
    include_futures: false,          // Skip futures (set true if needed)
    include_options: true,           // Download options
    strike_range: 200,               // ±200 from ATM
    max_strikes_per_side: 9,         // Max 9 strikes
    expiry_filter: ExpiryFilter::NearestWeekly,  // Only nearest expiry
};
```

## Expiry Filtering

### Available Filters

```rust
pub enum ExpiryFilter {
    NearestWeekly,      // Only nearest weekly expiry
    NearestMonthly,     // Only nearest monthly expiry
    AllActive,          // All active expiries
    Specific(NaiveDate),// Specific expiry date
}
```

### Example Usage

```rust
// Get only nearest weekly expiry
let filter = FilterConfig {
    expiry_filter: ExpiryFilter::NearestWeekly,
    ..Default::default()
};

// Get specific expiry
let expiry_date = NaiveDate::from_ymd(2023, 12, 28);
let filter = FilterConfig {
    expiry_filter: ExpiryFilter::Specific(expiry_date),
    ..Default::default()
};
```

## Asset-Specific Details

### NIFTY
- **Strike Increment**: 50 points
- **Lot Size**: 50
- **Typical Price**: ~23,500
- **Example Strikes**: 23,000, 23,050, 23,100, ..., 24,000

### BANKNIFTY
- **Strike Increment**: 100 points
- **Lot Size**: 15
- **Typical Price**: ~49,000
- **Example Strikes**: 48,000, 48,100, 48,200, ..., 50,000

### FINNIFTY
- **Strike Increment**: 50 points
- **Lot Size**: 40
- **Typical Price**: ~22,000
- **Example Strikes**: 21,500, 21,550, 21,600, ..., 22,500

## Export Token Mapping

The system can export all extracted tokens to JSON for reference:

```rust
extractor.export_tokens_to_file("data/extracted_tokens.json").await?;
```

**Output JSON Structure:**
```json
{
  "NIFTY": {
    "underlying_name": "NIFTY",
    "spot_token": "99926000",
    "spot_symbol": "NIFTY 50",
    "futures": [
      {
        "token": "12345",
        "symbol": "NIFTY23DEC23FUT",
        "expiry": "28DEC2023",
        "lot_size": 50
      }
    ],
    "options": [
      {
        "token": "54321",
        "symbol": "NIFTY23DEC2350000CE",
        "strike": 23500.0,
        "option_type": "CE",
        "expiry": "28DEC2023",
        "lot_size": 50
      }
    ]
  },
  "BANKNIFTY": { ... },
  "FINNIFTY": { ... }
}
```

## Summary

✅ **No Manual Token Lookup**: System automatically finds all tokens  
✅ **Intelligent Pattern Matching**: Multiple strategies to identify instruments  
✅ **Automatic Categorization**: Futures and options automatically separated  
✅ **Strike Range Calculation**: ATM and surrounding strikes auto-calculated  
✅ **Expiry Filtering**: Nearest weekly/monthly expiry auto-selected  
✅ **Multi-Asset Support**: NIFTY, BANKNIFTY, FINNIFTY all supported  
✅ **Export Capability**: Token mapping can be saved for reference  

## Next Steps

1. **Run Token Extraction**:
   ```bash
   cargo run --bin extract_tokens --release
   ```

2. **Review Extracted Tokens**:
   ```bash
   cat data/extracted_tokens.json
   ```

3. **Sync Historical Data**:
   ```bash
   cargo run --bin sync_multi_asset --release
   ```

4. **Use in Your Code**:
   ```rust
   let extractor = TokenExtractor::new(instruments);
   let tokens = extractor.extract_asset_tokens("NIFTY");
   ```

The system handles all the complexity of token identification automatically! 🚀

