use crate::{Alpm, Error};
use alpm_sys::*;

use std::ffi::CString;

impl Alpm {
    pub fn log_action<S1: Into<Vec<u8>>, S2: Into<Vec<u8>>>(
        &self,
        prefix: S1,
        msg: S2,
    ) -> Result<(), Error> {
        let s = CString::new(msg).unwrap();
        let p = CString::new(prefix).unwrap();

        let ret = unsafe { alpm_logaction(self.handle, p.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }
}

#[macro_export]
macro_rules! log_action {
    ($handle:tt, $prefix:tt, $($arg:tt)*) => ({
        let mut s = format!($($arg)*);
        s.push('\n');
        $handle.log_action($prefix, s)
    })
}
