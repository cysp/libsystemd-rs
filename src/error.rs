use std;
use libc;


#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Nul(std::ffi::NulError),
}

impl Error {
    pub fn from_negative_errno(errno: libc::c_int) -> Error {
        Error::Io(std::io::Error::from_raw_os_error(-errno))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &Error::Io(ref error) => error.fmt(f),
            &Error::Nul(ref error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(ref error) => error.description(),
            &Error::Nul(ref error) => error.description(),
        }
    }
}


impl From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Error {
        Error::Nul(error)
    }
}
