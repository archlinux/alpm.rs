use crate::utils::*;
use crate::{Alpm, AlpmList, Depend, FreeMethod, Package};

use alpm_sys::alpm_fileconflicttype_t::*;
use alpm_sys::*;

use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::transmute;

#[derive(Debug)]
pub struct Conflict {
    pub(crate) inner: *mut alpm_conflict_t,
    pub(crate) drop: bool,
}

impl Drop for Conflict {
    fn drop(&mut self) {
        if self.drop {
            unsafe { alpm_conflict_free(self.inner) }
        }
    }
}

impl Conflict {
    pub fn package1_hash(&self) -> u64 {
        #[allow(clippy::identity_conversion)]
        unsafe {
            (*self.inner).package1_hash.into()
        }
    }

    pub fn package2_hash(&self) -> u64 {
        #[allow(clippy::identity_conversion)]
        unsafe {
            (*self.inner).package2_hash.into()
        }
    }

    pub fn package1(&self) -> &str {
        unsafe { from_cstr((*self.inner).package1) }
    }

    pub fn package2(&self) -> &str {
        unsafe { from_cstr((*self.inner).package2) }
    }

    pub fn reason(&self) -> Depend {
        Depend {
            inner: unsafe { (*self.inner).reason },
            drop: false,
            phantom: PhantomData,
        }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum FileConflictType {
    Target = ALPM_FILECONFLICT_TARGET as u32,
    Filesystem = ALPM_FILECONFLICT_FILESYSTEM as u32,
}

#[derive(Debug)]
pub struct FileConflict {
    pub(crate) inner: *mut alpm_fileconflict_t,
}

impl FileConflict {
    pub fn target(&self) -> &str {
        unsafe { from_cstr((*self.inner).target) }
    }

    pub fn conflict_type(&self) -> FileConflictType {
        let t = unsafe { (*self.inner).type_ };
        unsafe { transmute::<alpm_fileconflicttype_t, FileConflictType>(t) }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr((*self.inner).file) }
    }

    pub fn conflicting_target(&self) -> Option<&str> {
        let s = unsafe { from_cstr((*self.inner).ctarget) };

        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

impl Alpm {
    pub fn check_conflicts<'a>(
        &self,
        pkgs: impl IntoIterator<Item = Package<'a>>,
    ) -> AlpmList<Conflict> {
        let mut list = std::ptr::null_mut();

        for pkg in pkgs {
            list = unsafe { alpm_list_add(list, pkg.pkg as *mut c_void) };
        }

        let ret = unsafe { alpm_checkconflicts(self.handle, list) };
        unsafe { alpm_list_free(list) };
        AlpmList::new(self, ret, FreeMethod::FreeConflict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SigLevel;

    #[test]
    fn test_check_conflicts() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb("core", SigLevel::NONE).unwrap();
        handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        handle.register_syncdb("community", SigLevel::NONE).unwrap();

        let i3 = handle.syncdbs().find_satisfier("i3-wm").unwrap();
        let i3gaps = handle.syncdbs().find_satisfier("i3-gaps").unwrap();
        let mut conflicts = handle.check_conflicts(vec![i3, i3gaps]);
        let conflict = conflicts.next().unwrap();
        assert_eq!(conflict.package1(), "i3-gaps");
        assert_eq!(conflict.package2(), "i3-wm");

        let xterm = handle.syncdbs().find_satisfier("xterm").unwrap();
        let systemd = handle.syncdbs().find_satisfier("systemd").unwrap();
        let conflicts = handle.check_conflicts(vec![xterm, systemd]);
        assert!(conflicts.is_empty());
    }
}
