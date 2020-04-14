use crate::ruuvitag::{RuuviTag, SensorValuesNormalized, SensorValuesType};
use chrono::{DateTime, Utc};
use influxdb::{Client, InfluxDbWriteable};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[allow(non_snake_case)]
#[derive(InfluxDbWriteable, Debug)]
struct RuuviTagMeasurement {
    time: DateTime<Utc>,
    temperature: f64,
    humidity: f64,
    pressure: u32,
    accelerationX: i16,
    accelerationY: i16,
    accelerationZ: i16,
    batteryVoltage: f64,
    movementCounter: u32,
    measurementSequenceNumber: u32,
    #[tag]
    mac: String,
}

impl From<RuuviTag> for RuuviTagMeasurement {
    fn from(tag: RuuviTag) -> Self {
        let values_normalized = match tag.sensor_values {
            SensorValuesType::Normalized(values_normalized) => values_normalized,
            SensorValuesType::Raw(values_raw) => SensorValuesNormalized::from(&values_raw),
        };

        RuuviTagMeasurement {
            time: tag.time,
            temperature: values_normalized.temperature,
            humidity: values_normalized.humidity,
            pressure: values_normalized.pressure,
            accelerationX: values_normalized.acceleration_x,
            accelerationY: values_normalized.acceleration_y,
            accelerationZ: values_normalized.acceleration_z,
            batteryVoltage: values_normalized.battery_voltage,
            movementCounter: values_normalized.movement_counter,
            measurementSequenceNumber: values_normalized.measurement_sequence_number,
            mac: tag.mac.to_string(),
        }
    }
}

pub struct InfluxDBConnector {
    client: Client,
    receiver: UnboundedReceiver<RuuviTag>,
}

pub async fn run_influx_db(mut influxdb_connector: InfluxDBConnector, measurement_name: &str) {
    loop {
        match influxdb_connector.receiver.recv().await {
            Some(measurement) => {
                let _ = influxdb_connector
                    .client
                    .query(&RuuviTagMeasurement::from(measurement).into_query(measurement_name))
                    .await
                    .map_err(|e| eprintln!("{}", e));
            }
            None => {
                eprintln!("All senders have been dropped!");
                return;
            }
        }
    }
}

impl InfluxDBConnector {
    pub fn new(host: &str, db_name: &str) -> (InfluxDBConnector, UnboundedSender<RuuviTag>) {
        let (sender, receiver) = unbounded_channel();
        (
            InfluxDBConnector {
                client: Client::new(host, db_name),
                receiver,
            },
            sender,
        )
    }
}
