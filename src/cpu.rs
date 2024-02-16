use crate::meter::{Measurement, Meter};
use psutil::cpu::CpuPercentCollector;
use std::error::Error;

#[derive(Debug)]
pub struct CpuLoad {
    pub name: &'static str,
    cpc: CpuPercentCollector,
}

impl CpuLoad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name: "cpu_load",
            cpc: CpuPercentCollector::new()?,
        })
    }
}

impl Meter for CpuLoad {
    fn name(&self) -> &'static str {
        self.name
    }

    fn measure(&mut self) -> Measurement {
        let val: f32 = self.cpc.cpu_percent().unwrap();

        Measurement::new(&self.name(), val)
    }
}
