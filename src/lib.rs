// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;
use std::io::Read;
use std::io::Write;

use crate::screen_rev_a::ScreenRevA;

pub use crate::colors::Rgba;
pub use crate::geometry::{Coord, Rect};
pub use crate::image::{Framebuffer, Image};

pub mod colors;
mod geometry;
mod image;
mod screen_rev_a;
mod serial_port;

type Res<T> = Result<T, Box<dyn Error>>;

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
    fn expose_framebuffer(&mut self, img888: &Image, rect: &Rect, pos: &Coord) -> Res<()>;
}

pub fn new(portname: &str) -> Res<Box<dyn Screen>> {
    Ok(Box::new(ScreenRevA::new(portname)?))
}

pub trait ScreenPort: Read + Write {
    fn get_buf(&self) -> Vec<u8>;
}
