use crate::cpu::CpuLoad;
use crate::scheduler::{Scheduler, Task};
use crate::screen::ScreenRevA;
use crate::themes::load;
use clap::Parser;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::process;
use std::sync::mpsc;
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
    SimpleLogger::new().init().unwrap();

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
    let theme_name = args.theme;
    let theme = load(&theme_name)?;

    log::info!("using theme: {:?}", theme_name);

    let _scr = ScreenRevA::new("AUTO");
    let (tx, rx) = mpsc::channel();

    thread::spawn(|| {
        let mut scheduler = Scheduler::new(tx);
        let cpu_load = CpuLoad::new().unwrap();
        scheduler.register_task(Task::new(Box::new(cpu_load), Duration::from_millis(2000)));
        scheduler.start();
    });

    loop {
        let m = rx.recv().unwrap();
        println!("---- {}: {}", m.name, m.value)
    }
}
