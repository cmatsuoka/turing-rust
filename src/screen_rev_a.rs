// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Read, Write};

use crate::serial_port;
use crate::{Orientation, Res, Screen};

// Constants and protocol definitions from
// https://github.com/mathoudebine/turing-smart-screen-python

enum Command {
    Hello = 69,           // Asks the screen for its model: 3.5", 5" or 7"
    _Reset = 101,         // Resets the display
    Clear = 102,          // Clears the display to a white screen
    _ToBlack = 103,       // Makes the screen go black. NOT TESTED
    ScreenOff = 108,      // Turns the screen off
    ScreenOn = 109,       // Turns the screen on
    SetBrightness = 110,  // Sets the screen brightness
    SetOrientation = 121, // Sets the screen orientation
    DisplayBitmap = 197,  // Displays an image on the screen
}

// Subrevisions
const USBMONITOR35: &[u8] = &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01];

// Macro to prepare the command buffer
macro_rules! cmd {
    // 1) match cmd!(Command::...)
    ($a:expr) => {{
        &[0u8, 0, 0, 0, 0, $a as u8]
    }};
    // 2) match cmd!(Command::..., parameter)
    ($a:expr, $b:expr) => {{
        &[$b as u8, 0, 0, 0, 0, $a as u8]
    }};
    // 3) match cmd!(Command::DisplayBitmap, x0, y0, x1, y1)
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr) => {{
        &[
            (($b & 0x03ff) >> 2) as u8,
            ((($b & 0x0003) << 6) | (($c & 0x03ff) >> 4)) as u8,
            ((($c & 0x000f) << 4) | (($d & 0x03ff) >> 6)) as u8,
            ((($d & 0x003f) << 2) | (($e & 0x03ff) >> 8)) as u8,
            ($e & 0x00ff) as u8,
            $a as u8, // Command::DisplayBitmap
        ]
    }};
    // 4) match cmd!(Command::SetOrientation, o, width, height)
    ($a:expr, $b:expr, $c:expr, $d:expr) => {{
        &[
            0u8,
            0,
            0,
            0,
            0,
            $a as u8,          // Command::SetOrientation
            100u8 + $b,        // orientation
            ($c >> 8) as u8,   // width MSB
            ($c & 0xff) as u8, // width LSB
            ($d >> 8) as u8,   // height MSB
            ($d & 0xff) as u8, // height MSB
            0,
            0,
            0,
            0,
            0,
        ]
    }};
}

fn orientation(o: Orientation) -> u8 {
    match o {
        Orientation::Portrait => 0,
        Orientation::Landscape => 2,
        Orientation::ReversePortrait => 1,
        Orientation::ReverseLandscape => 3,
    }
}

pub struct ScreenRevA {
    port: serial_port::SerialPort,
    orientation: Orientation,
}

impl ScreenRevA {
    pub fn new(portname: &str) -> Res<Self> {
        let name = match portname {
            "AUTO" => serial_port::detect("USB35INCHIPSV2")?,
            name => name.to_string(),
        };
        log::debug!("create screen rev A on {}", name);

        Ok(Self {
            port: serial_port::SerialPort::new(&name, 115_200)?,
            orientation: Orientation::Portrait,
        })
    }
}

impl Screen for ScreenRevA {
    fn screen_size(&self) -> (usize, usize) {
        match self.orientation {
            Orientation::Portrait | Orientation::ReversePortrait => (320, 480),
            Orientation::Landscape | Orientation::ReverseLandscape => (480, 320),
        }
    }
    fn write(&mut self, data: &[u8]) -> Res<usize> {
        let n = self.port.write(data)?;
        Ok(n)
    }

    fn read(&mut self, n: usize) -> Res<Vec<u8>> {
        let mut data = vec![0; n];
        self.port.read_exact(&mut data)?;
        Ok(data)
    }

    fn init(&mut self) -> Res<()> {
        log::debug!("init screen");
        self.write(cmd!(Command::Hello))?;

        let res = self.read(6)?;
        if res != USBMONITOR35 {
            return Err("incompatible screen model".into());
        }

        Ok(())
    }

    fn clear(&mut self) -> Res<()> {
        log::debug!("clear screen");
        self.set_orientation(Orientation::Portrait)?; // Orientation must be PORTRAIT before clearing
        self.write(cmd!(Command::Clear))?;
        Ok(())
    }

    fn screen_on(&mut self) -> Res<()> {
        log::debug!("screen on");
        self.write(cmd!(Command::ScreenOn))?;
        Ok(())
    }

    fn screen_off(&mut self) -> Res<()> {
        log::debug!("screen off");
        self.write(cmd!(Command::ScreenOff))?;
        Ok(())
    }

    fn set_orientation(&mut self, o: Orientation) -> Res<()> {
        log::debug!("set screen orientation to {:?}", o);
        self.orientation = o.clone();
        let (width, height) = self.screen_size();
        self.write(cmd!(Command::SetOrientation, orientation(o), width, height))?;
        Ok(())
    }

    fn set_brightness(&mut self, level: usize) -> Res<()> {
        log::debug!("set screen brightness to {}", level);
        self.write(cmd!(Command::SetBrightness, !level))?;
        Ok(())
    }

    fn draw_bitmap(&mut self, data: &[u8], x: usize, y: usize, w: usize, h: usize) -> Res<()> {
        log::debug!("draw bitmap @{},{}+{}x{}", x, y, w, h);
        if w * h > data.len() {
            return Err("image dimensions larger than image data".into());
        }

        self.write(cmd!(Command::DisplayBitmap, x, y, x + w - 1, y + h - 1))?;

        let (mut start, mut end) = (0, 2 * w);
        for _ in 0..h {
            self.write(&data[start..end])?;
            (start, end) = (end, end + 2 * w);
        }

        Ok(())
    }
}
