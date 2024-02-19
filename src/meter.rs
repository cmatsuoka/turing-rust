use std::error::Error;

pub trait Meter {
    fn id(&self) -> u64;
    fn measure(&mut self) -> Result<f32, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct MeterConfig {
    pub key: String,
    pub interval: u32,
}
