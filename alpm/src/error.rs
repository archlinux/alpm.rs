use crate::Alpm;

use std::error;
use std::ffi::{CStr, NulError};
use std::fmt;
use std::str::Utf8Error;

use alpm_sys::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct AlpmError {
    pub(crate) code: alpm_errno_t,
}

impl Alpm {
    pub fn last_error(&self) -> Error {
        unsafe { alpm_errno(self.handle).into() }
    }
}

impl From<alpm_errno_t> for AlpmError {
    fn from(code: alpm_errno_t) -> AlpmError {
        AlpmError { code }
    }
}

impl fmt::Display for AlpmError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe { CStr::from_ptr(alpm_strerror(self.code)) };
        fmt.write_str(&s.to_string_lossy())
    }
}

impl AlpmError {
    pub fn code(&self) -> alpm_errno_t {
        self.code
    }
}

impl error::Error for AlpmError {}

#[derive(Debug)]
pub enum Error {
    Alpm(AlpmError),
    Nul(NulError),
    Utf8(Utf8Error),
}

impl From<alpm_errno_t> for Error {
    fn from(code: alpm_errno_t) -> Error {
        Error::Alpm(AlpmError { code })
    }
}

impl From<AlpmError> for Error {
    fn from(e: AlpmError) -> Error {
        Error::Alpm(e)
    }
}

impl From<NulError> for Error {
    fn from(e: NulError) -> Error {
        Error::Nul(e)
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Error {
        Error::Utf8(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Alpm(e) => e.fmt(fmt),
            Error::Nul(e) => e.fmt(fmt),
            Error::Utf8(e) => e.fmt(fmt),
        }
    }
}

impl error::Error for Error {}

#[cfg(test)]
mod tests {
    use crate::Alpm;

    #[test]
    fn display() {
        let handle = Alpm::new("/", "tests/db").unwrap();

        println!("{}", handle.last_error());
    }
}
