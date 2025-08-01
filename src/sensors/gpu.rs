use std::collections::HashMap;

use crate::sensors::sensor::{SensorValue};

use macsmc::{SmcClient, connect};

#[derive(Debug)]
pub struct GpuTemp {
    name: String,
    client: SmcClient,
}

/*
 DESCRIPTION             KEY    VALUE     TYPE
 GPU 1                   Tf14    40.1 °C  flt
 GPU 2                   Tf18    40.1 °C  flt
 GPU 3                   Tf19    39.1 °C  flt
*/

// static array of GPU temperature sensor keys
const GPU_TEMP_KEYS: [&str; 3] = ["Tf14", "Tf18", "Tf19"];

const GPU_SENSOR_NAMES: [&str; 3] = ["GPU 1", "GPU 2", "GPU 3"];

impl GpuTemp {
    pub fn new() -> Self {
        Self {
            name: "GPU Temperature".to_string(),
            client: connect().expect("Failed to connect to SMC"),
        }
    }
}

impl crate::sensors::sensor::Sensor for GpuTemp {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        for (i, &key) in GPU_TEMP_KEYS.iter().enumerate() {
            if let Ok(data) = self.client.read_key(key) {
                match data.as_temperature() {
                    Ok(temp) => {
                        let sensor_name = GPU_SENSOR_NAMES[i].to_string();
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

impl Default for GpuTemp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::sensor::Sensor;

    #[test]
    fn test_gpu_temp_sensor_creation() {
        let gpu_temp = GpuTemp::new();
        assert_eq!(gpu_temp.name(), "GPU Temperature");
    }

    #[test]
    fn test_gpu_temp_sensor_reading() {
        let mut gpu_temp = GpuTemp::new();

        match gpu_temp.read() {
            Ok(readings) => {
                assert_eq!(readings.len(), 3);

                let expected_sensors = ["GPU 1", "GPU 2", "GPU 3"];

                for sensor_name in &expected_sensors {
                    assert!(
                        readings.contains_key(*sensor_name),
                        "Missing sensor: {}",
                        sensor_name
                    );

                    if let Some(SensorValue::Temperature(temp)) = readings.get(*sensor_name) {
                        assert!(
                            temp.0 >= 0.0 && temp.0 <= 150.0,
                            "Temperature should be in valid range for {}",
                            sensor_name
                        );
                        println!("{}: {:.1} °C", sensor_name, temp.0);
                    } else {
                        panic!("Expected temperature value for {}", sensor_name);
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not read GPU temperature sensors: {}", e);
            }
        }
    }

    #[test]
    fn test_gpu_keys_and_names_match() {
        assert_eq!(GPU_TEMP_KEYS.len(), GPU_SENSOR_NAMES.len());
    }
}
