use std::error::Error;

pub struct Measurement {
    pub name: &'static str,
    pub value: f32,
}

impl Measurement {
    pub fn new(name: &'static str, value: f32) -> Self {
        Self { name, value }
    }
}

pub trait Meter {
    fn name(&self) -> &'static str;
    fn measure(&mut self) -> Result<f32, Box<dyn Error>>;
}
