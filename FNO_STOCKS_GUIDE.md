# 📈 F&O Stocks - Complete Guide

## Overview

The system now supports **automatic extraction of ALL F&O stocks** (stocks with Futures & Options) and their corresponding CE/PE options!

## What Are F&O Stocks?

F&O stocks are individual company stocks that have:
- **Futures contracts** (FUTSTK)
- **Options contracts** (OPTSTK) - both Call (CE) and Put (PE)

Examples: RELIANCE, TCS, HDFCBANK, INFY, ICICIBANK, etc.

## Quick Start

### 1. Extract Popular F&O Stocks (Recommended)

```bash
cargo run --bin extract_fno_stocks --release
```

**What it does:**
- Identifies all F&O stocks from instrument master
- Extracts detailed tokens for **20 popular stocks**
- Exports to `data/popular_fno_stocks_tokens.json`
- **Time**: ~2 minutes

**Popular stocks included:**
- RELIANCE, TCS, HDFCBANK, INFY, ICICIBANK
- HINDUNILVR, ITC, SBIN, BHARTIARTL, KOTAKBANK
- LT, AXISBANK, BAJFINANCE, ASIANPAINT, MARUTI
- TITAN, SUNPHARMA, WIPRO, ULTRACEMCO, TATAMOTORS

### 2. Extract ALL F&O Stocks (Optional)

```bash
cargo run --bin extract_all_fno_stocks --release
```

**What it does:**
- Extracts ALL F&O stocks (typically 150-200 stocks)
- Full details with futures and options
- Exports to `data/all_fno_stocks_complete.json`
- **Time**: ~5-10 minutes
- **File size**: ~50-100 MB

**⚠️ Warning:** Only run this if you need the complete list!

## Code Usage

### Get List of All F&O Stocks

```rust
use rustro::broker::TokenExtractor;

let extractor = TokenExtractor::new(instruments);

// Get all F&O stocks
let all_stocks = extractor.get_all_fno_stocks();
println!("Found {} F&O stocks", all_stocks.len());

// Get popular F&O stocks only
let popular = extractor.get_popular_fno_stocks();
```

### Extract Tokens for a Specific Stock

```rust
// Extract tokens for RELIANCE
let reliance_tokens = extractor.extract_asset_tokens("RELIANCE");

// Access spot token
if let Some(token) = reliance_tokens.spot_token {
    println!("RELIANCE spot token: {}", token);
}

// Access futures
for future in &reliance_tokens.futures {
    println!("Future: {} (expiry: {})", future.symbol, future.expiry);
}

// Access options
for option in &reliance_tokens.options {
    println!("Option: {} (strike: {}, type: {})", 
             option.symbol, option.strike, option.option_type);
}
```

### Get ATM Options for a Stock

```rust
// Get ATM options for RELIANCE at current price 2500
let atm_options = extractor.get_atm_options(
    "RELIANCE",  // stock name
    2500.0,      // current price
    50,          // strike increment (varies by stock)
    5            // ±5 strikes from ATM
);

// Returns options from 2250 to 2750
for opt in atm_options {
    println!("{} - Strike: {}", opt.symbol, opt.strike);
}
```

### Get Nearest Expiry Options

```rust
// Get options for nearest expiry only
let nearest = extractor.get_nearest_expiry_options("RELIANCE");

println!("Nearest expiry: {}", nearest[0].expiry);
println!("Total options: {}", nearest.len());
```

### Extract Popular Stocks in Bulk

```rust
// Extract all popular stocks at once
let popular_tokens = extractor.extract_popular_fno_stock_tokens();

for (stock, tokens) in popular_tokens {
    println!("{}: {} futures, {} options", 
             stock, 
             tokens.futures.len(), 
             tokens.options.len());
}
```

## Filter Logic for F&O Stocks

### Spot Token (Equity)

```rust
// Filters:
- name == stock_name OR symbol == stock_name
- exch_seg == "NSE"
- instrument_type == "EQUITY"
```

### Futures

```rust
// Filters:
- name == stock_name
- exch_seg == "NFO"
- instrument_type == "FUTSTK"
```

### Options

```rust
// Filters:
- name == stock_name
- exch_seg == "NFO"
- instrument_type == "OPTSTK"
- symbol.ends_with("CE") or symbol.ends_with("PE")
- strike within range (if specified)
```

## Strike Increments by Stock

Different stocks have different strike increments:

| Price Range | Strike Increment | Example Stocks |
|-------------|------------------|----------------|
| < 500 | 5 or 10 | ITC, WIPRO |
| 500-1000 | 10 or 20 | SBIN, AXISBANK |
| 1000-2000 | 20 or 50 | INFY, TCS |
| 2000-5000 | 50 or 100 | RELIANCE, HDFCBANK |
| > 5000 | 100 or 200 | MRF, EICHERMOT |

