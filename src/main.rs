use crate::cpu::*;
use crate::scheduler::{Scheduler, Task};
use crate::screen::ScreenRevA;
use crate::themes::load;
use clap::Parser;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

mod cpu;
mod meter;
mod scheduler;
mod screen;
mod themes;

#[derive(Parser)]
struct Args {
    /// Screen brightness
    #[arg(short, long)]
    brightness: Option<i32>,

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

    let theme_name = args.theme;
    let theme = Arc::new(load(&theme_name)?);

    log::info!("using theme: {:?}", theme_name);

    let _scr = ScreenRevA::new("AUTO");
    let (tx, rx) = mpsc::channel();

    let mut map = HashMap::new();
    for (stat_name, stat) in &theme.stats {
        for (name, _) in stat {
            let key = format!("{stat_name}:{name}");
            log::debug!("found {key}");
            map.insert(key, 0.0);
        }
    }

    let sched_theme = theme.clone();

    thread::spawn(move || {
        let mut scheduler = Scheduler::new(tx);

        let cpu_perc = CpuPercentage::new().unwrap();
        scheduler.register_task(Task::new(Box::new(cpu_perc), Duration::from_millis(2000)));

        let cpu_temp = CpuTemperature::new().unwrap();
        scheduler.register_task(Task::new(Box::new(cpu_temp), Duration::from_millis(3000)));

        scheduler.start();
    });

    loop {
        let m = rx.recv().unwrap();
        println!("---- {}: {:?}", m.name, map);
        map.get_mut(m.name).map(|val| {
            *val = m.value;
        });
    }
}
