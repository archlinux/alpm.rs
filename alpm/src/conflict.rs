use crate::utils::*;
use crate::{Alpm, AlpmList, Depend, FreeMethod, Package};

use alpm_sys::*;

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Conflict {
    pub(crate) inner: alpm_conflict_t,
    pub(crate) drop: bool,
}

impl Drop for Conflict {
    fn drop(&mut self) {
        if self.drop {
            unsafe { alpm_conflict_free(&mut self.inner) }
        }
    }
}

impl Conflict {
    pub fn package1_hash(&self) -> u64 {
        self.inner.package1_hash
    }

    pub fn package2_hash(&self) -> u64 {
        self.inner.package2_hash
    }

    pub fn package1(&self) -> &str {
        unsafe { from_cstr(self.inner.package1) }
    }

    pub fn package2(&self) -> &str {
        unsafe { from_cstr(self.inner.package2) }
    }

    pub fn reason(&self) -> Depend<'_> {
        Depend {
            inner: self.inner.reason,
            drop: false,
            phantom: PhantomData,
        }
    }
}

impl Alpm {
    pub fn check_conflicts(&self, pkgs: AlpmList<Package>) -> AlpmList<Conflict> {
        let ret = unsafe { alpm_checkconflicts(self.handle, pkgs.item) };

        AlpmList {
            handle: self,
            item: ret,
            free: FreeMethod::FreeConflict,
            _marker: PhantomData,
        }
    }
}
