# Copilot Instructions

## Project Overview

macstats-influx is a Rust application that collects Linux hardware sensor data via the `lm-sensors` library and uploads metrics to InfluxDB 2.0. Despite the name, it targets Linux systems using the `lm_sensors` crate (not macOS SMC).

## Build & Test

```bash
cargo build                       # Build
cargo test                        # Run all tests
cargo test --lib sensors::cpu     # Test a specific sensor module
cargo test --lib sensors::fan
cargo build --release             # Release build
cargo run                         # Run (needs /etc/macstats/config.toml or env vars)
```

Tests read real hardware sensors via `lm_sensors`. They tolerate failures from parallel test runs (lm-sensors can only be initialized once per process), so tests that can't initialize sensors print a warning instead of failing.

## Architecture

### Sensor Trait System

All sensors implement the `Sensor` trait (`src/sensors/sensor.rs`):

```rust
pub trait Sensor {
    fn name(&self) -> &str;
    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String>;
}
```

`SensorValue` is an enum with five variants, each wrapping an `f64` type alias (`Celsius`, `Rpm`, `Volt`, `Watt`, `BytesPerSec`):

```rust
pub enum SensorValue {
    Temperature(Celsius),
    Speed(Rpm),
    Voltage(Volt),
    Power(Watt),
    NetworkSpeed(BytesPerSec),
}
```

`SensorManager` holds a `Vec<Box<dyn Sensor>>` and aggregates all readings into a single `HashMap<String, SensorValue>` via `read_all()`.

### Data Flow

1. `main.rs` loads config (file at `/etc/macstats/config.toml` → env var fallback)
2. `create_sensor_manager()` conditionally registers sensors based on config booleans
3. Async loop calls `sensor_manager.read_all()` → `InfluxUploader::upload_sensor_data()`
4. Each `SensorValue` variant maps to an InfluxDB measurement: `temperature`, `fan_speed`, `voltage`, `power`
5. All points are tagged with `host` and `sensor` name

### Adding a New Sensor

Follow the pattern in `src/sensors/cpu.rs`:

1. Create a struct with a `name: String` field
2. Implement `new()`, `Default`, and the `Sensor` trait
3. In `read()`: initialize `lm_sensors`, create a chip pattern (e.g., `"coretemp-isa-0000"`), iterate chips → features → sub-features, filter for `"input"` sub-features, and insert into the result HashMap
4. Add the module to `src/sensors/mod.rs`
5. Wire it up in `create_sensor_manager()` in `main.rs` with a config boolean
6. Add the config field to `SensorConfig` in `src/config.rs`
7. Add tests following the existing pattern (create sensor, read, validate value ranges)

## Configuration

Three TOML sections in `config.toml` (loaded from `/etc/macstats/config.toml`):

- `[influxdb]`: host, org, token, bucket
- `[collection]`: interval_seconds, hostname
- `[sensors]`: boolean enable flags (`enable_cpu`, `enable_fan`, `enable_power`, `enable_misc`, `enable_network`)
- `[network]` (optional): `interface` — network interface name (e.g., `"enp173s0f1"`), required when `enable_network = true`

Env var fallback: `INFLUXDB_HOST`, `INFLUXDB_ORG`, `INFLUXDB_TOKEN`, `INFLUXDB_BUCKET`, `COLLECTION_INTERVAL`, `HOSTNAME`, `ENABLE_CPU`, `ENABLE_FAN`, `ENABLE_POWER`, `ENABLE_MISC`, `ENABLE_NETWORK`, `NETWORK_INTERFACE`.

## Key Conventions

- Errors are `String`-based (`Result<T, String>` in sensors, `Box<dyn Error>` at the application boundary)
- Sensor names are stored as immutable `String` fields set in `new()`
- The main loop logs errors but never stops — collection continues on failure
- Each sensor struct implements both `Default` and `new()` constructors
- Comments in sensor implementations are in Chinese (chip pattern references)
