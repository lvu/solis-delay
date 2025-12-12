use duration_string::DurationString;
use std::time::Duration;

use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub api_url: String,
    pub api_key_id: String,
    pub api_key_secret: String,
    pub inverter_sn: Option<String>,
    #[serde(default = "AppConfig::default_grid_charging_delay")]
    #[serde(deserialize_with = "AppConfig::deserialize_duration")]
    pub grid_charging_delay: Duration,
    #[serde(default = "AppConfig::default_min_battery_percent")]
    pub min_battery_percent: f64,
    #[serde(default = "AppConfig::default_max_battery_percent")]
    pub max_battery_percent: f64,
}

impl AppConfig {
    fn default_grid_charging_delay() -> Duration {
        Duration::from_secs(600)
    }

    fn default_min_battery_percent() -> f64 {
        90.0
    }

    fn default_max_battery_percent() -> f64 {
        95.0
    }

    fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let d = DurationString::from_string(s).map_err(serde::de::Error::custom)?;
        Ok(d.into())
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let _ = dotenvy::dotenv(); // ignore errors
        let config = envy::prefixed("SOLIS_").from_env::<Self>()?;
        Ok(AppConfig {
            api_url: config.api_url.trim_end_matches('/').to_string(),
            ..config
        })
    }
}
