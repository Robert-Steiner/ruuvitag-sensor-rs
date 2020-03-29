use crate::ruuvitag::{is_ruuvitag, RuuviTag};
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
    let mut adapter = adapters.into_iter().next().unwrap();

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
    let central_event_handler = central.clone();

    let on_event_handler = Box::new(move |event| match event {
        DeviceDiscovered(bd_addr) => {
            if let Some(peripheral) = central_event_handler.peripheral(bd_addr) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    if is_ruuvitag(&manufacturer_data) {
                        match RuuviTag::new(peripheral.properties().address, &manufacturer_data) {
                            Ok(new_tag) => {
                                let _ = event_sender.send(Event::DeviceDiscovered(new_tag));
                            }
                            Err(e) => eprintln!("Error DeviceDiscovered {}", e),
                        }
                    }
                }
            }
        }
        DeviceUpdated(bd_addr) => {
            if let Some(peripheral) = central_event_handler.peripheral(bd_addr) {
                if let Some(manufacturer_data) = peripheral.properties().manufacturer_data {
                    if is_ruuvitag(&manufacturer_data) {
                        match RuuviTag::new(peripheral.properties().address, &manufacturer_data) {
                            Ok(updated_tag) => {
                                let _ = event_sender.send(Event::DeviceUpdated(updated_tag));
                            }
                            Err(e) => eprintln!("Error DeviceUpdated {}", e),
                        }
                    }
                }
            }
        }
        _ => (),
    });

    central.on_event(on_event_handler);
}
