use std::collections::HashMap;

use crate::sensors::sensor::SensorValue;

#[derive(Debug)]
pub struct Fan {
    name: String,
}

impl Fan {
    pub fn new() -> Self {
        Self {
            name: "Fan".to_string(),
        }
    }
}

impl Default for Fan {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::sensors::sensor::Sensor for Fan {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        let sensors = lm_sensors::Initializer::default()
            .initialize()
            .map_err(|e| format!("Failed to initialize lm-sensors: {}", e))?;

        // 创建匹配 it8613-isa-0a30 的 chip pattern
        let chip_pattern = sensors
            .new_chip("it8613-isa-0a30")
            .map_err(|e| format!("Failed to create chip pattern: {}", e))?;

        for chip in sensors.chip_iter(Some(chip_pattern.as_ref())) {
            for feature in chip.feature_iter() {
                let feature_name = feature
                    .name()
                    .transpose()
                    .map_err(|e| format!("Failed to get feature name: {}", e))?
                    .unwrap_or("N/A");

                // 只读取 fan2
                if feature_name != "fan2" {
                    continue;
                }

                for sub_feature in feature.sub_feature_iter() {
                    let sub_name = format!("{}", sub_feature);
                    // 只读取 input 类型的风扇转速
                    if sub_name.contains("input") {
                        if let Ok(value) = sub_feature.value() {
                            ret.insert("Case Fan".to_string(), SensorValue::Speed(value.raw_value()));
                        }
                    }
                }
            }
        }

        if ret.is_empty() {
            return Err("No fan sensors found".to_string());
        }

        Ok(ret)
    }
}
