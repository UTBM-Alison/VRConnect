# VRConnect - Medical Vital Data Middleware

High-performance Rust middleware for real-time vital data processing via Socket.IO, Bluetooth Low Energy (BLE), and file recording.

## Features

- **Socket.IO v4 Server**: WebSocket input with automatic zlib decompression
- **Data Processing**: JSON cleaning, validation, and transformation
- **Multi-Output**: 
  - Console (compact/verbose)
  - BLE GATT server
  - **File recording with automatic rotation and archiving**
- **Debug Mode**: Complete data logging (input/output) for troubleshooting
- **Flexible Configuration**: CLI arguments or environment file

## Quick Start

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt-get install bluez libbluetooth-dev libdbus-1-dev pkg-config

# Enable Bluetooth
sudo systemctl enable bluetooth
sudo systemctl start bluetooth

# Add user to bluetooth group
sudo usermod -aG bluetooth $USER
# Log out and back in
```

### Installation
```bash
cargo build --release
```

### Configuration

Copy `.env.example` to `.env` and adjust values:
```bash
cp .env.example .env
```

### Run
```bash
# Using environment file
./target/release/vrconnect --config .env

# Using CLI arguments
./target/release/vrconnect --port 5000 --ble-enabled --output-file-enabled

# Debug mode
./target/release/vrconnect --debug --debug-output ./logs/debug.log
```

## CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `--config <PATH>` | Path to configuration file | `.env` |
| `--port <PORT>` | Socket.IO server port | `3000` |
| `--host <HOST>` | Socket.IO server host | `127.0.0.1` |
| `--verbose` | Enable verbose console output | `false` |
| `--ble-enabled` | Enable BLE output | `false` |
| `--ble-name <NAME>` | BLE device name | `VitalConnect` |
| `--ble-uuid <UUID>` | BLE service UUID | Auto-generated |
| `--output-file-enabled` | Enable file recording | `false` |
| `--output-file-base-path <PATH>` | Base directory for recordings | `./data/vrconnect/recording` |
| `--output-file-max-size-mb <SIZE>` | Max file size before rotation (MB) | `500` |
| `--output-file-archive-threshold-gb <SIZE>` | Archive threshold (GB) | `5` |
| `--output-file-critical-disk-percent <PCT>` | Critical disk usage % | `95` |
| `--debug` | Enable debug mode | `false` |
| `--debug-output <PATH>` | Debug log file path | `./logs/debug.log` |
| `--log-level <LEVEL>` | Log level (INFO/WARN/ERROR/DEBUG/SUCCESS) | `INFO` |

## Architecture
```
Socket.IO Input → Decompression → JSON Cleaning → Transformation → Outputs (Console + BLE + File)
```

### Data Flow

1. **Input**: Socket.IO v4 server receives vital data (possibly compressed)
2. **Decompression**: Automatic zlib decompression if detected
3. **Cleaning**: JSON sanitization (control chars, NaN/Infinity, decimal separators)
4. **Transformation**: VitalData → ProcessedData with type detection
5. **Output**: Multi-channel (console, BLE, and/or file)

### File Output

**Features:**
- **Automatic Rotation**: Files rotate at 500 MB (configurable)
- **Daily Organization**: Files organized in date folders (`YYYYMMDD`)
- **Automatic Archiving**: 
  - Archives when daily folder reaches 5 GB
  - Archives old days at startup
  - Archives previous day when date changes
- **JSON Lines Format**: One JSON object per line for easy parsing
- **Disk Monitoring**: Graceful shutdown at 95% disk usage
- **Complete Data Capture**: ALL tracks including full waveforms

**Directory Structure:**
```
/data/vrconnect/recording/
├── data/
│   ├── 20250115/
│   │   ├── vrconnect_20250115_080000_090000.json (500 MB - completed)
│   │   ├── vrconnect_20250115_090000_100000.json (500 MB - completed)
│   │   └── vrconnect_20250115_100000_ongoing.json (active)
│   └── 20250116/
│       └── vrconnect_20250116_000000_ongoing.json
└── archive/
    ├── 20250114/
    │   └── archive_20250114_000000_235959.zip
    └── 20250115/
        └── archive_20250115_080000_100000.zip
```

**File Naming:**
- Active: `vrconnect_YYYYMMDD_HHMMSS_ongoing.json`
- Completed: `vrconnect_YYYYMMDD_HHMMSS_HHMMSS.json` (start_end)
- Archive: `archive_YYYYMMDD_HHMMSS_HHMMSS.zip` (first_last)

### BLE Limitations

**Important**: BLE output only transmits **non-waveform tracks** (HR, SpO2, NIBP, etc.) due to MTU payload limits. Waveform data (ECG, PLETH, CO2) is excluded from BLE transmission.

File output includes ALL data including complete waveforms.

## BLE Connection

### Service Information
- **Service UUID**: Configurable (default: `12345678-1234-5678-1234-567812345678`)
- **Data Characteristic**: Read + Notify enabled
- **Data Format**: JSON with number/string tracks only

### Connect via Smartphone

**Android/iOS**: Install "nRF Connect" app
1. Scan for devices
2. Connect to "VitalConnect" (or custom name)
3. Enable notifications on data characteristic
4. Receive real-time JSON updates

## Debug Mode

Debug mode logs all incoming and outgoing data:
```bash
./vrconnect --debug --debug-output ./debug_session.log
```

**Logged Information**:
- Raw Socket.IO frames (text/binary)
- Decompressed data
- Cleaned JSON
- Transformed ProcessedData
- BLE output payloads
- File output operations
- Console output

**Warning**: Debug logs grow rapidly with high-frequency data. Use for troubleshooting only.

## Logging

Logs are written to `LOG_DIR` (default: `./logs/`) with daily rotation:
- `vrconnect-2025-01-15.log`
- `vrconnect-2025-01-16.log`

**Log Levels**:
- `SUCCESS`: Operation completed successfully
- `INFO`: General information
- `WARNING`: Non-critical issues
- `ERROR`: Errors requiring attention
- `DEBUG`: Detailed debugging information

## Testing
```bash
# Run all tests
cargo test

# Unit tests only
cargo test --lib

# Functional tests only
cargo test --test functional

# With coverage
cargo tarpaulin --out Html
```

## Performance Considerations

**File Output:**
- Optimized for high-frequency data (async I/O)
- Minimal overhead with buffered writes
- Automatic compression reduces storage by ~70%
- Background archiving doesn't block data recording

**Disk Space:**
- Monitor available space regularly
- Archive retention policy depends on your storage
- Example: 500 MB/file, 10 files/hour = ~120 GB/day uncompressed
- With compression: ~36 GB/day

## License

Proprietary - UTBM Project
