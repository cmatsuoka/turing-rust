// SPDX-License-Identifier: GPL-3.0-or-later

pub type Rgba = rgb::RGBA<u8>;

pub const BLACK: Rgba = Rgba::new(0, 0, 0, 255);
pub const WHITE: Rgba = Rgba::new(255, 255, 255, 255);
pub const TRANSPARENT: Rgba = Rgba::new(0, 0, 0, 0);
