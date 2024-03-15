// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::min;

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub fn clip(&self, width: usize, height: usize) -> Rect {
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

#[derive(Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "@{},{}", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord() {
        let pos = Coord::new(10, 20);
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
        assert_eq!(format!("{}", pos), "@10,20");
    }

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
