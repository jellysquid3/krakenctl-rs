use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, io};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_update_interval")]
    pub update_interval: f32,

    pub sensor: SensorSettings,
    pub pump: PumpSettings,
}

impl Config {
    pub fn load_from_path(path: &Path) -> Result<Config, io::Error> {
        let data = fs::read_to_string(path)?;
        toml::from_str(&data).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(from = "(f32, f32)")]
#[serde(into = "(f32, f32)")]
pub struct Step {
    pub temp: f32,
    pub speed: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Profile {
    pub steps: Vec<Step>,
}

#[derive(Serialize, Deserialize)]
pub struct SensorSettings {
    pub chip: String,
    pub feature: String,
}

#[derive(Serialize, Deserialize)]
pub struct PumpSettings {
    pub curve: Profile,

    #[serde(default = "default_smoothing")]
    pub smoothing: f32,
}

impl From<(f32, f32)> for Step {
    fn from((temp, speed): (f32, f32)) -> Self {
        Step { temp, speed }
    }
}

impl Into<(f32, f32)> for Step {
    fn into(self) -> (f32, f32) {
        (self.temp, self.speed)
    }
}

fn default_smoothing() -> f32 { 10.0 }
fn default_update_interval() -> f32 { 1.0 }