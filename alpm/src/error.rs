use crate::Alpm;

use std::error;
use std::ffi::CStr;
use std::fmt;
use std::mem::transmute;

use alpm_sys::_alpm_errno_t::*;
use alpm_sys::*;

pub type Result<T> = std::result::Result<T, Error>;

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    Ok = ALPM_ERR_OK as u32,
    Memory,
    System,
    BadPerms,
    NotAFile,
    NotADir,
    WrongArgs,
    DiskSpace,
    HandleNull,
    HandleNotNull,
    HandleLock,
    DbOpen,
    DbCreate,
    DbNull,
    DbNotNull,
    DbNotFound,
    DbInvalid,
    DbInvalidSig,
    DbVersion,
    DbWrite,
    DbRemove,
    ServerBadUrl,
    ServerNone,
    TransNotNull,
    TransNull,
    TransDupTarget,
    TransNotInitialized,
    TransNotPrepared,
    TransAbort,
    TransType,
    TransNotLocked,
    TransHookFailed,
    PkgNotFound,
    PkgIgnored,
    PkgInvalid,
    PkgInvalidChecksum,
    PkgInvalidSig,
    PkgMissingSig,
    PkgOpen,
    PkgCantRemove,
    PkgInvalidName,
    PkgInvalidArch,
    PkgRepoNotFound,
    SigMissing,
    SigInvalid,
    DltInvalid,
    DltPatchFailed,
    UnsatisfiedDeps,
    ConflictingDeps,
    FileConflicts,
    Retrieve,
    InvalidRegex,
    Libarchive,
    Libcurl,
    ExternalDownload,
    Gpgme,
    MissingCapabilitySignatures,
}

impl Error {
    pub(crate) unsafe fn new(err: alpm_errno_t) -> Error {
        transmute::<alpm_errno_t, Error>(err)
    }
}

impl Alpm {
    pub fn last_error(&self) -> Error {
        unsafe { Error::new(alpm_errno(self.handle)) }
    }
}

impl Error {
    pub fn ok(self) -> bool {
        self == Error::Ok
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let err = unsafe { transmute::<Error, alpm_errno_t>(*self) };
        let s = unsafe { CStr::from_ptr(alpm_strerror(err)) };
        fmt.write_str(&s.to_str().unwrap())
    }
}

impl error::Error for Error {}

#[cfg(test)]
mod tests {
    use crate::Alpm;

    #[test]
    fn display() {
        let handle = Alpm::new("/", "tests/db").unwrap();

        println!("{}", handle.last_error());
    }
}
