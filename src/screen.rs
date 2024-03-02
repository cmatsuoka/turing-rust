// SPDX-License-Identifier: GPL-3.0-or-later

use serialport::{SerialPort, SerialPortType};
use std::time::Duration;

use crate::Res;

// Constants and protocol definitions from
// https://github.com/mathoudebine/turing-smart-screen-python

pub enum Orientation {
    Portrait = 0,
    Landscape = 2,
    ReversePortrait = 1,
    ReverseLandscape = 3,
}

enum Command {
    Hello = 69,            // Asks the screen for its model: 3.5", 5" or 7"
    _Reset = 101,          // Resets the display
    Clear = 102,           // Clears the display to a white screen
    _ToBlack = 103,        // Makes the screen go black. NOT TESTED
    ScreenOff = 108,       // Turns the screen off
    ScreenOn = 109,        // Turns the screen on
    SetBrightness = 110,   // Sets the screen brightness
    _SetOrientation = 121, // Sets the screen orientation
    DisplayBitmap = 197,   // Displays an image on the screen
}

// Subrevisions
const USBMONITOR35: &[u8] = &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01];

pub trait Screen {
    fn screen_size(&self) -> (usize, usize);
    fn write(&mut self, data: Vec<u8>) -> Res<usize>;
    fn read(&mut self, n: usize) -> Res<Vec<u8>>;
    fn init(&mut self) -> Res<()>;
    fn clear(&mut self) -> Res<()>;
    fn screen_on(&mut self) -> Res<()>;
    fn screen_off(&mut self) -> Res<()>;
    fn set_orientation(&mut self, o: Orientation) -> Res<()>;
    fn set_brightness(&mut self, level: usize) -> Res<()>;
    fn draw_bitmap(&mut self, data: &[u8], x: usize, y: usize, w: usize, h: usize) -> Res<()>;
}

pub struct ScreenRevA {
    port: Box<dyn SerialPort>,
    orientation: Orientation,
}

impl ScreenRevA {
    pub fn new(portname: &str) -> Res<Self> {
        let name = match portname {
            "AUTO" => auto_detect_port("USB35INCHIPSV2")?,
            name => name.to_string(),
        };
        log::debug!("create screen rev A on {}", name);

        let port = serialport::new(name, 115_200)
            .timeout(Duration::from_millis(1000))
            .open()?;

        let orientation = Orientation::Portrait;

        Ok(Self { port, orientation })
    }

    // Each coordinate has 10 bits
    // [xxxx xxxx] [xxyy yyyy] [yyyy zzzz] [zzzz zzww] [wwww wwww]
    fn send_command(
        &mut self,
        cmd: Command,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
    ) -> Res<()> {
        let buf: [u8; 6] = [
            ((x0 & 0x03ff) >> 2) as u8,
            (((x0 & 0x0003) << 6) | ((y0 & 0x03ff) >> 4)) as u8,
            (((y0 & 0x000f) << 4) | ((x1 & 0x03ff) >> 6)) as u8,
            (((x1 & 0x003f) << 2) | ((y1 & 0x03ff) >> 8)) as u8,
            (y1 & 0x00ff) as u8,
            cmd as u8,
        ];

        self.write(buf.to_vec())?;

        Ok(())
    }
}

impl Screen for ScreenRevA {
    fn screen_size(&self) -> (usize, usize) {
        match self.orientation {
            Orientation::Portrait | Orientation::ReversePortrait => (320, 480),
            Orientation::Landscape | Orientation::ReverseLandscape => (480, 320),
        }
    }

    fn write(&mut self, data: Vec<u8>) -> Res<usize> {
        let n = self.port.write(&data)?;
        Ok(n)
    }

    fn read(&mut self, n: usize) -> Res<Vec<u8>> {
        let mut data = vec![0; n];
        self.port.read_exact(&mut data)?;
        Ok(data)
    }

    fn init(&mut self) -> Res<()> {
        let hello = vec![Command::Hello as u8; 6];
        self.write(hello)?;

        let res = self.read(6)?;
        if res != USBMONITOR35 {
            return Err("incompatible screen model".into());
        }

        Ok(())
    }

    fn clear(&mut self) -> Res<()> {
        self.set_orientation(Orientation::Portrait)?; // Orientation must be PORTRAIT before clearing
        self.send_command(Command::Clear, 0, 0, 0, 0)?;
        Ok(())
    }

    fn screen_on(&mut self) -> Res<()> {
        self.send_command(Command::ScreenOn, 0, 0, 0, 0)?;
        Ok(())
    }

    fn screen_off(&mut self) -> Res<()> {
        self.send_command(Command::ScreenOff, 0, 0, 0, 0)?;
        Ok(())
    }

    fn set_orientation(&mut self, o: Orientation) -> Res<()> {
        self.orientation = o;
        // TODO: implement orientation command
        Ok(())
    }

    fn set_brightness(&mut self, level: usize) -> Res<()> {
        self.send_command(Command::SetBrightness, !level, 0, 0, 0)?;
        Ok(())
    }

    fn draw_bitmap(&mut self, data: &[u8], x: usize, y: usize, w: usize, h: usize) -> Res<()> {
        self.send_command(Command::DisplayBitmap, x, y, x + w - 1, y + h - 1)?;

	if w * h > data.len() {
		return Err("image dimensions larger than image data".into());
	}

        let (mut start, mut end) = (0, 2 * w);
        for _ in 0..h {
            self.write(data[start..end].to_owned())?;
            (start, end) = (end, end + 2 * w);
        }

        Ok(())
    }
}

fn auto_detect_port(ser: &str) -> Res<String> {
    for p in serialport::available_ports()? {
        match p.port_type {
            SerialPortType::UsbPort(info) => {
                let serial = info.serial_number.as_ref().map_or("", String::as_str);
                if serial == ser {
                    return Ok(p.port_name);
                }
            }
            _ => todo!(),
        }
    }
    Err(format!("no serial device matching {}", ser).into())
}
