use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;
use simple_logger::SimpleLogger;
use xxhash_rust::xxh3::xxh3_64;

use crate::cpu::*;
use crate::meter::{Meter, MeterConfig};
use crate::render::Renderer;
use crate::scheduler::{Scheduler, Task};
use crate::screen::ScreenRevA;

mod cpu;
mod meter;
mod render;
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

    #[arg(short, long, value_name = "device", default_value_t = String::from("AUTO"))]
    port: String,

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

    log::info!("using theme: {theme_name}");

    let _scr = ScreenRevA::new("AUTO");

    let mut meter_map = HashMap::<u64, f32>::new();
    let meter_configs = themes::get_meter_list(&theme);
    for config in &meter_configs {
        let hash = xxh3_64(config.key.as_bytes());
        meter_map.insert(hash, 0.0);
    }

    let _sched_theme = theme.clone();
    let sched_meter_configs = meter_configs.clone();

    // Data collection thread: read pc stats.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut scheduler = Scheduler::new(tx);
        register_meters(&mut scheduler, sched_meter_configs);
        scheduler.start();
    });

    // Image rendering thread: prepare framebuffer and communicate
    // with device.
    let (dev_tx, dev_rx) = mpsc::sync_channel(1);
    thread::spawn(|| {
        let mut renderer = Renderer::new(dev_rx);
        renderer.start();
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
                    match dev_tx.try_send(meter_map.clone()) {
                        Ok(_) => {}
                        Err(err) => {
                            log::info!("renderer send error: {err}")
                        }
                    }
                    timeout = refresh_period;
                }
                continue;
            }
        };
        // println!("---- {}: {}", m.id, m.value);
        if let Some(val) = meter_map.get_mut(&m.id) {
            *val = m.value;
        }
    }
}

fn register_meters(scheduler: &mut Scheduler, meter_list: Vec<MeterConfig>) {
    for meter in meter_list {
        match create_meter(&meter.key) {
            Ok(m) => {
                let interval = Duration::from_secs(meter.interval.into());
                scheduler.register_task(Task::new(m, interval));
            }
            Err(err) => {
                log::warn!("cannot register {}: {}", meter.key, err);
            }
        }
    }
}

fn create_meter(name: &str) -> Result<Box<dyn Meter>, Box<dyn Error>> {
    let m: Box<dyn Meter> = match name {
        "CPU:PERCENTAGE" => Box::new(CpuPercentage::new()?),
        "CPU:TEMPERATURE" => Box::new(CpuTemperature::new()?),
        _ => return Err("invalid meter".into()),
    };

    Ok(m)
}
