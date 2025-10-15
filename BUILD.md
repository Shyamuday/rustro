# Build Instructions

## Prerequisites

### 1. Install Rust

Download and install Rust from [https://rustup.rs](https://rustup.rs)

**Windows:**
```powershell
# Download and run rustup-init.exe
# Or use winget:
winget install Rustlang.Rustup
```

**Linux/Mac:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, restart your terminal and verify:
```bash
rustc --version
cargo --version
```

### 2. Angel One Account

- Create an account at [Angel One](https://www.angelone.in)
- Enable API access
- Get your:
  - Client Code
  - Password
  - TOTP Secret (from Angel One app)

## Building the Project

### 1. Clone the Repository

```bash
cd rustro
```

### 2. Configure Credentials

Edit `config.toml`:

```toml
angel_one_client_code = "YOUR_CLIENT_CODE"
angel_one_password = "YOUR_PASSWORD"
angel_one_totp_secret = "YOUR_TOTP_SECRET_BASE32"
```

**For production, use environment variables:**

Create `.env` file (not tracked in git):
```bash
ANGEL_CLIENT_CODE=YOUR_CLIENT_CODE
ANGEL_PASSWORD=YOUR_PASSWORD
ANGEL_TOTP_SECRET=YOUR_TOTP_SECRET
```

### 3. Build

**Development build:**
```bash
cargo build
```

**Release build (optimized):**
```bash
cargo build --release
```

The binary will be created at:
- Development: `target/debug/rustro`
- Release: `target/release/rustro`

## Running the Bot

### Development Mode

```bash
cargo run
```

### Release Mode

```bash
cargo run --release
```

Or run the binary directly:

**Windows:**
```powershell
.\target\release\rustro.exe
```

**Linux/Mac:**
```bash
./target/release/rustro
```

### With Custom Config

```bash
CONFIG_PATH=my_config.toml cargo run --release
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test

```bash
cargo test test_rsi
```

### Run with Output

```bash
cargo test -- --nocapture
```

## Development Tools

### Check for Errors (Fast)

```bash
cargo check
```

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy --all-targets --all-features
```

### Generate Documentation

```bash
cargo doc --open
```

## Troubleshooting

### Build Errors

**OpenSSL Not Found (Linux):**
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# Fedora
sudo dnf install pkgconfig openssl-devel

# macOS
brew install openssl
```

**Compilation Errors:**
- Ensure Rust version is 1.70+: `rustc --version`
- Clean and rebuild: `cargo clean && cargo build`

### Runtime Errors

**Authentication Failed:**
- Verify TOTP secret is correct base32 encoding
- Check system time is synchronized
- Test TOTP: Use a TOTP app to verify codes

**Permission Denied:**
- Ensure `data/` directory exists and is writable
- Check file permissions: `chmod 755 data/`

**Network Errors:**
- Check internet connection
- Verify Angel One API is accessible
- Check firewall/proxy settings

## Production Deployment

### 1. Build Optimized Binary

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

### 2. Copy Binary to Server

```bash
scp target/release/rustro user@server:/opt/trading/
```

### 3. Setup as Service (Linux - systemd)

Create `/etc/systemd/system/rustro.service`:

```ini
[Unit]
Description=Rustro Trading Bot
After=network.target

[Service]
Type=simple
User=trading
WorkingDirectory=/opt/trading
Environment=CONFIG_PATH=/opt/trading/config.toml
ExecStart=/opt/trading/rustro
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

### 4. View Logs

```bash
sudo journalctl -u rustro -f
```

## Performance Optimization

### Profile Build

```bash
cargo build --release --profile release-optimized
```

Add to `Cargo.toml`:
```toml
[profile.release-optimized]
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

### Memory Usage

- Monitor with: `ps aux | grep rustro`
- Adjust ring buffer size in code if needed
- Enable log rotation to prevent disk fill

## Security Checklist

- [ ] Never commit credentials to Git
- [ ] Use `.env` file for secrets (gitignored)
- [ ] Rotate TOTP secret regularly
- [ ] Use separate API keys for dev/prod
- [ ] Enable 2FA on Angel One account
- [ ] Run bot with limited user privileges
- [ ] Monitor logs for suspicious activity
- [ ] Keep dependencies updated: `cargo update`

## Continuous Integration

### GitHub Actions Example

Create `.github/workflows/rust.yml`:

```yaml
name: Rust CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - run: cargo build --release
    - run: cargo test --all
    - run: cargo clippy -- -D warnings
```

## Docker Deployment (Optional)

Create `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rustro /usr/local/bin/rustro
WORKDIR /data
CMD ["rustro"]
```

Build and run:
```bash
docker build -t rustro .
docker run -v $(pwd)/config.toml:/data/config.toml -v $(pwd)/data:/data rustro
```

---

For more details, see [README.md](README.md)

