use std::collections::HashMap;

use crate::sensors::sensor::SensorValue;

#[derive(Debug)]
pub struct CoreTemp {
    name: String,
}

impl CoreTemp {
    pub fn new() -> Self {
        Self {
            name: "Core Temp".to_string(),
        }
    }
}

impl Default for CoreTemp {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::sensors::sensor::Sensor for CoreTemp {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        let sensors = lm_sensors::Initializer::default()
            .initialize()
            .map_err(|e| format!("Failed to initialize lm-sensors: {}", e))?;

        // 创建匹配 coretemp-isa-0000 的 chip pattern
        let chip_pattern = sensors
            .new_chip("coretemp-isa-0000")
            .map_err(|e| format!("Failed to create chip pattern: {}", e))?;

        for chip in sensors.chip_iter(Some(chip_pattern.as_ref())) {
            for feature in chip.feature_iter() {
                // 使用 feature 的 label (如 "Core 0") 而不是内部名称 (如 "temp1")
                let feature_name = match feature.label() {
                    Ok(label) => label,
                    Err(_) => feature
                        .name()
                        .transpose()
                        .map_err(|e| format!("Failed to get feature name: {}", e))?
                        .unwrap_or("N/A")
                        .to_string(),
                };

                for sub_feature in feature.sub_feature_iter() {
                    let sub_name = format!("{}", sub_feature);
                    // 只读取 input 类型的温度值
                    if sub_name.contains("input") {
                        if let Ok(value) = sub_feature.value() {
                            ret.insert(feature_name.clone(), SensorValue::Temperature(value.raw_value()));
                        }
                    }
                }
            }
        }

        if ret.is_empty() {
            return Err("No CPU temperature sensors found".to_string());
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::sensor::Sensor;

    #[test]
    fn test_core_temp_creation() {
        let sensor = CoreTemp::new();
        assert_eq!(sensor.name(), "Core Temp");
    }

    #[test]
    fn test_core_temp_default() {
        let sensor = CoreTemp::default();
        assert_eq!(sensor.name(), "Core Temp");
    }

    #[test]
    fn test_core_temp_reading() {
        let mut sensor = CoreTemp::new();

        match sensor.read() {
            Ok(readings) => {
                assert!(!readings.is_empty(), "Should have at least one temperature reading");

                for (name, value) in &readings {
                    if let SensorValue::Temperature(temp) = value {
                        println!("{}: {:.1}°C", name, temp);
                        assert!(*temp >= 0.0 && *temp <= 120.0, "Temperature should be in valid range");
                    } else {
                        panic!("Expected Temperature value for {}", name);
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not read CPU temperature sensors: {}", e);
            }
        }
    }
}
