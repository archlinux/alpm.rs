use crate::{
    free, Alpm, Backup, Conflict, Db, DbMut, Dep, DepMissing, Depend, FileConflict, Group,
    OwnedConflict, Package,
};

use std::ffi::{c_void, CStr};
use std::iter::ExactSizeIterator;
use std::iter::Iterator;
use std::marker::PhantomData;
use std::os::raw::c_char;

use alpm_sys::*;

pub unsafe trait AsAlpmListItem<'a> {
    unsafe fn as_alpm_list_item(handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self;
}

#[derive(Debug)]
pub struct AlpmList<'a, T> {
    pub(crate) handle: &'a Alpm,
    pub(crate) list: *mut alpm_list_t,
    pub(crate) current: *mut alpm_list_t,
    pub(crate) free: FreeMethod,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T> AlpmList<'a, T> {
    //TODO
}

impl<'a, T> IntoIterator for AlpmList<'a, T>
where
    T: AsAlpmListItem<'a>,
{
    type Item = T;
    type IntoIter = IntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

pub struct IntoIter<'a, T> {
    list: AlpmList<'a, T>,
}

impl<'a, T> ExactSizeIterator for IntoIter<'a, T> where T: AsAlpmListItem<'a> {}

impl<'a, T> Iterator for IntoIter<'a, T>
where
    T: AsAlpmListItem<'a>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let data = self.list.next_data();

        match data {
            Some(data) => unsafe {
                Some(T::as_alpm_list_item(self.list.handle, data, self.list.free))
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = unsafe { alpm_list_count(self.list.current) };
        (size, Some(size))
    }
}

impl<'a, T> AlpmList<'a, T> {
    pub(crate) fn new(
        handle: &'a Alpm,
        list: *mut alpm_list_t,
        free: FreeMethod,
    ) -> AlpmList<'a, T> {
        AlpmList {
            handle,
            list,
            current: list,
            free,
            _marker: PhantomData,
        }
    }

    fn next_data(&mut self) -> Option<*mut c_void> {
        if self.current.is_null() {
            None
        } else {
            let data = unsafe { (*(self.current)).data };
            self.current = unsafe { alpm_list_next(self.current) };

            Some(data)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_null()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FreeMethod {
    FreeList,
    FreeInner,
    FreeConflict,
    FreeFileConflict,
    FreeDepMissing,
    None,
}

unsafe impl<'a> AsAlpmListItem<'a> for Package<'a> {
    unsafe fn as_alpm_list_item(handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        Package::new(handle, ptr as *mut alpm_pkg_t)
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Group<'a> {
    unsafe fn as_alpm_list_item(handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        Group {
            inner: ptr as *mut alpm_group_t,
            handle,
        }
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Depend {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        Depend::from_ptr(ptr as *mut alpm_depend_t)
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Dep<'a> {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        Dep::from_ptr(ptr as *mut alpm_depend_t)
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Backup {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        Backup {
            inner: ptr as *mut alpm_backup_t,
        }
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for FileConflict {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        FileConflict {
            inner: ptr as *mut alpm_fileconflict_t,
        }
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for DepMissing {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        DepMissing {
            inner: ptr as *mut alpm_depmissing_t,
        }
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for OwnedConflict {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        OwnedConflict::from_ptr(ptr as *mut alpm_conflict_t)
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Conflict<'a> {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        Conflict::from_ptr(ptr as *mut alpm_conflict_t)
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Db<'a> {
    unsafe fn as_alpm_list_item(handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        Db {
            db: ptr as *mut alpm_db_t,
            handle,
        }
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for DbMut<'a> {
    unsafe fn as_alpm_list_item(handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        DbMut {
            inner: Db {
                db: ptr as *mut alpm_db_t,
                handle,
            },
        }
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for &'a str {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        let s = CStr::from_ptr(ptr as *mut c_char);
        s.to_str().unwrap()
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for String {
    unsafe fn as_alpm_list_item(_handle: &'a Alpm, ptr: *mut c_void, _free: FreeMethod) -> Self {
        let s = CStr::from_ptr(ptr as *mut c_char);
        let s = s.to_str().unwrap().to_string();
        free(ptr);
        s
    }
}

unsafe extern "C" fn fileconflict_free(ptr: *mut c_void) {
    alpm_fileconflict_free(ptr as *mut alpm_fileconflict_t);
}

unsafe extern "C" fn depmissing_free(ptr: *mut c_void) {
    alpm_depmissing_free(ptr as *mut alpm_depmissing_t);
}

pub(crate) unsafe extern "C" fn dep_free(ptr: *mut c_void) {
    alpm_dep_free(ptr as *mut alpm_depend_t);
}

unsafe extern "C" fn conflict_free(ptr: *mut c_void) {
    alpm_conflict_free(ptr as *mut alpm_conflict_t);
}

impl<'a, T> Drop for AlpmList<'a, T> {
    fn drop(&mut self) {
        match self.free {
            FreeMethod::None => {}
            FreeMethod::FreeList => {
                unsafe { alpm_list_free(self.list) };
            }
            FreeMethod::FreeInner => {
                unsafe { alpm_list_free_inner(self.current, Some(free)) };
                unsafe { alpm_list_free(self.list) };
            }
            FreeMethod::FreeConflict => {
                unsafe { alpm_list_free_inner(self.current, Some(conflict_free)) };
                unsafe { alpm_list_free(self.list) };
            }
            FreeMethod::FreeFileConflict => {
                unsafe { alpm_list_free_inner(self.current, Some(fileconflict_free)) };
                unsafe { alpm_list_free(self.list) };
            }
            FreeMethod::FreeDepMissing => {
                unsafe { alpm_list_free_inner(self.current, Some(depmissing_free)) };
                unsafe { alpm_list_free(self.list) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SigLevel;

    #[test]
    fn test_depends_list_free() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let mut depends = pkg.depends();
        assert_eq!(depends.next().unwrap().to_string(), "coreutils");
    }

    #[test]
    fn test_is_empty() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg.depends();
        assert!(!depends.is_empty());

        let pkg = db.pkg("tzdata").unwrap();
        let depends = pkg.depends();
        assert!(depends.is_empty());
    }

    #[test]
    fn test_string_list_free() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        handle.register_syncdb("community", SigLevel::NONE).unwrap();
        handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let mut required_by = pkg.required_by();
        assert_eq!("acpi_call", required_by.next().unwrap());
    }

    #[test]
    fn test_str_list_free() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("pacman").unwrap();
        let mut groups = pkg.groups();
        assert_eq!("base", groups.next().unwrap());
    }
}
