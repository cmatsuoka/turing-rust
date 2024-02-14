use crate::cpu::CpuLoad;
use crate::scheduler::{Scheduler, Task};
use crate::screen::ScreenRevA;
use crate::themes::load;
use simple_logger::SimpleLogger;
use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod cpu;
mod meter;
mod scheduler;
mod screen;
mod themes;

fn main() {
    // TODO: parse command line
    let args: Vec<String> = env::args().collect();
    let name = &args[1];

    SimpleLogger::new().init().unwrap();

    let theme = load(name).unwrap();

    let mut cpu_percent_collector = psutil::cpu::CpuPercentCollector::new().unwrap();
    let cpu_percent = cpu_percent_collector.cpu_percent().unwrap();

    println!("{}", cpu_percent);
    println!("{:?}", theme);

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
