use btleplug::api::{
    BDAddr, Central,
    CentralEvent::{
        DeviceConnected, DeviceDisconnected, DeviceDiscovered, DeviceLost, DeviceUpdated,
    },
    Peripheral,
};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
use clap::{crate_version, App, Arg};

use ruuvitag_sensor_rs::measurement::RuuviMeasurement;

use std::str::FromStr;
use std::thread;
use std::time::Duration;

fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

pub fn main() {
    let matches = App::new("Ruuvitag sensor gateway")
        .version(crate_version!())
        .arg(
            Arg::with_name("mac")
                .help("the mac address of the ruuvitag")
                .short("m")
                .long("mac")
                .multiple(true)
                .required(true)
                .value_name("MAC")
                .takes_value(true),
        )
        .get_matches();

    let ruuvi_tags: Vec<BDAddr> = matches
        .values_of("mac")
        .unwrap()
        .map(|e| BDAddr::from_str(e).unwrap())
        .collect();

    let manager = Manager::new().unwrap();

    let central = get_central(&manager);

    loop {
        thread::sleep(Duration::from_secs(5));
        for tag in ruuvi_tags.iter() {
            let resp = central.peripheral(*tag);
            match resp {
                Some(prop) => {
                    let hex_format = hex::encode(prop.properties().manufacturer_data.unwrap());
                    let decode_measurement = RuuviMeasurement::from_str(&hex_format);

                    match decode_measurement {
                        Ok(measurement) => println!("{:?}", measurement),
                        Err(_) => eprintln!("Decode error!"),
                    }
                }
                None => eprintln!("Tag not found {:?}!", tag),
            };
        }
    }
}
