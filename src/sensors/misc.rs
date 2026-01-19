use std::collections::HashMap;

use crate::sensors::sensor::SensorValue;

#[derive(Debug)]
pub struct MiscTemp {
    name: String,
}

impl MiscTemp {
    pub fn new() -> Self {
        Self {
            name: "Misc Temp".to_string(),
        }
    }
}

impl Default for MiscTemp {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::sensors::sensor::Sensor for MiscTemp {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        let sensors = lm_sensors::Initializer::default()
            .initialize()
            .map_err(|e| format!("Failed to initialize lm-sensors: {}", e))?;

        // 读取 WiFi 适配器温度 (iwlwifi_1-virtual-0)
        if let Ok(wifi_chip) = sensors.new_chip("iwlwifi_1-virtual-0") {
            for chip in sensors.chip_iter(Some(wifi_chip.as_ref())) {
                for feature in chip.feature_iter() {
                    let feature_name = match feature.label() {
                        Ok(label) => label,
                        Err(_) => feature
                            .name()
                            .transpose()
                            .ok()
                            .flatten()
                            .unwrap_or("temp1")
                            .to_string(),
                    };

                    // 只读取 temp1
                    if feature_name.contains("temp1") || feature_name == "temp1" {
                        for sub_feature in feature.sub_feature_iter() {
                            let sub_name = format!("{}", sub_feature);
                            if sub_name.contains("input") {
                                if let Ok(value) = sub_feature.value() {
                                    ret.insert("WiFi".to_string(), SensorValue::Temperature(value.raw_value()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 读取 NVMe SSD 温度 (nvme-pci-0100)
        if let Ok(nvme_chip) = sensors.new_chip("nvme-pci-0100") {
            for chip in sensors.chip_iter(Some(nvme_chip.as_ref())) {
                for feature in chip.feature_iter() {
                    let feature_name = match feature.label() {
                        Ok(label) => label,
                        Err(_) => feature
                            .name()
                            .transpose()
                            .ok()
                            .flatten()
                            .unwrap_or("N/A")
                            .to_string(),
                    };

                    // 只读取 Composite 温度
                    if feature_name == "Composite" {
                        for sub_feature in feature.sub_feature_iter() {
                            let sub_name = format!("{}", sub_feature);
                            if sub_name.contains("input") {
                                if let Ok(value) = sub_feature.value() {
                                    ret.insert("NVMe".to_string(), SensorValue::Temperature(value.raw_value()));
                                }
                            }
                        }
                    }
                }
            }
        }

        if ret.is_empty() {
            return Err("No misc temperature sensors found".to_string());
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::sensor::Sensor;

    #[test]
    fn test_misc_temp_creation() {
        let sensor = MiscTemp::new();
        assert_eq!(sensor.name(), "Misc Temp");
    }

    #[test]
    fn test_misc_temp_reading() {
        let mut sensor = MiscTemp::new();

        match sensor.read() {
            Ok(readings) => {
                for (name, value) in &readings {
                    if let SensorValue::Temperature(temp) = value {
                        println!("{}: {:.1}°C", name, temp);
                        assert!(*temp >= -40.0 && *temp <= 120.0, "Temperature should be in valid range");
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not read misc temperature sensors: {}", e);
            }
        }
    }
}
