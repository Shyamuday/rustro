# 🔐 How to Get Your Angel One TOTP Secret

## What You Need

To enable **automatic TOTP generation**, you need the **base32 TOTP secret** from Angel One.

This is NOT the 6-digit code that changes every 30 seconds. It's the **permanent secret key** that generates those codes.

---

## 📱 Method 1: From Your Authenticator App (If Recently Set Up)

If you recently set up 2FA and still have access to the setup screen:

### Google Authenticator
1. Unfortunately, Google Authenticator doesn't show the secret after setup
2. You'll need to use Method 2 or 3 below

### Authy
1. Open Authy app
2. Long-press on Angel One entry
3. Look for "Show Secret" or similar option
4. Copy the base32 string

### Microsoft Authenticator
1. Open the app
2. Tap on Angel One entry
3. Look for settings/details
4. May show the secret key

### Other Authenticator Apps
Most apps don't show the secret after initial setup. Proceed to Method 2.

---

## 🌐 Method 2: Reset 2FA on Angel One (Recommended)

This is the most reliable way to get your TOTP secret.

### Step 1: Login to Angel One SmartAPI Portal

1. Go to: **https://smartapi.angelone.in/**
2. Login with your credentials:
   - Client Code: `S736247`
   - Password: `Lumia620@`

### Step 2: Navigate to API Settings

1. Click on **"My Apps"** or **"API Settings"**
2. Look for your registered app (or create a new one)
3. Find **"2FA Settings"** or **"TOTP Configuration"**

### Step 3: Reset/Setup TOTP

1. Click **"Reset 2FA"** or **"Configure TOTP"**
2. You'll see a QR code and/or a text string
3. **IMPORTANT**: Before scanning the QR code, look for:
   - A text field labeled "Secret Key" or "Manual Entry Key"
   - A base32 string (looks like: `JBSWY3DPEHPK3PXP`)
   
4. **Copy this secret string** - this is what you need!

### Step 4: Complete the Setup

1. Scan the QR code with your authenticator app (or enter the secret manually)
2. Enter the 6-digit code to verify
3. Save the setup

### Step 5: Update config.toml

Open `config.toml` and update:

```toml
angel_one_totp_secret = "JBSWY3DPEHPK3PXP"  # Replace with your actual secret
```

---

## 🔍 Method 3: Extract from QR Code

If you only see a QR code and no text secret:

### Option A: Use a QR Code Reader

1. Take a screenshot of the QR code
2. Use an online QR decoder: https://zxing.org/w/decode
3. Upload the screenshot
4. The decoded text will look like:
   ```
   otpauth://totp/AngelOne:S736247?secret=JBSWY3DPEHPK3PXP&issuer=AngelOne
   ```
5. Extract the part after `secret=` and before `&` - that's your TOTP secret!

### Option B: Use a Browser Extension

1. Install a QR code reader extension (like "QR Code Reader" for Chrome)
2. Scan the QR code on the Angel One page
3. Extract the secret from the decoded URL

---

## ✅ What the TOTP Secret Looks Like

The secret is a **base32 encoded string**:

### Valid Format:
- Only contains: `A-Z` (uppercase) and `2-7` (numbers)
- Usually 16-32 characters long
- Examples:
  - `JBSWY3DPEHPK3PXP`
  - `GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ`
  - `MFRGGZDFMZTWQ2LK`

### Invalid (These are NOT the secret):
- ❌ `476428` (This is a TOTP code, not the secret)
- ❌ Contains lowercase letters
- ❌ Contains special characters like `@`, `#`, etc.
- ❌ Your password or API key

---

## 🔧 Update Your Configuration

Once you have the secret, update `config.toml`:

```toml
angel_one_totp_secret = "YOUR_ACTUAL_BASE32_SECRET_HERE"
```

For example:
```toml
angel_one_totp_secret = "JBSWY3DPEHPK3PXP"
```

**Remove the quotes if they're part of what you copied!**

---

## 🧪 Test the Setup

After updating `config.toml`:

1. Run the bot:
   ```bash
   cargo run --release
   ```

2. The bot should now:
   - ✅ Auto-generate TOTP codes
   - ✅ Login automatically
   - ✅ No manual prompt for TOTP

3. You'll see:
   ```
   🚀 Starting Rustro Trading Bot...
   ✅ Configuration loaded
   🔐 Initializing session...
   🆕 No tokens found - logging in
   ✅ Login successful, tokens expire at: 2025-11-13 03:30:00 UTC
   ```

---

## 🆘 Troubleshooting

### "Invalid TOTP secret"
- Check that you copied the entire secret
- Ensure there are no spaces or line breaks
- Verify it only contains A-Z and 2-7
- Make sure you didn't copy the TOTP code instead

### "Authentication Failed"
- The secret might be incorrect
- Try resetting 2FA and getting a fresh secret
- Verify your system time is correct (TOTP is time-based)

### "Parse error: EOF while parsing"
- This means the API key setup is wrong (we already fixed this!)
- If you still see this, check your `angel_one_api_key` in config.toml

### Still Getting Prompted for TOTP
- Check that `angel_one_totp_secret` is NOT set to `"YOUR_TOTP_SECRET"`
- Verify you saved the config.toml file
- Restart the bot after updating config

---

## 🔒 Security Notes

1. **Keep your TOTP secret safe!**
   - Anyone with this secret can generate your 2FA codes
   - Don't share it or commit it to Git

2. **Backup your secret**
   - Save it in a password manager
   - Store it securely in case you need to reconfigure

3. **Your authenticator app still works**
   - Both the app and the bot use the same secret
   - They'll generate the same codes at the same time

---

## 📞 Need Help?

If you can't find your TOTP secret:

1. **Contact Angel One Support**
   - They can help you reset 2FA
   - Phone: 1800 123 4555
   - Email: support@angelbroking.com

2. **Use Manual Entry (Current Method)**
   - Keep `angel_one_totp_secret = "YOUR_TOTP_SECRET"`
   - Enter codes manually when prompted
   - Still works fine, just less convenient

---

**Once you have the secret configured, the bot will run completely automatically! 🚀**

