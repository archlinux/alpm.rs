use crate::utils::*;
use crate::{Alpm, AlpmList, Depend, FreeMethod, Package};

use alpm_sys::alpm_fileconflicttype_t::*;
use alpm_sys::*;

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
        unsafe { (*self.inner).package1_hash }
    }

    pub fn package2_hash(&self) -> u64 {
        unsafe { (*self.inner).package2_hash }
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
    pub fn check_conflicts(&self, pkgs: AlpmList<Package>) -> AlpmList<Conflict> {
        let ret = unsafe { alpm_checkconflicts(self.handle, pkgs.list) };
        AlpmList::new(self, ret, FreeMethod::FreeConflict)
    }
}
