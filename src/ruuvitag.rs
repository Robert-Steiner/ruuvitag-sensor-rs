use btleplug::api::BDAddr;
use chrono::{DateTime, Utc};
use colored::*;
use ruuvi_sensor_protocol::{
    Acceleration, AccelerationVector, BatteryPotential, Humidity, MeasurementSequenceNumber,
    MovementCounter, Pressure, Temperature,
};
use ruuvi_sensor_protocol::{ParseError, SensorValues as SensorValuesRaw};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
#[serde(remote = "BDAddr")]
struct BDAddrDef {
    #[serde(getter = "BDAddr::to_string")]
    address: String,
}

impl From<BDAddrDef> for BDAddr {
    fn from(def: BDAddrDef) -> BDAddr {
        BDAddr::from_str(&def.address[..]).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SensorValuesType {
    Raw(SensorValues),
    Normalized(SensorValuesNormalized),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuuviTag {
    #[serde(with = "BDAddrDef")]
    pub mac: BDAddr,
    pub time: DateTime<Utc>,
    pub sensor_values: SensorValuesType,
}

impl RuuviTag {
    pub fn new(mac: BDAddr, manufacturer_data: &[u8]) -> Result<RuuviTag, ParseError> {
        let sensor_values = from_manufacturer_data(manufacturer_data)?;
        Ok(RuuviTag {
            mac,
            time: Utc::now(),
            sensor_values: SensorValuesType::Raw(SensorValues::from(&sensor_values)),
        })
    }

    pub fn normalize_sensor_values(self) -> RuuviTag {
        match self.sensor_values {
            SensorValuesType::Raw(sensor_values) => RuuviTag {
                mac: self.mac,
                time: self.time,
                sensor_values: SensorValuesType::Normalized(SensorValuesNormalized::from(
                    &sensor_values,
                )),
            },
            _ => RuuviTag { ..self },
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SensorValues {
    pub temperature: i32,
    pub humidity: u32,
    pub pressure: u32,
    pub acceleration_x: i16,
    pub acceleration_y: i16,
    pub acceleration_z: i16,
    pub battery_voltage: u16,
    pub movement_counter: u32,
    pub measurement_sequence_number: u32,
}

impl From<&SensorValuesRaw> for SensorValues {
    fn from(sensor_values_raw: &SensorValuesRaw) -> Self {
        let AccelerationVector(acc_x, acc_y, acc_z) = sensor_values_raw
            .acceleration_vector_as_milli_g()
            .unwrap_or(AccelerationVector(0, 0, 0));

        SensorValues {
            temperature: sensor_values_raw
                .temperature_as_millicelsius()
                .unwrap_or_default(),
            humidity: sensor_values_raw.humidity_as_ppm().unwrap_or_default(),
            pressure: sensor_values_raw.pressure_as_pascals().unwrap_or_default(),
            acceleration_x: acc_x,
            acceleration_y: acc_y,
            acceleration_z: acc_z,
            battery_voltage: sensor_values_raw
                .battery_potential_as_millivolts()
                .unwrap_or_default(),
            movement_counter: sensor_values_raw.movement_counter().unwrap_or_default(),
            measurement_sequence_number: sensor_values_raw
                .measurement_sequence_number()
                .unwrap_or_default(),
        }
    }
}

impl fmt::Display for SensorValues {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self as &dyn fmt::Debug).fmt(f)
    }
}

impl fmt::Display for SensorValuesType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Raw(ruuvi_tag) => write!(f, "{}", &ruuvi_tag),
            Self::Normalized(ruuvi_tag) => write!(f, "{}", &ruuvi_tag),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
        SensorValuesNormalized {
            temperature: sensor_values.temperature as f64 / 1000_f64,
            humidity: sensor_values.humidity as f64 / 10000_f64,
            pressure: (sensor_values.pressure as f64 / 100_f64) as u32,
            acceleration_x: sensor_values.acceleration_x,
            acceleration_y: sensor_values.acceleration_y,
            acceleration_z: sensor_values.acceleration_z,
            battery_voltage: sensor_values.battery_voltage as f64 / 1000_f64,
            movement_counter: sensor_values.movement_counter,
            measurement_sequence_number: sensor_values.measurement_sequence_number,
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
            "{} {} : {}",
            self.time,
            self.mac.to_string().bold(),
            self.sensor_values
        )
    }
}

pub fn is_ruuvitag(data: &[u8]) -> bool {
    let manufacturer_id = u16::from(data[0]) + (u16::from(data[1]) << 8);
    manufacturer_id == 0x0499
}

pub fn from_manufacturer_data(data: &[u8]) -> Result<SensorValuesRaw, ParseError> {
    if data.len() > 2 {
        let id = u16::from(data[0]) + (u16::from(data[1]) << 8);
        SensorValuesRaw::from_manufacturer_specific_data(id, &data[2..])
    } else {
        Err(ParseError::EmptyValue)
    }
}
