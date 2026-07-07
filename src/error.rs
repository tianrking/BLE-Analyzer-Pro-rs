use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Usb(rusb::Error),
    Io(io::Error),
    InvalidConfig(String),
    NoDevices,
    OpenFailed(String),
}

impl From<rusb::Error> for Error {
    fn from(value: rusb::Error) -> Self {
        Self::Usb(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Usb(err) => write!(f, "USB error: {err}"),
            Error::Io(err) => write!(f, "I/O error: {err}"),
            Error::InvalidConfig(msg) => write!(f, "invalid config: {msg}"),
            Error::NoDevices => write!(f, "no WCH BLE Analyzer MCU devices found"),
            Error::OpenFailed(msg) => write!(f, "could not open any MCU device: {msg}"),
        }
    }
}

impl std::error::Error for Error {}
