use std::error::Error;

pub struct Measurement {
    pub name: String,
    pub value: f32,
}

impl Measurement {
    pub fn new(name: &str, value: f32) -> Self {
        Self {
            name: name.to_string(),
            value: value,
        }
    }
}

pub trait Meter {
    fn name(&self) -> &'static str;
    fn measure(&mut self) -> Result<f32, Box<dyn Error>>;
}
