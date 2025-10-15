# 🦀 Rust Installation & Running the Trading Bot

## 📋 Prerequisites

Before running the trading bot, you need:

1. ✅ Rust toolchain (rustc + cargo)
2. ✅ Angel One API credentials
3. ✅ Windows 10/11 (64-bit)

---

## 🔧 Step 1: Install Rust

### **Method 1: Using rustup (Recommended)**

1. **Download the installer:**

   - Visit: https://rustup.rs/
   - Or direct download: https://win.rustup.rs/x86_64

2. **Run the installer:**

   - Double-click `rustup-init.exe`
   - Press `1` to proceed with default installation
   - Wait for installation to complete

3. **Restart your terminal:**

   - Close PowerShell/Command Prompt
   - Open a new terminal

4. **Verify installation:**

   ```powershell
   cargo --version
   rustc --version
   rustup --version
   ```

   Expected output:

   ```
   cargo 1.xx.x
   rustc 1.xx.x
   rustup 1.xx.x
   ```

### **Method 2: Using Chocolatey**

If you have Chocolatey package manager:

```powershell
choco install rust
```

### **Method 3: Using winget**

If you have Windows Package Manager:

```powershell
winget install Rustlang.Rustup
```

---

## ⚙️ Step 2: Configure Credentials

1. **Open `config.toml` in a text editor**

2. **Update the following fields with your Angel One credentials:**

```toml
# Angel One SmartAPI Credentials
angel_one_client_code = "YOUR_CLIENT_CODE"        # Your Angel One client ID
angel_one_password = "YOUR_PASSWORD"              # Your Angel One password
angel_one_totp_secret = "YOUR_TOTP_SECRET"        # Base32 TOTP secret from authenticator app
angel_one_api_key = "YOUR_API_KEY"                # From Angel One developer portal
angel_one_secret_key = "YOUR_SECRET_KEY"          # From Angel One developer portal
```

3. **Save the file**

> **⚠️ Security Note:** Never commit `config.toml` with real credentials to Git!
> In production, use environment variables instead.

### **How to get TOTP Secret:**

1. When setting up 2FA in Angel One mobile app
2. Instead of scanning QR code, click "Can't scan?"
3. Copy the **Base32 secret key** (e.g., `JBSWY3DPEHPK3PXP`)
4. Paste it in `config.toml` as `angel_one_totp_secret`

---

## 🏗️ Step 3: Build the Project

Open PowerShell in the project directory and run:

```powershell
cargo build --release
```

**What this does:**

- ✅ Downloads all dependencies (tokio, reqwest, serde, etc.)
- ✅ Compiles the entire project with optimizations
- ✅ Creates executable at `target/release/rustro.exe`

**Expected time:** 2-5 minutes (first time only)

---

## 🚀 Step 4: Run the Trading Bot

### **Option A: Run with Cargo**

```powershell
cargo run --release
```

### **Option B: Run the Executable Directly**

```powershell
.\target\release\rustro.exe
```

### **Option C: Run with Custom Config**

```powershell
$env:CONFIG_PATH="path/to/custom/config.toml"; cargo run --release
```

---

## 📊 Expected Output

When the bot starts successfully, you'll see:

```
🚀 Starting Rustro Trading Bot...
✅ Configuration loaded
🔐 Initializing session...
✅ Valid tokens loaded from file
🔑 Tokens expire at: 2024-12-26T18:30:00Z
📥 Downloading instrument master...
✅ Instrument master downloaded: 25000+ instruments
✅ NIFTY token: 99926000
📡 WebSocket connected and subscribed
✅ Session initialized successfully
🏁 Trading bot starting main loop...
⏰ Market opens at 09:15:00 IST - waiting 120 minutes
```

---

## 🛑 Stopping the Bot

### **Graceful Shutdown:**

Press `Ctrl+C` in the terminal:

```
⚠️  Ctrl+C received - initiating graceful shutdown
🛑 Starting shutdown sequence...
⚠️  Closing 1 open positions
💾 Saved 3 trades
✅ Shutdown completed in 5s
👋 Goodbye!
```

This will:

- ✅ Close all open positions
- ✅ Save daily trades to `data/trades_YYYYMMDD.json`
- ✅ Write final audit logs
- ✅ Clean exit

---

## 📁 Directory Structure

After running the bot, these directories/files are created:

```
rustro/
├── data/
│   ├── events.jsonl                    # Event audit trail
│   ├── bars_nifty_daily.jsonl          # Daily OHLCV bars
│   ├── bars_nifty_hourly.jsonl         # Hourly OHLCV bars
│   ├── trades_20241226.json            # Daily trades
│   ├── tokens.json                     # Cached Angel One JWT tokens
│   └── tokens/
│       ├── angelone_master_20241226.csv
│       └── index_options_20241226.json
├── target/
│   └── release/
│       └── rustro.exe                  # Compiled executable
└── config.toml                         # Configuration file
```

