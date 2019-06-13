use crate::Alpm;

use std::error;
use std::ffi::CStr;
use std::fmt;

use alpm_sys::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub(crate) code: alpm_errno_t,
}

impl Alpm {
    pub fn last_error(&self) -> Error {
        unsafe { alpm_errno(self.handle).into() }
    }
}

impl Error {
    pub fn ok(&self) -> bool {
        self.code == alpm_errno_t::ALPM_ERR_OK
    }
}

impl From<alpm_errno_t> for Error {
    fn from(code: alpm_errno_t) -> Error {
        Error { code }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe { CStr::from_ptr(alpm_strerror(self.code)) };
        fmt.write_str(&s.to_str().unwrap())
    }
}

impl Error {
    pub fn code(&self) -> alpm_errno_t {
        self.code
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
