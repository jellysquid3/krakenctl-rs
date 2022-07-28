#[macro_use]
extern crate log;

use std::path::Path;
use std::thread;
use std::time::Duration;

use clap::{Parser, Subcommand};
use lm_sensors::prelude::*;
use lm_sensors::value::Kind;
use lm_sensors::SubFeatureRef;

use config::*;
use krakenusb::{self, KrakenUsbDevice};

mod config;

#[derive(Parser)]
struct CliArgs {
    #[clap(long)]
    verbose: bool,

    #[clap(subcommand)]
    action: CliAction,
}

#[derive(Subcommand)]
enum CliAction {
    Run(CliRunArgs),
    ListSensors,
}

#[derive(Parser)]
struct CliRunArgs {
    #[clap(long)]
    config: String,
}

fn main() {
    let args = CliArgs::parse();

    env_logger::builder()
        .filter_level(if args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .try_init()
        .expect("Failed to initialize logging");

    match args.action {
        CliAction::Run(action) => run(&action),
        CliAction::ListSensors => list_sensors(),
    }
}

fn run(action: &CliRunArgs) {
    let config = Config::load_from_path(Path::new(&action.config))
        .expect("Failed to load configuration file");

    info!(
        "Update interval: Every {:.2} second(s)",
        config.update_interval
    );
    info!(
        "Sensor source: (chip = '{}', feature = '{}')",
        config.sensor.chip, config.sensor.feature
    );

    let mut device = krakenusb::find_device()
        .expect("Couldn't find Kraken driver");

    let sensors = lm_sensors::Initializer::default()
        .initialize()
        .expect("Failed to initialize sensors interface");

    let sensor_chip = sensors
        .chip_iter(None)
        .find(|chip| chip.name().map_or(false, |name| name == config.sensor.chip))
        .expect("Failed to find sensor chip");

    let sensor_feature = sensor_chip
        .feature_iter()
        .find(|feature| {
            feature
                .label()
                .map_or(false, |label| label == config.sensor.feature)
        })
        .expect("Failed to find sensor feature");

    let mut sensor = sensor_feature
        .sub_feature_by_kind(Kind::TemperatureInput)
        .expect("Couldn't find sub-feature");

    run_loop(&mut device, &mut sensor, &config);
}

fn run_loop(device: &mut Box<dyn KrakenUsbDevice>, sensor: &mut SubFeatureRef, config: &Config) {
    let update_interval = config.update_interval;
    let pump_settings = &config.pump;

    let mut prev_average: Option<f32> = None;

    let cutoff_freq = (1.0 / update_interval) / pump_settings.smoothing;
    let alpha = 1.0 - f32::exp(-2.0 * std::f32::consts::PI * cutoff_freq);

    loop {
        let sample = sensor.raw_value().expect("Failed to read sensor value") as f32;

        let average = match prev_average {
            Some(ema) => alpha * sample + (1.0 - alpha) * ema,
            None => sample,
        };

        let target_duty = interpolate_profile(&pump_settings.curve, average);
        debug!(
            "Temp (raw = {:.1} C, avg = {:.1} C) - Duty: {:.1}%",
            sample, average, target_duty
        );

        let duty = round_to_interval(target_duty, 5.0);
        device.set_fixed_speed(duty);

        prev_average = Some(average);

        thread::sleep(Duration::from_secs_f32(update_interval));
    }
}

fn list_sensors() {}

fn round_to_interval(value: f32, step: f32) -> u8 {
    let rounded = step * (value / step).round();
    rounded as u8
}

fn interpolate_profile(profile: &Profile, current_temp: f32) -> f32 {
    let steps = &profile.steps;

    let mut min = steps.first().unwrap();
    let mut max = steps.last().unwrap();

    for step in steps {
        if step.temp <= current_temp {
            min = step;
        } else if step.temp >= current_temp {
            max = step;
            break;
        }
    }
    if min.temp == max.temp {
        return min.speed;
    }

    min.speed + (current_temp - min.temp) / (max.temp - min.temp) * (max.speed - min.speed)
}
