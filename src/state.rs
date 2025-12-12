use crate::config::AppConfig;
use crate::solis_client::InverterDetailInfo;
use chrono::{DateTime, TimeDelta, Utc};

#[derive(Debug)]
pub struct AppState {
    pub grid_active: bool,
    pub allow_grid_charging: bool,
    pub last_grid_switch_on_time: DateTime<Utc>,
}

impl AppState {
    pub fn new(config: &AppConfig, now: DateTime<Utc>) -> Self {
        Self {
            grid_active: true,
            allow_grid_charging: true,
            last_grid_switch_on_time: now - config.grid_charging_delay - TimeDelta::seconds(1),
        }
    }

    pub fn update(
        &self,
        detail: &InverterDetailInfo,
        config: &AppConfig,
        now: DateTime<Utc>,
    ) -> Self {
        let new_grid_active = detail.u_ac1 > 2.0; // Margin for noise
        let new_grid_switch_on_time = if new_grid_active && !self.grid_active {
            now
        } else {
            self.last_grid_switch_on_time
        };
        let new_allow_grid_charging =
            new_grid_active  // prevent charging when grid is back
            && now > new_grid_switch_on_time + config.grid_charging_delay  // prevent charging too soon
            && detail.battery_percent < config.max_battery_percent  // prevent charging when battery is full
            && (self.allow_grid_charging || detail.battery_percent < config.min_battery_percent) // hysteresis
        ;

        Self {
            grid_active: new_grid_active,
            allow_grid_charging: new_allow_grid_charging,
            last_grid_switch_on_time: new_grid_switch_on_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::AppConfig;
    use crate::solis_client::InverterDetailInfo;
    use crate::solis_client::InverterState;
    use chrono::{DateTime, TimeZone, Utc};
    use std::time::Duration;

    fn charge_delay() -> Duration {
        Duration::from_secs(300)
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()
    }

    fn long_ago() -> DateTime<Utc> {
        now() - charge_delay() - TimeDelta::seconds(1)
    }

    fn recently() -> DateTime<Utc> {
        now() - TimeDelta::seconds(1)
    }

    fn test_config() -> AppConfig {
        AppConfig {
            api_url: "http://127.0.0.1:8080".to_string(),
            api_key_id: "test".to_string(),
            api_key_secret: "test".to_string(),
            inverter_sn: Some("1234567890".to_string()),
            grid_charging_delay: charge_delay(),
            min_battery_percent: 90.0,
            max_battery_percent: 95.0,
        }
    }

    fn test_detail(battery_percent: f64, grid_active: bool) -> InverterDetailInfo {
        InverterDetailInfo {
            state: InverterState::Online,
            battery_percent: battery_percent,
            u_ac1: if grid_active { 210.0 } else { 0.0 },
        }
    }

    #[test]
    fn test_switch_off_when_no_grid() {
        let config = test_config();
        let state = AppState {
            grid_active: true,
            allow_grid_charging: true,
            last_grid_switch_on_time: long_ago(),
        };
        let detail = test_detail(50.0, false);
        let new_state = state.update(&detail, &config, now());
        assert!(!new_state.grid_active);
        assert!(!new_state.allow_grid_charging);
        assert_eq!(new_state.last_grid_switch_on_time, long_ago());
    }

    #[test]
    fn test_no_switch_on_right_after_grid_back() {
        let config = test_config();
        let state = AppState {
            grid_active: false,
            allow_grid_charging: false,
            last_grid_switch_on_time: long_ago(),
        };
        let detail = test_detail(50.0, true);
        let new_state = state.update(&detail, &config, now());
        assert!(new_state.grid_active);
        assert!(!new_state.allow_grid_charging);
        assert_eq!(new_state.last_grid_switch_on_time, now());
    }

    #[test]
    fn test_no_switch_on_soon_after_grid_back() {
        let config = test_config();
        let state = AppState {
            grid_active: true,
            allow_grid_charging: false,
            last_grid_switch_on_time: recently(),
        };
        let detail = test_detail(50.0, true);
        let new_state = state.update(&detail, &config, now());
        assert!(new_state.grid_active);
        assert!(!new_state.allow_grid_charging);
        assert_eq!(new_state.last_grid_switch_on_time, recently());
    }

    #[test]
    fn test_switch_on_after_delay() {
        let config = test_config();
        let state = AppState {
            grid_active: true,
            allow_grid_charging: false,
            last_grid_switch_on_time: long_ago(),
        };
        let detail = test_detail(50.0, true);
        let new_state = state.update(&detail, &config, now());
        assert!(new_state.allow_grid_charging);
    }

    #[test]
    fn test_switch_off_when_charged() {
        let config = test_config();
        let state = AppState {
            grid_active: true,
            allow_grid_charging: true,
            last_grid_switch_on_time: long_ago(),
        };
        let detail = test_detail(96.0, true);
        let new_state = state.update(&detail, &config, now());
        assert!(!new_state.allow_grid_charging);
    }

    #[test]
    fn test_no_switch_on_while_slightly_discharging() {
        let config = test_config();
        let state = AppState {
            grid_active: true,
            allow_grid_charging: false,
            last_grid_switch_on_time: long_ago(),
        };
        let detail = test_detail(92.0, true);
        let new_state = state.update(&detail, &config, now());
        assert!(!new_state.allow_grid_charging);
    }

    #[test]
    fn test_no_switch_off_while_charging() {
        let config = test_config();
        let state = AppState {
            grid_active: true,
            allow_grid_charging: true,
            last_grid_switch_on_time: long_ago(),
        };
        let detail = test_detail(92.0, true);
        let new_state = state.update(&detail, &config, now());
        assert!(new_state.allow_grid_charging);
    }
}
