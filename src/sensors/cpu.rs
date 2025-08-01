use std::{collections::HashMap, sync::Arc};

use crate::sensors::sensor::{Sensor, SensorStatus, SensorValue};

use macsmc::{SmcClient, connect};

#[derive(Debug)]
pub struct CoreTemp {
    name: String,
    sensor_type: String,
    client: SmcClient,
}

/*
 DESCRIPTION             KEY    VALUE     TYPE
 CPU Efficiency Core 1   Te05    37.9 °C  flt
 CPU Efficiency Core 2  Tp01    37.7 °C  flt
 CPU Efficiency Core 3  Tp05    36.8 °C  flt
 CPU Efficiency Core 4  Tp09    37.9 °C  flt
 CPU Performance Core 1  Tp0D    36.8 °C  flt
 CPU Performance Core 2  Tp0b    37.3 °C  flt
 CPU Performance Core 3  Tp0f    37.9 °C  flt
 CPU Performance Core 4  Tp0j    36.7 °C  flt
*/

// static array of cpu core keys
const CPU_CORE_KEYS: [&str; 8] = [
    "Tp01", "Tp05", "Tp09", "Tp0D", "Tp0b", "Tp0f", "Tp0j", "Te05",
];

const CPU_SENSOR_NAME: [&str; 8] = [
    "CPU Efficiency Core 1",
    "CPU Efficiency Core 2",
    "CPU Efficiency Core 3",
    "CPU Efficiency Core 4",
    "CPU Performance Core 1",
    "CPU Performance Core 2",
    "CPU Performance Core 3",
    "CPU Performance Core 4",
];

impl CoreTemp {
    pub fn new() -> Self {
        Self {
            name: "Core Temp".to_string(),
            sensor_type: "Thermal".to_string(),
            client: connect().expect("Failed to connect to SMC"),
        }
    }
}

impl crate::sensors::sensor::Sensor for CoreTemp {
    fn name(&self) -> &str {
        &self.name
    }

    fn sensor_type(&self) -> &str {
        &self.sensor_type
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();
        for (i, &key) in CPU_CORE_KEYS.iter().enumerate() {
            if let Ok(data) = self.client.read_key(key) {
                match data.as_temperature() {
                    Ok(temp) => {
                        let sensor_name = CPU_SENSOR_NAME[i].to_string();
                        ret.insert(sensor_name, SensorValue::Temperature(temp));
                    }
                    Err(_) => {
                        return Err(format!("Failed to read temperature for key: {}", key));
                    }
                }
            } else {
                return Err(format!("Failed to read key: {}", key));
            }
        }
        Ok(ret)
    }
}
