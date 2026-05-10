
#[derive(Debug)]
pub enum Error {
    CapacityExceeded,
    InvalidUtf8
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::CapacityExceeded => write!(fmt, "Fixed Capacity exceeded"),
            Error::InvalidUtf8 => write!(fmt, "invalid utf-8 sequence"),
        }
    }
}