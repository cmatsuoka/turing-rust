// SPDX-License-Identifier: GPL-3.0-or-later

pub mod cpu;
pub mod framebuffer;
pub mod meter;
pub mod render;
pub mod scheduler;
pub mod screen;
pub mod themes;

use std::error::Error;

type Res<T> = Result<T, Box<dyn Error>>;
