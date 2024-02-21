use std::collections::HashMap;
use std::error::Error;

use crate::themes::DeviceMeter;

pub trait Meter {
    fn id(&self) -> u64;
    fn measure(&mut self) -> Result<f32, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct MeterConfig {
    pub id: u64,
    pub interval: u32,
    pub layout: DeviceMeter,
}

pub type Measurements = HashMap<u64, f32>;
