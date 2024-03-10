// SPDX-License-Identifier: GPL-3.0-or-later

use rusttype;

use crate::{Rect, Res, Rgba, Screen};

pub struct Font<'a> {
    font: rusttype::Font<'a>,
}

impl Font<'_> {
    pub fn from_data(data: Vec<u8>) -> Res<Self> {
        if let Some(font) = rusttype::Font::try_from_vec(data) {
            Ok(Self { font })
        } else {
            Err("cannot load font data".into())
        }
    }
}

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

    pub fn draw_text(
        &mut self,
        font: &Font,
        size: f32,
        color: Rgba,
        x: usize,
        y: usize,
        msg: &str,
    ) -> Res<Rect> {
        let scale = rusttype::Scale { x: size, y: size };
        let c = color;

        // From rusttype ascii.rs:
        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.font.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);
        let glyphs: Vec<_> = font.font.layout(msg, scale, offset).collect();

        let h = (size + v_metrics.ascent).ceil() as usize;
        let w = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;

        for g in glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|xx, yy, v| {
                    let xx = x as i32 + xx as i32 + bb.min.x;
                    let yy = y as i32 + yy as i32 + bb.min.y;

                    // There's still a possibility that the glyph clips the boundaries of the bitmap
                    if xx >= 0 && xx < self.width as i32 && yy >= 0 && yy < self.height as i32 {
                        let xx = xx as usize;
                        let yy = yy as usize;
                        let bg = &mut self.fb888[yy * self.width + xx];
                        Self::blit_alpha(bg, c, v);
                    }
                });
            }
        }

        let rect = Rect::new(x, y, w, h);
        log::debug!("draw text: '{}' @{},{}, bb {}", msg, x, y, rect);

        Ok(rect)
    }

    pub fn render_on(&mut self, scr: &mut Box<dyn Screen>, rect: &Rect) -> Res<()> {
        scr.expose_framebuffer(&self.fb888, rect)?;
        Ok(())
    }

    fn blit_alpha(bg: &mut Rgba, c: Rgba, v: f32) {
        // short circuit known cases
        if v < 0.01f32 {
            return;
        }
        if v > 0.99f32 {
            *bg = c;
            return;
        }

        let a = (255.0 * v) as u16;
        let ac = 0x00ff - a;
        bg.r = ((bg.r as u16 * ac + c.r as u16 * a) >> 8) as u8;
        bg.g = ((bg.g as u16 * ac + c.g as u16 * a) >> 8) as u8;
        bg.b = ((bg.b as u16 * ac + c.b as u16 * a) >> 8) as u8;
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

    #[test]
    fn test_blit_alpha() {
        for tc in vec![
            (0.0, Rgba::new(0x40, 0x80, 0xc0, 0xff)),
            (0.5, Rgba::new(0x20, 0x7f, 0xde, 0xff)),
            (1.0, Rgba::new(0x00, 0x80, 0xff, 0xff)),
        ] {
            let bg = &mut Rgba::new(0x40, 0x80, 0xc0, 0xff);
            let c = Rgba::new(0x00, 0x80, 0xff, 0x00);

            Framebuffer::blit_alpha(bg, c, tc.0);
            assert_eq!(bg.r, tc.1.r);
            assert_eq!(bg.g, tc.1.g);
            assert_eq!(bg.b, tc.1.b);
        }
    }
}
