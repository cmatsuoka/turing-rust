// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::min;

use crate::colors;
use crate::{Coord, Rect, Res, Rgba, Screen};

/// The Image struct contains the width, height, and pixel data of an
/// RGBA image.
#[derive(Debug, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Rgba>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![colors::BLACK; width * height],
        }
    }

    #[inline]
    pub fn full(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    pub fn copy_image(&mut self, image: &Image, crop: &Rect, dest: &Coord) {
        let crop = crop.clip(
            min(image.width, self.width - dest.x),
            min(image.height, self.height - dest.y),
        );

        for y in 0..crop.h {
            let offset = (dest.y + y) * self.width + crop.x + dest.x;
            let src_offset = (crop.y + y) * image.width + crop.x;
            self.buffer[offset..offset + crop.w]
                .copy_from_slice(&image.buffer[src_offset..src_offset + crop.w]);
        }
    }

    pub fn blend_image(&mut self, image: &Image, crop: &Rect, dest: &Coord) {
        let crop = crop.clip(
            min(image.width, self.width - dest.x),
            min(image.height, self.height - dest.y),
        );

        let mut offset = dest.y * self.width + dest.x;
        let mut src_offset = crop.y * image.width + crop.x;

        for _ in 0..crop.h {
            for x in 0..crop.w {
                let fg = image.buffer[src_offset + x];
                if fg.a > 0 {
                    let bg = &mut self.buffer[offset + x];
                    Self::blend_alpha(bg, fg);
                }
            }
            offset += self.width;
            src_offset += image.width;
        }
    }

    /// Blend the image with a background.
    ///
    /// Alpha blend the cropped portion of the image with the supplied background
    /// at the given position. The resulting image overwrites the cropped area
    /// of the source image.
    ///
    /// * `crop`: the area of the image to blend.
    /// * `pos`: the coordinates inside the background image.
    /// * `background`: the background image.
    pub fn blend_to_background(&mut self, crop: &Rect, pos: &Coord, background: &Image) {
        let mut offset = pos.y * background.width + pos.x;
        let mut src_offset = crop.y * self.width + crop.x;

        for _ in 0..crop.h {
            for x in 0..crop.w {
                let fg = self.buffer[src_offset + x];
                let mut bg = background.buffer[offset + x];
                Self::blend_alpha(&mut bg, fg);
                self.buffer[src_offset + x] = bg;
            }
            offset += background.width;
            src_offset += self.width;
        }
    }

    pub fn render_on(&self, scr: &mut Box<dyn Screen>, crop: &Rect, pos: &Coord) -> Res<()> {
        scr.display_image(self, crop, pos)?;
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
    fn test_full() {
        let image = Image::new(20, 30);
        assert_eq!(image.full(), Rect::new(0, 0, 20, 30));
    }
    #[test]
    fn test_copy_image() {
        let image = Image {
            buffer: vec![
                Rgba::new(1, 1, 1, 1),
                Rgba::new(2, 2, 2, 2),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
            ],
            width: 2,
            height: 2,
        };

        let mut background = Image::new(3, 3);
        background.copy_image(&image, &image.full(), &Coord::new(1, 1));

        assert_eq!(
            background.buffer,
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
            buffer: vec![
                Rgba::new(1, 1, 1, 1),
                Rgba::new(2, 2, 2, 2),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
            ],
            width: 2,
            height: 2,
        };

        let mut background = Image::new(3, 3);
        background.copy_image(&image, &Rect::new(0, 1, 2, 1), &Coord::new(1, 1));

        assert_eq!(
            background.buffer,
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
            buffer: vec![
                Rgba::new(1, 1, 1, 1),
                Rgba::new(2, 2, 2, 2),
                Rgba::new(3, 3, 3, 3),
                Rgba::new(4, 4, 4, 4),
            ],
            width: 2,
            height: 2,
        };

        let mut background = Image::new(3, 3);
        background.copy_image(&image, &Rect::new(0, 0, 2, 2), &Coord::new(2, 2));

        assert_eq!(
            background.buffer,
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

            Image::blend_alpha(bg, tc.0);
            assert_eq!(bg.r, tc.1.r);
            assert_eq!(bg.g, tc.1.g);
            assert_eq!(bg.b, tc.1.b);
        }
    }

    #[test]
    fn test_blend_image() {
        let image = Image {
            buffer: vec![
                Rgba::new(0x80, 0x40, 0x20, 0xff),
                Rgba::new(0x80, 0x40, 0x20, 0x80),
                Rgba::new(0x80, 0x40, 0x20, 0x40),
                Rgba::new(0x80, 0x40, 0x20, 0x00),
            ],
            width: 2,
            height: 2,
        };

        let mut background = Image::new(3, 3);
        background.blend_image(&image, &Rect::new(0, 0, 3, 3), &Coord::new(1, 1));

        assert_eq!(
            background.buffer,
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

    #[test]
    fn test_blend_to_background() {
        let mut image = Image {
            buffer: vec![
                Rgba::new(0x80, 0x40, 0x20, 0xff),
                Rgba::new(0x80, 0x40, 0x20, 0x80),
                Rgba::new(0x80, 0x40, 0x20, 0x40),
                Rgba::new(0x80, 0x40, 0x20, 0x00),
            ],
            width: 2,
            height: 2,
        };

        let background = Image::new(3, 3);
        image.blend_to_background(&Rect::new(0, 0, 3, 3), &Coord::new(1, 1), &background);

        assert_eq!(
            image.buffer,
            &[
                Rgba::new(0x80, 0x40, 0x20, 0xff),
                Rgba::new(0x40, 0x20, 0x10, 0xff),
                Rgba::new(0x20, 0x10, 0x08, 0xff),
                Rgba::new(0x00, 0x00, 0x00, 0xff),
            ]
        );
    }
}
