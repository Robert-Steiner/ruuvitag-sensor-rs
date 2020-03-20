use hex;
use math::round;
use std::num::ParseIntError;
use std::str::FromStr;

pub fn twos_complement(val: i32, bits: u8) -> i32 {
    if (val & (1 << (bits - 1))) != 0 {
        return val - (1 << bits);
    }
    val
}

fn rshift(val: u16, n: u8) -> u64 {
    (val as u64 % 0x100000000) >> n
}

#[derive(Debug, Default)]
pub struct RuuviMeasurement {
    pub data_format: u8,
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: u16,
    pub acceleration_x: i32,
    pub acceleration_y: i32,
    pub acceleration_z: i32,
    pub battery_voltage: f64,
    pub tx_power: u8,
    pub movement_counter: u8,
    pub sequence_number: u16,
    pub mac: String,
}

impl FromStr for RuuviMeasurement {
    type Err = ParseIntError;

    fn from_str(hex_string: &str) -> Result<Self, Self::Err> {
        let data_format = u8::from_str_radix(&hex_string[4..6], 16)?;
        let temperature = get_temperature(&hex_string).unwrap();
        let humidity = get_humidity(&hex_string).unwrap();
        let pressure = get_pressure(&hex_string).unwrap();
        let (acceleration_x, acceleration_y, acceleration_z) =
            get_acceleration(&hex_string).unwrap();
        let (battery_voltage, tx_power) = get_power_info(&hex_string);
        let battery_voltage = battery_voltage.unwrap_or_default();
        let tx_power = tx_power.unwrap_or_default();
        let movement_counter = get_movement_counter(&hex_string);
        let sequence_number = get_measurement_sequence_number(&hex_string);
        let mac = get_mac(&hex_string);

        Ok(RuuviMeasurement {
            data_format,
            temperature,
            humidity,
            pressure,
            acceleration_x,
            acceleration_y,
            acceleration_z,
            battery_voltage,
            tx_power,
            movement_counter,
            sequence_number,
            mac,
        })
    }
}

// functions ported from python to rust
// source: https://github.com/ttu/ruuvitag-sensor/blob/master/ruuvitag_sensor/decoder.py
// Format of the transmitted data https://github.com/ruuvi/ruuvi-sensor-protocols/blob/master/dataformat_05.md

fn get_temperature(data: &str) -> Option<f64> {
    let temperature_raw = i16::from_str_radix(&data[6..10], 16).unwrap();

    if temperature_raw == 0x7FFF {
        return None;
    }

    let temp_1 = i16::from_str_radix(&data[6..8], 16).unwrap();
    let temp_2 = i16::from_str_radix(&data[8..10], 16).unwrap();

    let temperature = twos_complement(((temp_1 << 8) + temp_2) as i32, 16) as f64 / 200_f64;
    Some(round::ceil(temperature, 2))
}

fn get_humidity(data: &str) -> Option<f64> {
    let humidity_raw = u16::from_str_radix(&data[10..14], 16).unwrap();

    if humidity_raw == 0xFFFF {
        return None;
    }

    let humidity_1 = u16::from_str_radix(&data[10..12], 16).unwrap();
    let humidity_2 = u16::from_str_radix(&data[12..14], 16).unwrap();

    let humidity = ((humidity_1 & 0xFF) << 8 | humidity_2 & 0xFF) as f64 / 400_f64;
    Some(round::ceil(humidity, 2))
}

fn get_pressure(data: &str) -> Option<u16> {
    let pressure_raw = u16::from_str_radix(&data[14..18], 16).unwrap();

    if pressure_raw == 0xFFFF {
        return None;
    }

    let pressure_1 = u16::from_str_radix(&data[14..16], 16).unwrap();
    let pressure_2 = u16::from_str_radix(&data[16..18], 16).unwrap();

    let pressure = ((pressure_1 & 0xFF) << 8 | pressure_2 & 0xFF) as i32 + 50000;
    Some(round::ceil(pressure as f64 / 100_f64, 2) as u16)
}

fn get_acceleration(data: &str) -> Option<(i32, i32, i32)> {
    let acc_x_raw = u16::from_str_radix(&data[18..22], 16).unwrap() as i16;
    let acc_y_raw = u16::from_str_radix(&data[22..26], 16).unwrap() as i16;
    let acc_z_raw = u16::from_str_radix(&data[26..30], 16).unwrap() as i16;

    if acc_x_raw == 0x7FFF || acc_y_raw == 0x7FFF || acc_z_raw == 0x7FFF {
        return None;
    }

    let acc_x_1 = u16::from_str_radix(&data[18..20], 16).unwrap();
    let acc_x_2 = u16::from_str_radix(&data[20..22], 16).unwrap();

    let acc_y_1 = u16::from_str_radix(&data[22..24], 16).unwrap();
    let acc_y_2 = u16::from_str_radix(&data[24..26], 16).unwrap();

    let acc_z_1 = u16::from_str_radix(&data[26..28], 16).unwrap();
    let acc_z_2 = u16::from_str_radix(&data[28..30], 16).unwrap();

    let acc_x = twos_complement(((acc_x_1 << 8) + acc_x_2) as i32, 16);
    let acc_y = twos_complement(((acc_y_1 << 8) + acc_y_2) as i32, 16);
    let acc_z = twos_complement(((acc_z_1 << 8) + acc_z_2) as i32, 16);

    Some((acc_x, acc_y, acc_z))
}

fn get_power_info(data: &str) -> (Option<f64>, Option<u8>) {
    let power_info_1 = u16::from_str_radix(&data[30..32], 16).unwrap();
    let power_info_2 = u16::from_str_radix(&data[32..34], 16).unwrap();

    let power_info = (power_info_1 & 0xFF) << 8 | (power_info_2 & 0xFF);
    let battery_voltage = rshift(power_info, 5) + 1600;
    let tx_power = (power_info & 0b11111) * 2 - 40;

    let mut battery_voltage_opt = None;

    if rshift(power_info, 5) != 0b11111111111 {
        battery_voltage_opt = Some(round::ceil(battery_voltage as f64, 3));
    }

    let mut tx_power_opt = None;

    if (power_info & 0b11111) != 0b11111 {
        tx_power_opt = Some(tx_power as u8);
    }

    (battery_voltage_opt, tx_power_opt)
}

fn get_movement_counter(data: &str) -> u8 {
    let movement_counter_raw = u8::from_str_radix(&data[34..36], 16).unwrap();
    movement_counter_raw & 0xFF
}

fn get_measurement_sequence_number(data: &str) -> u16 {
    let measurement_sequence_number_1 = u16::from_str_radix(&data[36..38], 16).unwrap();
    let measurement_sequence_number_2 = u16::from_str_radix(&data[38..40], 16).unwrap();
    (measurement_sequence_number_1 & 0xFF) << 8 | measurement_sequence_number_2 & 0xFF
}

fn get_mac(data: &str) -> String {
    let mut mac = String::new();
    for x in hex::decode(&data[40..42]).unwrap() {
        mac.push_str(&format!("{:X}", x)[..]);
    }

    for x in hex::decode(&data[44..]).unwrap() {
        mac.push_str(&format!(":{:X}", x)[..]);
    }
    mac
}
