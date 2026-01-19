use std::collections::HashMap;

use crate::sensors::sensor::SensorValue;

#[derive(Debug)]
pub struct Power {
    name: String,
}

impl Power {
    pub fn new() -> Self {
        Self {
            name: "Power".to_string(),
        }
    }
}

impl Default for Power {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::sensors::sensor::Sensor for Power {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        let sensors = lm_sensors::Initializer::default()
            .initialize()
            .map_err(|e| format!("Failed to initialize lm-sensors: {}", e))?;

        // 创建匹配 rapl_monitor-virtual-0 的 chip pattern
        let chip_pattern = sensors
            .new_chip("rapl_monitor-virtual-0")
            .map_err(|e| format!("Failed to create chip pattern: {}", e))?;

        for chip in sensors.chip_iter(Some(chip_pattern.as_ref())) {
            for feature in chip.feature_iter() {
                let feature_name = feature
                    .name()
                    .transpose()
                    .map_err(|e| format!("Failed to get feature name: {}", e))?
                    .unwrap_or("N/A");

                // 只读取 power1
                if feature_name != "power1" {
                    continue;
                }

                for sub_feature in feature.sub_feature_iter() {
                    let sub_name = format!("{}", sub_feature);
                    // 只读取 average 或 input 类型的功率值
                    if sub_name.contains("average") || sub_name.contains("input") {
                        if let Ok(value) = sub_feature.value() {
                            ret.insert("CPU Power".to_string(), SensorValue::Power(value.raw_value()));
                            break; // 只需要一个值
                        }
                    }
                }
            }
        }

        if ret.is_empty() {
            return Err("No power sensors found".to_string());
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::sensor::Sensor;

    #[test]
    fn test_power_sensor_creation() {
        let power = Power::new();
        assert_eq!(power.name(), "Power");
    }

    #[test]
    fn test_power_sensor_reading() {
        let mut power = Power::new();

        match power.read() {
            Ok(readings) => {
                assert!(readings.contains_key("CPU Power"), "Missing CPU Power sensor");

                if let Some(SensorValue::Power(power_val)) = readings.get("CPU Power") {
                    assert!(
                        *power_val >= 0.0,
                        "Power value should be non-negative"
                    );
                    println!("CPU Power: {:.1} W", power_val);
                } else {
                    panic!("Expected power value for CPU Power");
                }
            }
            Err(e) => {
                println!("Warning: Could not read power sensors: {}", e);
            }
        }
    }
}
