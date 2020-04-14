use crate::ble::Event;
use crate::ble::Event::{DeviceDiscovered, DeviceUpdated};
use crate::influx::{run_influx_db, InfluxDBConnector};
use crate::ruuvitag::RuuviTag;
use atty::Stream;
use btleplug::api::BDAddr;
use serde_json;
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

        let func = if ruuvitags_macs.is_empty() {
            let without_filter: Box<dyn Fn(RuuviTag) -> ()> = Box::new(|tag: RuuviTag| {
                let _ = sender.send(tag);
            });
            without_filter
        } else {
            let with_filter: Box<dyn Fn(RuuviTag) -> ()> = Box::new(|tag: RuuviTag| {
                if ruuvitags_macs.contains(&tag.mac) {
                    let _ = sender.send(tag);
                }
            });
            with_filter
        };

        loop {
            let event = self.receiver.recv().unwrap();
            if let DeviceUpdated(tag) = event {
                func(tag);
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

    pub fn write(self, normalize: bool) {
        loop {
            let event = self.receiver.recv().unwrap();
            match event {
                DeviceDiscovered(tag) | DeviceUpdated(tag) => {
                    let maybe_normalized = if normalize {
                        tag.normalize_sensor_values()
                    } else {
                        tag
                    };

                    if atty::is(Stream::Stdout) {
                        println!("{}", maybe_normalized);
                    } else {
                        println!("{}", serde_json::to_string(&maybe_normalized).unwrap());
                    }
                }
            }
        }
    }
}
