use crate::ble::Event;
use crate::ble::Event::{DeviceDiscovered, DeviceUpdated};
use crate::influx::{run_influx_db, InfluxDBConnector};
use btleplug::api::BDAddr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use tokio::runtime;

pub struct Controller {
    receiver: Receiver<Event>,
}

impl Controller {
    pub fn new() -> (Controller, Sender<Event>) {
        let (sender, receiver) = mpsc::channel();
        (Controller { receiver }, sender)
    }

    pub fn collect(
        self,
        ruuvitags_macs: &[BDAddr],
        influxdb_url: &str,
        influxdb_db_name: &str,
        influxdb_measurement_name: &str,
    ) {
        let (influx_client, sender) = InfluxDBConnector::new(influxdb_url, influxdb_db_name);
        let measurement_name = influxdb_measurement_name.to_owned();
        thread::spawn(move || {
            let mut rt = runtime::Builder::new()
                .threaded_scheduler()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move { run_influx_db(influx_client, &measurement_name[..]).await });
        });

        loop {
            let event = self.receiver.recv().unwrap();
            if let DeviceUpdated(tag) = event {
                if ruuvitags_macs.contains(&tag.mac) {
                    let _ = sender.send(tag);
                }
            }
        }
    }

    pub fn find(self) {
        loop {
            let event = self.receiver.recv().unwrap();
            if let DeviceDiscovered(tag) = event {
                println!("Found RuuviTag: {}", tag.mac);
            }
        }
    }

    pub fn show(self) {
        loop {
            let event = self.receiver.recv().unwrap();
            match event {
                DeviceDiscovered(tag) | DeviceUpdated(tag) => println!("{}", tag),
            };
        }
    }
}
