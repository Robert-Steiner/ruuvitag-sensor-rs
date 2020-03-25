use btleplug::api::BDAddr;
use ruuvitag_sensor_rs::ble::{collect, find_ruuvitags, get_central};
use ruuvitag_sensor_rs::influx::{run_influx_db, InfluxDBConnector};
use std::thread;
use tokio::runtime;

use std::str::FromStr;

#[derive(structopt::StructOpt, Debug)]
enum Args {
    Collect {
        #[structopt(
            short = "m",
            long = "mac",
            required = true,
            help = "MAC address of the RuuviTag."
        )]
        macs: Vec<String>,
        #[structopt(
            short = "s",
            long = "rate",
            default_value = "5",
            help = "Scanning rate in seconds."
        )]
        scanning_rate: u16,
        #[structopt(
            long = "influxdb_url",
            default_value = "http://localhost:8086",
            help = "URL of the InfluxDB instance."
        )]
        influxdb_url: String,
        #[structopt(
            long = "influxdb_db_name",
            default_value = "ruuvi",
            help = "Name of the InfluxDB database."
        )]
        influxdb_db_name: String,
    },
    Find {},
}
#[paw::main]
fn main(args: Args) -> Result<(), std::io::Error> {
    let central = get_central();

    match args {
        Args::Collect {
            macs,
            scanning_rate,
            influxdb_url,
            influxdb_db_name,
        } => {
            let ruuvi_tags: Vec<BDAddr> =
                macs.iter().map(|e| BDAddr::from_str(&e).unwrap()).collect();
            let (influx_client, sender) = InfluxDBConnector::new(&influxdb_url, &influxdb_db_name);

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
        Args::Find {} => {
            find_ruuvitags(&central);
        }
    }
    Ok(())
}
