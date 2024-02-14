use crate::meter::{Measurement, Meter};
use psutil::cpu::CpuPercentCollector;
use std::error::Error;

#[derive(Debug)]
pub struct CpuLoad<'a> {
    pub name: &'a str,
    cpc: CpuPercentCollector,
}

impl CpuLoad<'_> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name: "cpu_load",
            cpc: CpuPercentCollector::new()?,
        })
    }
}

impl Meter for CpuLoad<'_> {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn measure(&mut self) -> Measurement {
        let val: f32 = self.cpc.cpu_percent().unwrap();

        Measurement::new(&self.name(), val)
    }
}