---

## 🔍 Monitoring the Bot

### **1. Event Logs (Real-time):**

```powershell
Get-Content data/events.jsonl -Wait -Tail 20
```

### **2. Daily Trades:**

```powershell
Get-Content data/trades_20241226.json | ConvertFrom-Json | Format-Table
```

### **3. Check Current Positions:**

The bot logs position updates every minute:

```
🔍 Running hourly analysis...
✅ Hourly aligned with daily
🎯 Entry signal generated!
📈 Executing entry: CE @ 19500.0
✅ Order placed: ORDER123456
```

---

## 🐛 Troubleshooting

### **Issue 1: "cargo: command not found"**

**Solution:**

- Restart your terminal after installing Rust
- Or manually add to PATH:
  ```powershell
  $env:PATH += ";$env:USERPROFILE\.cargo\bin"
  ```

### **Issue 2: "Authentication failed"**

**Solution:**

- Check credentials in `config.toml`
- Verify TOTP secret is correct (Base32 format)
- Make sure Angel One account is active

### **Issue 3: "Failed to download instrument master"**

**Solution:**

- Check internet connection
- Angel One API might be down (check status)
- Try again during market hours

### **Issue 4: "No trading day (weekend or holiday)"**

**Solution:**

- This is expected! The bot automatically waits
- It uses NSE holiday calendar
- Will resume on next trading day

### **Issue 5: Build errors**

**Solution:**

```powershell
# Clean build artifacts and rebuild
cargo clean
cargo build --release
```

---

## 🧪 Testing (Paper Trading Mode)

To test without real money:

1. **Enable paper trading in `config.toml`:**

```toml
enable_paper_trading = true
```

2. **Run the bot:**

```powershell
cargo run --release
```

**Paper mode features:**

- ✅ Simulated order fills (5 bps slippage)
- ✅ No real API calls for orders
- ✅ Still fetches real market data
- ✅ Same logic as live trading

---

## 📈 Performance Optimization

The project is already optimized for release builds:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip debug symbols (smaller binary)
```

**Typical performance:**

- Binary size: ~5-8 MB
- Memory usage: ~20-50 MB
- Cold start: <2 seconds
- Order execution: <100ms

---

## 🔒 Security Best Practices

### **1. Never commit credentials:**

Add to `.gitignore`:

```
config.toml
data/tokens.json
*.log
```

### **2. Use environment variables in production:**

```powershell
$env:ANGEL_CLIENT_CODE="YOUR_CODE"
$env:ANGEL_PASSWORD="YOUR_PASSWORD"
$env:ANGEL_TOTP_SECRET="YOUR_SECRET"
cargo run --release
```

### **3. Rotate API keys regularly:**

Update keys in Angel One developer portal monthly.

---

## 📚 Additional Commands

### **Check for updates:**

```powershell
rustup update
```

### **Format code:**

```powershell
cargo fmt
```

### **Run linter:**

```powershell
cargo clippy
```

### **Run tests:**

```powershell
cargo test
```

### **View documentation:**

```powershell
cargo doc --open
```

### **Check dependencies:**

```powershell
cargo tree
```

---

## 🎯 Quick Start Checklist

- [ ] Install Rust via rustup
- [ ] Verify: `cargo --version` works
- [ ] Update `config.toml` with Angel One credentials
- [ ] Get TOTP secret from authenticator app
- [ ] Run: `cargo build --release`
- [ ] Run: `cargo run --release`
- [ ] Verify bot starts and authenticates
- [ ] Check `data/events.jsonl` for logs
- [ ] Monitor first trade execution
- [ ] Test graceful shutdown with `Ctrl+C`

---

## 🆘 Support

If you encounter issues:

1. **Check logs:** `data/events.jsonl`
2. **Verify config:** `config.toml` credentials
3. **Check market hours:** Bot only runs 9:15 AM - 3:30 PM IST
4. **Verify trading day:** Not weekends/holidays
5. **Angel One API status:** Check official status page

---

## 🚀 Ready to Go!

You're all set! The bot will:

✅ Auto-authenticate with Angel One API
✅ Download NIFTY instrument tokens
✅ Analyze daily trend (ADX on 1D bars)
✅ Wait for hourly alignment (ADX on 1H bars)
✅ Execute CE/PE trades at ATM strikes
✅ Manage stop loss & trailing stops
✅ Auto-exit at 3:20 PM
✅ Save all trades & audit logs

**Happy Trading! 📈**
