// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::min;

use crate::{Rect, Res, Rgba, Screen};

#[derive(Debug, Clone)]
pub struct Framebuffer {
    width: usize,
    height: usize,
    fb888: Vec<Rgba>,
    fb565_raw: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            fb888: vec![Rgba::new(0, 0, 0, 0xff); width * height],
            fb565_raw: vec![0; 2 * width * height],
        }
    }

    pub fn copy_from(&mut self, bitmap: &[Rgba], rect: &Rect) {
        let h = min::<usize>(rect.h, self.height);
        for y in 0..h {
            let offset = y * self.width;
            let src_offset = y * rect.w;
            let w = min::<usize>(rect.w, self.width);
            self.fb888[offset..offset + w].copy_from_slice(&bitmap[src_offset..src_offset + w]);
        }
    }

    // RGB565 bit packing:
    // [rrrr rggg] [gggb bbbb]  =(LE)=>  [gggb bbbb] [rrrr rggg]
    fn downmix(&mut self) {
        let mut j = 0;
        for i in 0..self.fb888.len() {
            let p = self.fb888[i];
            self.fb565_raw[j] = ((p.g & 0x1c) << 3) | (p.b >> 3);
            j += 1;
            self.fb565_raw[j] = (p.r & 0xf8) | (p.g >> 5);
            j += 1;
        }
    }

    pub fn render_on(&mut self, scr: &mut Box<dyn Screen>) -> Res<()> {
        let (width, height) = scr.screen_size();
        self.downmix();
        scr.draw_bitmap(&self.fb565_raw, 0, 0, width, height)?;

        Ok(())
    }
}
