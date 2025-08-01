use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub influxdb: InfluxDbConfig,
    pub collection: CollectionConfig,
    pub sensors: SensorConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InfluxDbConfig {
    pub host: String,
    pub org: String,
    pub token: String,
    pub bucket: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionConfig {
    pub interval_seconds: u64,
    pub hostname: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorConfig {
    pub enable_cpu: bool,
    pub enable_fan: bool,
    pub enable_gpu: bool,
    pub enable_misc_temp: bool,
    pub enable_power: bool,
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let influxdb = InfluxDbConfig {
            host: std::env::var("INFLUXDB_HOST").unwrap_or_else(|_| "http://localhost:8086".to_string()),
            org: std::env::var("INFLUXDB_ORG")?,
            token: std::env::var("INFLUXDB_TOKEN")?,
            bucket: std::env::var("INFLUXDB_BUCKET").unwrap_or_else(|_| "macstats".to_string()),
        };

        let collection = CollectionConfig {
            interval_seconds: std::env::var("COLLECTION_INTERVAL")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| {
                gethostname::gethostname()
                    .into_string()
                    .unwrap_or_else(|_| "unknown".to_string())
            }),
        };

        let sensors = SensorConfig {
            enable_cpu: std::env::var("ENABLE_CPU").unwrap_or_else(|_| "true".to_string()).parse()?,
            enable_fan: std::env::var("ENABLE_FAN").unwrap_or_else(|_| "true".to_string()).parse()?,
            enable_gpu: std::env::var("ENABLE_GPU").unwrap_or_else(|_| "true".to_string()).parse()?,
            enable_misc_temp: std::env::var("ENABLE_MISC_TEMP").unwrap_or_else(|_| "true".to_string()).parse()?,
            enable_power: std::env::var("ENABLE_POWER").unwrap_or_else(|_| "true".to_string()).parse()?,
        };

        Ok(Config {
            influxdb,
            collection,
            sensors,
        })
    }
}