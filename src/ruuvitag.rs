use ruuvi_sensor_protocol::{ParseError, SensorValues};

#[derive(Debug)]
pub struct RuuviTag {
    pub mac: String,
    pub sensor_values: SensorValues,
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
