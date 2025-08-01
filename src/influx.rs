use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::sensors::sensor::SensorValue;

pub struct InfluxUploader {
    client: Client,
    bucket: String,
    hostname: String,
}

impl InfluxUploader {
    pub fn new(config: &Config) -> Self {
        let client = Client::new(
            config.influxdb.host.clone(),
            config.influxdb.org.clone(),
            config.influxdb.token.clone(),
        );

        Self {
            client,
            bucket: config.influxdb.bucket.clone(),
            hostname: config.collection.hostname.clone(),
        }
    }

    pub async fn upload_sensor_data(
        &self,
        sensor_data: HashMap<String, SensorValue>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos() as i64;

        let mut points = Vec::new();

        for (sensor_name, sensor_value) in sensor_data {
            let point = match sensor_value {
                SensorValue::Temperature(temp) => {
                    DataPoint::builder("temperature")
                        .tag("host", &self.hostname)
                        .tag("sensor", &sensor_name)
                        .field("celsius", temp.0 as f64)
                        .timestamp(timestamp)
                        .build()?
                }
                SensorValue::Speed(rpm) => {
                    DataPoint::builder("fan_speed")
                        .tag("host", &self.hostname)
                        .tag("sensor", &sensor_name)
                        .field("rpm", rpm.0 as f64)
                        .timestamp(timestamp)
                        .build()?
                }
                SensorValue::Voltage(volt) => {
                    DataPoint::builder("voltage")
                        .tag("host", &self.hostname)
                        .tag("sensor", &sensor_name)
                        .field("volts", volt.0 as f64)
                        .timestamp(timestamp)
                        .build()?
                }
                SensorValue::Power(watt) => {
                    DataPoint::builder("power")
                        .tag("host", &self.hostname)
                        .tag("sensor", &sensor_name)
                        .field("watts", watt.0 as f64)
                        .timestamp(timestamp)
                        .build()?
                }
            };
            points.push(point);
        }

        if !points.is_empty() {
            let len = points.len();
            self.client
                .write(&self.bucket, stream::iter(points))
                .await?;
            println!("Uploaded {} data points to InfluxDB", len);
        }

        Ok(())
    }
}