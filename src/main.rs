use std::any::type_name;
use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use bevy_reflect::{Reflect, Struct};
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

    let mut map = HashMap::<&str, f32>::new();
    let name_list = get_name_list(&theme);
    for name in &name_list {
        map.insert(&name, 0.0);
    }

    let sched_theme = theme.clone();
    let sched_name_list = name_list.clone();

    thread::spawn(move || {
        let mut scheduler = Scheduler::new(tx);

        for name in sched_name_list {
            match meter_factory(&name) {
                Ok(m) => {
                    scheduler.register_task(Task::new(m, Duration::from_secs(2)));
                }
                Err(err) => {
                    log::warn!("cannot register meter: {err}");
                }
            }
        }
        scheduler.start();
    });

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
                    log::debug!("measurements: {:?}", map);
                    timeout = refresh_period;
                }
                continue;
            }
        };
        // println!("---- {}: {}", m.name, m.value);
        map.get_mut(m.name).map(|val| {
            *val = m.value;
        });
    }
}

fn get_name_list(theme: &themes::Theme) -> Vec<String> {
    let mut res = Vec::<String>::new();

    for (i, value) in theme.stats.iter_fields().enumerate() {
        let name = theme.stats.name_at(i).unwrap();
        let mut interval: u32 = 2;
        match name {
            "interval" => (), // TODO: handle interval overrides
            device_name => {
                let devstats = value.downcast_ref::<Option<themes::DeviceStats>>().unwrap();
                match devstats {
                    Some(ds) => {
                        get_meter_names(&device_name, ds, &mut res);
                    }
                    _ => (),
                }
            }
        }
    }

    res
}

fn get_meter_names(device_name: &str, ds: &themes::DeviceStats, names: &mut Vec<String>) {
    for (i, v) in ds.iter_fields().enumerate() {
        let meter_name = ds.name_at(i).unwrap();
        match meter_name {
            "interval" => (), // TODO: handle interval overrides
            _ => match v.downcast_ref::<Option<themes::DeviceMeter>>() {
                Some(Some(val)) => names.push(format!(
                    "{}:{}",
                    device_name.to_uppercase(),
                    meter_name.to_uppercase()
                )),
                _ => (),
            },
        }
    }
}

fn meter_factory(name: &str) -> Result<Box<dyn Meter>, Box<dyn Error>> {
    let m: Box<dyn Meter> = match name {
        "CPU:PERCENTAGE" => Box::new(CpuPercentage::new()?),
        "CPU:TEMPERATURE" => Box::new(CpuTemperature::new()?),
        _ => return Err(format!("invalid meter {name}").into()),
    };

    Ok(m)
}
