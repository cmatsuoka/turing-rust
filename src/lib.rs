// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::min;
use std::error::Error;

use crate::screen_rev_a::ScreenRevA;

pub mod framebuffer;
pub mod screen_rev_a;
pub mod serial_port;

type Res<T> = Result<T, Box<dyn Error>>;
type Rgba = rgb::RGBA<u8>;

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
    fn draw_bitmap(&mut self, data: &[Rgba], rect: &Rect) -> Res<()>;
}

pub fn new(portname: &str) -> Res<Box<dyn Screen>> {
    Ok(Box::new(ScreenRevA::new(portname)?))
}
