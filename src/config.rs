use chrono::TimeDelta;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub api_url: String,
    pub api_key_id: String,
    pub api_key_secret: String,
    pub inverter_sn: Option<String>,
    #[serde(default = "AppConfig::default_grid_charging_delay")]
    pub grid_charging_delay: TimeDelta,
    #[serde(default = "AppConfig::default_min_battery_percent")]
    pub min_battery_percent: f64,
    #[serde(default = "AppConfig::default_max_battery_percent")]
    pub max_battery_percent: f64,
}

impl AppConfig {
    fn default_grid_charging_delay() -> TimeDelta {
        TimeDelta::seconds(600)
    }

    fn default_min_battery_percent() -> f64 {
        90.0
    }

    fn default_max_battery_percent() -> f64 {
        95.0
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
