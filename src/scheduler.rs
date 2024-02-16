use crate::meter::{Measurement, Meter};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub struct Task {
    meter: Box<dyn Meter>,
    period: Duration,
    last: Instant,
}

impl Task {
    pub fn new(meter: Box<dyn Meter>, period: Duration) -> Self {
        Self {
            meter: meter,
            period: period,
            last: Instant::now() - Duration::from_secs(86400), // a long time ago
        }
    }
}

pub struct Scheduler {
    ch: mpsc::Sender<Measurement>,
    tasks: Vec<Task>,
}

impl Scheduler {
    pub fn new(ch: mpsc::Sender<Measurement>) -> Self {
        Self {
            ch: ch,
            tasks: Vec::new(),
        }
    }

    pub fn register_task(&mut self, task: Task) {
        log::info!("register {}", task.meter.name());
        self.tasks.push(task);
    }

    pub fn start(&mut self) {
        log::info!("start scheduler");
        loop {
            let now = Instant::now();
            for task in &mut self.tasks {
                // if timer expired, run our task
                if task.last.elapsed() >= task.period {
                    task.last = now;
                    let val = match task.meter.measure() {
                        Ok(val) => val,
                        Err(err) => {
                            log::warn!("measurement error: {}", err);
                            0.0
                        }
                    };

                    let m = Measurement::new(task.meter.name(), val);
                    match self.ch.send(m) {
                        Ok(_) => (),
                        Err(err) => {
                            log::warn!("cannot send measurement: {}", err);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
