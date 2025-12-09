mod config;
mod solis_client;
mod state;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use chrono::{TimeDelta, Utc};
use config::AppConfig;
use log::{info, warn};
use signal_hook::consts::TERM_SIGNALS;
use solis_client::{InverterCommand, InverterState, SolisApi};
use state::AppState;

const EXECUTION_INTERVAL: TimeDelta = TimeDelta::seconds(30);

fn worker_step(
    api: &SolisApi,
    config: &AppConfig,
    state: &mut AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let inverter_sn = config.inverter_sn.as_ref().unwrap();
    let detail = api.get_inverter_detail(inverter_sn)?;
    info!("detail: {:?}", detail);
    if detail.state == InverterState::Offline {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "inverter is offline",
        )));
    }

    let new_state = state.update(&detail, config, Utc::now());
    if new_state.allow_grid_charging != state.allow_grid_charging {
        let value = if new_state.allow_grid_charging {
            "1"
        } else {
            "0"
        };
        info!("updating allow_grid_charging to {}", value);
        api.update_parameter_value_if_needed(
            inverter_sn,
            InverterCommand::AllowGridCharging,
            value,
        )?;
    }
    *state = new_state;
    info!("new state: {:?}", &state);

    Ok(())
}

fn worker(api: &SolisApi, config: &AppConfig) {
    let term = Arc::new(AtomicBool::new(false));
    for &signal in TERM_SIGNALS {
        let _ = signal_hook::flag::register(signal, Arc::clone(&term));
    }
    let mut state = AppState::new(config, Utc::now());
    let mut last_exec = Utc::now() - EXECUTION_INTERVAL;
    while !term.load(std::sync::atomic::Ordering::Relaxed) {
        if Utc::now() - last_exec > EXECUTION_INTERVAL {
            if let Err(e) = worker_step(api, config, &mut state) {
                warn!("error: {:#?}", e);
            }
            last_exec = Utc::now();
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn main() {
    env_logger::init();
    let config = AppConfig::new().unwrap();
    let api = SolisApi::new(
        config.api_url.clone(),
        config.api_key_id.clone(),
        config.api_key_secret.clone(),
    );

    if config.inverter_sn.is_none() {
        println!("SOLIS_INVERTER_SN is not set; here are the inverters:");
        match api.get_inverters() {
            Ok(inverters) => {
                for inverter in inverters {
                    println!("ID: {}, SN: {}", inverter.id, inverter.sn);
                }
            }
            Err(e) => {
                println!("error: {:?}", e);
                std::process::exit(1);
            }
        }
        std::process::exit(2);
    }

    worker(&api, &config);
}
