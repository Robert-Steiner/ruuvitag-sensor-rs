use crate::ruuvitag::RuuviTag;
use chrono::{DateTime, Utc};
use influxdb::{Client, InfluxDbWriteable, Timestamp};
use ruuvi_sensor_protocol::{
    Acceleration, AccelerationVector, BatteryPotential, Humidity, MeasurementSequenceNumber,
    MovementCounter, Pressure, Temperature,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

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
        let AccelerationVector(acc_x, acc_y, acc_z) =
            tag.sensor_values.acceleration_vector_as_milli_g().unwrap();
        RuuviTagMeasurement {
            time: Timestamp::Now.into(),
            temperature: tag.sensor_values.temperature_as_millicelsius().unwrap() as f64 / 1000_f64,
            humidity: tag.sensor_values.humidity_as_ppm().unwrap() as f64 / 10000_f64,
            pressure: (tag.sensor_values.pressure_as_pascals().unwrap() as f64 / 100_f64) as u32,
            accelerationX: acc_x,
            accelerationY: acc_y,
            accelerationZ: acc_z,
            batteryVoltage: tag.sensor_values.battery_potential_as_millivolts().unwrap() as f64
                / 1000_f64,
            movementCounter: tag.sensor_values.movement_counter().unwrap(),
            measurementSequenceNumber: tag.sensor_values.measurement_sequence_number().unwrap(),
            mac: tag.mac,
        }
    }
}

pub struct InfluxDBConnector {
    client: Client,
    receiver: UnboundedReceiver<RuuviTag>,
}

pub async fn run_influx_db(mut influxdb_connector: InfluxDBConnector) {
    loop {
        match influxdb_connector.receiver.recv().await {
            Some(measurement) => {
                let _ = influxdb_connector
                    .client
                    .query(
                        &RuuviTagMeasurement::from(measurement).into_query("ruuvi_measurements_rs"),
                    )
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
