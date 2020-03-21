use btleplug::api::BDAddr;
use clap::{crate_version, value_t, App, Arg};

use ruuvitag_sensor_rs::ble::{collect, find_ruuvi_tags, get_central, scan};

use std::str::FromStr;

pub fn main() {
    let matches = App::new("Ruuvitag sensor gateway")
        .version(crate_version!())
        .arg(
            Arg::with_name("mac")
                .help("the mac address of the ruuvitag")
                .short("m")
                .long("mac")
                .multiple(true)
                .value_name("MAC")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .help("timeout in seconds")
                .short("t")
                .long("timeout")
                .value_name("TIMEOUT in SEC")
                .takes_value(true),
        )
        .subcommand(App::new("scan"))
        .subcommand(App::new("find"))
        .get_matches();

    let central = get_central();

    if let Some(_is_present) = matches.subcommand_matches("scan") {
        scan(&central);
    } else if let Some(_is_present) = matches.subcommand_matches("find") {
        find_ruuvi_tags(&central);
    } else {
        let ruuvi_tags: Vec<BDAddr> = matches
            .values_of("mac")
            .unwrap()
            .map(|e| BDAddr::from_str(e).unwrap())
            .collect();
        let timeout = value_t!(matches.value_of("timeout"), u64).unwrap_or(5);
        collect(&central, &ruuvi_tags, timeout);
    }
}
