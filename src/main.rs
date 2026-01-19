mod config;
mod influx;
mod sensors;

use config::Config;
use influx::InfluxUploader;
use sensors::sensor::SensorManager;
use sensors::*;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting macstats-influx...");

    let config = match Config::load_from_file("/etc/macstats/config.toml") {
        Ok(config) => {
            println!("Loaded configuration from /etc/macstats/config.toml");
            config
        }
        Err(_) => {
            println!("Failed to load /etc/macstats/config.toml, trying environment variables...");
            Config::load_from_env()?
        }
    };

    let uploader = InfluxUploader::new(&config);
    let mut sensor_manager = create_sensor_manager(&config);

    println!(
        "Collecting sensor data every {} seconds and uploading to InfluxDB at {}",
        config.collection.interval_seconds, config.influxdb.host
    );

    let mut interval = time::interval(Duration::from_secs(config.collection.interval_seconds));

    loop {
        interval.tick().await;

        match collect_and_upload(&mut sensor_manager, &uploader).await {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error collecting/uploading sensor data: {}", e);
            }
        }
    }
}

fn create_sensor_manager(config: &Config) -> SensorManager {
    let mut manager = SensorManager::new();

    if config.sensors.enable_cpu {
        manager.add_sensor(Box::new(cpu::CoreTemp::new()));
        println!("Enabled CPU temperature sensors");
    }

    if config.sensors.enable_fan {
        manager.add_sensor(Box::new(fan::Fan::new()));
        println!("Enabled fan speed sensors");
    }

    if config.sensors.enable_power {
        manager.add_sensor(Box::new(power::Power::new()));
        println!("Enabled power sensors");
    }

    if config.sensors.enable_misc {
        manager.add_sensor(Box::new(misc::MiscTemp::new()));
        println!("Enabled misc temperature sensors (WiFi, NVMe)");
    }

    manager
}

async fn collect_and_upload(
    sensor_manager: &mut SensorManager,
    uploader: &InfluxUploader,
) -> Result<(), Box<dyn std::error::Error>> {
    let sensor_data = sensor_manager.read_all()?;
    
    if !sensor_data.is_empty() {
        uploader.upload_sensor_data(sensor_data).await?;
    } else {
        println!("No sensor data collected");
    }

    Ok(())
}
