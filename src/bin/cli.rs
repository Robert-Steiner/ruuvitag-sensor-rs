use btleplug::api::{BDAddr, Central, ParseBDAddrError};
use exitfailure::ExitFailure;
use failure::ResultExt;
use ruuvitag_sensor_rs::ble::{get_central, register_event_handler};
use ruuvitag_sensor_rs::controller::Controller;
use std::str::FromStr;

fn parse_address(address: &str) -> Result<BDAddr, ParseBDAddrError> {
    BDAddr::from_str(&address)
}

#[derive(structopt::StructOpt, Debug)]
enum Args {
    Collect {
        #[structopt(
            short = "m",
            long = "mac",
            required = true,
            help = "MAC address of the RuuviTag.",
            parse(try_from_str = parse_address)
        )]
        ruuvitags_macs: Vec<BDAddr>,
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
        #[structopt(
            long = "influxdb_measurement_name",
            default_value = "ruuvi_measurements",
            help = "Name of the measurement."
        )]
        influxdb_measurement_name: String,
    },
    Find {},
    Show {},
}

#[paw::main]
fn main(args: Args) -> Result<(), ExitFailure> {
    let (controller, event_tx) = Controller::new();

    let central =
        get_central().with_context(|_| "could not initialize bluetooth adapter".to_string())?;

    central.active(false);
    register_event_handler(event_tx, &central);

    central
        .start_scan()
        .with_context(|_| "could not start bluetooth scan".to_string())?;

    match args {
        Args::Collect {
            ruuvitags_macs,
            influxdb_url,
            influxdb_db_name,
            influxdb_measurement_name,
        } => {
            controller.collect(
                &ruuvitags_macs,
                &influxdb_url,
                &influxdb_db_name,
                &influxdb_measurement_name,
            );
        }
        Args::Find {} => {
            controller.find();
        }
        Args::Show {} => {
            controller.show();
        }
    }
    Ok(())
}
