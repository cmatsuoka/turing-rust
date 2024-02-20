use std::collections::HashMap;
use std::error::Error;

pub trait Meter {
    fn id(&self) -> u64;
    fn measure(&mut self) -> Result<f32, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct MeterConfig {
    pub name: String,
    pub interval: u32,
}

pub type Measurements = HashMap<u64, f32>;
