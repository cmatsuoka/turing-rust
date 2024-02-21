use std::collections::HashMap;
use std::sync::mpsc;

use crate::meter::{Measurements, MeterConfig};
use crate::themes::DeviceMeter;

pub struct Renderer {
    ch: mpsc::Receiver<Measurements>,
    widgets: HashMap<u64, DeviceMeter>,
}

impl Renderer {
    pub fn new(ch: mpsc::Receiver<Measurements>, configs: Vec<MeterConfig>) -> Self {
        let mut widgets = HashMap::<u64, DeviceMeter>::new();
        for cfg in configs {
            widgets.insert(cfg.id, cfg.layout);
        }
        Self { ch, widgets }
    }

    pub fn start(&mut self) {
        loop {
            match self.ch.recv() {
                Ok(measurements) => {
                    self.render(measurements);
                }
                Err(err) => {
                    log::warn!("renderer receive error: {err}");
                }
            }
        }
    }

    fn render(&self, measurements: Measurements) {
        log::debug!("measurements: {:?}", measurements);
    }
}
