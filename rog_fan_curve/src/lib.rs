//! The [acpi_call](https://github.com/mkottman/acpi_call) kernel module is needed for this crate to interacti with acpi.
//!
//! # Example
//!
//! ```no_run
//! # use rog_fan_curve::{
//! #     Curve,
//! #     Board,
//! #     Fan,
//! #     CurveError,
//! # };
//! # fn main() -> Result<(), CurveError> {
//! let mut curve = Curve::new();
//! 
//! curve.set_point(0,  30,   0);
//! curve.set_point(1,  40,   1);
//! curve.set_point(2,  50,   4);
//! curve.set_point(3,  60,   4);
//! curve.set_point(4,  70,  13);
//! curve.set_point(5,  80,  40);
//! curve.set_point(6,  90, 100);
//! curve.set_point(7, 100, 100);
//! 
//! let board = Board::from_name("GA401IV").unwrap();
//! 
//! curve.apply(board, Fan::Cpu)?;
//! curve.apply(board, Fan::Gpu)?;
//! 
//! # Ok(())
//! # }
//! ```
//!
//! # Fan speeds and temperatures
//!
//! Temperatures are in degrees celcius.
//!
//! Fan speeds are roughly a percentage fan speed. The scale is non linear and values
//! over 100 seem to result in slightly higher fan speeds. A value of 0 will turn the fan off.
//!
//! A temperature, speed pair indicates fan speed over a certain temerature,
//! e.g. 40c:10% means the fan will run at 10% speed when the temperature is over 40C.
//!
//! # Config string format
//!
//! Config strings follow the format
//! ```text
//! <t>c:<s>%,<t>c:<s>%,<t>c:<s>%,<t>c:<s>%,<t>c:<s>%,<t>c:<s>%,<t>c:<s>%,<t>c:<s>%
//! ```
//! where t is temperature and s is fan speed.
//!
//! Curves must have exactly 8 pairs. This format should match the one used by
//! [atrofac](https://github.com/cronosun/atrofac).
//!
//! #### Example
//!
//! ```text
//! 30c:0%,40c:5%,50c:10%,60c:20%,70c:35%,80c:55%,90c:65%,100c:65%`
//! ```
//!
//! # Serde
//!
//! `Curve` implements Serialize and Deserialize to and from the config string format.
//!
//! #### Example
//!
//! In `Cargo.toml`
//! ```toml
//! rog_fan_curve = { version = "*", features = ["serde"] }
//! ```
//!
//! ```
//! # use rog_fan_curve::Curve;
//! # fn main() -> serde_json::Result<()> {
//! # #[cfg(feature = "serde")] {
//! let json = "\"30c:0%,40c:5%,50c:10%,60c:20%,70c:35%,80c:55%,90c:65%,100c:75%\"";
//!
//! let curve: Curve = serde_json::from_str(json)?;
//!
//! let new_json = serde_json::to_string(&curve)?;
//! assert_eq!(json, new_json);
//! # }
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "serde")]
mod serde_impl;

use std::io::prelude::*;
use std::fmt;
use std::fs::OpenOptions;

#[derive(Debug)]
pub enum CurveError {
    Acpi(String),
    InvalidFan(Fan),
    Io(std::io::Error),
}

impl fmt::Display for CurveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Acpi(acpi_response) => {
                write!(f, "acpi_call returned an error `{}`", acpi_response)
            }
            Self::InvalidFan(fan) => {
                write!(f, "Fan `{:?}` not supported on this board.", fan)
            }
            Self::Io(err) => {
                write!(f, "failed to write to file {}", err)
            }
        }
    }
}

impl std::error::Error for CurveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => {
                Some(err)
            }
            _ => None
        }
    }
}

impl From<std::io::Error> for CurveError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug, PartialEq)]
pub enum UnsafeCurveError {
    TempOutOfRange(u8),
    SpeedTooLow(u8),
}

