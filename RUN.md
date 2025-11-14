# How to Run Rustro Trading Bot

This guide provides step-by-step instructions to run the Rustro trading bot on your system.

## 📋 Prerequisites

Before running the bot, ensure you have:

1. **Rust installed** (version 1.70 or higher)

   - Install from [rustup.rs](https://rustup.rs)
   - Verify installation: `rustc --version`

2. **Angel One Trading Account**

   - Active trading account with Angel One
   - API access enabled

3. **API Credentials**

   - Client Code
   - Password
   - MPIN
   - TOTP Secret (Base32 encoded)
   - API Key and Secret Key

4. **Network Access**
   - Internet connection for API calls
   - Port access for WebSocket connections (if not using paper trading)

## 🚀 Quick Start

### Step 1: Clone the Repository

```bash
git clone <your-repo-url>
cd rustro
```

### Step 2: Build the Project

Build the project in release mode for optimal performance:

```bash
cargo build --release
```

**Note:** First build may take several minutes as it compiles all dependencies.

### Step 3: Configure the Bot

1. Open `config.toml` in a text editor
2. Update the following fields with your credentials:

```toml
angel_one_client_code = "YOUR_CLIENT_CODE"
angel_one_password = "YOUR_PASSWORD"
angel_one_mpin = "YOUR_MPIN"
angel_one_totp_secret = "YOUR_TOTP_SECRET_BASE32"
angel_one_api_key = "YOUR_API_KEY"
angel_one_secret_key = "YOUR_SECRET_KEY"
```

3. **Important:** Set paper trading mode for initial testing:

```toml
enable_paper_trading = true  # Set to false for live trading
```

### Step 4: Create Data Directory

**⚠️ IMPORTANT:** The bot needs a `data` directory to store logs, events, and historical data. Create it before running:

```bash
# On Windows (PowerShell)
mkdir data

# On Windows (CMD)
mkdir data

# On Linux/Mac
mkdir -p data
```

**Note:** While the bot attempts to create this automatically, it's recommended to create it manually to avoid permission issues, especially on Windows.

### Step 5: Run the Bot

#### Option A: Using Cargo (Recommended)

**Release mode (optimized):**

```bash
cargo run --release
```

**Debug mode (faster compilation, slower execution):**

```bash
cargo run
```

**With custom config path:**

```bash
CONFIG_PATH=config.toml cargo run --release
```

#### Option B: Using Pre-built Binary

After building, run the executable directly:

**Windows:**

```bash
target\release\rustro.exe
```

**Linux/Mac:**

```bash
./target/release/rustro
```

#### Option C: Using Batch Script (Windows)

Simply double-click `run.bat` or run from command prompt:

```bash
run.bat
```

This script:

- Shows a welcome message
- Runs the bot in release mode
- Pauses after execution (useful for viewing errors)

## 🖥️ Platform-Specific Instructions

### Windows

1. **Using PowerShell:**

   ```powershell
   cd C:\path\to\rustro
   cargo run --release
   ```

2. **Using Command Prompt:**

   ```cmd
   cd C:\path\to\rustro
   cargo run --release
   ```

3. **Using run.bat:**
   - Double-click `run.bat` in File Explorer
   - Or run: `.\run.bat` in PowerShell/CMD

### Linux/Mac

1. **Using Terminal:**

   ```bash
   cd /path/to/rustro
   cargo run --release
   ```

2. **Run as background process:**

   ```bash
   nohup cargo run --release > bot.log 2>&1 &
   ```

3. **Using systemd (Linux):**
   Create a service file for automatic startup (see Advanced section below)

## ⚙️ Configuration Options

### Paper Trading vs Live Trading

**Paper Trading (Recommended for testing):**

```toml
enable_paper_trading = true
```

- Simulates trades without real money
- No actual orders placed
- Safe for testing strategies

**Live Trading:**

```toml
enable_paper_trading = false
```

- Places real orders
- Uses real money
- ⚠️ **Use with caution!**

### Logging Configuration

```toml
log_level = "info"  # Options: "trace", "debug", "info", "warn", "error"
audit_trail_enabled = true  # Enable event logging
```

### Trading Windows

```toml
entry_window_start = "10:00:00"  # Start time for entries (IST)
entry_window_end = "15:00:00"   # End time for entries (IST)
eod_exit_time = "15:20:00"      # Mandatory exit time (IST)
```

## 🔍 Verifying the Bot is Running

When the bot starts successfully, you should see:

```
🚀 Starting Rustro Trading Bot...
✅ Configuration loaded
🔐 Initializing session...
✅ Valid tokens loaded from file
📡 WebSocket enabled for real-time data
✅ Session initialized successfully
🏁 Trading bot starting main loop...
```

### Check Logs

The bot creates several log files in the `data/` directory:

- `data/events.jsonl` - All events (trades, signals, errors)
- `data/trades_YYYYMMDD.json` - Daily trade summary
- `data/positions_YYYYMMDD.jsonl` - Position history
- `data/daily_bias_latest.json` - Latest daily bias calculation

## 🛑 Stopping the Bot

### Graceful Shutdown

Press `Ctrl+C` to initiate graceful shutdown:

- Bot will close all open positions
- Save all pending data
- Complete shutdown sequence

**Expected output:**

```
⚠️  Ctrl+C received - initiating graceful shutdown
🛑 Starting shutdown sequence...
💾 Saved X trades
✅ Shutdown completed in Xs
👋 Goodbye!
```

### Force Stop

If the bot is unresponsive:

- Press `Ctrl+C` multiple times
- On Windows: Close the terminal window
- On Linux: `kill -9 <process_id>`

## 🔧 Troubleshooting

### Build Errors

**Error: "rustc not found"**

- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart terminal after installation

**Error: "linker not found" (Linux)**

- Install build essentials: `sudo apt-get install build-essential`

### Runtime Errors

**Error: "Configuration file not found"**

- Ensure `config.toml` exists in the project root
- Or set `CONFIG_PATH` environment variable

**Error: "Authentication failed"**

- Verify TOTP secret is correct (Base32 format)
- Check system time is synchronized
- Ensure Angel One account is active

**Error: "WebSocket connection failed"**

- Check internet connection
- Verify firewall settings
- Bot will fall back to REST API automatically

**Error: "File I/O error: The system cannot find the file specified. (os error 2)"**

This error typically occurs when the `data/` directory doesn't exist or cannot be created. Fix it by:

1. **Manually create the data directory:**

   ```bash
   # Windows PowerShell
   mkdir data

   # Windows CMD
   mkdir data

   # Linux/Mac
   mkdir -p data
   ```

2. **Check directory permissions:**

   - Ensure you have write permissions in the project directory
   - On Windows, run PowerShell/CMD as Administrator if needed

3. **Verify the directory was created:**

   ```bash
   # Windows
   dir data

   # Linux/Mac
   ls -la data
   ```

4. **If the error persists:**
   - Check if antivirus is blocking file creation
   - Verify disk space is available
   - Try running from a different directory location

**Error: "Insufficient daily bars"**

- Bot needs historical data to start
- Wait for initial data sync (happens automatically)
- Or run historical sync manually (see Advanced section)

### Data Issues

**Missing historical data:**

- Bot automatically syncs data on first run
- Check `data/bars/` directory for sync reports
- Manual sync: See Advanced section

**Token expiration:**

- Bot automatically refreshes tokens
- Check `data/tokens.json` for token status

## 📊 Monitoring the Bot

### Real-time Monitoring

Watch the console output for:

- Entry signals: `🎯 Entry signal generated!`
- Position updates: `📈 Position updated: ...`
- Risk warnings: `⚠️ Risk check failed: ...`

### Log Analysis

View recent events:

```bash
# Windows PowerShell
Get-Content data\events.jsonl -Tail 50

# Linux/Mac
tail -n 50 data/events.jsonl
```

### Daily Reports

Check daily trade summary:

```bash
# Windows
type data\trades_YYYYMMDD.json

# Linux/Mac
cat data/trades_YYYYMMDD.json
```

## 🔐 Security Best Practices

1. **Never commit credentials** to Git

   - Add `config.toml` to `.gitignore`
   - Use environment variables in production

2. **Use environment variables:**

   ```bash
   export ANGEL_CLIENT_CODE="..."
   export ANGEL_PASSWORD="..."
   export ANGEL_TOTP_SECRET="..."
   ```

3. **Separate API keys** for dev/prod environments

4. **Enable 2FA** on your Angel One account

5. **Start with paper trading** before going live

## 🚀 Advanced Usage

### Running with Environment Variables

```bash
# Windows PowerShell
$env:CONFIG_PATH="config.toml"
cargo run --release

# Linux/Mac
CONFIG_PATH=config.toml cargo run --release
```

### Running as a Service (Linux)

Create `/etc/systemd/system/rustro.service`:

```ini
[Unit]
Description=Rustro Trading Bot
After=network.target

[Service]
Type=simple
User=your_username
WorkingDirectory=/path/to/rustro
ExecStart=/path/to/rustro/target/release/rustro
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable rustro
sudo systemctl start rustro
sudo systemctl status rustro
```

### Manual Historical Data Sync

Run the sync utility:

```bash
cargo run --release --bin sync_multi_asset
```

### Running Multiple Instances

Each instance needs:

- Separate config file
- Separate data directory (or different session UUIDs)
- Different API keys (if rate limiting is an issue)

## 📝 Next Steps

After successfully running the bot:

1. **Monitor for a few days** in paper trading mode
2. **Review daily reports** in `data/trades_*.json`
3. **Adjust strategy parameters** in `config.toml` if needed
4. **Check risk settings** before going live
5. **Enable live trading** only after thorough testing

## 🆘 Getting Help

If you encounter issues:

1. Check the console output for error messages
2. Review `data/events.jsonl` for detailed event logs
3. Check existing documentation:
   - `README.md` - Project overview
   - `QUICKSTART.md` - Quick setup guide
   - `CREDENTIALS_SETUP.md` - Credential configuration
4. Review logs in `data/` directory

## ⚠️ Important Notes

- **Market Hours:** Bot only trades during NSE market hours (9:15 AM - 3:30 PM IST)
- **Holidays:** Bot automatically skips trading on NSE holidays
- **First Run:** Initial data sync may take several minutes
- **Paper Trading:** Always test in paper mode first
- **Risk Management:** Review and adjust risk parameters before live trading

---

**Happy Trading! 🦀**
