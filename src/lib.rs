// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::min;
use std::error::Error;
use std::io::Read;
use std::io::Write;

use crate::screen_rev_a::ScreenRevA;

pub mod framebuffer;
pub mod screen_rev_a;
pub mod serial_port;

type Res<T> = Result<T, Box<dyn Error>>;

pub type Rgba = rgb::RGBA<u8>;

#[derive(Debug, Clone)]
pub struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    fn clip(&self, width: usize, height: usize) -> Rect {
        if self.x >= width || self.y >= height {
            Rect::new(self.x, self.y, 0, 0)
        } else {
            let w = min(self.w, width - self.x);
            let h = min(self.h, height - self.y);
            Rect::new(self.x, self.y, w, h)
        }
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "@{},{}+{}x{}", self.x, self.y, self.w, self.h)
    }
}

#[derive(Debug, Clone)]
pub enum Orientation {
    Portrait = 0,
    Landscape = 1,
    ReversePortrait = 2,
    ReverseLandscape = 3,
}

pub trait Screen {
    fn screen_size(&self) -> (usize, usize);
    fn write(&mut self, data: &[u8]) -> Res<usize>;
    fn read(&mut self, n: usize) -> Res<Vec<u8>>;
    fn init(&mut self) -> Res<()>;
    fn clear(&mut self) -> Res<()>;
    fn screen_on(&mut self) -> Res<()>;
    fn screen_off(&mut self) -> Res<()>;
    fn set_orientation(&mut self, o: Orientation) -> Res<()>;
    fn set_brightness(&mut self, level: usize) -> Res<()>;
    fn expose_framebuffer(&mut self, fb888: &[Rgba], rect: &Rect) -> Res<()>;
}

pub fn new(portname: &str) -> Res<Box<dyn Screen>> {
    Ok(Box::new(ScreenRevA::new(portname)?))
}

pub trait ScreenPort: Read + Write {
    fn get_buf(&self) -> Vec<u8>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let r = Rect::new(2, 3, 4, 5);
        assert_eq!(r.x, 2);
        assert_eq!(r.y, 3);
        assert_eq!(r.w, 4);
        assert_eq!(r.h, 5);
        assert_eq!(format!("{}", r), "@2,3+4x5");
    }

    #[test]
    fn test_rect_clip() {
        for tc in vec![
            (Rect::new(5, 10, 15, 20), Rect::new(5, 10, 15, 20)), // fully inside
            (Rect::new(5, 10, 50, 50), Rect::new(5, 10, 20, 25)), // clipped
            (Rect::new(30, 10, 50, 50), Rect::new(30, 10, 0, 0)), // off-screen x
            (Rect::new(5, 40, 50, 50), Rect::new(5, 40, 0, 0)),   // off-screen y
        ] {
            let r = tc.0.clip(25, 35);
            assert_eq!(r.x, tc.1.x);
            assert_eq!(r.y, tc.1.y);
            assert_eq!(r.w, tc.1.w);
            assert_eq!(r.h, tc.1.h);
        }
    }
}
