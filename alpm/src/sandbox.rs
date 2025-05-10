use alpm_sys::*;
use std::ffi::CString;

use crate::{Alpm, Result};

impl Alpm {
    #[cfg(not(feature = "git"))]
    pub fn sandbox_setup_child<S: Into<Vec<u8>>>(&mut self, user: S, path: S) -> Result<()> {
        let user = CString::new(user).unwrap();
        let path = CString::new(path).unwrap();
        let ret = unsafe { alpm_sandbox_setup_child(self.as_ptr(), user.as_ptr(), path.as_ptr()) };
        self.check_ret(ret)
    }

    #[cfg(feature = "git")]
    pub fn sandbox_setup_child<S: Into<Vec<u8>>>(
        &mut self,
        user: S,
        path: S,
        restrict_syscalls: bool,
    ) -> Result<()> {
        let user = CString::new(user).unwrap();
        let path = CString::new(path).unwrap();
        let ret = unsafe {
            alpm_sandbox_setup_child(
                self.as_ptr(),
                user.as_ptr(),
                path.as_ptr(),
                restrict_syscalls,
            )
        };
        self.check_ret(ret)
    }

    #[cfg(feature = "git")]
    #[doc(alias = "disable_sandbox")]
    pub fn sandbox_disabled(&self) -> bool {
        unsafe { alpm_option_get_disable_sandbox(self.as_ptr()) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox() {
        let mut handle = Alpm::new("/", "tests/db/").unwrap();
        if cfg!(feature = "git") {
            assert_eq!(handle.sandbox_user(), Some("root"));
        } else {
            assert_eq!(handle.sandbox_user(), None);
        }
        handle.set_sandbox_user(Some("foo")).unwrap();
        assert_eq!(handle.sandbox_user(), Some("foo"));
        handle.set_sandbox_user(Option::<&str>::None).unwrap();
        assert_eq!(handle.sandbox_user(), None);
    }
}
