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

        let ret = unsafe { alpm_logaction(self.as_ptr(), p.as_ptr(), s.as_ptr()) };
        self.check_ret(ret)
    }
}

/// Logs a formatted message.
///
/// A wrapper around the [`Alpm::log_action`] function that acts similar to writeln!.
///
/// Unline the [`Alpm::log_action`] function this macro automatically appends a newline to the message.
///
/// The first argument is a handle to alpm.
///
/// The second is the message prefix. This is usually the name of your program doing the logging.
///
/// The third and following arguments are the same as [`format!`]. The third argument must be a string
/// literal.
///
/// Like [Alpm::log_action`] this returns a Result.
///
/// # Examples
///
/// ```
/// # use alpm::{Alpm, log_action};
/// # let handle = Alpm::new("/", "tests/db").unwrap();
/// log_action!(handle, "coolprogram", "starting transaction").unwrap();
/// log_action!(handle, "coolprogram", "installing package {}", "xorg").unwrap();
/// log_action!(handle, "coolprogram", "installing packages {pkg1} {pkg2}", pkg1 = "xorg", pkg2 = "git").unwrap();
/// ```
#[macro_export]
macro_rules! log_action {
    ($handle:tt, $prefix:tt, $($arg:tt)*) => ({
        let mut s = format!($($arg)*);
        s.reserve_exact(1);
        s.push('\n');
        $handle.log_action($prefix, s)
    })
}
