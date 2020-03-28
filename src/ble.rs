use crate::ruuvitag::{from_manufacturer_data, is_ruuvitag, RuuviTag};
use btleplug::api::{
    Central,
    CentralEvent::{DeviceDiscovered, DeviceUpdated},
    Peripheral,
};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
use std::sync::mpsc::Sender;

pub fn get_central() -> ConnectedAdapter {
    let manager = Manager::new().unwrap();
    let adapters = manager.adapters().unwrap();
    let mut adapter = adapters.into_iter().nth(0).unwrap();

    // reset the adapter -- clears out any errant state
    adapter = manager.down(&adapter).unwrap();
    adapter = manager.up(&adapter).unwrap();

    adapter.connect().unwrap()
}

pub enum Event {
    DeviceDiscovered(RuuviTag),
    DeviceUpdated(RuuviTag),
}

pub fn register_event_handler(event_sender: Sender<Event>, central: &ConnectedAdapter) {
    let central_clone = central.clone();
    central.on_event(Box::new(move |event| match event {
        DeviceDiscovered(bd_addr) => {
            if let Some(peripheral) = central_clone.peripheral(bd_addr) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    if is_ruuvitag(&manufacturer_data) {
                        let sensor_values = from_manufacturer_data(&manufacturer_data);
                        match sensor_values {
                            Ok(data) => {
                                let _ = event_sender.send(Event::DeviceDiscovered(RuuviTag {
                                    mac: peripheral.properties().address,
                                    sensor_values: data,
                                }));
                            }
                            Err(_) => eprintln!("Error DeviceDiscovered"),
                        }
                    }
                }
            }
        }
        DeviceUpdated(bd_addr) => {
            if let Some(peripheral) = central_clone.peripheral(bd_addr) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    if is_ruuvitag(&manufacturer_data) {
                        let sensor_values = from_manufacturer_data(&manufacturer_data);
                        match sensor_values {
                            Ok(data) => {
                                let _ = event_sender.send(Event::DeviceUpdated(RuuviTag {
                                    mac: peripheral.properties().address,
                                    sensor_values: data,
                                }));
                            }
                            Err(_) => eprintln!("Error DeviceUpdated"),
                        }
                    }
                }
            }
        }
        _ => (),
    }));
}
