use std::error::Error;

pub struct Measurement {
    pub id: u64,
    pub value: f32,
}

impl Measurement {
    pub fn new(id: u64, value: f32) -> Self {
        Self { id, value }
    }
}

pub trait Meter {
    fn id(&self) -> u64;
    fn measure(&mut self) -> Result<f32, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct MeterConfig {
    pub key: String,
    pub interval: u32,
}
