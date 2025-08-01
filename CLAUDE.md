# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust application (`macstats-influx`) that collects macOS hardware sensor data via the System Management Controller (SMC) and is designed to send metrics to InfluxDB. The project consists of two main components:

1. **Main application** (`src/`): Sensor data collection and InfluxDB integration
2. **macsmc library** (`macsmc/`): Low-level SMC interface for macOS hardware sensors

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the application (requires config.toml or environment variables)
cargo run

# Run tests
cargo test

# Build in release mode
cargo build --release

# Test specific sensor modules
cargo test --lib sensors::cpu
cargo test --lib sensors::fan
```

## Configuration

The application supports two configuration methods:

1. **config.toml file** (preferred):
   ```toml
   [influxdb]
   host = "https://your-influxdb-host"
   org = "your-org"
   token = "your-token"
   bucket = "your-bucket"
   
   [collection]
   interval_seconds = 10
   hostname = "your-hostname"
   
   [sensors]
   enable_cpu = true
   enable_fan = true
   enable_gpu = true
   enable_misc_temp = true
   enable_power = true
   ```

2. **Environment variables** (see .env.example):
   - INFLUXDB_HOST, INFLUXDB_ORG, INFLUXDB_TOKEN, INFLUXDB_BUCKET
   - COLLECTION_INTERVAL, HOSTNAME
   - ENABLE_CPU, ENABLE_FAN, ENABLE_GPU, ENABLE_MISC_TEMP, ENABLE_POWER

## Architecture

### Sensor System Architecture

The application uses a modular sensor architecture centered around the `Sensor` trait:

- **`sensors/sensor.rs`**: Core trait and `SensorManager` for coordinating all sensors
- **`sensors/cpu.rs`**: CPU temperature sensors using SMC keys (`Tp01`, `Tp05`, etc.)
- **`sensors/fan.rs`**: Fan speed sensors using SMC key `F0Ac`
- **`sensors/gpu.rs`**: GPU sensors (implementation pending)
- **`sensors/misc_temp.rs`**: Miscellaneous temperature sensors (implementation pending)
- **`sensors/power.rs`**: Power consumption sensors (implementation pending)

Each sensor returns a `HashMap<String, SensorValue>` where `SensorValue` is an enum supporting:
- `Temperature(Celsius)`
- `Speed(Rpm)`
- `Percentage(Percentage)`
- `Voltage(Volt)`
- `Power(Watt)`

### SMC Library Integration

The `macsmc` library provides the low-level interface to macOS SMC:
- **`SmcClient`**: Main client for reading SMC keys
- **SMC keys**: Hardware-specific 4-character codes (e.g., `F0Ac` for fan speed, `Tp01` for CPU core temperature)
- **Type conversion**: Automatic parsing of SMC data to typed values

## Dependencies

- **influxdb2 (0.5.2)**: InfluxDB client for metrics storage
- **macsmc (local)**: Custom SMC library for macOS hardware access
- **tokio (1.0)**: Async runtime for periodic data collection
- **serde + toml**: Configuration file parsing
- **futures (0.3)**: Stream processing for InfluxDB uploads
- **gethostname (0.4)**: System hostname detection

## macOS-Specific Requirements

This project only works on macOS due to SMC dependency. The `macsmc` library includes a compile-time check that prevents builds on non-macOS systems.

## Testing Strategy

Tests verify sensor functionality by:
1. Creating sensor instances through `SensorManager`
2. Reading actual hardware values
3. Validating data types and reasonable value ranges
4. Printing sensor readings for debugging

When adding new sensors, follow the existing test pattern in `src/sensors/sensor.rs`.

## InfluxDB Integration

The application uploads sensor data to InfluxDB 2.0 with the following structure:

### Measurements and Fields
- **temperature**: celsius (float)
- **fan_speed**: rpm (float) 
- **power**: watts (float)
- **voltage**: volts (float)
- **percentage**: percent (float)

### Tags
- **host**: hostname from config
- **sensor**: specific sensor name (e.g., "CPU Performance Core 1", "Fan", "GPU 1")

### Data Flow
1. Sensors read from SMC every `interval_seconds`
2. Data converted to InfluxDB DataPoints with timestamps
3. Batched upload to configured InfluxDB instance
4. Automatic retry on connection failures

## Grafana Visualization

A complete Grafana dashboard JSON is included (`grafana-dashboard.json`) with:
- CPU Core Temperatures panel
- System Component Temperatures panel  
- Fan Speed panel
- Power Consumption panel
- Voltage Levels panel
- System Percentages panel

Import the dashboard and update the datasource UID to match your InfluxDB connection.