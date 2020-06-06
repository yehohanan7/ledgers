use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    SegmentFull(u64),
}

impl Error {
    pub fn is_segment_full(&self) -> bool {
        match self {
            Error::SegmentFull(_) => true,
            _ => false,
        }
    }
}

impl std::convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IOError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(e) => write!(f, "io error {}", e),
            Error::SegmentFull(id) => write!(f, "segment {} is full", id),
        }
    }
}
