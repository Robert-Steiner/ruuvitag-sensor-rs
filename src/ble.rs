use btleplug::api::{BDAddr, Central, CentralEvent::DeviceDiscovered, Peripheral};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};

use crate::measurement::{is_ruuvi_tag, RuuviMeasurement};

use std::str::FromStr;
use std::thread;
use std::time::Duration;

pub fn get_central() -> ConnectedAdapter {
    let manager = Manager::new().unwrap();
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

pub fn scan(central: &ConnectedAdapter) {
    central.start_scan().unwrap();
    thread::sleep(Duration::from_secs(2));
    for device in central.peripherals().into_iter() {
        println!("{}", device);
    }
}

pub fn collect(central: &ConnectedAdapter, ruuvi_tags: &Vec<BDAddr>, timeout: u64) {
    loop {
        thread::sleep(Duration::from_secs(timeout));
        for tag in ruuvi_tags.iter() {
            if let Some(peripheral) = central.peripheral(*tag) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    let hex_format = hex::encode(manufacturer_data);
                    let decode_measurement = RuuviMeasurement::from_str(&hex_format);
                    match decode_measurement {
                        Ok(measurement) => println!("{:#?}", measurement),
                        Err(_) => eprintln!("Decode error!"),
                    }
                }
            } else {
                eprintln!("Tag not found {:?}!", tag)
            }
        }
    }
}

pub fn find_ruuvi_tags(central: &ConnectedAdapter) {
    let central_clone = central.clone();
    central.on_event(Box::new(move |event| match event {
        DeviceDiscovered(bd_addr) => {
            if let Some(peripheral) = central_clone.peripheral(bd_addr) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    let hex_format = hex::encode(manufacturer_data);
                    if is_ruuvi_tag(&hex_format) {
                        println!("New Ruuvi tag: {}", peripheral.properties().address)
                    }
                }
            }
        }
        _ => (),
    }));
    thread::sleep(Duration::from_secs(15));
}