**Note:** The system automatically detects available strikes from the instrument master.

## Example: RELIANCE Options

### Typical Structure

```
RELIANCE (Current Price: 2,500)
├── Spot: RELIANCE-EQ (NSE)
├── Futures:
│   ├── RELIANCE24DEC24FUT (expiry: 26DEC2024)
│   ├── RELIANCE24JAN25FUT (expiry: 30JAN2025)
│   └── RELIANCE24FEB25FUT (expiry: 27FEB2025)
└── Options:
    ├── Strike 2400:
    │   ├── RELIANCE24DEC242400CE
    │   └── RELIANCE24DEC242400PE
    ├── Strike 2450:
    │   ├── RELIANCE24DEC242450CE
    │   └── RELIANCE24DEC242450PE
    ├── Strike 2500 (ATM):
    │   ├── RELIANCE24DEC242500CE
    │   └── RELIANCE24DEC242500PE
    ├── Strike 2550:
    │   ├── RELIANCE24DEC242550CE
    │   └── RELIANCE24DEC242550PE
    └── Strike 2600:
        ├── RELIANCE24DEC242600CE
        └── RELIANCE24DEC242600PE
```

## Output Files

### 1. `data/all_fno_stocks.json`

Simple list of all F&O stock names:

```json
[
  "RELIANCE",
  "TCS",
  "HDFCBANK",
  "INFY",
  ...
]
```

### 2. `data/popular_fno_stocks_tokens.json`

Detailed tokens for popular stocks:

```json
{
  "RELIANCE": {
    "underlying_name": "RELIANCE",
    "spot_token": "2885",
    "spot_symbol": "RELIANCE-EQ",
    "futures": [
      {
        "token": "54321",
        "symbol": "RELIANCE24DEC24FUT",
        "expiry": "26DEC2024",
        "lot_size": 250
      }
    ],
    "options": [
      {
        "token": "65432",
        "symbol": "RELIANCE24DEC242500CE",
        "strike": 2500.0,
        "option_type": "CE",
        "expiry": "26DEC2024",
        "lot_size": 250
      },
      {
        "token": "65433",
        "symbol": "RELIANCE24DEC242500PE",
        "strike": 2500.0,
        "option_type": "PE",
        "expiry": "26DEC2024",
        "lot_size": 250
      }
    ]
  }
}
```

### 3. `data/all_fno_stocks_complete.json`

Complete details for ALL F&O stocks (large file, ~50-100 MB).

### 4. `data/all_fno_stocks_summary.json`

Summary with counts for all stocks:

```json
[
  {
    "stock": "RELIANCE",
    "has_spot": true,
    "futures_count": 3,
    "options_count": 456,
    "ce_count": 228,
    "pe_count": 228
  },
  {
    "stock": "TCS",
    "has_spot": true,
    "futures_count": 3,
    "options_count": 412,
    "ce_count": 206,
    "pe_count": 206
  }
]
```

## Integration with Trading Bot

### Example: Trade RELIANCE Options

```rust
use rustro::broker::TokenExtractor;

// Get RELIANCE tokens
let extractor = TokenExtractor::new(instruments);
let reliance = extractor.extract_asset_tokens("RELIANCE");

// Get current price (from spot market data)
let current_price = 2500.0;

// Get ATM options
let atm_options = extractor.get_atm_options("RELIANCE", current_price, 50, 3);

// Filter for CE options only
let ce_options: Vec<_> = atm_options.iter()
    .filter(|o| o.option_type == "CE")
    .collect();

// Place order for ATM CE
if let Some(atm_ce) = ce_options.iter().find(|o| o.strike as i32 == 2500) {
    println!("Trading: {} (token: {})", atm_ce.symbol, atm_ce.token);
    // broker.place_order(&atm_ce.token, quantity, price).await?;
}
```

### Example: Multi-Stock Strategy

```rust
// Get popular stocks
let popular_tokens = extractor.extract_popular_fno_stock_tokens();

// For each stock, get ATM options
for (stock, tokens) in popular_tokens {
    if let Some(spot_token) = tokens.spot_token {
        // Get current price from market data
        let price = get_current_price(&spot_token).await?;
        
        // Get ATM options
        let atm = extractor.get_atm_options(&stock, price, 50, 2);
        
        // Apply your strategy
        analyze_and_trade(&stock, &atm).await?;
    }
}
```

## Popular F&O Stocks by Sector

