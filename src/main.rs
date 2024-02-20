use std::error::Error;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use clap::Parser;
use simple_logger::SimpleLogger;
use xxhash_rust::xxh3::xxh3_64;

use crate::cpu::*;
use crate::meter::{Measurements, Meter, MeterConfig};
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

    let mut meter_map = Measurements::new();
    let meter_configs = themes::get_meter_list(&theme);
    for config in &meter_configs {
        let hash = xxh3_64(config.name.as_bytes());
        meter_map.insert(hash, 0.0);
    }

    // Image rendering thread: prepare framebuffer and communicate
    // with device.
    let (tx, rx) = mpsc::sync_channel(1);
    thread::spawn(|| {
        let mut renderer = Renderer::new(rx);
        renderer.start();
    });

    // Main loop: collect pc stats.
    let mut scheduler = Scheduler::new(tx, refresh_period);
    register_meters(&mut scheduler, meter_configs);
    scheduler.start(meter_map);

    Ok(())
}

fn register_meters(scheduler: &mut Scheduler, meter_list: Vec<MeterConfig>) {
    for meter in meter_list {
        match create_meter(&meter.name) {
            Ok(m) => {
                let interval = Duration::from_secs(meter.interval.into());
                scheduler.register_task(Task::new(m, interval));
            }
            Err(err) => {
                log::warn!("cannot register {}: {}", meter.name, err);
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
