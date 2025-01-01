use crate::{
    Alpm, AlpmList, AlpmListMut, DependMissing, Error, OwnedConflict, OwnedFileConflict, Package,
    Result,
};

use alpm_sys::_alpm_transflag_t::*;
use alpm_sys::*;

use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::ptr;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub struct TransFlag: u32 {
        const NONE = 0;
        const NO_DEPS = ALPM_TRANS_FLAG_NODEPS;
        const NO_SAVE = ALPM_TRANS_FLAG_NOSAVE;
        const NO_DEP_VERSION = ALPM_TRANS_FLAG_NODEPVERSION;
        const CASCADE = ALPM_TRANS_FLAG_CASCADE;
        const RECURSE = ALPM_TRANS_FLAG_RECURSE;
        const DB_ONLY = ALPM_TRANS_FLAG_DBONLY;
        const NO_HOOKS = ALPM_TRANS_FLAG_NOHOOKS;
        const ALL_DEPS = ALPM_TRANS_FLAG_ALLDEPS;
        const DOWNLOAD_ONLY = ALPM_TRANS_FLAG_DOWNLOADONLY;
        const NO_SCRIPTLET = ALPM_TRANS_FLAG_NOSCRIPTLET;
        const NO_CONFLICTS = ALPM_TRANS_FLAG_NOCONFLICTS;
        const NEEDED = ALPM_TRANS_FLAG_NEEDED;
        const ALL_EXPLICIT = ALPM_TRANS_FLAG_ALLEXPLICIT;
        const UNNEEDED = ALPM_TRANS_FLAG_UNNEEDED;
        const RECURSE_ALL = ALPM_TRANS_FLAG_RECURSEALL;
        const NO_LOCK = ALPM_TRANS_FLAG_NOLOCK;
    }
}

#[derive(Debug)]
pub enum PrepareData<'a> {
    PkgInvalidArch(AlpmListMut<&'a Package>),
    UnsatisfiedDeps(AlpmListMut<DependMissing>),
    ConflictingDeps(AlpmListMut<OwnedConflict>),
}

pub struct PrepareError<'a> {
    error: Error,
    data: *mut alpm_list_t,
    _marker: PhantomData<&'a ()>,
}

impl Debug for PrepareError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrepareError")
            .field("error", &self.error())
            .field("data", &self.data())
            .finish()
    }
}

impl Display for PrepareError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl<'a> From<PrepareError<'a>> for Error {
    fn from(err: PrepareError<'a>) -> Error {
        err.error
    }
}

impl StdError for PrepareError<'_> {}

