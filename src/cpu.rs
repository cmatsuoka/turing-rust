use crate::meter::Meter;
use psutil::{cpu, sensors};
use std::error::Error;

// CPU percentage

#[derive(Debug)]
pub struct CpuPercentage {
    pub name: &'static str,
    cpc: cpu::CpuPercentCollector,
}

impl CpuPercentage {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name: "CPU:PERCENTAGE",
            cpc: cpu::CpuPercentCollector::new()?,
        })
    }
}

impl Meter for CpuPercentage {
    fn name(&self) -> &'static str {
        self.name
    }

    fn measure(&mut self) -> Result<f32, Box<dyn Error>> {
        let val: f32 = self.cpc.cpu_percent()?;

        Ok(val)
    }
}

// CPU temperature

#[derive(Debug)]
pub struct CpuTemperature {
    pub name: &'static str,
}

impl CpuTemperature {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name: "CPU:TEMPERATURE",
        })
    }
}

impl Meter for CpuTemperature {
    fn name(&self) -> &'static str {
        self.name
    }

    fn measure(&mut self) -> Result<f32, Box<dyn Error>> {
        let temps = sensors::temperatures();
        for temp in temps {
            match &temp {
                Ok(t) => {
                    if t.unit() == "k10temp" && t.label() == Some("Tccd1") {
                        return Ok(t.current().celsius() as f32);
                    }
                }
                Err(_) => (),
            };
        }

        Ok(0.0)
    }
}
