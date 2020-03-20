use btleplug::api::{Central, Peripheral};
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

    let ruuvi_tags: Vec<&str> = matches.values_of("mac").unwrap().collect();

    let manager = Manager::new().unwrap();

    let central = get_central(&manager);
    loop {
        // start scanning for devices
        central.start_scan().unwrap();

        thread::sleep(Duration::from_secs(2));
        for device in central.peripherals().into_iter() {
            if ruuvi_tags
                .iter()
                .any(|x| x == &device.address().to_string())
            {
                let hex_format = hex::encode(device.properties().manufacturer_data.unwrap());
                let measurement = RuuviMeasurement::from_str(&hex_format);

                println!("{:?}", measurement);
            }
        }
    }
}
