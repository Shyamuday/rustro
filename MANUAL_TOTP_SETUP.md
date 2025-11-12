# Manual TOTP Setup Guide

## ✅ What's Been Configured

The bot has been modified to accept **manual TOTP input** from the terminal instead of auto-generating it.

### Your Credentials (in config.toml)
- ✅ Client Code: `S736247`
- ✅ Password: `Lumia620@`
- ✅ API Key: `TJrbZ2ba`
- ✅ Secret Key: `c3359fe8-44a8-48f7-8a49-5ef84d780ea4`
- ✅ TOTP: **Manual Entry** (you'll be prompted)
- ✅ Paper Trading: **ENABLED** (safe testing mode)

## 🔐 How Manual TOTP Works

### When You Start the Bot

```bash
cargo run --release
```

You'll see:

```
🚀 Starting Rustro Trading Bot...
✅ Configuration loaded
🔄 Tokens expired - logging in

🔐 Angel One Login Required
Please enter the 6-digit TOTP code from your authenticator app:
> _
```

**Just enter the 6-digit code from your authenticator app and press Enter!**

### When Will You Need to Enter TOTP?

1. **At Startup**: Every time you start the bot
2. **Daily at 3:30 AM**: When tokens expire (if bot runs overnight)
3. **After Errors**: If authentication fails and needs retry

### Token Validity

Once you enter TOTP and login successfully:
- ✅ Tokens valid until **3:30 AM next day**
- ✅ No need to re-enter TOTP during the day
- ✅ Bot auto-refreshes as needed

## 🚀 Next Steps

### Before Running

You need to install C++ build tools. Choose ONE option:

#### Option A: Visual Studio C++ Tools (Recommended)

1. Open **Visual Studio Installer**
2. Click **"Modify"** on Visual Studio 2022 Community
3. Check **"Desktop development with C++"**
4. Click **"Modify"** and install

#### Option B: MinGW (Quick Alternative)

Run PowerShell **as Administrator**:

```powershell
choco install mingw -y
rustup default stable-x86_64-pc-windows-gnu
```

### After Installing Build Tools

1. **Close and reopen your terminal**
2. Navigate to project:
   ```bash
   cd C:\Users\Admin\Desktop\paradigm\rustro
   ```
3. **Build the project**:
   ```bash
   cargo build --release
   ```
4. **Run the bot**:
   ```bash
   cargo run --release
   ```
5. **Enter TOTP when prompted**

## 📊 What Happens After Login

### Paper Trading Mode (Currently Enabled)

- ✅ Connects to Angel One for real market data
- ✅ Runs strategy and generates signals
- ✅ Simulates order execution (no real trades)
- ✅ Tracks P&L in memory
- ✅ Safe for testing!

### To Enable Live Trading

Edit `config.toml`:

```toml
enable_paper_trading = false
```

⚠️ **Only do this after thorough testing!**

## 🛑 Stopping the Bot

Press **Ctrl+C** to stop gracefully:

```
[INFO] Ctrl+C received - initiating graceful shutdown
[WARN] Closing 1 open positions
[INFO] Shutdown sequence completed
```

The bot will:
1. Close all open positions
2. Save trade history
3. Exit cleanly

## 📝 Monitoring

### Console Output
Real-time logs show:
- Login status
- Market data
- Strategy signals
- Position updates
- P&L changes

### Files Created
- `data/events.jsonl` - Full event audit trail
- `data/tokens.json` - Saved tokens (auto-refresh)
- `data/trades_YYYYMMDD.json` - Daily trade history
- `data/bars/` - Historical bar data

## ⚠️ Important Notes

1. **TOTP Timing**: The 6-digit code changes every 30 seconds. Enter it quickly!
2. **Token Storage**: Tokens are saved to `data/tokens.json` for reuse
3. **Paper Trading**: Currently enabled - no real money at risk
4. **Market Hours**: Bot only trades 9:15 AM - 3:30 PM IST
5. **Holidays**: Automatically detects market holidays

## 🔒 Security

- Your credentials are in `config.toml` (local file)
- **Never commit config.toml to Git!**
- Consider using environment variables for production
- Tokens are encrypted in `data/tokens.json`

## 🆘 Troubleshooting

### "Authentication Failed"
- Check TOTP code is correct
- Ensure you entered it within 30 seconds
- Verify password is correct

### "Build Failed - linker not found"
- Install C++ build tools (see above)
- Restart terminal after installation

### "Market Closed"
- Normal outside 9:15 AM - 3:30 PM IST
- Bot will wait for market open

## 📞 Support

Check the logs in `data/events.jsonl` for detailed error messages.

---

**Ready to trade! 🚀**










