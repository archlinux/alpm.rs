use crate::Alpm;

use std::error;
use std::ffi::CStr;
use std::fmt;
use std::mem::transmute;

use alpm_sys::_alpm_errno_t::*;
use alpm_sys::*;

pub type Result<T> = std::result::Result<T, Error>;

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum Error {
    Ok = ALPM_ERR_OK as u32,
    Memory = ALPM_ERR_MEMORY as u32,
    System = ALPM_ERR_SYSTEM as u32,
    BadPerms = ALPM_ERR_BADPERMS as u32,
    NotAFile = ALPM_ERR_NOT_A_FILE as u32,
    NotADir = ALPM_ERR_NOT_A_DIR as u32,
    WrongArgs = ALPM_ERR_WRONG_ARGS as u32,
    DiskSpace = ALPM_ERR_DISK_SPACE as u32,
    HandleNull = ALPM_ERR_HANDLE_NULL as u32,
    HandleNotNull = ALPM_ERR_HANDLE_NOT_NULL as u32,
    HandleLock = ALPM_ERR_HANDLE_LOCK as u32,
    DbOpen = ALPM_ERR_DB_OPEN as u32,
    DbCreate = ALPM_ERR_DB_CREATE as u32,
    DbNull = ALPM_ERR_DB_NULL as u32,
    DbNotNull = ALPM_ERR_DB_NOT_NULL as u32,
    DbNotFound = ALPM_ERR_DB_NOT_FOUND as u32,
    DbInvalid = ALPM_ERR_DB_INVALID as u32,
    DbInvalidSig = ALPM_ERR_DB_INVALID_SIG as u32,
    DbVersion = ALPM_ERR_DB_VERSION as u32,
    DbWrite = ALPM_ERR_DB_WRITE as u32,
    DbRemove = ALPM_ERR_DB_REMOVE as u32,
    ServerBadUrl = ALPM_ERR_SERVER_BAD_URL as u32,
    ServerNone = ALPM_ERR_SERVER_NONE as u32,
    TransNotNull = ALPM_ERR_TRANS_NOT_NULL as u32,
    TransNull = ALPM_ERR_TRANS_NULL as u32,
    TransDupTarget = ALPM_ERR_TRANS_DUP_TARGET as u32,
    TransDupFileName = ALPM_ERR_TRANS_DUP_FILENAME as u32,
    TransNotInitialized = ALPM_ERR_TRANS_NOT_INITIALIZED as u32,
    TransNotPrepared = ALPM_ERR_TRANS_NOT_PREPARED as u32,
    TransAbort = ALPM_ERR_TRANS_ABORT as u32,
    TransType = ALPM_ERR_TRANS_TYPE as u32,
    TransNotLocked = ALPM_ERR_TRANS_NOT_LOCKED as u32,
    TransHookFailed = ALPM_ERR_TRANS_HOOK_FAILED as u32,
    PkgNotFound = ALPM_ERR_PKG_NOT_FOUND as u32,
    PkgIgnored = ALPM_ERR_PKG_IGNORED as u32,
    PkgInvalid = ALPM_ERR_PKG_INVALID as u32,
    PkgInvalidChecksum = ALPM_ERR_PKG_INVALID_CHECKSUM as u32,
    PkgInvalidSig = ALPM_ERR_PKG_INVALID_SIG as u32,
    PkgMissingSig = ALPM_ERR_PKG_MISSING_SIG as u32,
    PkgOpen = ALPM_ERR_PKG_OPEN as u32,
    PkgCantRemove = ALPM_ERR_PKG_CANT_REMOVE as u32,
    PkgInvalidName = ALPM_ERR_PKG_INVALID_NAME as u32,
    PkgInvalidArch = ALPM_ERR_PKG_INVALID_ARCH as u32,
    SigMissing = ALPM_ERR_SIG_MISSING as u32,
    SigInvalid = ALPM_ERR_SIG_INVALID as u32,
    UnsatisfiedDeps = ALPM_ERR_UNSATISFIED_DEPS as u32,
    ConflictingDeps = ALPM_ERR_CONFLICTING_DEPS as u32,
    FileConflicts = ALPM_ERR_FILE_CONFLICTS as u32,
    Retrieve = ALPM_ERR_RETRIEVE as u32,
    InvalidRegex = ALPM_ERR_INVALID_REGEX as u32,
    Libarchive = ALPM_ERR_LIBARCHIVE as u32,
    Libcurl = ALPM_ERR_LIBCURL as u32,
    ExternalDownload = ALPM_ERR_EXTERNAL_DOWNLOAD as u32,
    Gpgme = ALPM_ERR_GPGME as u32,
    MissingCapabilitySignatures = ALPM_ERR_MISSING_CAPABILITY_SIGNATURES as u32,
}

impl Error {
    pub(crate) unsafe fn new(err: alpm_errno_t) -> Error {
        transmute::<alpm_errno_t, Error>(err)
    }
}

impl Alpm {
    pub fn last_error(&self) -> Error {
        unsafe { Error::new(alpm_errno(self.as_ptr())) }
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
        fmt.write_str(s.to_str().unwrap())
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
