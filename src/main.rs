use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};


use clap::Parser;
use simple_logger::SimpleLogger;

use crate::cpu::*;
use crate::meter::Meter;
use crate::scheduler::{Scheduler, Task};
use crate::screen::ScreenRevA;

mod cpu;
mod meter;
mod scheduler;
mod screen;
mod themes;

#[derive(Parser)]
#[command(name = "turing-screen")]
#[command(about = "A lightweight turing smart screen updater")]
struct Args {
    /// Set screen brightness in 0-255 range
    #[arg(short, long, value_name = "level")]
    brightness: Option<i32>,

    /// Screen refresh period in seconds
    #[arg(short, long, value_name = "num", default_value_t = 5)]
    refresh: u64,

    #[arg(value_name = "theme_name")]
    theme: String,
}

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init()?;

    let refresh_period = Duration::from_secs(args.refresh);
    let theme_name = args.theme;
    let theme = Arc::new(themes::load(&theme_name)?);

    log::info!("using theme: {:?}", theme_name);

    let _scr = ScreenRevA::new("AUTO");
    let (tx, rx) = mpsc::channel();

    let mut meter_map = HashMap::<&str, f32>::new();
    let meter_configs = themes::get_meter_list(&theme);
    for config in &meter_configs {
        meter_map.insert(&config.key, 0.0);
    }

    let _sched_theme = theme.clone();
    let sched_meter_configs = meter_configs.clone();

    // Data collection thread: read pc stats.
    thread::spawn(move || {
        let mut scheduler = Scheduler::new(tx);

        for meter in sched_meter_configs {
            match create_meter(&meter.key) {
                Ok(m) => {
                    scheduler.register_task(Task::new(m, Duration::from_secs(2)));
                }
                Err(err) => {
                    log::warn!("cannot register {}: {}", meter.key, err);
                }
            }
        }
        scheduler.start();
    });

    // Main dispatcher: collect meter readings and send data to
    // the renderer thread.
    // TODO: use recv_deadline when it stabilizes
    let mut timeout = refresh_period;
    loop {
        let now = Instant::now();
        let m = match rx.recv_timeout(timeout) {
            Ok(m) => {
                let age = now.elapsed();
                timeout = if timeout > age {
                    timeout - age
                } else {
                    Duration::ZERO
                };
                m
            }
            Err(err) => {
                if err == mpsc::RecvTimeoutError::Timeout {
                    log::debug!("measurements: {:?}", meter_map);
                    timeout = refresh_period;
                }
                continue;
            }
        };
        // println!("---- {}: {}", m.name, m.value);
        if let Some(val) = meter_map.get_mut(m.name) {
            *val = m.value;
        }
    }
}

fn create_meter(name: &str) -> Result<Box<dyn Meter>, Box<dyn Error>> {
    let m: Box<dyn Meter> = match name {
        "CPU:PERCENTAGE" => Box::new(CpuPercentage::new()?),
        "CPU:TEMPERATURE" => Box::new(CpuTemperature::new()?),
        _ => return Err(format!("invalid meter {name}").into()),
    };

    Ok(m)
}
