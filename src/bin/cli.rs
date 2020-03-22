use btleplug::api::BDAddr;
use clap::{crate_version, value_t, App, Arg};
use ruuvitag_sensor_rs::ble::{collect, find_ruuvi_tags, get_central, scan};
use ruuvitag_sensor_rs::influx::{run_influx_db, InfluxDBStore};
use std::thread;
use tokio::runtime;

use std::str::FromStr;

pub fn main() {
    let matches = App::new("RuuviTags Sensor CLI")
        .version(crate_version!())
        .subcommand(
            App::new("collect")
                .help("Collects data from RuuviTags and sends them to an InfluxDB instance.")
                .arg(
                    Arg::with_name("mac")
                        .help("The MAC address of the RuuviTag.")
                        .short("m")
                        .long("mac")
                        .multiple(true)
                        .value_name("MAC")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("scanning_rate")
                        .help("The scanning rate in seconds.")
                        .short("s")
                        .long("rate")
                        .value_name("RATE in SEC")
                        .takes_value(true)
                        .default_value("5"),
                )
                .arg(
                    Arg::with_name("influxdb_url")
                        .help("The URL of the InfluxDB instance.")
                        .long("influxdb_url")
                        .value_name("INFLUXDB_URL")
                        .takes_value(true)
                        .default_value("http://localhost:8086"),
                )
                .arg(
                    Arg::with_name("influxdb_db_name")
                        .help("The name of the InfluxDB database.")
                        .long("influxdb_db_name")
                        .value_name("INFLUXDB_DB_NAME")
                        .takes_value(true)
                        .default_value("ruuvi"),
                ),
        )
        .subcommand(App::new("scan").help("Scans for all active BLE devices."))
        .subcommand(App::new("find").help("Finds all active BLE RuuviTags."))
        .get_matches();

    let central = get_central();

    if let Some(_is_present) = matches.subcommand_matches("scan") {
        scan(&central);
    } else if let Some(_is_present) = matches.subcommand_matches("find") {
        find_ruuvi_tags(&central);
    } else if let Some(collect_match) = matches.subcommand_matches("collect") {
        let ruuvi_tags: Vec<BDAddr> = collect_match
            .values_of("mac")
            .unwrap()
            .map(|e| BDAddr::from_str(e).unwrap())
            .collect();

        let scanning_rate = value_t!(collect_match.value_of("scanning_rate"), u64).unwrap();
        let influxdb_url = collect_match.value_of("influxdb_url").unwrap();
        let influxdb_db_name = collect_match.value_of("influxdb_db_name").unwrap();

        let (influx_client, sender) = InfluxDBStore::new(influxdb_url, influxdb_db_name);

        thread::spawn(|| {
            let mut rt = runtime::Builder::new()
                .threaded_scheduler()
                .enable_all()
                .build()
                .unwrap();
            let _ = rt.block_on(async move { run_influx_db(influx_client).await });
        });

        collect(&central, sender, &ruuvi_tags, scanning_rate);
    }
}
