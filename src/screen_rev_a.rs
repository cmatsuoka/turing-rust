// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Read, Write};

use crate::serial_port;
use crate::{Coord, Image, Rect};
use crate::{Orientation, Res, Screen, ScreenPort};

// Constants and protocol definitions from
// https://github.com/mathoudebine/turing-smart-screen-python

const WIDTH: usize = 320;
const HEIGHT: usize = 480;

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
    port: Box<dyn ScreenPort>,
    orientation: Orientation,
    fb565_raw: Vec<u8>,
}

impl ScreenRevA {
    pub fn new(portname: &str) -> Res<Self> {
        let name = match portname {
            "AUTO" => serial_port::detect("USB35INCHIPSV2")?,
            name => name.to_string(),
        };
        log::debug!("create screen rev A on {}", name);

        Ok(Self {
            port: Box::new(serial_port::SerialPort::new(&name, 115_200)?),
            orientation: Orientation::Portrait,
            fb565_raw: vec![0u8; 2 * WIDTH * HEIGHT],
        })
    }

    // RGB565 bit packing:
    // [rrrr rggg] [gggb bbbb]  =(LE)=>  [gggb bbbb] [rrrr rggg]
    fn downmix(&mut self, image: &Image, rect: &Rect, pos: &Coord) {
        let (width, _) = self.screen_size(); // screen width in pixels
        let mut ofs888 = rect.y * image.width + rect.x; // image vector offset in pixels
        let mut ofs565 = 2 * (pos.y * width + pos.x); // fb565 vector offset in bytes
        let stride = width * 2; // width of rgb565 framebuffer in bytes

        for _ in 0..rect.h {
            // line start y coordinate
            let mut src = ofs888;
            let mut dest = ofs565;
            for _ in 0..rect.w {
                let p = image.buffer[src];
                src += 1;
                self.fb565_raw[dest] = ((p.g & 0x1c) << 3) | (p.b >> 3);
                self.fb565_raw[dest + 1] = (p.r & 0xf8) | (p.g >> 5);
                dest += 2;
            }
            ofs888 += image.width;
            ofs565 += stride;
        }
    }
}

