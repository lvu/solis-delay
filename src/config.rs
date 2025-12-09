use chrono::TimeDelta;

#[derive(Debug)]
pub struct AppConfig {
    pub api_url: String,
    pub api_key_id: String,
    pub api_key_secret: String,
    pub inverter_sn: Option<String>,
    pub grid_charging_delay: TimeDelta,
    pub min_battery_percent: f64,
    pub max_battery_percent: f64,
}

impl AppConfig {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv()?;
        let api_url = dotenv::var("SOLIS_API_URL")?;
        let api_url = api_url.trim_end_matches('/');
        let grid_charging_delay = TimeDelta::seconds(
            dotenv::var("SOLIS_GRID_CHARGING_DELAY")
                .unwrap_or_else(|_| "600".to_string())
                .parse::<i64>()?,
        );
        let min_battery_percent = dotenv::var("SOLIS_MIN_BATTERY_PERCENT")
            .unwrap_or_else(|_| "90".to_string())
            .parse::<f64>()?;
        let max_battery_percent = dotenv::var("SOLIS_MAX_BATTERY_PERCENT")
            .unwrap_or_else(|_| "95".to_string())
            .parse::<f64>()?;
        Ok(Self {
            api_url: api_url.to_string(),
            api_key_id: dotenv::var("SOLIS_KEY_ID")?,
            api_key_secret: dotenv::var("SOLIS_KEY_SECRET")?,
            inverter_sn: dotenv::var("SOLIS_INVERTER_SN").ok(),
            grid_charging_delay,
            min_battery_percent,
            max_battery_percent,
        })
    }
}