#[derive(Debug, Clone)]
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
    
    /// Create a `Curve` from a config string
    ///
    /// See the crate level documentation for information about the config string format.
    ///
    /// Err results will contain a human readable string describing why the config string isn't valid.
    pub fn from_config_str(config_str: &str) -> Result<Self, String> {
        let config_str = config_str.trim().trim_end_matches(",");
        let pairs = config_str.split(",");
        
        let mut curve = Curve::new();
        let mut pair_i = 0;
        for pair in pairs {
            if pair_i >= 8 {
                return Err("Too many pairs.".into())
            }
            
            let pair_str = pair.trim();
            let mut pair = pair_str.split(":");
            let temp = pair.next()
                .ok_or_else(|| format!("Invalid format: {}", pair_str))?;
            let speed = pair.next()
                .ok_or_else(|| format!("Invalid format: {}", pair_str))?;
            
            // let temp = temp.strip_suffix("c").ok_or_else(|| "Tempruature must have a c".into())?;
            if !temp.ends_with("c") {
                return Err("Tempruature must have a c".into())
            }
            let temp = &temp[..temp.len()-1];
            // let speed = speed.strip_suffix("%").ok_or_else(|| "Speed must have a %".into())?;
            if !speed.ends_with("%") {
                return Err("Speed must have a %".into())
            }
            let speed = &speed[..speed.len()-1];
            
            let temp = temp.parse().map_err(|_| format!("Invalid number: {}", temp))?;
            let speed = speed.parse().map_err(|_| format!("Invalid number: {}", speed))?;
            
            curve.set_point(pair_i, temp, speed);
            
            pair_i += 1;
        }
        
        if pair_i < 7 {
            return Err("Too few pairs.".into())
        }
        
        Ok(curve)
    }
    
    /// Create a config string for a `Curve`
    ///
    /// See the crate level documentation for information about the config string format.
    pub fn as_config_string(&self) -> String {
        let mut out = String::new();
        
        for i in 0..8 {
            let temp = self.curve[i];
            let speed = self.curve[i+8];
            out.push_str(&format!("{}c:{}%,", temp, speed));
        }
        out.pop();
        
        out
    }
    
    /// Checks if the curve is "safe"
    ///
    /// This checks if the curve falls within some arbitrary limitations.
    /// The limitations should match the ones used by [atrofac](https://github.com/cronosun/atrofac/blob/master/ADVANCED.md#limits)
    /// and armoury crate.
    pub fn check_safety(&self, fan: Fan) -> Result<(), UnsafeCurveError> {
        let mut last_speed = 0;
        for i in 0..8 {
            let temp = self.curve[i];
            let speed = self.curve[i+8];
            
            let i = i as u8;
            
            let min_temp = (i * 10) + 30;
            let max_temp = (i * 10) + 39;
            
            let min_speed = match i {
                0 | 1 | 2 | 3 => 0,
                4 => match fan {
                    Fan::Cpu => 31,
                    Fan::Gpu => 34,
                },
                5 => match fan {
                    Fan::Cpu => 49,
                    Fan::Gpu => 51,
                },
                6 | 7 => match fan {
                    Fan::Cpu => 56,
                    Fan::Gpu => 61,
                },
                _ => unreachable!("Invalid fan index"),
            };
            
            if temp < min_temp || temp > max_temp {
                return Err(UnsafeCurveError::TempOutOfRange(i))
            }
            
            if speed < min_speed || speed < last_speed {
                return Err(UnsafeCurveError::SpeedTooLow(i))
            }
            
            last_speed = speed;
        }
        
        Ok(())
    }
    
    /// Sets a point on the fan curve
    ///
    /// # Arguments
    ///
    /// * `n` - The point on the fan curve to change, must be between 0 and 7 inclusive
    /// * `temp` - Degrees celcius
    /// * `speed` - Fan speed (see crate level documentation)
    ///
    /// # Panics
    ///
    /// Panics if n isn't in [0..=7]
    pub fn set_point(&mut self, n: u8, temp: u8, speed: u8) {
        assert!(n <= 7); // must be >= 0 due to type
        let n = n as usize;
        self.curve[n] = temp;
        self.curve[n+8] = speed;
    }
    
    /// Applies the fan curve via acpi_call
    pub fn apply(&self, board: Board, fan: Fan) -> Result<(), CurveError> {
        assert_eq!(board, Board::Ga401);
        let fan_addr = fan.address().ok_or(CurveError::InvalidFan(fan))?;
        let command = make_command(self, fan_addr);
        // dbg!(&command);
        acpi_call(&command)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Board {
    Ga401,
}

impl Board {
    /// Gets the board name from the kernel using `/sys/class/dmi/id/board_name`
    pub fn from_board_name() -> Option<Self> {
        let name = std::fs::read_to_string("/sys/class/dmi/id/board_name").ok()?;
        Self::from_name(name.trim())
    }
    
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "GA401IV" => Some(Board::Ga401),
            _ => None,
        }
    }
}

/// A fan, some boards may not have all fans.
#[derive(Debug, Clone, Copy, PartialEq)]
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
    
    #[test]
    fn test_parse() {
        let config_str = "30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%";
        
        let curve = Curve::from_config_str(config_str).unwrap();
        
        assert_eq!(&curve.curve, &[
            30, 49, 59, 69,
            79, 89, 99, 109,
            1, 2, 3, 4,
            31, 49, 56, 58,
        ]);
    }
    
    #[test]
    fn as_config_str() {
        let config_str = "30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%";
        let curve = Curve::from_config_str(config_str).unwrap();
        
        let out_config_str = curve.as_config_string();
        
        assert_eq!(&out_config_str, "30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%");
    }
    
    #[test]
    fn check_safety() {
        let config_str = "30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%";
        let curve = Curve::from_config_str(config_str).unwrap();
        assert_eq!(curve.check_safety(Fan::Cpu), Ok(()));
        
        let config_str = "41c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%";
        let curve = Curve::from_config_str(config_str).unwrap();
        assert_eq!(curve.check_safety(Fan::Cpu), Err(UnsafeCurveError::TempOutOfRange(0)));
        
        let config_str = "30c:1%,49c:2%,59c:3%,69c:4%,79c:30%,89c:49%,99c:56%,109c:58%";
        let curve = Curve::from_config_str(config_str).unwrap();
        assert_eq!(curve.check_safety(Fan::Cpu), Err(UnsafeCurveError::SpeedTooLow(4)));
    }
}
