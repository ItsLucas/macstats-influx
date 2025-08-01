use std::collections::HashMap;

use crate::sensors::sensor::{Sensor, SensorValue};

use macsmc::{SmcClient, connect};

#[derive(Debug)]
pub struct Power {
    name: String,
    sensor_type: String,
    client: SmcClient,
}

/*
 DESCRIPTION           KEY   VALUE     TYPE
 DC In                 PDTR     4.8 W  flt
 Other 3.3V High Side  PO3R     0.6 W  flt
 Other 5V High Side    PO5R     0.1 W  flt
 System Total          PSTR     4.8 W  flt
*/

// static array of power sensor keys
const POWER_KEYS: [&str; 4] = ["PDTR", "PO3R", "PO5R", "PSTR"];

const POWER_SENSOR_NAMES: [&str; 4] = [
    "DC In",
    "Other 3.3V High Side",
    "Other 5V High Side",
    "System Total",
];

impl Power {
    pub fn new() -> Self {
        Self {
            name: "Power".to_string(),
            sensor_type: "Power".to_string(),
            client: connect().expect("Failed to connect to SMC"),
        }
    }
}

impl crate::sensors::sensor::Sensor for Power {
    fn name(&self) -> &str {
        &self.name
    }

    fn sensor_type(&self) -> &str {
        &self.sensor_type
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        for (i, &key) in POWER_KEYS.iter().enumerate() {
            if let Ok(data) = self.client.read_key(key) {
                match data.as_power() {
                    Ok(power) => {
                        let sensor_name = POWER_SENSOR_NAMES[i].to_string();
                        ret.insert(sensor_name, SensorValue::Power(power));
                    }
                    Err(_) => {
                        return Err(format!("Failed to read power for key: {}", key));
                    }
                }
            } else {
                return Err(format!("Failed to read key: {}", key));
            }
        }

        Ok(ret)
    }
}

impl Default for Power {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(power.sensor_type(), "Power");
    }

    #[test]
    fn test_power_sensor_reading() {
        let mut power = Power::new();

        match power.read() {
            Ok(readings) => {
                assert_eq!(readings.len(), 4);

                let expected_sensors = [
                    "DC In",
                    "Other 3.3V High Side",
                    "Other 5V High Side",
                    "System Total",
                ];

                for sensor_name in &expected_sensors {
                    assert!(
                        readings.contains_key(*sensor_name),
                        "Missing sensor: {}",
                        sensor_name
                    );

                    if let Some(SensorValue::Power(power_val)) = readings.get(*sensor_name) {
                        assert!(
                            *power_val >= macsmc::Watt(0.0),
                            "Power value should be non-negative for {}",
                            sensor_name
                        );
                        println!("{}: {:.1} W", sensor_name, power_val);
                    } else {
                        panic!("Expected power value for {}", sensor_name);
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not read power sensors: {}", e);
            }
        }
    }

    #[test]
    fn test_power_keys_and_names_match() {
        assert_eq!(POWER_KEYS.len(), POWER_SENSOR_NAMES.len());
    }
}