impl PrepareError<'_> {
    pub fn error(&self) -> Error {
        self.error
    }

    #[doc(hidden)]
    pub fn data(&self) -> PrepareData {
        self.try_data().expect("prepare error has no data")
    }

    // TODO replace data() with this on next major bump
    pub fn try_data(&self) -> Option<PrepareData> {
        match self.error {
            Error::PkgInvalidArch => unsafe {
                let list = AlpmListMut::from_ptr(self.data);
                Some(PrepareData::PkgInvalidArch(list))
            },
            Error::UnsatisfiedDeps => unsafe {
                let list = AlpmListMut::from_ptr(self.data);
                Some(PrepareData::UnsatisfiedDeps(list))
            },
            Error::ConflictingDeps => unsafe {
                let list = AlpmListMut::from_ptr(self.data);
                Some(PrepareData::ConflictingDeps(list))
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum CommitData {
    FileConflict(AlpmListMut<OwnedFileConflict>),
    PkgInvalid(AlpmListMut<String>),
}

pub struct CommitError {
    error: Error,
    data: *mut alpm_list_t,
}

impl Debug for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitError")
            .field("error", &self.error())
            .field("data", &self.data())
            .finish()
    }
}

impl Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl From<CommitError> for Error {
    fn from(err: CommitError) -> Error {
        err.error
    }
}

impl StdError for CommitError {}

impl CommitError {
    pub fn error(&self) -> Error {
        self.error
    }

    #[doc(hidden)]
    pub fn data(&self) -> CommitData {
        self.try_data().expect("commit error has no data")
    }

    // TODO replace data() with this on next major bump
    pub fn try_data(&self) -> Option<CommitData> {
        match self.error {
            Error::FileConflicts => unsafe {
                let list = AlpmListMut::from_ptr(self.data);
                Some(CommitData::FileConflict(list))
            },
            Error::PkgInvalid | Error::PkgInvalidSig | Error::PkgInvalidChecksum => unsafe {
                let list = AlpmListMut::from_ptr(self.data);
                Some(CommitData::PkgInvalid(list))
            },
            _ => None,
        }
    }
}

impl Alpm {
    pub fn trans_flags(self) -> TransFlag {
        let flags = unsafe { alpm_trans_get_flags(self.as_ptr()) };
        TransFlag::from_bits(flags as u32).unwrap()
    }

    pub fn trans_prepare(&mut self) -> std::result::Result<(), PrepareError> {
        let mut list = ptr::null_mut();
        let ret = unsafe { alpm_trans_prepare(self.as_ptr(), &mut list) };
        let err = self.check_ret(ret);

        if let Err(err) = err {
            Err(PrepareError {
                error: err,
                data: list,
                _marker: PhantomData,
            })
        } else {
            Ok(())
        }
    }

    pub fn trans_commit(&mut self) -> std::result::Result<(), CommitError> {
        let mut list = ptr::null_mut();
        let ret = unsafe { alpm_trans_commit(self.as_ptr(), &mut list) };
        let err = self.check_ret(ret);

        if let Err(err) = err {
            Err(CommitError {
                error: err,
                data: list,
            })
        } else {
            Ok(())
        }
    }

    pub fn trans_interrupt(&mut self) -> Result<()> {
        let ret = unsafe { alpm_trans_interrupt(self.as_ptr()) };
        self.check_ret(ret)
    }

    pub fn trans_add(&self) -> AlpmList<&Package> {
        let list = unsafe { alpm_trans_get_add(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn trans_remove(&self) -> AlpmList<&Package> {
        let list = unsafe { alpm_trans_get_remove(self.as_ptr()) };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn trans_release(&mut self) -> Result<()> {
        let ret = unsafe { alpm_trans_release(self.as_ptr()) };
        self.check_ret(ret)
    }
}

impl Alpm {
    pub fn trans_init(&self, flags: TransFlag) -> Result<()> {
        let ret = unsafe { alpm_trans_init(self.as_ptr(), flags.bits() as i32) };
        self.check_ret(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnyEvent, Error, Event, LogLevel, SigLevel};

    fn logcb(_level: LogLevel, msg: &str, _: &mut ()) {
        print!("{}", msg);
    }

    fn eventcb(event: AnyEvent, _: &mut ()) {
        match event.event() {
            Event::DatabaseMissing(x) => println!("missing database: {}", x.dbname()),
            _ => println!("event: {:?}", event),
        }
    }

    #[test]
    #[ignore]
    fn test_trans() {
        let mut handle = Alpm::new("/", "tests/db").unwrap();
        let flags = TransFlag::DB_ONLY;

        handle.set_log_cb((), logcb);
        handle.set_event_cb((), eventcb);

        let db = handle.register_syncdb_mut("core", SigLevel::NONE).unwrap();
        db.add_server("https://ftp.rnl.tecnico.ulisboa.pt/pub/archlinux/core/os/x86_64")
            .unwrap();
        let db = handle
            .syncdbs()
            .iter()
            .find(|db| db.name() == "core")
            .unwrap();
        let pkg = db.pkg("filesystem").unwrap();

        handle.trans_init(flags).unwrap();
        handle.trans_add_pkg(pkg).unwrap();
        handle.trans_prepare().unwrap();
        // Due to age the mirror now returns 404 for the package.
        // But we're only testing that the function is called correctly anyway.
        assert!(handle.trans_commit().unwrap_err().error() == Error::Retrieve);
    }
}
