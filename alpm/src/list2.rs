use crate::{
    free, Alpm, Backup, Conflict, Db, DbMut, Dep, DepMissing, Depend, DependMissing, FileConflict,
    Group, LoadedPackage, OwnedConflict, OwnedFileConflict, Package, Pkg,
};

use std::ffi::{c_void, CStr};
use std::fmt;
use std::iter::{ExactSizeIterator, FromIterator, Iterator};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use std::ptr;

use alpm_sys::*;

pub unsafe trait IntoAlpmListItem {
    unsafe fn into_item(ptr: *mut c_void) -> Self;
}

pub unsafe trait AsAlpmListItem<'a> {
    type Borrow: 'a;

    unsafe fn as_item(ptr: *mut c_void) -> Self::Borrow;
}

pub unsafe trait IntoAlpmListItemPtr {
    type Ptr;
    unsafe fn into_item_ptr(&self) -> *mut Self::Ptr;
}

pub struct Iter<'l, T> {
    current: AlpmList<'l, T>,
}

pub struct IntoIter<T>
where
    T: DropAlpmListItem,
{
    current: AlpmListMut<T>,
}

#[derive(Clone, Copy, Debug)]
pub struct AlpmList<'l, T> {
    _marker: PhantomData<(&'l (), T)>,
    list: *mut alpm_list_t,
}

pub struct AlpmListMut<T>
where
    T: DropAlpmListItem,
{
    _marker: PhantomData<T>,
    list: *mut alpm_list_t,
}

impl<T> Drop for AlpmListMut<T>
where
    T: DropAlpmListItem,
{
    fn drop(&mut self) {
        let mut list = self.list;
        let start = list;

        while !list.is_null() {
            unsafe { T::drop_item((*list).data) };
            list = unsafe { (*list).next };
        }

        unsafe { alpm_list_free(start) }
    }
}

impl<T> FromIterator<T> for AlpmListMut<T>
where
    T: DropAlpmListItem + IntoAlpmListItemPtr,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = AlpmListMut::new();

        for item in iter {
            list.push(item);
        }

        list
    }
}

impl<T> Extend<T> for AlpmListMut<T>
where
    T: DropAlpmListItem + IntoAlpmListItemPtr,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T> AlpmListMut<T>
where
    T: DropAlpmListItem + IntoAlpmListItemPtr,
{
    pub fn push(&mut self, t: T) {
        unsafe { self.list = alpm_list_add(self.list, t.into_item_ptr() as *mut c_void) };
        std::mem::forget(t);
    }
}

impl<T> IntoIterator for AlpmListMut<T>
where
    T: DropAlpmListItem + IntoAlpmListItem,
{
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { current: self }
    }
}

impl<'l, T> IntoIterator for &'l AlpmListMut<T>
where
    T: DropAlpmListItem + AsAlpmListItem<'l>,
{
    type IntoIter = Iter<'l, T>;
    type Item = T::Borrow;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            current: self.as_list(),
        }
    }
}

