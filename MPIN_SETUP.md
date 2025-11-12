# 🔐 Angel One MPIN Setup Guide

## ⚠️ Important Change

Angel One has updated their API authentication and now requires **MPIN (Mobile PIN)** instead of password for login.

## What is MPIN?

MPIN is a 4-6 digit PIN that you use to login to the Angel One mobile app. It's different from your trading password.

---

## 📱 How to Find/Set Your MPIN

### Option 1: Check Your Angel One Mobile App

1. Open the **Angel One mobile app**
2. When you login, you use your MPIN (not password)
3. This is the 4-6 digit PIN you enter
4. **This is what you need!**

### Option 2: Reset Your MPIN

If you don't remember your MPIN:

1. Open Angel One mobile app
2. On login screen, tap **"Forgot MPIN?"**
3. Follow the verification process (OTP to registered mobile)
4. Set a new MPIN (4-6 digits)
5. **Remember this MPIN** - you'll need it for the bot

### Option 3: Contact Angel One Support

- **Phone**: 1800 123 4555
- **Email**: support@angelbroking.com
- Ask them to help you reset your MPIN

---

## ⚙️ Update Your Configuration

Once you have your MPIN, update `config.toml`:

```toml
angel_one_mpin = "1234"  # Replace with your actual MPIN (4-6 digits)
```

### Full Configuration Example:

```toml
angel_one_client_code = "S736247"
angel_one_password = "Lumia620@"           # Keep this (might be needed for other operations)
angel_one_mpin = "1234"                    # Your 4-6 digit MPIN
angel_one_totp_secret = "YOUR_TOTP_SECRET" # Or leave as is for manual entry
angel_one_api_key = "TJrbZ2ba"
angel_one_secret_key = "c3359fe8-44a8-48f7-8a49-5ef84d780ea4"
```

---

## 🚀 Run the Bot

After updating your MPIN in `config.toml`:

```bash
cargo run --release
```

The bot will now:
1. ✅ Use your MPIN for authentication
2. ✅ Prompt for TOTP code (or auto-generate if you have the secret)
3. ✅ Login successfully
4. ✅ Start trading!

---

## 🔍 Verify It's Working

When you run the bot, you should see:

```
🚀 Starting Rustro Trading Bot...
✅ Configuration loaded
🔐 Initializing session...
🆕 No tokens found - logging in
🔐 Using MPIN for authentication

🔐 Angel One Login Required
Please enter the 6-digit TOTP code from your authenticator app:
(Attempt 1/3)
> [enter your TOTP]

📡 Login API Response:
   Status: 200 OK
   Body: {"status":true,"message":"SUCCESS","data":{...}}

✅ Login successful! Tokens expire at: 2025-11-13 03:30:00 UTC
```

---

## ❌ Troubleshooting

### Error: "LoginbyPassword is not allowed"

This means you haven't set the MPIN yet. Update `config.toml` with:

```toml
angel_one_mpin = "YOUR_ACTUAL_MPIN"
```

### Error: "Invalid MPIN"

- Double-check your MPIN is correct
- Try logging into the Angel One mobile app with the same MPIN
- If it doesn't work there either, reset your MPIN

### Error: "Invalid TOTP"

- Make sure you're entering the code quickly (expires in 30 seconds)
- The TOTP code changes every 30 seconds - wait for a fresh one
- You have 3 attempts before the bot exits

### Still Using Password?

The bot will detect if MPIN is not set and fall back to password, but Angel One API will reject it. You **must** set the MPIN.

---

## 🔒 Security Notes

1. **Keep your MPIN secure**
   - Don't share it with anyone
   - Don't commit `config.toml` to Git

2. **MPIN vs Password**
   - MPIN: Used for API login (4-6 digits)
   - Password: Your trading password (keep it in config for now)

3. **TOTP is still required**
   - MPIN replaces password
   - TOTP (2FA) is still mandatory for security

---

## 📊 What Happens After Login

Once authenticated with MPIN + TOTP:

1. ✅ Bot receives JWT and Feed tokens
2. ✅ Tokens saved to `data/tokens.json`
3. ✅ Valid until 3:30 AM IST next day
4. ✅ No need to re-enter MPIN/TOTP until tokens expire
5. ✅ Bot starts monitoring market and executing strategy

---

## 🆘 Need Help?

If you're stuck:

1. **Check the logs** - they now show detailed error messages
2. **Verify your MPIN** - test it in the Angel One mobile app
3. **Contact Angel One** - they can help with MPIN issues
4. **Check this guide** - make sure you've updated `config.toml` correctly

---

**Ready to trade with MPIN authentication! 🚀**

