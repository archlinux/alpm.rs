use crate::utils::*;

use crate::{AlpmList, Package, Pkg};

use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::fmt;
use std::io::{self, Read};
use std::marker::PhantomData;
use std::mem::transmute;
use std::os::raw::c_uchar;
use std::ptr::NonNull;
use std::slice;
use std::{cmp::Ordering, ops::Deref};

use _alpm_db_usage_t::*;
use _alpm_download_event_type_t::*;
use _alpm_pkgfrom_t::*;

use _alpm_loglevel_t::*;
use _alpm_pkgreason_t::*;
use _alpm_pkgvalidation_t::*;
use _alpm_progress_t::*;
use _alpm_siglevel_t::*;
use alpm_sys::*;

use bitflags::bitflags;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
#[must_use]
pub enum FetchResult {
    Ok,
    Err,
    FileExists,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct SigLevel: u32 {
        const NONE = 0;
        const PACKAGE = ALPM_SIG_PACKAGE;
        const PACKAGE_OPTIONAL = ALPM_SIG_PACKAGE_OPTIONAL;
        const PACKAGE_MARGINAL_OK = ALPM_SIG_PACKAGE_MARGINAL_OK;
        const PACKAGE_UNKNOWN_OK = ALPM_SIG_PACKAGE_UNKNOWN_OK;
        const DATABASE = ALPM_SIG_DATABASE;
        const DATABASE_OPTIONAL = ALPM_SIG_DATABASE_OPTIONAL;
        const DATABASE_MARGINAL_OK = ALPM_SIG_DATABASE_MARGINAL_OK;
        const DATABASE_UNKNOWN_OK = ALPM_SIG_DATABASE_UNKNOWN_OK;
        const USE_DEFAULT = ALPM_SIG_USE_DEFAULT;
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Usage: u32 {
        const NONE = 0;
        const SYNC = ALPM_DB_USAGE_SYNC;
        const SEARCH = ALPM_DB_USAGE_SEARCH;
        const INSTALL = ALPM_DB_USAGE_INSTALL;
        const UPGRADE = ALPM_DB_USAGE_UPGRADE;
        const ALL = ALPM_DB_USAGE_ALL;
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct LogLevel: u32 {
        const NONE = 0;
        const ERROR = ALPM_LOG_ERROR;
        const WARNING = ALPM_LOG_WARNING;
        const DEBUG = ALPM_LOG_DEBUG;
        const FUNCTION = ALPM_LOG_FUNCTION;
    }
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum Progress {
    AddStart = ALPM_PROGRESS_ADD_START as u32,
    UpgradeStart = ALPM_PROGRESS_UPGRADE_START as u32,
    DowngradeStart = ALPM_PROGRESS_DOWNGRADE_START as u32,
    ReinstallStart = ALPM_PROGRESS_REINSTALL_START as u32,
    RemoveStart = ALPM_PROGRESS_REMOVE_START as u32,
    ConflictsStart = ALPM_PROGRESS_CONFLICTS_START as u32,
    DiskspaceStart = ALPM_PROGRESS_DISKSPACE_START as u32,
    IntegrityStart = ALPM_PROGRESS_INTEGRITY_START as u32,
    LoadStart = ALPM_PROGRESS_LOAD_START as u32,
    KeyringStart = ALPM_PROGRESS_KEYRING_START as u32,
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum PackageFrom {
    File = ALPM_PKG_FROM_FILE as u32,
    LocalDb = ALPM_PKG_FROM_LOCALDB as u32,
    SyncDb = ALPM_PKG_FROM_SYNCDB as u32,
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum PackageReason {
    Explicit = ALPM_PKG_REASON_EXPLICIT as u32,
    Depend = ALPM_PKG_REASON_DEPEND as u32,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct PackageValidation: u32 {
        const UNKNOWN = ALPM_PKG_VALIDATION_UNKNOWN;
        const NONE = ALPM_PKG_VALIDATION_NONE;
        const MD5SUM = ALPM_PKG_VALIDATION_MD5SUM;
        const SHA256SUM = ALPM_PKG_VALIDATION_SHA256SUM;
        const SIGNATURE = ALPM_PKG_VALIDATION_SIGNATURE;
    }
}

pub struct Group {
    inner: UnsafeCell<alpm_group_t>,
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Group")
            .field("name", &self.name())
            .field("packages", &self.packages())
            .finish()
    }
}

impl Group {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *mut alpm_group_t) -> &'a Group {
        &*(ptr as *mut Group)
    }

    pub(crate) fn as_ptr(&self) -> *mut alpm_group_t {
        self.inner.get()
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.as_ptr()).name) }
    }

    pub fn packages(&self) -> AlpmList<&Package> {
        let pkgs = unsafe { (*self.as_ptr()).packages };
        unsafe { AlpmList::from_ptr(pkgs) }
    }
}

pub struct ChangeLog<'a> {
    pub(crate) pkg: &'a Pkg,
    stream: NonNull<c_void>,
}

