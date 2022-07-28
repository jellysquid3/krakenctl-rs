krakenctl-rs
===

A simple and lightweight application for automatically controlling the pump speed of the NZXT Kraken AIO water coolers on Linux systems.
Currently supports configuring a fan curve that reacts to other thermal sensors provided by lm_sensors.

### Supported Devices
- NZXT Kraken X53, X63, and X73
- ... and maybe others, but I don't have any other devices to verify with. Don't get your hopes up.

### Requirements

- lm_sensors (you should have ran sensors-detect as well)
- hidapi

You will need both the header files and the library files. Most package managers should provide what you need, just
search around.

### Building

```
cargo build --release
```

The `krakenctl` executable can be found in `target/release` afterwards, assuming you haven't done something weird to your Rust toolchain.

### Usage

```
krakenctl run --config <file>
krakenctl --help
```
#### Running as a systemD service

The following service file can be installed to `/etc/systemd/service/krakenctl.service` for convenience sake.

```
[Unit]
Description=NZXT Kraken pump speed management

[Service]
Type=simple
ExecStart=/opt/krakenctl run --config /opt/krakenctl.toml

[Install]
WantedBy=multi-user.target
```

You will need to copy the binary from `target/release/krakenctl` to a location which the service can access (such
as `/opt/krakenctl` in this example) and create the config for it.

Reload the daemon with `systemctl daemon-reload` and enable/start the service with `systemctl enable --now krakenctl.service`.

### Configuration

```toml
# The rate at which sensors will be polled to update the pump speed
update_interval = 2.0

[pump]
# The pump speed curve, specified as an array of (tempC, duty%) points. The reading from the temperature sensor
# will be used to linearly interpolate between these points.
curve = [ [50.0, 50.0], [60.0, 60.0], [70.0, 70.0], [80.0, 90.0], [90.0, 100.0] ]
# The smoothing factor to use for filtering the temperature sensor
# An exponential moving average is used as low-pass filter to avoid sudden jumps in pump speed.
# Higher values increase the amount of smoothing that is done.
smoothing = 10.0

[sensor]
# These values can be obtained from running lm_sensors and looking at the output
# The name of the chip to monitor
chip = "k10temp-pci-00c3"
# The feature of the chip to monitor
feature = "Tctl"
```