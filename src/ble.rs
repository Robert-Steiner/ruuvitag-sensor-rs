use btleplug::api::{BDAddr, Central, CentralEvent::DeviceDiscovered, Peripheral};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
use tokio::sync::mpsc::UnboundedSender;

use crate::ruuvitag::{from_manufacturer_data, is_ruuvitag, RuuviTag};

use std::thread;
use std::time::Duration;

pub fn get_central() -> ConnectedAdapter {
    let manager = Manager::new().unwrap();
    let adapters = manager.adapters().unwrap();
    let mut adapter = adapters.into_iter().nth(0).unwrap();

    // reset the adapter -- clears out any errant state
    adapter = manager.down(&adapter).unwrap();
    adapter = manager.up(&adapter).unwrap();

    adapter.connect().unwrap()
}

pub fn collect(
    central: &ConnectedAdapter,
    sender: UnboundedSender<RuuviTag>,
    ruuvitags_macs: &Vec<BDAddr>,
    scanning_rate: u16,
) {
    loop {
        thread::sleep(Duration::from_secs(scanning_rate.into()));
        for mac in ruuvitags_macs.iter() {
            if let Some(peripheral) = central.peripheral(*mac) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    let sensor_values = from_manufacturer_data(&manufacturer_data);
                    match sensor_values {
                        Ok(data) => {
                            let _ = sender.send(RuuviTag {
                                mac: peripheral.properties().address.to_string(),
                                sensor_values: data,
                            });
                        }
                        Err(_) => eprintln!("Parse error!"),
                    }
                }
            } else {
                eprintln!("RuuviTag not found {}!", mac)
            }
        }
    }
}

pub fn find_ruuvitags(central: &ConnectedAdapter) {
    let central_clone = central.clone();
    central.on_event(Box::new(move |event| match event {
        DeviceDiscovered(bd_addr) => {
            if let Some(peripheral) = central_clone.peripheral(bd_addr) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    if is_ruuvitag(&manufacturer_data) {
                        println!("Found RuuviTag: {}", peripheral.properties().address)
                    }
                }
            }
        }
        _ => (),
    }));
    thread::sleep(Duration::from_secs(15));
}
