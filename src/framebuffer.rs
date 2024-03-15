// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::min;

use crate::{Coord, Image, Rect, Res, Rgba, Screen};

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
            fb888: vec![crate::BLACK; width * height],
        }
    }

    pub fn copy_image(&mut self, image: &Image, crop: &Rect, dest: &Coord) {
        let crop = crop.clip(
            min(image.width, self.width - dest.x),
            min(image.height, self.height - dest.y),
        );

        for y in 0..crop.h {
            let offset = (dest.y + y) * self.width + crop.x + dest.x;
            let src_offset = (crop.y + y) * image.width + crop.x;
            self.fb888[offset..offset + crop.w]
                .copy_from_slice(&image.buffer[src_offset..src_offset + crop.w]);
        }
    }

    pub fn blend_image(&mut self, image: &Image, crop: &Rect, dest: &Coord) {
        let crop = crop.clip(
            min(image.width, self.width - dest.x),
            min(image.height, self.height - dest.y),
        );

        for y in 0..crop.h {
            let offset = (dest.y + y) * self.width + dest.x;
            let src_offset = (crop.y + y) * image.width + crop.x;
            for x in 0..crop.w {
                let fg = image.buffer[src_offset + x];
                if fg.a > 0 {
                    let bg = &mut self.fb888[offset + x];
                    Self::blend_alpha(bg, fg);
                }
            }
        }
    }

    pub fn render_on(&mut self, scr: &mut Box<dyn Screen>, rect: &Rect) -> Res<()> {
        scr.expose_framebuffer(&self.fb888, rect)?;
        Ok(())
    }

    #[inline]
    fn blend_alpha(bg: &mut Rgba, fg: Rgba) {
        // short circuit cases
        if fg.a == 0x00 {
            return;
        }
        if fg.a == 0xff {
            *bg = fg;
            return;
        }

        let a = fg.a as u16;
        let ac = 0x00ff - a;
        bg.r = ((bg.r as u16 * ac + fg.r as u16 * a) >> 8) as u8;
        bg.g = ((bg.g as u16 * ac + fg.g as u16 * a) >> 8) as u8;
        bg.b = ((bg.b as u16 * ac + fg.b as u16 * a) >> 8) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_image() {
        let image = Image {
            buffer: &mut [
                Rgba::new(1, 1, 1, 1),
                Rgba::new(2, 2, 2, 2),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
            ],
            width: 2,
            height: 2,
        };

        let mut fb = Framebuffer::new(3, 3);
        fb.copy_image(&image, &Rect::new(0, 0, 20, 20), &Coord::new(1, 1));

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
    fn test_copy_image_cropped() {
        let image = Image {
            buffer: &mut [
                Rgba::new(1, 1, 1, 1),
                Rgba::new(2, 2, 2, 2),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
            ],
            width: 2,
            height: 2,
        };

        let mut fb = Framebuffer::new(3, 3);
        fb.copy_image(&image, &Rect::new(0, 1, 2, 1), &Coord::new(1, 1));

        assert_eq!(
            fb.fb888,
            &[
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
            ]
        );
    }

    #[test]
    fn test_copy_image_clipped() {
        let image = Image {
            buffer: &mut [
                Rgba::new(1, 1, 1, 1),
                Rgba::new(2, 2, 2, 2),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
            ],
            width: 2,
            height: 2,
        };

        let mut fb = Framebuffer::new(3, 3);
        fb.copy_image(&image, &Rect::new(0, 0, 2, 2), &Coord::new(2, 2));

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
    fn test_blend_alpha() {
        for tc in vec![
            (
                Rgba::new(0x00, 0x80, 0xff, 0x00),
                Rgba::new(0x40, 0x80, 0xc0, 0xff),
            ),
            (
                Rgba::new(0x00, 0x80, 0xff, 0x80),
                Rgba::new(0x1f, 0x7f, 0xde, 0xff),
            ),
            (
                Rgba::new(0x00, 0x80, 0xff, 0xff),
                Rgba::new(0x00, 0x80, 0xff, 0xff),
            ),
        ] {
            let bg = &mut Rgba::new(0x40, 0x80, 0xc0, 0xff);

            Framebuffer::blend_alpha(bg, tc.0);
            assert_eq!(bg.r, tc.1.r);
            assert_eq!(bg.g, tc.1.g);
            assert_eq!(bg.b, tc.1.b);
        }
    }

    #[test]
    fn test_blend_image() {
        let image = Image {
            buffer: &mut [
                Rgba::new(0x80, 0x40, 0x20, 0xff),
                Rgba::new(0x80, 0x40, 0x20, 0x80),
                Rgba::new(0x80, 0x40, 0x20, 0x40),
                Rgba::new(0x80, 0x40, 0x20, 0x00),
            ],
            width: 2,
            height: 2,
        };

        let mut fb = Framebuffer::new(3, 3);
        fb.blend_image(&image, &Rect::new(0, 0, 3, 3), &Coord::new(1, 1));

        assert_eq!(
            fb.fb888,
            &[
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0x80, 0x40, 0x20, 255),
                Rgba::new(0x40, 0x20, 0x10, 255),
                Rgba::new(0, 0, 0, 255),
                Rgba::new(0x20, 0x10, 0x08, 255),
                Rgba::new(0x00, 0x00, 0x00, 255),
            ]
        );
    }
}
