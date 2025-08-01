use std::{collections::HashMap, sync::Arc};

use crate::sensors::sensor::{Sensor, SensorStatus, SensorValue};

use macsmc::{self, SmcClient, connect};

#[derive(Debug)]
pub struct Fan {
    name: String,
    sensor_type: String,
    client: SmcClient,
}

impl Fan {
    pub fn new() -> Self {
        Self {
            name: "Fan".to_string(),
            sensor_type: "Fan".to_string(),
            client: connect().expect("Failed to connect to SMC"),
        }
    }
}

impl crate::sensors::sensor::Sensor for Fan {
    fn name(&self) -> &str {
        &self.name
    }

    fn sensor_type(&self) -> &str {
        &self.sensor_type
    }

    fn read(&mut self) -> Result<HashMap<String, SensorValue>, String> {
        let mut ret = HashMap::new();

        if let Ok(data) = self.client.read_key("F0Ac") {
            match data.as_rpm() {
                Ok(rpm) => {
                    ret.insert(self.name.clone(), SensorValue::Speed(rpm));
                    Ok(ret)
                }
                Err(_) => Err("Failed to read fan speed".to_string()),
            }
        } else {
            Err("Failed to read fan data".to_string())
        }
    }
}