impl<T> AlpmListMut<T>
where
    T: DropAlpmListItem,
{
    pub fn new() -> Self {
        AlpmListMut {
            list: ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    pub fn as_list(&self) -> AlpmList<T> {
        AlpmList {
            list: self.list,
            _marker: PhantomData,
        }
    }
}

impl<'l, T> Iter<'l, T> {
    fn next_data(&mut self) -> Option<*mut c_void> {
        if self.current.list.is_null() {
            None
        } else {
            let data = unsafe { (*(self.current.list)).data };
            self.current.list = unsafe { alpm_list_next(self.current.list) };

            Some(data)
        }
    }
}

impl<'l, T> Iterator for Iter<'l, T>
where
    T: AsAlpmListItem<'l>,
{
    type Item = T::Borrow;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.next_data();

        match data {
            Some(data) => unsafe { Some(T::as_item(data)) },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = unsafe { alpm_list_count(self.current.list) };
        (size, Some(size))
    }
}

impl<T> IntoIter<T>
where
    T: DropAlpmListItem,
{
    fn next_data(&mut self) -> Option<*mut c_void> {
        if self.current.list.is_null() {
            None
        } else {
            let data = unsafe { (*(self.current.list)).data };
            self.current.list = unsafe { alpm_list_next(self.current.list) };

            Some(data)
        }
    }
}

impl<T> Iterator for IntoIter<T>
where
    T: DropAlpmListItem + IntoAlpmListItem,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.next_data();

        match data {
            Some(data) => unsafe { Some(T::into_item(data)) },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = unsafe { alpm_list_count(self.current.list) };
        (size, Some(size))
    }
}

pub trait WithAlpmList<T> {
    fn with_alpm_list<F: FnOnce(AlpmList<T>)>(self, f: F);
}

impl<'l, T> WithAlpmList<T> for AlpmList<'l, T> {
    fn with_alpm_list<F: FnOnce(AlpmList<'l, T>)>(self, f: F) {
        f(self)
    }
}

impl<T> WithAlpmList<T> for AlpmListMut<T>
where
    T: DropAlpmListItem,
{
    fn with_alpm_list<F: FnOnce(AlpmList<T>)>(self, f: F) {
        f(self.as_list())
    }
}

pub unsafe trait CastAlpmList<T>: Sized {
    fn cast_alpm_list(list: AlpmList<Self>) -> AlpmList<T> {
        AlpmList {
            list: list.list,
            _marker: PhantomData,
        }
    }
}

//unsafe impl<T> CastAlpmList<T> for T {}
//unsafe impl<'a> CastAlpmList<&'a str> for String {}
//unsafe impl<T> CastAlpmList<T> for &T {}
//unsafe impl<'a, T, U> CastAlpmList<U> for T where T: AsAlpmListItem<'a, Borrow = U> {}
unsafe impl<T, U> CastAlpmList<U> for T
where
    T: IntoAlpmListItemPtr,
    U: IntoAlpmListItemPtr<Ptr = T::Ptr>,
{
}

impl<T, U, I> WithAlpmList<U> for I
where
    T: DropAlpmListItem + IntoAlpmListItemPtr,
    U: IntoAlpmListItemPtr<Ptr = T::Ptr>,
    I: Iterator<Item = T>,
{
    fn with_alpm_list<F: FnOnce(AlpmList<U>)>(self, f: F) {
        let list = AlpmListMut::from_iter(self);
        println!("{}", type_name_of_val(&list));
        let list = list.as_list();
        let list = T::cast_alpm_list(list);
        f(list)
    }
}

unsafe impl<'a, T> AsAlpmListItem<'a> for &T
where
    T: AsAlpmListItem<'a>,
{
    type Borrow = T::Borrow;

    unsafe fn as_item(ptr: *mut c_void) -> T::Borrow {
        T::as_item(ptr)
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Dep<'a> {
    type Borrow = Self;

    unsafe fn as_item(ptr: *mut c_void) -> Self::Borrow {
        Dep::from_ptr(ptr as *mut _)
    }
}

unsafe impl<'a> IntoAlpmListItem for Dep<'a> {
    unsafe fn into_item(ptr: *mut c_void) -> Self {
        Dep::from_ptr(ptr as *mut _)
    }
}

unsafe impl<'a> IntoAlpmListItemPtr for Dep<'a> {
    type Ptr = alpm_depend_t;

    unsafe fn into_item_ptr(&self) -> *mut alpm_depend_t {
        self.as_ptr()
    }
}

unsafe impl<'a> AsAlpmListItem<'a> for Depend {
    type Borrow = Dep<'a>;

    unsafe fn as_item(ptr: *mut c_void) -> Self::Borrow {
        Dep::from_ptr(ptr as *mut _)
    }
}

unsafe impl IntoAlpmListItem for Depend {
    unsafe fn into_item(ptr: *mut c_void) -> Self {
        Depend::from_ptr(ptr as *mut _)
    }
}

unsafe impl IntoAlpmListItemPtr for Depend {
    type Ptr = alpm_depend_t;

    unsafe fn into_item_ptr(&self) -> *mut alpm_depend_t {
        self.as_ptr()
    }
}

unsafe impl<T> IntoAlpmListItemPtr for &T where T: IntoAlpmListItemPtr {
    type Ptr = T::Ptr;

    unsafe fn into_item_ptr(&self) -> *mut T::Ptr {
        T::into_item_ptr(*self)
    }
}

pub struct AlpmListFromIter<T>
where
    T: DropAlpmListItem,
{
    list: AlpmListMut<T>,
}

pub unsafe trait DropAlpmListItem {
    unsafe fn drop_item(ptr: *mut c_void);
}

unsafe impl DropAlpmListItem for Depend {
    unsafe fn drop_item(ptr: *mut c_void) {
        Depend::from_ptr(ptr as *mut _);
    }
}

unsafe impl<'a> DropAlpmListItem for Dep<'a> {
    unsafe fn drop_item(ptr: *mut c_void) {
    }
}

unsafe impl<T> DropAlpmListItem for &T {
    unsafe fn drop_item(ptr: *mut c_void) {
    }
}

#[cfg(test)]
mod tests {
    use crate::AsDep;

    use super::*;

    fn foo<'a, L: WithAlpmList<Dep<'a>>>(list: L) {
        list.with_alpm_list(|list| println!("{:?}", list));
    }

    #[test]
    fn test_list2() {
        let dep = Depend::new("a");
        let a = vec![dep];

        foo(a.iter());
        foo(a.into_iter());

        let dep = Depend::new("a");
        let dep = dep.as_dep();
        let a = vec![dep];

        foo(a.iter());
        foo(a.into_iter());

    }
}

pub fn type_name_of_val<T: ?Sized>(_val: &T) -> &'static str {
    std::any::type_name::<T>()
}
