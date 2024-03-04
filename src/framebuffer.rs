// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{Rect, Res, Rgba, Screen};

#[derive(Debug, Clone)]
pub struct Framebuffer {
    width: usize,
    height: usize,
    fb888: Vec<Rgba>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            fb888: vec![Rgba::new(0, 0, 0, 0xff); width * height],
        }
    }

    pub fn copy_from(&mut self, bitmap: &[Rgba], rect: &Rect) {
        let r = rect.clip(self.width, self.height);

        for y in 0..r.h {
            let offset = (r.y + y) * self.width + r.x;
            let src_offset = y * r.w;
            self.fb888[offset..offset + r.w].copy_from_slice(&bitmap[src_offset..src_offset + r.w]);
        }
    }

    pub fn render_on(&mut self, scr: &mut Box<dyn Screen>, rect: &Rect) -> Res<()> {
        scr.draw_bitmap(&self.fb888, rect)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_from() {
        let bitmap = &[
            Rgba::new(1, 1, 1, 1),
            Rgba::new(2, 2, 2, 2),
            Rgba::new(3, 3, 3, 3),
            Rgba::new(4, 4, 4, 4),
        ];

        let mut fb = Framebuffer::new(3, 3);
        fb.copy_from(bitmap, &Rect::new(1, 1, 2, 2));

        assert_eq!(
            fb.fb888,
            &[
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(1, 1, 1, 1),
                Rgba::new(2, 2, 2, 2),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
            ]
        );
    }

    #[test]
    fn test_copy_from_clipped() {
        let bitmap = &[
            Rgba::new(1, 1, 1, 1),
            Rgba::new(2, 2, 2, 2),
            Rgba::new(3, 3, 3, 3),
            Rgba::new(4, 4, 4, 4),
        ];

        let mut fb = Framebuffer::new(3, 3);
        fb.copy_from(bitmap, &Rect::new(2, 2, 2, 2));

        assert_eq!(
            fb.fb888,
            &[
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(1, 1, 1, 1),
            ]
        );
    }
}
