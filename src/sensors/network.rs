use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use crate::sensors::sensor::{Sensor, SensorValue};

pub struct Network {
    name: String,
    interface: String,
    last_rx_bytes: Option<u64>,
    last_tx_bytes: Option<u64>,
    last_read_time: Option<Instant>,
}

impl Network {
    pub fn new(interface: &str) -> Self {
        Self {
            name: "Network".to_string(),
            interface: interface.to_string(),
            last_rx_bytes: None,
            last_tx_bytes: None,
            last_read_time: None,
        }
    }

    fn read_sysfs_value(&self, stat: &str) -> Result<u64, String> {
        let path = format!("/sys/class/net/{}/statistics/{}", self.interface, stat);
        fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?
            .trim()
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse {}: {}", path, e))
    }
}

impl Sensor for Network {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let rx_bytes = self.read_sysfs_value("rx_bytes")?;
        let tx_bytes = self.read_sysfs_value("tx_bytes")?;
        let now = Instant::now();

        let mut ret = HashMap::new();

        if let (Some(prev_rx), Some(prev_tx), Some(prev_time)) =
            (self.last_rx_bytes, self.last_tx_bytes, self.last_read_time)
        {
            let elapsed = now.duration_since(prev_time).as_secs_f64();
            // Skip if counters decreased (interface reset) or no time elapsed
            if elapsed > 0.0 && rx_bytes >= prev_rx && tx_bytes >= prev_tx {
                let rx_speed = (rx_bytes - prev_rx) as f64 / elapsed;
                let tx_speed = (tx_bytes - prev_tx) as f64 / elapsed;

                ret.insert(
                    format!("{} RX", self.interface),
                    SensorValue::NetworkSpeed(rx_speed),
                );
                ret.insert(
                    format!("{} TX", self.interface),
                    SensorValue::NetworkSpeed(tx_speed),
                );
            }
        }

        self.last_rx_bytes = Some(rx_bytes);
        self.last_tx_bytes = Some(tx_bytes);
        self.last_read_time = Some(now);

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let sensor = Network::new("enp173s0f1");
        assert_eq!(sensor.name(), "Network");
        assert_eq!(sensor.interface, "enp173s0f1");
    }

    #[test]
    fn test_network_first_read_returns_empty() {
        let mut sensor = Network::new("enp173s0f1");
        match sensor.read() {
            Ok(readings) => {
                assert!(readings.is_empty(), "First read should return empty (no baseline yet)");
            }
            Err(e) => {
                println!("Warning: Could not read network stats: {}", e);
            }
        }
    }

    #[test]
    fn test_network_second_read_returns_speed() {
        let mut sensor = Network::new("enp173s0f1");

        match sensor.read() {
            Ok(_) => {}
            Err(e) => {
                println!("Warning: Could not read network stats: {}", e);
                return;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        match sensor.read() {
            Ok(readings) => {
                assert_eq!(readings.len(), 2, "Should have RX and TX readings");

                for (name, value) in &readings {
                    if let SensorValue::NetworkSpeed(bps) = value {
                        println!("{}: {:.2} B/s", name, bps);
                        assert!(*bps >= 0.0, "Speed should be non-negative");
                    } else {
                        panic!("Expected NetworkSpeed value for {}", name);
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not read network stats: {}", e);
            }
        }
    }

    #[test]
    fn test_network_invalid_interface() {
        let mut sensor = Network::new("nonexistent0");
        let result = sensor.read();
        assert!(result.is_err(), "Should fail for non-existent interface");
    }
}