impl Screen for ScreenRevA {
    fn screen_size(&self) -> (usize, usize) {
        match self.orientation {
            Orientation::Portrait | Orientation::ReversePortrait => (WIDTH, HEIGHT),
            Orientation::Landscape | Orientation::ReverseLandscape => (HEIGHT, WIDTH),
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

    /// Send an image to the screen.
    ///
    /// Display a cropped portion of the image on the turing screen.
    ///
    /// * `image`: the RGBA image.
    /// * `crop`: the area of the image to display.
    /// * `pos`: the screen coordinates to show the cropped area.
    fn display_image(&mut self, image: &Image, crop: &Rect, pos: &Coord) -> Res<()> {
        log::debug!("display image {} {}", crop, pos);
        let (width, height) = self.screen_size(); // size of screen in pixels
        let r = crop.clip(width - pos.x, height - pos.y);

        if r.w == 0 || r.h == 0 {
            return Ok(());
        }

        self.downmix(image, &r, pos);
        self.write(cmd!(
            Command::DisplayBitmap,
            pos.x,
            pos.y,
            pos.x + r.w - 1,
            pos.y + r.h - 1
        ))?;

        let stride = 2 * width; // width of screen in bytes
        let mut start = pos.y * stride + (2 * pos.x); // line start offset in bytes
        let mut end = start + 2 * r.w; // line end offset in bytes
        for _ in 0..r.h {
            self.write(&self.fb565_raw[start..end].to_owned())?;
            start += stride;
            end += stride;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    use crate::Rgba;

    type FakePort = Cursor<Vec<u8>>;

    impl ScreenPort for FakePort {
        fn get_buf(&self) -> Vec<u8> {
            self.get_ref().to_vec()
        }
    }

    fn fake_screen(port: FakePort) -> ScreenRevA {
        let scr = ScreenRevA {
            port: Box::new(port),
            fb565_raw: Vec::<u8>::new(),
            orientation: Orientation::Portrait,
        };
        return scr;
    }
    #[test]
    fn test_screen_size() -> Res<()> {
        let fake_port = FakePort::new(Vec::new());
        let scr = fake_screen(fake_port);
        let (w, h) = scr.screen_size();
        assert_eq!(w, WIDTH);
        assert_eq!(h, HEIGHT);
        Ok(())
    }

    #[test]
    fn test_write() -> Res<()> {
        let fake_port = FakePort::new(Vec::new());
        let mut scr = fake_screen(fake_port);
        let res = scr.write(&[1, 2, 3, 4, 5])?;
        assert_eq!(res, 5);
        assert_eq!(scr.port.get_buf(), vec![1, 2, 3, 4, 5]);
        Ok(())
    }

    #[test]
    fn test_read() -> Res<()> {
        let fake_port = FakePort::new(vec![1u8, 2, 3, 4, 5]);
        let mut scr = fake_screen(fake_port);
        let buf = scr.read(5)?;
        assert_eq!(buf, &[1, 2, 3, 4, 5]);
        Ok(())
    }

    #[test]
    fn test_init() -> Res<()> {
        let fake_port = FakePort::new(vec![0u8, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1]);
        let mut scr = ScreenRevA {
            port: Box::new(fake_port),
            fb565_raw: Vec::<u8>::new(),
            orientation: Orientation::Portrait,
        };
        assert!(scr.init().is_ok());
        Ok(())
    }

    #[test]
    fn test_init_fail() -> Res<()> {
        let fake_port = FakePort::new(vec![0u8, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2]);
        let mut scr = fake_screen(fake_port);
        assert!(scr.init().is_err());
        Ok(())
    }

    #[test]
    fn test_clear() -> Res<()> {
        let fake_port = FakePort::new(Vec::new());
        let mut scr = fake_screen(fake_port);
        scr.clear()?;
        assert_eq!(
            scr.port.get_buf(),
            vec![
                0, 0, 0, 0, 0, 121, // Command::SetOrientation
                100, // Orientation::Portrait
                1, 64, // Width MSB:LSB (0x0140 == 320)
                1, 224, // Height MSB:LSB (0x01e0 == 480)
                0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 102 // Command::Clear
            ]
        );
        Ok(())
    }

    #[test]
    fn test_screen_on() -> Res<()> {
        let fake_port = FakePort::new(Vec::new());
        let mut scr = fake_screen(fake_port);
        scr.screen_on()?;
        assert_eq!(scr.port.get_buf(), vec![0, 0, 0, 0, 0, 109]);
        Ok(())
    }

    #[test]
    fn test_screen_off() -> Res<()> {
        let fake_port = FakePort::new(Vec::new());
        let mut scr = fake_screen(fake_port);
        scr.screen_off()?;
        assert_eq!(scr.port.get_buf(), vec![0, 0, 0, 0, 0, 108]);
        Ok(())
    }

    #[test]
    fn test_set_brightness() -> Res<()> {
        let fake_port = FakePort::new(Vec::new());
        let mut scr = fake_screen(fake_port);
        scr.set_brightness(0x55)?;
        assert_eq!(scr.port.get_buf(), vec![0xaa, 0, 0, 0, 0, 110]);
        Ok(())
    }

    #[test]
    fn test_downmix() -> Res<()> {
        let fake_port = FakePort::new(Vec::<u8>::new());
        let mut scr = fake_screen(fake_port);
        scr.fb565_raw = vec![0u8; 2 * 320 * 20]; // framebuffer is 320x20

        // image is 20x10
        let mut rgba = vec![Rgba::new(0x80, 0x80, 0x80, 0x00); 20 * 10];

        // 2x2 area to be converted
        rgba[21] = Rgba::new(0xff, 0x00, 0xff, 0x1f);
        rgba[22] = Rgba::new(0x11, 0x22, 0x44, 0x88);
        rgba[41] = Rgba::new(0x00, 0xff, 0x00, 0xff);
        rgba[42] = Rgba::new(0x55, 0xaa, 0xff, 0x00);

        let image = Image {
            buffer: rgba,
            width: 20,
            height: 10,
        };

        // rgb565 data contains only the converted area
        let r = Rect::new(1, 1, 2, 2);
        scr.downmix(&image, &r, &Coord::new(1, 1));

        let mut expected = vec![0u8; 2 * 320 * 20];
        expected[321 * 2 + 0] = 0b00011111;
        expected[321 * 2 + 1] = 0b11111000;
        expected[322 * 2 + 0] = 0b00001000;
        expected[322 * 2 + 1] = 0b00010001;
        expected[641 * 2 + 0] = 0b11100000;
        expected[641 * 2 + 1] = 0b00000111;
        expected[642 * 2 + 0] = 0b01011111;
        expected[642 * 2 + 1] = 0b01010101;

        assert_eq!(scr.fb565_raw, expected);

        Ok(())
    }

    #[test]
    fn test_display_image() -> Res<()> {
        let fake_port = FakePort::new(Vec::<u8>::new());
        let mut scr = fake_screen(fake_port);
        scr.fb565_raw = vec![0u8; 2 * 320 * 2];
        let image = Image {
            buffer: vec![Rgba::new(4, 4, 4, 0); 320 * 2],
            width: 320,
            height: 2,
        };
        scr.display_image(&image, &Rect::new(1, 1, 4, 1), &Coord::new(1, 1))?;
        assert_eq!(
            scr.port.get_buf(),
            vec![0x00, 0x40, 0x10, 0x10, 0x01, 197, 32, 0, 32, 0, 32, 0, 32, 0]
        );
        Ok(())
    }

    #[test]
    fn test_display_image_offscreen() -> Res<()> {
        let fake_port = FakePort::new(Vec::<u8>::new());
        let mut scr = fake_screen(fake_port);
        let image = Image {
            buffer: vec![Rgba::new(4, 4, 4, 0); 320 * 2],
            width: 320,
            height: 2,
        };
        scr.display_image(&image, &Rect::new(10, 20, 0, 0), &Coord::new(0, 0))?;
        assert_eq!(scr.port.get_buf(), vec![]);
        Ok(())
    }
}
