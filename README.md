# ruuvitag-sensor-rs

A small CLI that you can use to find nearby RuuviTags or to collect sensor data of the RuuviTags and send them to an InfluxDB instance. 
The influx data schema is the same one that used in [ruuvi.grafana-dashboards.json](https://github.com/ruuvi/ruuvi.grafana-dashboards.json) and should work out of the box.


This CLI was successfully tested on a Raspberry PI 3B+.

## Perquisites

- rustup
- Linux
- BlueZ bluetooth library

## Build

```
git clone https://github.com/Robert-Steiner/ruuvitag-sensor-rs.git
cd ruuvitag-sensor-rs
cargo install --path .
sudo setcap 'cap_net_raw,cap_net_admin+eip' `which ruuvi-cli`
```

## Usage

### Find nearby RuuviTags

Command:

`ruuvi-cli find`

Output:

```
Found RuuviTag: FD:AA:AA:AA:AA:AA
Found RuuviTag: EC:BB:BB:BB:BB:BB
...
```

### Collect sensor data

Command:

`ruuvi-cli collect -m FD:AA:AA:AA:AA:AA -m EC:BB:BB:BB:BB:BB`

To see all `collect` options run:

`ruuvi-cli collect -h`
