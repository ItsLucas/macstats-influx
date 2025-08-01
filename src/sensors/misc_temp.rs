use std::collections::HashMap;

use crate::sensors::sensor::{SensorValue};

use macsmc::{SmcClient, connect};

#[derive(Debug)]
pub struct MiscTemp {
    name: String,
    client: SmcClient,
}

/*
 DESCRIPTION             KEY    VALUE     TYPE
 Airport                 TW0P    30.9 °C  flt
 Heatpipe 1              Th0H    31.9 °C  flt
 Memory Proximity        Ts0S    31.7 °C  flt
 NAND                    TH0x    31.3 °C  flt
 Palm Rest               Ts0P    31.4 °C  flt
 Power Supply 1 Alt      Tp0C    31.9 °C  flt
*/

// static array of miscellaneous temperature sensor keys
const MISC_TEMP_KEYS: [&str; 6] = ["TW0P", "Th0H", "Ts0S", "TH0x", "Ts0P", "Tp0C"];

const MISC_SENSOR_NAMES: [&str; 6] = [
    "Airport",
    "Heatpipe 1",
    "Memory Proximity",
    "NAND",
    "Palm Rest",
    "Power Supply 1 Alt",
];

impl MiscTemp {
    pub fn new() -> Self {
        Self {
            name: "Miscellaneous Temperature".to_string(),
            client: connect().expect("Failed to connect to SMC"),
        }
    }
}

impl crate::sensors::sensor::Sensor for MiscTemp {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        for (i, &key) in MISC_TEMP_KEYS.iter().enumerate() {
            if let Ok(data) = self.client.read_key(key) {
                match data.as_temperature() {
                    Ok(temp) => {
                        let sensor_name = MISC_SENSOR_NAMES[i].to_string();
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

impl Default for MiscTemp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::sensor::Sensor;

    #[test]
    fn test_misc_temp_sensor_creation() {
        let misc_temp = MiscTemp::new();
        assert_eq!(misc_temp.name(), "Miscellaneous Temperature");
    }

    #[test]
    fn test_misc_temp_sensor_reading() {
        let mut misc_temp = MiscTemp::new();

        match misc_temp.read() {
            Ok(readings) => {
                assert_eq!(readings.len(), 6);

                let expected_sensors = [
                    "Airport",
                    "Heatpipe 1",
                    "Memory Proximity",
                    "NAND",
                    "Palm Rest",
                    "Power Supply 1 Alt",
                ];

                for sensor_name in &expected_sensors {
                    assert!(
                        readings.contains_key(*sensor_name),
                        "Missing sensor: {}",
                        sensor_name
                    );

                    if let Some(SensorValue::Temperature(temp)) = readings.get(*sensor_name) {
                        assert!(
                            temp.0 >= 0.0 && temp.0 <= 100.0,
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
                println!(
                    "Warning: Could not read miscellaneous temperature sensors: {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_misc_keys_and_names_match() {
        assert_eq!(MISC_TEMP_KEYS.len(), MISC_SENSOR_NAMES.len());
    }

    #[test]
    fn test_individual_sensors() {
        let mut misc_temp = MiscTemp::new();

        match misc_temp.read() {
            Ok(readings) => {
                if let Some(SensorValue::Temperature(airport_temp)) = readings.get("Airport") {
                    assert!(
                        airport_temp.0 >= 20.0 && airport_temp.0 <= 60.0,
                        "Airport temperature seems unreasonable: {:.1}°C",
                        airport_temp.0
                    );
                }

                if let Some(SensorValue::Temperature(palm_temp)) = readings.get("Palm Rest") {
                    assert!(
                        palm_temp.0 >= 20.0 && palm_temp.0 <= 50.0,
                        "Palm Rest temperature seems unreasonable: {:.1}°C",
                        palm_temp.0
                    );
                }

                if let Some(SensorValue::Temperature(memory_temp)) =
                    readings.get("Memory Proximity")
                {
                    assert!(
                        memory_temp.0 >= 20.0 && memory_temp.0 <= 80.0,
                        "Memory Proximity temperature seems unreasonable: {:.1}°C",
                        memory_temp.0
                    );
                }
            }
            Err(_) => {}
        }
    }
}
