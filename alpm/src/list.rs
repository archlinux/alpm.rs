use crate::{free, Alpm, Conflict, DepMissing, Depend, Group, Package};

use std::ffi::{c_void, CStr};
use std::iter::Iterator;
use std::marker::PhantomData;
use std::os::raw::c_char;

use alpm_sys::*;

#[derive(Debug)]
pub(crate) enum FreeMethod {
    FreeList,
    FreeInner,
    FreeConflict,
    FreeDepMissing,
    None,
}

#[derive(Debug)]
pub struct AlpmList<'a, T: 'a> {
    pub(crate) handle: &'a Alpm,
    pub(crate) item: *mut alpm_list_t,
    pub(crate) free: FreeMethod,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T> AlpmList<'a, T> {
    pub fn iter(&'a self) -> AlpmList<'a, T> {
        AlpmList {
            handle: self.handle,
            item: self.item,
            free: FreeMethod::None,
            _marker: self._marker,
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Package<'a>> {
    type Item = Package<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.item.is_null() {
                None
            } else {
                let data = (*(self.item)).data;
                let data = data as *mut alpm_pkg_t;
                self.item = alpm_list_next(self.item);
                let pkg = Package {
                    pkg: data,
                    handle: self.handle,
                    drop: false,
                };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Group<'a>> {
    type Item = Group<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.item.is_null() {
                None
            } else {
                let data = (*(self.item)).data;
                let data = data as *mut alpm_group_t;
                self.item = alpm_list_next(self.item);
                let group = Group {
                    handle: self.handle,
                    inner: data,
                };
                Some(group)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Depend<'a>> {
    type Item = Depend<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.item.is_null() {
                None
            } else {
                let data = (*(self.item)).data;
                let data = data as *mut alpm_depend_t;
                self.item = alpm_list_next(self.item);
                let pkg = Depend {
                    inner: data,
                    drop: false,
                    phantom: PhantomData,
                };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, DepMissing> {
    type Item = DepMissing;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.item.is_null() {
                None
            } else {
                let data = (*(self.item)).data;
                let data = data as *mut alpm_depmissing_t;
                self.item = alpm_list_next(self.item);
                let pkg = DepMissing { inner: data };
                Some(pkg)
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, Conflict> {
    type Item = Depend<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.item.is_null() {
                None
            } else {
                let data = (*(self.item)).data;
                let data = data as *mut alpm_depend_t;
                self.item = alpm_list_next(self.item);
                let pkg = Depend {
                    inner: data,
                    drop: false,
                    phantom: PhantomData,
                };
                Some(pkg)
            }
        }
    }
}

unsafe extern "C" fn depmissing_free(ptr: *mut c_void) {
    alpm_depmissing_free(ptr as *mut alpm_depmissing_t);
}

unsafe extern "C" fn conflict_free(ptr: *mut c_void) {
    alpm_conflict_free(ptr as *mut alpm_conflict_t);
}

impl<'a, T> Drop for AlpmList<'a, T> {
    fn drop(&mut self) {
        match self.free {
            FreeMethod::None => {}
            FreeMethod::FreeList => {
                unsafe { alpm_list_free(self.item) };
            }
            FreeMethod::FreeInner => {
                unsafe { alpm_list_free_inner(self.item, Some(free)) };
                unsafe { alpm_list_free(self.item) };
            }
            FreeMethod::FreeConflict => {
                unsafe { alpm_list_free_inner(self.item, Some(depmissing_free)) };
                unsafe { alpm_list_free(self.item) };
            }
            FreeMethod::FreeDepMissing => {
                unsafe { alpm_list_free_inner(self.item, Some(conflict_free)) };
                unsafe { alpm_list_free(self.item) };
            }
        }
    }
}

impl<'a> Iterator for AlpmList<'a, &'a str> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.item.is_null() {
                None
            } else {
                let data = (*(self.item)).data;
                let data = data as *const c_char;
                self.item = alpm_list_next(self.item);
                let s = CStr::from_ptr(data);
                Some(s.to_str().unwrap())
            }
        }
    }
}

impl<'a> IntoIterator for &'a AlpmList<'a, Package<'a>> {
    type Item = Package<'a>;
    type IntoIter = AlpmList<'a, Package<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
