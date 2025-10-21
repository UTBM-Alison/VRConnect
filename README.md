# VitalConnect - Rust Edition

A high-performance Rust implementation for receiving, processing, and transmitting real-time vital data via Socket.IO and Bluetooth Low Energy (BLE).

## Features

- Socket.IO input with async client
- Automatic zlib decompression
- JSON processing and cleaning
- Data transformation with statistics
- Console output (compact/verbose modes with colors)
- **BLE GATT server for smartphone connectivity**
- Clean architecture with 7 modules
- Production-ready error handling

## Quick Start

### Prerequisites
```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt-get install bluez libbluetooth-dev libdbus-1-dev pkg-config

# Enable Bluetooth
sudo systemctl enable bluetooth
sudo systemctl start bluetooth

# Add user to bluetooth group
sudo usermod -aG bluetooth $USER
# Log out and back in!
```

### Build & Run
```bash
cargo build --release
cargo run --release
```

### Options
```bash
cargo run -- --help

# Examples:
cargo run -- --port 5000 --verbose true
cargo run -- --ble-device-name "MyMonitor" --log-level debug
```

## Smartphone Connection

1. Install "nRF Connect" (Android) or "LightBlue" (iOS)
2. Scan for "VitalConnect"
3. Connect and subscribe to data characteristic
4. Receive real-time JSON vital data!

**Service UUID:** `12345678-1234-5678-1234-567812345678`

## Architecture
```
Socket.IO → Decompression → Processing → Transformation → Outputs (Console + BLE)
```

See documentation for more details.
