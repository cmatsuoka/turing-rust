use std::collections::HashMap;
use std::sync::mpsc;

use crate::meter::{Measurements, MeterConfig};
use crate::themes;

pub struct Renderer {
    ch: mpsc::Receiver<Measurements>,
    widgets: HashMap<u64, themes::DeviceMeter>,
}

impl Renderer {
    pub fn new(ch: mpsc::Receiver<Measurements>, configs: Vec<MeterConfig>) -> Self {
        let mut widgets = HashMap::<u64, themes::DeviceMeter>::new();
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
        for (id, value) in measurements {
            render_widget(&self.widgets[&id], value);
        }
    }
}

fn render_widget(widget: &themes::DeviceMeter, value: f32) {
    if let Some(w) = &widget.text {
        render_text(w, value);
    } else if let Some(w) = &widget.graph {
        render_graph(w, value);
    }
}

fn render_text(text: &themes::Text, value: f32) {
    log::debug!("    Text: {}", value);
}

fn render_graph(graph: &themes::Graph, value: f32) {
    log::debug!("    Graph: {}", value);
}
