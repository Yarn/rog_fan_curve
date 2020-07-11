//! The [acpi_call](https://github.com/mkottman/acpi_call) kernel module is needed for this crate to interacti with acpi.
//!
//! ```no_run
//! # use rog_fan_curve::{
//! #     Curve,
//! #     Board,
//! #     Fan,
//! #     CurveError,
//! # };
//! # fn foo() -> Result<(), CurveError> {
//! let mut curve = Curve::new();
//! 
//! curve.set_point(0, 0x1e, 0x00);
//! curve.set_point(1, 0x2d, 0x01);
//! curve.set_point(2, 0x32, 0x04);
//! curve.set_point(3, 0x3c, 0x04);
//! curve.set_point(4, 0x46, 0x13);
//! curve.set_point(5, 0x50, 0x40);
//! curve.set_point(6, 0x5a, 0x64);
//! curve.set_point(7, 0x64, 0x64);
//! 
//! let board = Board::from_name("GA401IV").unwrap();
//! 
//! curve.apply(board, Fan::Cpu)?;
//! curve.apply(board, Fan::Gpu)?;
//! 
//! # Ok(())
//! # }
//! ```

use std::io::prelude::*;
use std::fs::OpenOptions;

#[derive(Debug)]
pub enum CurveError {
    Acpi(String),
    InvalidFan,
    Io(std::io::Error),
}

impl From<std::io::Error> for CurveError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug)]
pub struct Curve {
    curve: [u8; 16],
}

impl Curve {
    pub fn new() -> Self {
        Self {
            curve: [
                0x1e, 0x2d, 0x32, 0x3c,
                0x46, 0x50, 0x5a, 0x64,
                0x12, 0x12, 0x12, 0x20,
                0x30, 0x40, 0x64, 0x64,
            ],
        }
    }
    
    /// Sets a point on the fan curve
    ///
    /// # Arguments
    ///
    /// * `n` - The point on the fan curve to change, must be between 0 and 7 inclusive
    /// * `temp` - Degrees celcius
    /// * `speed` -> Percent? fan speed
    pub fn set_point(&mut self, n: u8, temp: u8, speed: u8) {
        assert!(n <= 7); // must be >= 0 due to type
        let n = n as usize;
        self.curve[n] = temp;
        self.curve[n+8] = speed;
    }
    
    /// Applies the fan curve via acpi_call
    pub fn apply(&self, board: Board, fan: Fan) -> Result<(), CurveError> {
        assert_eq!(board, Board::Ga401iv);
        let fan_addr = fan.address().ok_or(CurveError::InvalidFan)?;
        let command = make_command(self, fan_addr);
        // dbg!(&command);
        acpi_call(&command)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Board {
    Ga401iv,
}

impl Board {
    /// Gets the board name from the kernel using `/sys/class/dmi/id/board_name`
    pub fn from_board_name() -> Option<Self> {
        let name = std::fs::read_to_string("/sys/class/dmi/id/board_name").ok()?;
        Self::from_name(name.trim())
    }
    
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "GA401IV" => Some(Board::Ga401iv),
            _ => None,
        }
    }
}

/// A fan, some boards may not have all fans.
#[derive(Debug, Clone, Copy)]
pub enum Fan {
    Cpu,
    Gpu,
}

impl Fan {
    fn address(&self) -> Option<u32> {
        match self {
            Fan::Cpu => Some(0x40),
            Fan::Gpu => Some(0x44),
        }
    }
}

fn make_command(curve: &Curve, fan: u32) -> String {
    let mut command = "\\_SB.PCI0.SBRG.EC0.SUFC ".to_string();
    
    for param_bytes in curve.curve.chunks_exact(4) {
        let mut param = param_bytes[0] as u32;
        param += (param_bytes[1] as u32) << 0x08;
        param += (param_bytes[2] as u32) << 0x10;
        param += (param_bytes[3] as u32) << 0x18;
        command.push_str(&format!("{:0>#010x} ", param));
    }
    
    command.push_str(&format!("{:#x}", fan));
    
    command
}

fn acpi_call(command: &str) -> Result<(), CurveError> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/proc/acpi/call")?;
    
    file.write_all(command.as_bytes())?;
    
    file.seek(std::io::SeekFrom::Start(0))?;
    
    let mut out = String::new();
    file.read_to_string(&mut out)?;
    
    // dbg!(&out);
    
    if out.starts_with("Error:") {
        return Err(CurveError::Acpi(out))
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_make_command() {
        let curve = Curve {
            curve: [
                0x1e, 0x2d, 0x32, 0x3c,
                0x46, 0x50, 0x5a, 0x64,
                0x00, 0x01, 0x04, 0x04,
                0x13, 0x40, 0x64, 0x64,
            ],
        };
        
        let command = make_command(&curve, 0x40);
        dbg!(&command);
        
        assert_eq!(&command, "\\_SB.PCI0.SBRG.EC0.SUFC 0x3c322d1e 0x645a5046 0x04040100 0x64644013 0x40")
    }
    
    #[test]
    fn set_point() {
        let mut curve = Curve {
            curve: [
                0x0a, 0x0a, 0x0a, 0x0a,
                0x0a, 0x0a, 0x0a, 0x0a,
                0x0a, 0x0a, 0x0a, 0x0a,
                0x0a, 0x0a, 0x0a, 0x0a,
            ],
        };
        
        curve.set_point(0, 0, 1);
        curve.set_point(1, 2, 3);
        curve.set_point(2, 4, 5);
        curve.set_point(3, 6, 7);
        curve.set_point(4, 8, 9);
        curve.set_point(5, 10, 11);
        curve.set_point(6, 12, 13);
        curve.set_point(7, 14, 15);
        
        assert_eq!(&curve.curve, &[
             0,  2,  4,  6,
             8, 10, 12, 14,
             1,  3,  5,  7,
             9, 11, 13, 15,
        ]);
    }
}
