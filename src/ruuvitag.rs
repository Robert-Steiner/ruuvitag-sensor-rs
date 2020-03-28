use btleplug::api::BDAddr;
use chrono::{DateTime, Utc};
use colored::*;
use ruuvi_sensor_protocol::{
    Acceleration, AccelerationVector, BatteryPotential, Humidity, MeasurementSequenceNumber,
    MovementCounter, Pressure, Temperature,
};
use ruuvi_sensor_protocol::{ParseError, SensorValues};
use std::fmt;

#[derive(Debug)]
pub struct RuuviTag {
    pub mac: BDAddr,
    pub time: DateTime<Utc>,
    pub sensor_values: SensorValues,
}

impl RuuviTag {
    pub fn new(mac: BDAddr, manufacturer_data: &[u8]) -> Result<RuuviTag, ParseError> {
        let sensor_values = from_manufacturer_data(manufacturer_data)?;
        Ok(RuuviTag {
            mac,
            time: Utc::now(),
            sensor_values,
        })
    }
}

pub struct SensorValuesNormalized {
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: u32,
    pub acceleration_x: i16,
    pub acceleration_y: i16,
    pub acceleration_z: i16,
    pub battery_voltage: f64,
    pub movement_counter: u32,
    pub measurement_sequence_number: u32,
}

impl From<&SensorValues> for SensorValuesNormalized {
    fn from(sensor_values: &SensorValues) -> Self {
        let AccelerationVector(acc_x, acc_y, acc_z) =
            sensor_values.acceleration_vector_as_milli_g().unwrap();

        SensorValuesNormalized {
            temperature: sensor_values.temperature_as_millicelsius().unwrap() as f64 / 1000_f64,
            humidity: sensor_values.humidity_as_ppm().unwrap() as f64 / 10000_f64,
            pressure: (sensor_values.pressure_as_pascals().unwrap() as f64 / 100_f64) as u32,
            acceleration_x: acc_x,
            acceleration_y: acc_y,
            acceleration_z: acc_z,
            battery_voltage: sensor_values.battery_potential_as_millivolts().unwrap() as f64
                / 1000_f64,
            movement_counter: sensor_values.movement_counter().unwrap(),
            measurement_sequence_number: sensor_values.measurement_sequence_number().unwrap(),
        }
    }
}

impl fmt::Display for SensorValuesNormalized {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:>5} °C | {:>7} RH-% | {:>4} Pa | ACC-X {:>5} G | ACC-Y {:>5} G | ACC-Z {:>5} G | {:>5} mV | movement {:>3} | seq# {:>5}",
            self.temperature.to_string().blue(),
            self.humidity.to_string().yellow(),
            self.pressure.to_string().green(),
            self.acceleration_x,
            self.acceleration_y,
            self.acceleration_z,
            self.battery_voltage.to_string().cyan(),
            self.movement_counter.to_string().red(),
            self.measurement_sequence_number.to_string().magenta()
        )
    }
}

impl fmt::Display for RuuviTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} : {}",
            self.mac.to_string().bold(),
            SensorValuesNormalized::from(&self.sensor_values)
        )
    }
}

pub fn is_ruuvitag(data: &[u8]) -> bool {
    let manufacturer_id = u16::from(data[0]) + (u16::from(data[1]) << 8);
    manufacturer_id == 0x0499
}

pub fn from_manufacturer_data(data: &[u8]) -> Result<SensorValues, ParseError> {
    if data.len() > 2 {
        let id = u16::from(data[0]) + (u16::from(data[1]) << 8);
        SensorValues::from_manufacturer_specific_data(id, &data[2..])
    } else {
        Err(ParseError::EmptyValue)
    }
}
