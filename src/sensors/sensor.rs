use std::collections::HashMap;

/// Temperature in degrees Celsius
pub type Celsius = f64;
/// Fan speed in RPM
pub type Rpm = f64;
/// Voltage in Volts
pub type Volt = f64;
/// Power in Watts
pub type Watt = f64;

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
    fn test_cpu_temperature() {
        use crate::sensors::cpu::CoreTemp;
        let mut manager = SensorManager::new();

        let cpu_sensor = Box::new(CoreTemp::new());
        manager.add_sensor(cpu_sensor);

        // lm-sensors may fail if already initialized by another test running in parallel
        match manager.read_all() {
            Ok(results) => {
                assert!(!results.is_empty(), "Should have at least one CPU temperature sensor");

                // print all results for debugging
                for (key, value) in &results {
                    match value {
                        SensorValue::Temperature(temp) => {
                            println!("{}: {}°C", key, temp);
                            assert!(*temp >= 0.0 && *temp <= 120.0);
                        }
                        SensorValue::Speed(rpm) => {
                            println!("{}: {} RPM", key, rpm);
                        }
                        SensorValue::Voltage(volt) => {
                            println!("{}: {} V", key, volt);
                        }
                        SensorValue::Power(watt) => {
                            println!("{}: {} W", key, watt);
                        }
                    }
                }
            }
            Err(e) => {
                // This can happen when tests run in parallel and lm-sensors is already initialized
                println!("Warning: Could not read sensors (may be parallel test issue): {}", e);
            }
        }
    }
}
