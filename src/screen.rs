use serialport::{SerialPort, SerialPortType};
use std::error::Error;
use std::time::Duration;

// Constants and protocol definitions from
// https://github.com/mathoudebine/turing-smart-screen-python

enum Orientation {
    Portrait = 0,
    _Landscape = 2,
    _ReversePortrait = 1,
    _ReverseLandscape = 3,
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
    _DisplayBitmap = 197,  // Displays an image on the screen
}

// Subrevisions
const USBMONITOR35: &[u8] = &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01];

trait ScreenOperations {
    fn write(&mut self, data: Vec<u8>) -> Result<usize, Box<dyn Error>>;
    fn read(&mut self, n: usize) -> Result<Vec<u8>, Box<dyn Error>>;
    fn init(&mut self) -> Result<(), Box<dyn Error>>;
    fn clear(&mut self) -> Result<(), Box<dyn Error>>;
    fn screen_on(&mut self) -> Result<(), Box<dyn Error>>;
    fn screen_off(&mut self) -> Result<(), Box<dyn Error>>;
    fn set_orientation(&mut self, o: Orientation) -> Result<(), Box<dyn Error>>;
    fn set_brightness(&mut self, level: u8) -> Result<(), Box<dyn Error>>;
    fn draw_bitmap(
        &mut self,
        path: String,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct ScreenRevA {
    port: Box<dyn SerialPort>,
    orientation: Orientation,
}

impl ScreenRevA {
    pub fn new(portname: &str) -> Result<Self, Box<dyn Error>> {
        let name = match portname {
            "AUTO" => auto_detect_port("USB35INCHIPSV2")?,
            name => name.to_string(),
        };

        let port = serialport::new(name, 115_200)
            .timeout(Duration::from_millis(1000))
            .open()?;

        let orientation = Orientation::Portrait;

        Ok(Self {
            port,
            orientation,
        })
    }
}

impl ScreenOperations for ScreenRevA {
    fn write(&mut self, data: Vec<u8>) -> Result<usize, Box<dyn Error>> {
        let n = self.port.write(&data)?;
        Ok(n)
    }

    fn read(&mut self, n: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data = vec![0; n];
        self.port.read_exact(&mut data)?;
        Ok(data)
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        let hello = vec![Command::Hello as u8; 6];
        self.write(hello)?;

        let res = self.read(6)?;
        if res != USBMONITOR35 {
            return Err("incompatible screen model".into());
        }

        Ok(())
    }

    fn clear(&mut self) -> Result<(), Box<dyn Error>> {
        self.set_orientation(Orientation::Portrait)?; // Orientation must be PORTRAIT before clearing
        self.write(vec![Command::Clear as u8, 0, 0, 0, 0])?;
        Ok(())
    }

    fn screen_on(&mut self) -> Result<(), Box<dyn Error>> {
        self.write(vec![Command::ScreenOn as u8, 0, 0, 0, 0])?;
        Ok(())
    }

    fn screen_off(&mut self) -> Result<(), Box<dyn Error>> {
        self.write(vec![Command::ScreenOff as u8, 0, 0, 0, 0])?;
        Ok(())
    }

    fn set_orientation(&mut self, o: Orientation) -> Result<(), Box<dyn Error>> {
        self.orientation = o;
        // TODO: implement orientation command
        Ok(())
    }

    fn set_brightness(&mut self, level: u8) -> Result<(), Box<dyn Error>> {
        self.write(vec![Command::SetBrightness as u8, !level, 0, 0, 0])?;
        Ok(())
    }

    fn draw_bitmap(
        &mut self,
        _path: String,
        _x: u32,
        _y: u32,
        _w: u32,
        _h: u32,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

fn auto_detect_port(ser: &str) -> Result<String, Box<dyn Error>> {
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
