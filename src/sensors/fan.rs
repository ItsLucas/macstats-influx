use std::collections::HashMap;

use crate::sensors::sensor::{SensorValue};

use macsmc::{self, SmcClient, connect};

#[derive(Debug)]
pub struct Fan {
    name: String,
    client: SmcClient,
}

impl Fan {
    pub fn new() -> Self {
        Self {
            name: "Fan".to_string(),
            client: connect().expect("Failed to connect to SMC"),
        }
    }
}

impl crate::sensors::sensor::Sensor for Fan {
    fn name(&self) -> &str {
        &self.name
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
