use std::error::Error;

use psutil::{cpu, sensors};
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;

use crate::meter::Meter;

// CPU percentage

#[derive(Debug)]
pub struct CpuPercentage {
    pub id: u64,
    cpc: cpu::CpuPercentCollector,
}

impl CpuPercentage {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id: const_xxh3(b"CPU:PERCENTAGE"),
            cpc: cpu::CpuPercentCollector::new()?,
        })
    }
}

impl Meter for CpuPercentage {
    fn id(&self) -> u64 {
        self.id
    }

    fn measure(&mut self) -> Result<f32, Box<dyn Error>> {
        let val: f32 = self.cpc.cpu_percent()?;

        Ok(val)
    }
}

// CPU temperature

#[derive(Debug)]
pub struct CpuTemperature {
    pub id: u64,
}

impl CpuTemperature {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id: const_xxh3(b"CPU:TEMPERATURE"),
        })
    }
}

impl Meter for CpuTemperature {
    fn id(&self) -> u64 {
        self.id
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
