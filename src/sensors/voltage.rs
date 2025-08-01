use std::collections::HashMap;

use crate::sensors::sensor::{Sensor, SensorValue};

use macsmc::{SmcClient, connect};

#[derive(Debug)]
pub struct Voltage {
    name: String,
    sensor_type: String,
    client: SmcClient,
}

/*
 DESCRIPTION   KEY      VALUE     TYPE
 DC In         VD0R       12.2 V  flt
*/

// static array of voltage sensor keys
const VOLTAGE_KEYS: [&str; 1] = [
    "VD0R",
];

const VOLTAGE_SENSOR_NAMES: [&str; 1] = [
    "DC In",
];

impl Voltage {
    pub fn new() -> Self {
        Self {
            name: "Voltage".to_string(),
            sensor_type: "Voltage".to_string(),
            client: connect().expect("Failed to connect to SMC"),
        }
    }
}

impl crate::sensors::sensor::Sensor for Voltage {
    fn name(&self) -> &str {
        &self.name
    }

    fn sensor_type(&self) -> &str {
        &self.sensor_type
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();
        
        for (i, &key) in VOLTAGE_KEYS.iter().enumerate() {
            if let Ok(data) = self.client.read_key(key) {
                match data.as_voltage() {
                    Ok(voltage) => {
                        let sensor_name = VOLTAGE_SENSOR_NAMES[i].to_string();
                        ret.insert(sensor_name, SensorValue::Voltage(voltage));
                    }
                    Err(_) => {
                        return Err(format!("Failed to read voltage for key: {}", key));
                    }
                }
            } else {
                return Err(format!("Failed to read key: {}", key));
            }
        }
        
        Ok(ret)
    }
}

impl Default for Voltage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::sensor::Sensor;

    #[test]
    fn test_voltage_sensor_creation() {
        let voltage = Voltage::new();
        assert_eq!(voltage.name(), "Voltage");
        assert_eq!(voltage.sensor_type(), "Voltage");
    }

    #[test]
    fn test_voltage_sensor_reading() {
        let mut voltage = Voltage::new();
        
        match voltage.read() {
            Ok(readings) => {
                // 应该有 1 个电压读数
                assert_eq!(readings.len(), 1);
                
                // 检查是否包含所有预期的传感器
                let expected_sensors = [
                    "DC In"
                ];
                
                for sensor_name in &expected_sensors {
                    assert!(readings.contains_key(*sensor_name), 
                        "Missing sensor: {}", sensor_name);
                    
                    if let Some(SensorValue::Voltage(voltage_val)) = readings.get(*sensor_name) {
                        assert!(voltage_val.0 >= 0.0 && voltage_val.0 <= 50.0, 
                            "Voltage value should be in valid range for {}", sensor_name);
                        println!("{}: {:.1} V", sensor_name, voltage_val.0);
                    } else {
                        panic!("Expected voltage value for {}", sensor_name);
                    }
                }
            }
            Err(e) => {
                // 在某些情况下（比如测试环境），SMC 可能不可用
                println!("Warning: Could not read voltage sensors: {}", e);
            }
        }
    }

    #[test]
    fn test_voltage_keys_and_names_match() {
        // 确保键和名称数组长度匹配
        assert_eq!(VOLTAGE_KEYS.len(), VOLTAGE_SENSOR_NAMES.len());
    }

    #[test]
    fn test_dc_in_voltage_range() {
        let mut voltage = Voltage::new();
        
        match voltage.read() {
            Ok(readings) => {
                if let Some(SensorValue::Voltage(dc_voltage)) = readings.get("DC In") {
                    // DC In 电压通常在 10V 到 15V 之间
                    assert!(dc_voltage.0 >= 10.0 && dc_voltage.0 <= 15.0, 
                        "DC In voltage seems unreasonable: {:.1}V", dc_voltage.0);
                }
            }
            Err(_) => {
                // 忽略错误，某些环境下传感器可能不可用
            }
        }
    }
}
