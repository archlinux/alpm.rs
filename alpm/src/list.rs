use crate::{
    free, AlpmListMut, Backup, Conflict, Db, DbMut, Dep, DepMissing, Depend, DependMissing,
    FileConflict, Group, LoadedPackage, OwnedConflict, OwnedFileConflict, Package, Pkg,
};

use std::ffi::{c_void, CStr};
use std::fmt;
use std::fmt::Debug;
use std::iter::{ExactSizeIterator, Iterator};
use std::marker::PhantomData;
use std::os::raw::c_char;

use alpm_sys::*;

#[doc(hidden)]
pub unsafe trait IntoAlpmListItem: Sized {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self;
    unsafe fn drop_item(ptr: *mut c_void) {
        Self::into_list_item(ptr);
    }
}

pub struct AlpmList<'l, T> {
    _marker: PhantomData<(&'l (), T)>,
    list: *mut alpm_list_t,
}

unsafe impl<T: Send> Send for AlpmList<'_, T> {}
unsafe impl<T: Sync> Sync for AlpmList<'_, T> {}

impl<T> Clone for AlpmList<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AlpmList<'_, T> {}

impl<T: IntoAlpmListItem + Debug> fmt::Debug for AlpmList<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AlpmList ")?;
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> AlpmList<'_, T> {
    pub(crate) unsafe fn from_ptr<'a>(list: *mut alpm_list_t) -> AlpmList<'a, T> {
        AlpmList {
            _marker: PhantomData,
            list,
        }
    }

    pub(crate) fn as_ptr(self) -> *mut alpm_list_t {
        self.list
    }

    pub fn is_empty(self) -> bool {
        self.list.is_null()
    }

    pub fn len(self) -> usize {
        unsafe { alpm_list_count(self.list) }
    }
}

impl<'l, T: IntoAlpmListItem> AlpmList<'l, T> {
    pub fn first(self) -> Option<T> {
        if self.list.is_null() {
            None
        } else {
            unsafe { Some(T::into_list_item((*self.list).data)) }
        }
    }

    pub fn last(self) -> Option<T> {
        if self.list.is_null() {
            None
        } else {
            unsafe { Some(T::into_list_item((*(*self.list).prev).data)) }
        }
    }

    pub fn iter(self) -> Iter<'l, T> {
        self.into_iter()
    }

    pub fn to_list_mut(&self) -> AlpmListMut<T> {
        let list = unsafe { alpm_list_copy(self.list) };
        unsafe { AlpmListMut::from_ptr(list) }
    }
}

impl<'l, T: IntoAlpmListItem> IntoIterator for AlpmList<'l, T> {
    type IntoIter = Iter<'l, T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            list: self.list,
            _marker: PhantomData,
        }
    }
}

pub struct Iter<'l, T> {
    _marker: PhantomData<(&'l (), T)>,
    list: *mut alpm_list_t,
}

unsafe impl<T: Send> Send for Iter<'_, T> {}
unsafe impl<T: Sync> Sync for Iter<'_, T> {}

impl<T: IntoAlpmListItem + Debug> Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("Iter")
                .field("list", &AlpmList::<T>::from_ptr(self.list))
                .finish()
        }
    }
}

impl<T> Iter<'_, T> {
    fn next_data(&mut self) -> Option<*mut c_void> {
        if self.list.is_null() {
            None
        } else {
            let data = unsafe { (*(self.list)).data };
            self.list = unsafe { alpm_list_next(self.list) };

            Some(data)
        }
    }
}

impl<T: IntoAlpmListItem> Iterator for Iter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.next_data().map(|i| T::into_list_item(i)) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = unsafe { alpm_list_count(self.list) };
        (size, Some(size))
    }
}

impl<T: IntoAlpmListItem> ExactSizeIterator for Iter<'_, T> {}

unsafe impl IntoAlpmListItem for &Dep {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Dep::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &Conflict {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Conflict::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &Pkg {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Pkg::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &Package {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Package::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &Db {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Db::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &Group {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Group::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &DepMissing {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        DepMissing::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &FileConflict {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        FileConflict::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &str {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        let s = CStr::from_ptr(ptr as *mut c_char);
        s.to_str().unwrap()
    }
}

// owned

unsafe impl IntoAlpmListItem for Depend {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Depend::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for OwnedConflict {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        OwnedConflict::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for LoadedPackage<'_> {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        LoadedPackage::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for DbMut<'_> {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        DbMut::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for DependMissing {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        DependMissing::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for &Backup {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        Backup::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for OwnedFileConflict {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        OwnedFileConflict::from_ptr(ptr as _)
    }
}

unsafe impl IntoAlpmListItem for String {
    unsafe fn into_list_item(ptr: *mut c_void) -> Self {
        let s = CStr::from_ptr(ptr as *mut c_char);
        let ret = s.to_str().unwrap().to_string();
        free(ptr);
        ret
    }
    unsafe fn drop_item(ptr: *mut c_void) {
        free(ptr)
    }
}
