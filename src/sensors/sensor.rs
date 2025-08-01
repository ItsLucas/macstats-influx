use macsmc::{Celsius, Rpm, Volt, Watt};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SensorValue {
    Temperature(Celsius),
    Speed(Rpm),
    Voltage(Volt),
    Power(Watt),
}

pub trait Sensor {
    fn name(&self) -> &str;

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String>;
}

pub struct SensorManager {
    sensors: Vec<Box<dyn Sensor>>,
}

impl SensorManager {
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
        }
    }

    pub fn add_sensor(&mut self, sensor: Box<dyn Sensor>) {
        self.sensors.push(sensor);
    }

    pub fn read_all(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut results = HashMap::new();

        for sensor in &mut self.sensors {
            let _result = match sensor.read() {
                Ok(values) => {
                    for (key, value) in values {
                        results.insert(key, value);
                    }
                }
                Err(e) => {
                    return Err(format!("Error reading sensor {}: {}", sensor.name(), e));
                }
            };
        }

        Ok(results)
    }
}

impl Default for SensorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_speed() {
        use crate::sensors::fan::Fan;
        let mut manager = SensorManager::new();

        let fan_sensor = Box::new(Fan::new());
        manager.add_sensor(fan_sensor);

        let results = manager.read_all().unwrap();
        assert!(results.contains_key("Fan"));
        if let Some(result) = results.get("Fan") {
            if let Some(SensorValue::Speed(rpm)) = results.get("Fan") {
                println!("Fan is running at {} RPM", rpm.0);
                assert!(*rpm >= macsmc::Rpm(1200.0) && *rpm <= macsmc::Rpm(6000.0));
            } else {
                panic!("Expected a Speed value for Fan, found: {:?}", result);
            }
        } else {
            panic!("Fan sensor not found in results");
        }
    }

    #[test]
    fn test_cpu_temperature() {
        use crate::sensors::cpu::CoreTemp;
        let mut manager = SensorManager::new();

        let cpu_sensor = Box::new(CoreTemp::new());
        manager.add_sensor(cpu_sensor);

        let results = manager.read_all().unwrap();
        assert!(results.contains_key("CPU Performance Core 1"));
        if let Some(result) = results.get("CPU Performance Core 1") {
            if let SensorValue::Temperature(temp) = result {
                // Use `result`, not results.get("Fan")
                println!("CPU Core 1 temperature is {}°C", temp.0);
                assert!(temp.0 >= 0.0 && temp.0 <= 100.0); // Compare f32 values
            } else {
                panic!(
                    "Expected a Temperature value for CPU Core 1, found: {:?}",
                    result
                );
            }
        } else {
            panic!("CPU Core 1 sensor not found in results");
        }

        // print all results for debugging
        for (key, value) in results {
            match value {
                SensorValue::Temperature(temp) => {
                    println!("{}: {}°C", key, temp.0);
                }
                SensorValue::Speed(rpm) => {
                    println!("{}: {} RPM", key, rpm.0);
                }
                SensorValue::Voltage(volt) => {
                    println!("{}: {} V", key, volt.0);
                }
                SensorValue::Power(watt) => {
                    println!("{}: {} W", key, watt.0);
                }
            }
        }
    }
}
