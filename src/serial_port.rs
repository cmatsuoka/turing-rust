// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Read;
use std::io::Write;
use std::time::Duration;

use crate::Res;
use crate::ScreenPort;

pub struct SerialPort {
    port: Box<dyn serialport::SerialPort>,
}

impl SerialPort {
    pub fn new(path: &str, baud_rate: u32) -> Res<Self> {
        Ok(Self {
            port: serialport::new(path, baud_rate)
                .timeout(Duration::from_millis(1000))
                .open()?,
        })
    }
}

impl Read for SerialPort {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.port.read(buf)
    }
}

impl Write for SerialPort {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.port.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.port.flush()
    }
}

impl ScreenPort for SerialPort {
    fn get_buf(&self) -> Vec<u8> {
        Vec::<u8>::new()
    }
}

pub fn detect(ser: &str) -> Res<String> {
    for p in serialport::available_ports()? {
        match p.port_type {
            serialport::SerialPortType::UsbPort(info) => {
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