### Banking & Finance
- HDFCBANK, ICICIBANK, SBIN, AXISBANK, KOTAKBANK, BAJFINANCE

### IT & Technology
- TCS, INFY, WIPRO, TECHM, HCLTECH

### Energy & Oil
- RELIANCE, ONGC, BPCL, IOC

### Automobile
- MARUTI, TATAMOTORS, M&M, EICHERMOT

### FMCG
- HINDUNILVR, ITC, NESTLEIND, BRITANNIA

### Telecom
- BHARTIARTL, IDEA (now merged)

### Pharma
- SUNPHARMA, DRREDDY, CIPLA, DIVISLAB

### Metals & Mining
- TATASTEEL, HINDALCO, JSWSTEEL, VEDL

### Infrastructure
- LT, ULTRACEMCO, GRASIM

### Consumer Goods
- TITAN, ASIANPAINT, PIDILITIND

## Lot Sizes

F&O stock lot sizes vary by stock:

| Stock | Lot Size | Example |
|-------|----------|---------|
| RELIANCE | 250 | 1 lot = 250 shares |
| TCS | 150 | 1 lot = 150 shares |
| HDFCBANK | 550 | 1 lot = 550 shares |
| INFY | 300 | 1 lot = 300 shares |
| ITC | 1600 | 1 lot = 1600 shares |

**Note:** Lot sizes are automatically extracted from the instrument master.

## Best Practices

### 1. Start with Popular Stocks
```bash
# Extract popular stocks first (faster)
cargo run --bin extract_fno_stocks --release
```

### 2. Use Dynamic Extraction
```rust
// Don't hardcode tokens, extract dynamically
let tokens = extractor.extract_asset_tokens("RELIANCE");
```

### 3. Check Liquidity
```rust
// Popular stocks have more options
let popular = extractor.get_popular_fno_stocks();
```

### 4. Filter by Expiry
```rust
// Trade nearest expiry for better liquidity
let nearest = extractor.get_nearest_expiry_options("RELIANCE");
```

### 5. Respect Lot Sizes
```rust
// Check lot size before placing order
let lot_size = reliance.futures[0].lot_size;
let quantity = lot_size * num_lots;
```

## Comparison: Indices vs Stocks

| Feature | Indices (NIFTY, etc.) | F&O Stocks |
|---------|----------------------|------------|
| Strike Increment | Fixed (50/100) | Varies (5-200) |
| Lot Size | Fixed | Varies by stock |
| Liquidity | Very high | Varies |
| Volatility | Lower | Higher |
| Capital Required | Higher | Lower (smaller lot sizes) |
| Number of Options | ~18 per expiry | Varies (50-500) |

## Troubleshooting

### Issue: Stock not found

**Solution:** Check if the stock has F&O:
```rust
let all_stocks = extractor.get_all_fno_stocks();
if all_stocks.contains(&"STOCKNAME".to_string()) {
    // Stock has F&O
}
```

### Issue: No options found

**Reason:** Stock might only have futures, not options.

**Solution:** Check futures:
```rust
let tokens = extractor.extract_asset_tokens("STOCKNAME");
println!("Futures: {}, Options: {}", 
         tokens.futures.len(), 
         tokens.options.len());
```

### Issue: Strike increment unknown

**Solution:** Extract all strikes and calculate:
```rust
let tokens = extractor.extract_asset_tokens("STOCKNAME");
let strikes: Vec<i32> = tokens.options.iter()
    .map(|o| o.strike as i32)
    .collect();
strikes.sort();
strikes.dedup();

// Calculate increment
if strikes.len() >= 2 {
    let increment = strikes[1] - strikes[0];
    println!("Strike increment: {}", increment);
}
```

## Summary

✅ **Automatic extraction** of all F&O stocks  
✅ **Popular stocks** pre-configured (20 stocks)  
✅ **Complete list** available (150-200 stocks)  
✅ **Futures and options** both supported  
✅ **CE and PE** automatically identified  
✅ **Lot sizes** extracted automatically  
✅ **Strike increments** detected from data  
✅ **Multiple expiries** supported  
✅ **Easy integration** with trading bot  

## Next Steps

1. **Extract popular stocks**:
   ```bash
   cargo run --bin extract_fno_stocks --release
   ```

2. **Review the output**:
   ```bash
   cat data/popular_fno_stocks_tokens.json
   ```

3. **Use in your code**:
   ```rust
   let tokens = extractor.extract_asset_tokens("RELIANCE");
   ```

4. **Trade options**:
   ```rust
   let atm = extractor.get_atm_options("RELIANCE", 2500.0, 50, 5);
   ```

**You now have access to ALL F&O stocks and their options!** 🚀

