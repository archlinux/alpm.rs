use alpm_sys::*;

use crate::{
    free, AlpmList, Backup, Conflict, Db, DbMut, Dep, DepMissing, Depend, DependMissing,
    FileConflict, IntoAlpmListItem, Iter, LoadedPackage, OwnedConflict, OwnedFileConflict, Package,
    Pkg,
};

use std::ffi::c_void;
use std::fmt;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use std::ptr;

extern "C" {
    fn strndup(cs: *const c_char, n: usize) -> *mut c_char;
}

#[doc(hidden)]
pub unsafe trait BorrowAlpmListItem<'a> {
    type Borrow: IntoAlpmListItem;
}

#[doc(hidden)]
pub unsafe trait IntoAlpmListPtr: Sized {
    type Output: IntoAlpmListItem;
    fn into_ptr(self) -> *mut c_void {
        ManuallyDrop::new(self).as_ptr()
    }
    fn as_ptr(&self) -> *mut c_void;
}

pub struct AlpmListMut<T: IntoAlpmListItem> {
    _marker: PhantomData<T>,
    list: *mut alpm_list_t,
}

unsafe impl<T: IntoAlpmListItem + Send> Send for AlpmListMut<T> {}
unsafe impl<T: IntoAlpmListItem + Sync> Sync for AlpmListMut<T> {}

impl<'a, T: IntoAlpmListItem + BorrowAlpmListItem<'a>> fmt::Debug for AlpmListMut<T>
where
    T::Borrow: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.list(), f)
    }
}

pub struct IntoIter<T: IntoAlpmListItem> {
    list: *mut alpm_list_t,
    start: *mut alpm_list_t,
    _marker: PhantomData<T>,
}

unsafe impl<T: IntoAlpmListItem + Send> Send for IntoIter<T> {}
unsafe impl<T: IntoAlpmListItem + Sync> Sync for IntoIter<T> {}

impl<'a, T: IntoAlpmListItem + BorrowAlpmListItem<'a>> Debug for IntoIter<T>
where
    T::Borrow: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("Iter")
                .field("list", &AlpmList::<T::Borrow>::from_ptr(self.list))
                .finish()
        }
    }
}

impl<T: IntoAlpmListItem> Drop for AlpmListMut<T> {
    fn drop(&mut self) {
        let mut list = self.list;
        let start = self.list;

        while !list.is_null() {
            unsafe { T::drop_item((*list).data) };
            list = unsafe { (*list).next };
        }

        unsafe { alpm_list_free(start) }
    }
}

impl<T: IntoAlpmListItem> IntoIterator for AlpmListMut<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        let list = ManuallyDrop::new(self);
        IntoIter {
            list: list.list,
            start: list.list,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: IntoAlpmListItem> IntoIterator for &'a AlpmListMut<T>
where
    T: BorrowAlpmListItem<'a>,
{
    type IntoIter = Iter<'a, T::Borrow>;
    type Item = T::Borrow;

    fn into_iter(self) -> Self::IntoIter {
        self.list().into_iter()
    }
}

impl<T: IntoAlpmListPtr> FromIterator<T> for AlpmListMut<T::Output> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = AlpmListMut::new();
        list.extend(iter);
        list
    }
}

impl<T: IntoAlpmListPtr> Extend<T> for AlpmListMut<T::Output> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<'a, T: IntoAlpmListItem + BorrowAlpmListItem<'a>> AlpmListMut<T> {
    pub fn list(&self) -> AlpmList<T::Borrow> {
        unsafe { AlpmList::from_ptr(self.list) }
    }
}

impl<T: IntoAlpmListItem> AlpmListMut<T> {
    pub(crate) unsafe fn from_ptr(list: *mut alpm_list_t) -> AlpmListMut<T> {
        AlpmListMut {
            list,
            _marker: PhantomData,
        }
    }

    pub fn new() -> AlpmListMut<T> {
        AlpmListMut {
            list: ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        unsafe { alpm_list_count(self.list) }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut curr = self.list;

        while !curr.is_null() {
            let item = unsafe { ManuallyDrop::new(T::into_list_item((*curr).data)) };
            let next = unsafe { (*curr).next };
            if !f(&item) {
                ManuallyDrop::into_inner(item);
                unsafe { self.list = alpm_list_remove_item(self.list, curr) };
                unsafe { free(curr as _) };
            }
            curr = next;
        }
    }

    pub fn remove(&mut self, n: usize) -> Option<T> {
        if n >= self.len() {
            return None;
        }

        let item = unsafe { alpm_list_nth(self.list, n) };
        unsafe { self.list = alpm_list_remove_item(self.list, item) };
        let ret = unsafe { Some(T::into_list_item((*item).data)) };
        unsafe { free(item as _) };
        ret
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.list.is_null() {
            return None;
        }

        let last = unsafe { (*self.list).prev };
        unsafe { self.list = alpm_list_remove_item(self.list, last) };
        let ret = unsafe { Some(T::into_list_item((*last).data)) };
        unsafe { free(last as _) };
        ret
    }
}

impl<T: IntoAlpmListItem> AlpmListMut<T> {
    pub fn push<U: IntoAlpmListPtr<Output = T>>(&mut self, t: U) {
        unsafe { self.list = alpm_list_add(self.list, U::into_ptr(t)) }
    }
}

impl AlpmListMut<String> {
    pub fn push_str(&mut self, s: &str) {
        let s = unsafe { strndup(s.as_bytes().as_ptr() as _, s.len()) };
        unsafe { self.list = alpm_list_add(self.list, s as *mut c_void) };
    }
}

// cant deref so manual impl
impl<T: IntoAlpmListItem> AlpmListMut<T> {
    pub fn is_empty(&self) -> bool {
        self.list.is_null()
    }
}

impl<'a, T: IntoAlpmListItem + BorrowAlpmListItem<'a>> AlpmListMut<T> {
    pub fn first(&self) -> Option<T::Borrow> {
        self.list().first()
    }

