use crate::measurement::RuuviMeasurement;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use influxdb::{Client, Timestamp};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(InfluxDbWriteable)]
struct RuuviMeasurementInflux {
    time: DateTime<Utc>,
    temperature: f64,
    humidity: f64,
    acceleration_x: i32,
    acceleration_y: i32,
    acceleration_z: i32,
    battery_voltage: f64,
    tx_power: u8,
    movement_counter: u8,
    sequence_number: u16,
    pressure: u16,
    #[tag]
    mac: String,
}

pub struct InfluxDBStore {
    client: Client,
    receiver: UnboundedReceiver<RuuviMeasurement>,
}

pub async fn run_influx_db(mut influx_client: InfluxDBStore) {
    loop {
        match influx_client.receiver.recv().await {
            Some(measurement) => {
                let convert = RuuviMeasurementInflux {
                    time: Timestamp::Now.into(),
                    temperature: measurement.temperature,
                    humidity: measurement.humidity,
                    pressure: measurement.pressure,
                    acceleration_x: measurement.acceleration_x,
                    acceleration_y: measurement.acceleration_y,
                    acceleration_z: measurement.acceleration_z,
                    battery_voltage: measurement.battery_voltage,
                    tx_power: measurement.tx_power,
                    movement_counter: measurement.movement_counter,
                    sequence_number: measurement.sequence_number,
                    mac: measurement.mac,
                };
                let _ = influx_client
                    .client
                    .query(&convert.into_query("ruuvi_measurements_rs"))
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

impl InfluxDBStore {
    pub fn new(host: &str, db_name: &str) -> (InfluxDBStore, UnboundedSender<RuuviMeasurement>) {
        let (sender, receiver) = unbounded_channel();
        (
            InfluxDBStore {
                client: Client::new(host, db_name),
                receiver,
            },
            sender,
        )
    }
}
