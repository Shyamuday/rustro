# 🔐 Credentials Setup Guide

## ✅ Your Angel One API Credentials

I've configured most of your credentials in `config.toml`. Here's what you have:

```toml
angel_one_client_code = "S736247"
angel_one_password = "541992"
angel_one_api_key = "TJrbZ2ba"
angel_one_secret_key = "c3359fe8-44a8-48f7-8a49-5ef84d780ea4"
```

## 🔴 **CRITICAL: You Need the TOTP Secret**

The bot needs the **base32 TOTP secret** (not the 6-digit code that changes).

### How to Get Your TOTP Secret

#### Option 1: From Authenticator App (If You Set It Up Recently)

If you set up TOTP recently, you may have saved the base32 secret or QR code.

**The secret looks like this:**
```
JBSWY3DPEHPK3PXP
```
- All uppercase letters (A-Z) and numbers (2-7)
- Usually 16-32 characters long
- This is what the bot needs!

#### Option 2: From Angel One Dashboard

1. Go to https://smartapi.angelone.in/
2. Login with your credentials
3. Go to "My Apps" → "TradeKing"
4. Look for TOTP/2FA settings
5. You should see a QR code or base32 secret
6. Copy the base32 text string

#### Option 3: Reset TOTP (If You Can't Find It)

If you can't find your TOTP secret:
1. Go to Angel One SmartAPI dashboard
2. Disable TOTP/2FA
3. Re-enable it
4. **Save the base32 secret** when shown
5. Also scan QR in your authenticator app

### Current TOTP Code (476428)

You mentioned the current TOTP code is **476428**, but the bot needs the **permanent secret** that generates these codes, not the code itself.

---

## 📝 **Once You Have the TOTP Secret**

Update `config.toml`:

```toml
angel_one_totp_secret = "JBSWY3DPEHPK3PXP"  # Your actual base32 secret
```

---

## 🚀 **Quick Test (Without TOTP for Now)**

If you want to test while getting the TOTP secret, you can temporarily use a workaround:

### Create `.env` file:
```bash
ANGEL_CLIENT_CODE=S736247
ANGEL_PASSWORD=541992
ANGEL_TOTP_CODE=476428  # Manual entry (changes every 30s)
ANGEL_API_KEY=TJrbZ2ba
ANGEL_SECRET_KEY=c3359fe8-44a8-48f7-8a49-5ef84d780ea4
```

---

## 🎯 **What the Bot Does With These**

### Authentication Flow:
```
1. Bot reads config.toml
2. Generates TOTP from secret (every 30 seconds)
3. Calls Angel One login API with:
   - client_code: S736247
   - password: 541992
   - totp: <generated code>
   - API headers with your API key
4. Gets JWT token + Feed token
5. Saves tokens to data/tokens.json
6. Uses tokens for all subsequent API calls
7. Auto-refreshes before expiry (3:30 AM IST next day)
```

---

## 🔒 **Security Best Practices**

### ✅ DO:
- Keep TOTP secret safe and private
- Use `.gitignore` (already configured)
- Use environment variables in production
- Rotate secrets every 6 months
- Enable 2FA on Angel One account

### ❌ DON'T:
- Commit credentials to Git
- Share credentials publicly
- Use same credentials for dev/prod
- Store credentials in plain text on shared systems

---

## 🧪 **Testing Authentication**

Once you have the TOTP secret, test authentication:

### 1. Update config.toml
```toml
angel_one_totp_secret = "YOUR_BASE32_SECRET"
enable_paper_trading = true  # Safe testing
```

### 2. Run
```bash
cargo run --release
```

### 3. Expected Output
```
🚀 Starting Rustro Trading Bot...
✅ Configuration loaded
🔐 Initializing session...
🆕 No tokens found - logging in
✅ Login successful
🔑 Tokens expire at: 2025-10-16T03:30:00Z
✅ Session initialized successfully
```

### 4. If Login Fails
- Check TOTP secret is correct base32
- Verify system time is synchronized
- Test TOTP with authenticator app to confirm secret
- Check Angel One account is active

---

## 📞 **Getting Help**

### If TOTP Secret Not Available:

**Option A: Contact Me**
Send me a message when you have the TOTP secret, and I'll help you configure it.

**Option B: Manual TOTP Entry**
I can create a version that prompts for TOTP code each time (less automated but works).

**Option C: Angel One Support**
Contact Angel One support to get your TOTP secret or help resetting it.

---

## ✅ **Current Status**

### Configured ✅
- ✅ Client Code: S736247
- ✅ Password/PIN: 541992
- ✅ API Key: TJrbZ2ba
- ✅ Secret Key: c3359fe8-44a8-48f7-8a49-5ef84d780ea4

### Needed ⚠️
- ⚠️ TOTP Secret (base32 format)

### Once TOTP is Added:
- ✅ Bot will authenticate automatically
- ✅ Bot will generate TOTP codes automatically
- ✅ Bot will refresh tokens automatically
- ✅ You're ready to trade!

---

## 🎉 **You're Almost There!**

**Just need the TOTP secret and you're done!**

Once configured:
1. `cargo build --release`
2. `cargo run --release`
3. Bot handles everything else automatically!

**The bot is 100% complete and waiting for your TOTP secret!** 🚀

---

## 📚 **Documentation**

- Main guide: `100_PERCENT_COMPLETE.md`
- Quick start: `QUICKSTART.md`
- Usage guide: `USAGE.md`
- This guide: `CREDENTIALS_SETUP.md`