    pub fn last(&self) -> Option<T::Borrow> {
        self.list().last()
    }

    pub fn iter(&self) -> Iter<T::Borrow> {
        unsafe { AlpmList::from_ptr(self.list).into_iter() }
    }
}

impl<T: IntoAlpmListItem> Drop for IntoIter<T> {
    fn drop(&mut self) {
        let mut list = self.list;

        while !list.is_null() {
            unsafe { T::drop_item((*list).data) };
            list = unsafe { (*list).next };
        }

        unsafe { alpm_list_free(self.start) }
    }
}

impl<T: IntoAlpmListItem> IntoIter<T> {
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

impl<T: IntoAlpmListItem> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.next_data().map(|i| T::into_list_item(i)) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = unsafe { alpm_list_count(self.list) };
        (size, Some(size))
    }
}

// ptr impl

unsafe impl<'a, T: IntoAlpmListPtr> IntoAlpmListPtr for &&'a T
where
    &'a T::Output: BorrowAlpmListItem<'a>,
{
    type Output = <&'a T::Output as BorrowAlpmListItem<'a>>::Borrow;
    fn as_ptr(&self) -> *mut c_void {
        T::as_ptr(*self)
    }
}

unsafe impl IntoAlpmListPtr for Depend {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Dep::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &Dep {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Dep::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &Pkg {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Pkg::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &Package {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Pkg::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for LoadedPackage<'_> {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Pkg::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &Db {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Db::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for DbMut<'_> {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Db::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for DependMissing {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        DepMissing::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &DepMissing {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        DepMissing::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &Conflict {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Conflict::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for OwnedConflict {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Conflict::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &FileConflict {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        FileConflict::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for OwnedFileConflict {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        FileConflict::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for &Backup {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        Backup::as_ptr(self) as _
    }
}

unsafe impl IntoAlpmListPtr for String {
    type Output = Self;
    fn as_ptr(&self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
    fn into_ptr(self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
}

unsafe impl IntoAlpmListPtr for &String {
    type Output = String;
    fn as_ptr(&self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
    fn into_ptr(self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
}

unsafe impl IntoAlpmListPtr for &str {
    type Output = String;
    fn as_ptr(&self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
    fn into_ptr(self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
}

// ptr impl on &T because String is special

unsafe impl IntoAlpmListPtr for &&str {
    type Output = String;
    fn as_ptr(&self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
    fn into_ptr(self) -> *mut c_void {
        unsafe { strndup(self.as_bytes().as_ptr() as _, self.len()) as *mut c_void }
    }
}

unsafe impl<'a> IntoAlpmListPtr for &'a Depend {
    type Output = &'a Dep;
    fn as_ptr(&self) -> *mut c_void {
        Dep::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a Dep {
    type Output = &'a Dep;
    fn as_ptr(&self) -> *mut c_void {
        Dep::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a Package {
    type Output = &'a Pkg;
    fn as_ptr(&self) -> *mut c_void {
        Pkg::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a Pkg {
    type Output = &'a Pkg;
    fn as_ptr(&self) -> *mut c_void {
        Pkg::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &LoadedPackage<'a> {
    type Output = &'a Pkg;
    fn as_ptr(&self) -> *mut c_void {
        Pkg::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a Db {
    type Output = &'a Db;
    fn as_ptr(&self) -> *mut c_void {
        Db::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &DbMut<'a> {
    type Output = DbMut<'a>;
    fn as_ptr(&self) -> *mut c_void {
        Db::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &'a DependMissing {
    type Output = &'a DepMissing;
    fn as_ptr(&self) -> *mut c_void {
        DepMissing::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a DepMissing {
    type Output = &'a DepMissing;
    fn as_ptr(&self) -> *mut c_void {
        DepMissing::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a Conflict {
    type Output = &'a Conflict;
    fn as_ptr(&self) -> *mut c_void {
        Conflict::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &'a OwnedConflict {
    type Output = &'a Conflict;
    fn as_ptr(&self) -> *mut c_void {
        Conflict::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a FileConflict {
    type Output = &'a FileConflict;
    fn as_ptr(&self) -> *mut c_void {
        FileConflict::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &'a OwnedFileConflict {
    type Output = &'a FileConflict;
    fn as_ptr(&self) -> *mut c_void {
        FileConflict::as_ptr(self) as _
    }
}

unsafe impl<'a> IntoAlpmListPtr for &&'a Backup {
    type Output = &'a Backup;
    fn as_ptr(&self) -> *mut c_void {
        Backup::as_ptr(self) as _
    }
}

// borrow impl

unsafe impl<'a, T: BorrowAlpmListItem<'a>> BorrowAlpmListItem<'a> for &T {
    type Borrow = T::Borrow;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for Depend {
    type Borrow = &'a Dep;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for DbMut<'a> {
    type Borrow = DbMut<'a>;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &Db {
    type Borrow = &'a Db;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &Dep {
    type Borrow = &'a Dep;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &Pkg {
    type Borrow = &'a Pkg;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &Package {
    type Borrow = &'a Pkg;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for LoadedPackage<'a> {
    type Borrow = &'a Pkg;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for DependMissing {
    type Borrow = &'a DepMissing;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &DepMissing {
    type Borrow = &'a DepMissing;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &Conflict {
    type Borrow = &'a Conflict;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for OwnedConflict {
    type Borrow = &'a Conflict;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &FileConflict {
    type Borrow = &'a FileConflict;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for OwnedFileConflict {
    type Borrow = &'a FileConflict;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &Backup {
    type Borrow = &'a Backup;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for String {
    type Borrow = &'a str;
}

unsafe impl<'a> BorrowAlpmListItem<'a> for &str {
    type Borrow = &'a str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Alpm, SigLevel};

    #[test]
    fn test_depends_list_free() {
        let mut list = AlpmListMut::new();
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("aaaa"));
    }

    #[test]
    fn test_depends_list_free_iter() {
        let mut list = AlpmListMut::new();
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("aaaa"));
        let mut iter = list.into_iter();
        iter.next().unwrap();
    }

    #[test]
    fn test_depends_list_pop() {
        let mut list = AlpmListMut::new();
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("bbbb"));
        list.push(Depend::new("cccc"));
        list.push(Depend::new("dddd"));
        assert_eq!(list.pop().unwrap().name(), "dddd");
        assert_eq!(list.pop().unwrap().name(), "cccc");
        assert_eq!(list.pop().unwrap().name(), "bbbb");
        assert_eq!(list.pop().unwrap().name(), "aaaa");
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn test_depends_list_remove() {
        let mut list = AlpmListMut::new();
        list.push(Depend::new("aaaa"));
        list.push(Depend::new("bbbb"));
        list.push(Depend::new("cccc"));
        list.push(Depend::new("dddd"));
        assert_eq!(list.remove(0).unwrap().name(), "aaaa");
        assert_eq!(list.remove(2).unwrap().name(), "dddd");
        assert_eq!(list.remove(3), None);
    }

    #[test]
    fn test_depends_list_from_iter() {
        let vec = vec![
            Depend::new("aaaa"),
            Depend::new("bbbb"),
            Depend::new("cccc"),
            Depend::new("dddd"),
        ];

        let list = vec.clone().into_iter().collect::<AlpmListMut<_>>();
        let mut iter = list.iter();

        assert_eq!(iter.next().unwrap().name(), "aaaa");
        assert_eq!(iter.next().unwrap().name(), "bbbb");
        assert_eq!(iter.next().unwrap().name(), "cccc");
        assert_eq!(iter.next().unwrap().name(), "dddd");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_depends_list_debug() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();

        println!("{:#?}", db.pkgs());
        println!("{:#?}", pkg.depends());
    }

    #[test]
    fn test_depends_list_free2() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg.depends();
        assert_eq!(depends.first().unwrap().to_string(), "coreutils");
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
        let required_by = pkg.required_by();
        assert_eq!("acpi_call", required_by.first().unwrap());
    }

    #[test]
    fn test_str_list_free() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("pacman").unwrap();
        let groups = pkg.groups();
        assert_eq!("base", groups.first().unwrap());
    }

    #[test]
    fn test_push() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("pacman").unwrap();

        let mut list = AlpmListMut::new();
        list.push(pkg);
        assert_eq!(list.first().unwrap().name(), "pacman");

        let mut list = AlpmListMut::new();
        list.push(Depend::new("a"));
        list.push(Depend::new("b"));
        list.push(Depend::new("c"));

        let mut list = AlpmListMut::new();
        list.push("a".to_string());
        list.push("b".to_string());
        list.push("c".to_string());

        let mut list = AlpmListMut::new();
        list.push("a".to_string());
        list.push_str("b");
        list.push_str("c");
    }

    #[test]
    fn test_retain() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let mut pkgs = db.pkgs().to_list_mut();
        pkgs.retain(|p| p.name().starts_with('a'));

        assert!(!pkgs.is_empty());
        pkgs.iter().for_each(|p| assert!(p.name().starts_with('a')));
    }
}
