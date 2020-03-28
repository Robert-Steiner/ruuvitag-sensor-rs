use btleplug::api::BDAddr;
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
    pub sensor_values: SensorValues,
}

impl fmt::Display for RuuviTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let AccelerationVector(acc_x, acc_y, acc_z) =
            self.sensor_values.acceleration_vector_as_milli_g().unwrap();

        let temperature = (self.sensor_values.temperature_as_millicelsius().unwrap() as f64
            / 1000_f64)
            .to_string();
        let humidity =
            (self.sensor_values.humidity_as_ppm().unwrap() as f64 / 10000_f64).to_string();
        let pressure = ((self.sensor_values.pressure_as_pascals().unwrap() as f64 / 100_f64)
            as u32)
            .to_string();
        let acceleration_x = acc_x.to_string();
        let acceleration_y = acc_y.to_string();
        let acceleration_z = acc_z.to_string();
        let battery_voltage = (self
            .sensor_values
            .battery_potential_as_millivolts()
            .unwrap() as f64
            / 1000_f64)
            .to_string();
        let movement_counter = (self.sensor_values.movement_counter().unwrap()).to_string();
        let measurement_sequence_number =
            (self.sensor_values.measurement_sequence_number().unwrap()).to_string();
        let mac = self.mac.to_string();

        write!(f, "{} : {:>5} °C | {:>7} RH-% | {:>4} Pa | ACC-X {:>5} G | ACC-Y {:>5} G | ACC-Z {:>5} G | {:>5} mV | movement {:>3} | seq# {:>5}",
            mac.bold(),
            temperature.blue(),
            humidity.yellow(),
            pressure.green(),
            acceleration_x,
            acceleration_y,
            acceleration_z,
            battery_voltage.cyan(),
            movement_counter.red(),
            measurement_sequence_number.magenta()
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
