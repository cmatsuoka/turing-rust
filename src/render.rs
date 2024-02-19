use std::collections::HashMap;
use std::sync::mpsc;

pub struct Renderer {
    ch: mpsc::Receiver<HashMap<u64, f32>>,
}

impl Renderer {
    pub fn new(ch: mpsc::Receiver<HashMap<u64, f32>>) -> Self {
        Self { ch }
    }

    pub fn start(&mut self) {
        loop {
            match self.ch.recv() {
                Ok(map) => {
                    log::debug!("measurement: {:?}", map);
                }
                Err(err) => {
                    log::warn!("renderer receive error: {err}");
                }
            }
        }
    }
}