impl fmt::Debug for ChangeLog<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChangeLog").field("pkg", &self.pkg).finish()
    }
}

impl Drop for ChangeLog<'_> {
    fn drop(&mut self) {
        unsafe { alpm_pkg_changelog_close(self.pkg.as_ptr(), self.as_ptr()) };
    }
}

impl Read for ChangeLog<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            alpm_pkg_changelog_read(
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                self.pkg.as_ptr(),
                self.as_ptr(),
            )
        };
        Ok(ret)
    }
}

impl ChangeLog<'_> {
    pub(crate) unsafe fn new(pkg: &Pkg, ptr: *mut c_void) -> ChangeLog {
        ChangeLog {
            pkg,
            stream: NonNull::new_unchecked(ptr),
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut c_void {
        self.stream.as_ptr()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum Match {
    No,
    Yes,
    Inverted,
}

#[repr(transparent)]
pub struct Backup {
    inner: alpm_backup_t,
}

unsafe impl Send for Backup {}
unsafe impl Sync for Backup {}

impl fmt::Debug for Backup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Backup")
            .field("hash", &self.hash())
            .field("name", &self.name())
            .finish()
    }
}

impl Backup {
    pub(crate) unsafe fn from_ptr<'a>(ptr: *mut alpm_backup_t) -> &'a Backup {
        &*(ptr as *mut Backup)
    }

    pub(crate) fn as_ptr(&self) -> *const alpm_backup_t {
        &self.inner
    }

    pub fn hash(&self) -> &str {
        unsafe { from_cstr((*self.as_ptr()).hash) }
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.as_ptr()).name) }
    }
}

pub struct AnyDownloadEvent<'a> {
    event: alpm_download_event_type_t,
    data: *mut c_void,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for AnyDownloadEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnyDownloadEvent")
            .field("event", &self.event())
            .finish()
    }
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DownloadEventType {
    Init = ALPM_DOWNLOAD_INIT as u32,
    Retry = ALPM_DOWNLOAD_RETRY as u32,
    Progress = ALPM_DOWNLOAD_PROGRESS as u32,
    Completed = ALPM_DOWNLOAD_COMPLETED as u32,
}

impl<'a> AnyDownloadEvent<'a> {
    pub(crate) unsafe fn new(
        event: alpm_download_event_type_t,
        data: *mut c_void,
    ) -> AnyDownloadEvent<'a> {
        AnyDownloadEvent {
            event,
            data,
            marker: PhantomData,
        }
    }

    #[allow(clippy::useless_conversion)]
    pub fn event(&self) -> DownloadEvent {
        let event =
            unsafe { transmute::<alpm_download_event_type_t, DownloadEventType>(self.event) };
        match event {
            DownloadEventType::Init => {
                let data = self.data as *const alpm_download_event_init_t;
                let event = DownloadEventInit {
                    optional: unsafe { (*data).optional != 0 },
                };
                DownloadEvent::Init(event)
            }
            DownloadEventType::Progress => {
                let data = self.data as *const alpm_download_event_progress_t;
                let event = DownloadEventProgress {
                    downloaded: unsafe { (*data).downloaded.into() },
                    total: unsafe { (*data).total.into() },
                };
                DownloadEvent::Progress(event)
            }
            DownloadEventType::Retry => {
                let data = self.data as *const alpm_download_event_retry_t;
                let event = DownloadEventRetry {
                    resume: unsafe { (*data).resume != 0 },
                };
                DownloadEvent::Retry(event)
            }
            DownloadEventType::Completed => {
                let data = self.data as *mut alpm_download_event_completed_t;
                let result = match unsafe { (*data).result.cmp(&0) } {
                    Ordering::Equal => DownloadResult::Success,
                    Ordering::Greater => DownloadResult::UpToDate,
                    Ordering::Less => DownloadResult::Failed,
                };
                let event = DownloadEventCompleted {
                    total: unsafe { (*data).total.into() },
                    result,
                };
                DownloadEvent::Completed(event)
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DownloadEvent {
    Init(DownloadEventInit),
    Progress(DownloadEventProgress),
    Retry(DownloadEventRetry),
    Completed(DownloadEventCompleted),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventInit {
    pub optional: bool,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventProgress {
    pub downloaded: i64,
    pub total: i64,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventRetry {
    pub resume: bool,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventCompleted {
    pub total: i64,
    pub result: DownloadResult,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
#[must_use]
pub enum DownloadResult {
    Success,
    UpToDate,
    Failed,
}

pub struct Signature {
    sig: NonNull<c_uchar>,
    len: usize,
}

impl Signature {
    pub(crate) unsafe fn new(sig: *mut c_uchar, len: usize) -> Signature {
        Signature {
            sig: NonNull::new_unchecked(sig),
            len,
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut c_uchar {
        self.sig.as_ptr()
    }

    pub fn sig(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.sig()).finish()
    }
}

impl Deref for Signature {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.sig()
    }
}

impl Drop for Signature {
    fn drop(&mut self) {
        unsafe { crate::free(self.as_ptr() as _) }
    }
}
