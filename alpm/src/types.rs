use crate::utils::*;
use crate::{
    Alpm, AlpmList, AlpmListMut, Conflict, Db, Dep, DependMissing, Error, OwnedConflict,
    OwnedFileConflict, Package, PgpKey,
};

use std::ffi::c_void;
use std::io::{self, Read};
use std::marker::PhantomData;
use std::mem::{transmute, ManuallyDrop};
#[cfg(feature = "mtree")]
use std::ptr;

use _alpm_db_usage_t::*;
use _alpm_event_type_t::*;
use _alpm_hook_when_t::*;
use _alpm_loglevel_t::*;
use _alpm_pkgfrom_t::*;
use _alpm_pkgreason_t::*;
use _alpm_pkgvalidation_t::*;
use _alpm_progress_t::*;
use _alpm_question_type_t::*;
use _alpm_siglevel_t::*;
use alpm_sys::*;

#[cfg(feature = "mtree")]
use libarchive::archive::Handle;
#[cfg(feature = "mtree")]
use libarchive::reader::ReaderEntry;
#[cfg(feature = "mtree")]
use libarchive3_sys::ffi::*;

use bitflags::bitflags;

#[derive(Debug)]
pub struct LogCb<'a> {
    pub(crate) cb: alpm_cb_log,
    pub(crate) marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct DownloadCb<'a> {
    pub(crate) cb: alpm_cb_download,
    pub(crate) marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct FetchCb<'a> {
    pub(crate) cb: alpm_cb_fetch,
    pub(crate) marker: PhantomData<&'a ()>,
}

#[cfg(not(feature = "git"))]
#[derive(Debug)]
pub struct TotalDownloadCb<'a> {
    pub(crate) cb: alpm_cb_totaldl,
    pub(crate) marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct EventCb<'a> {
    pub(crate) cb: alpm_cb_event,
    pub(crate) marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct QuestionCb<'a> {
    pub(crate) cb: alpm_cb_question,
    pub(crate) marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct ProgressCb<'a> {
    pub(crate) cb: alpm_cb_progress,
    pub(crate) marker: PhantomData<&'a ()>,
}

impl<'a> LogCb<'a> {
    pub fn none() -> LogCb<'static> {
        LogCb {
            marker: PhantomData,
            cb: None,
        }
    }
}

impl<'a> DownloadCb<'a> {
    pub fn none() -> DownloadCb<'static> {
        DownloadCb {
            marker: PhantomData,
            cb: None,
        }
    }
}

impl<'a> FetchCb<'a> {
    pub fn none() -> FetchCb<'static> {
        FetchCb {
            marker: PhantomData,
            cb: None,
        }
    }
}

#[cfg(not(feature = "git"))]
impl<'a> TotalDownloadCb<'a> {
    pub fn none() -> TotalDownloadCb<'static> {
        TotalDownloadCb {
            marker: PhantomData,
            cb: None,
        }
    }
}

impl<'a> EventCb<'a> {
    pub fn none() -> EventCb<'static> {
        EventCb {
            marker: PhantomData,
            cb: None,
        }
    }
}

impl<'a> QuestionCb<'a> {
    pub fn none() -> QuestionCb<'static> {
        QuestionCb {
            marker: PhantomData,
            cb: None,
        }
    }
}

impl<'a> ProgressCb<'a> {
    pub fn none() -> ProgressCb<'static> {
        ProgressCb {
            marker: PhantomData,
            cb: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum FetchCbReturn {
    Ok,
    Err,
    FileExists,
}

bitflags! {
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
    pub struct PackageValidation: u32 {
        const UNKNOWN = ALPM_PKG_VALIDATION_UNKNOWN;
        const NONE = ALPM_PKG_VALIDATION_NONE;
        const MD5SUM = ALPM_PKG_VALIDATION_MD5SUM;
        const SHA256SUM = ALPM_PKG_VALIDATION_SHA256SUM;
        const SIGNATURE = ALPM_PKG_VALIDATION_SIGNATURE;
    }
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum EventType {
    CheckDepsStart = ALPM_EVENT_CHECKDEPS_START as u32,
    CheckDepsDone = ALPM_EVENT_CHECKDEPS_DONE as u32,
    FileConflictsStart = ALPM_EVENT_FILECONFLICTS_START as u32,
    FileConflictsDone = ALPM_EVENT_FILECONFLICTS_DONE as u32,
    ResolveDepsStart = ALPM_EVENT_RESOLVEDEPS_START as u32,
    ResolveDepsDone = ALPM_EVENT_RESOLVEDEPS_DONE as u32,
    InterConflictsStart = ALPM_EVENT_INTERCONFLICTS_START as u32,
    InterConflictsDone = ALPM_EVENT_INTERCONFLICTS_DONE as u32,
    TransactionStart = ALPM_EVENT_TRANSACTION_START as u32,
    TransactionDone = ALPM_EVENT_TRANSACTION_DONE as u32,
    PackageOperationStart = ALPM_EVENT_PACKAGE_OPERATION_START as u32,
    PackageOperationDone = ALPM_EVENT_PACKAGE_OPERATION_DONE as u32,
    IntegrityStart = ALPM_EVENT_INTEGRITY_START as u32,
    IntegrityDone = ALPM_EVENT_INTEGRITY_DONE as u32,
    LoadStart = ALPM_EVENT_LOAD_START as u32,
    LoadDone = ALPM_EVENT_LOAD_DONE as u32,
    ScriptletInfo = ALPM_EVENT_SCRIPTLET_INFO as u32,
    #[cfg(not(feature = "git"))]
    RetrieveStart = ALPM_EVENT_RETRIEVE_START as u32,
    #[cfg(not(feature = "git"))]
    RetrieveDone = ALPM_EVENT_RETRIEVE_DONE as u32,
    #[cfg(not(feature = "git"))]
    RetrieveFailed = ALPM_EVENT_RETRIEVE_FAILED as u32,
    #[cfg(not(feature = "git"))]
    PkgDownloadStart = ALPM_EVENT_PKGDOWNLOAD_START as u32,
    #[cfg(not(feature = "git"))]
    PkgDownloadDone = ALPM_EVENT_PKGDOWNLOAD_DONE as u32,
    #[cfg(not(feature = "git"))]
    PkgDownloadFailed = ALPM_EVENT_PKGDOWNLOAD_FAILED as u32,
    #[cfg(feature = "git")]
    RetrieveStart = ALPM_EVENT_DB_RETRIEVE_START as u32,
    #[cfg(feature = "git")]
    RetrieveDone = ALPM_EVENT_DB_RETRIEVE_DONE as u32,
    #[cfg(feature = "git")]
    RetrieveFailed = ALPM_EVENT_DB_RETRIEVE_FAILED as u32,
    #[cfg(feature = "git")]
    PkgDownloadStart = ALPM_EVENT_PKG_RETRIEVE_START as u32,
    #[cfg(feature = "git")]
    PkgDownloadDone = ALPM_EVENT_PKG_RETRIEVE_DONE as u32,
    #[cfg(feature = "git")]
    PkgDownloadFailed = ALPM_EVENT_PKG_RETRIEVE_FAILED as u32,
    DiskSpaceStart = ALPM_EVENT_DISKSPACE_START as u32,
    DiskSpaceDone = ALPM_EVENT_DISKSPACE_DONE as u32,
    OptDepRemoval = ALPM_EVENT_OPTDEP_REMOVAL as u32,
    DatabaseMissing = ALPM_EVENT_DATABASE_MISSING as u32,
    KeyringStart = ALPM_EVENT_KEYRING_START as u32,
    KeyringDone = ALPM_EVENT_KEYRING_DONE as u32,
    KeyDownloadStart = ALPM_EVENT_KEY_DOWNLOAD_START as u32,
    KeyDownloadDone = ALPM_EVENT_KEY_DOWNLOAD_DONE as u32,
    PacnewCreated = ALPM_EVENT_PACNEW_CREATED as u32,
    PacsaveCreated = ALPM_EVENT_PACSAVE_CREATED as u32,
    HookStart = ALPM_EVENT_HOOK_START as u32,
    HookDone = ALPM_EVENT_HOOK_DONE as u32,
    HookRunStart = ALPM_EVENT_HOOK_RUN_START as u32,
    HookRunDone = ALPM_EVENT_HOOK_RUN_DONE as u32,
}

#[derive(Debug)]
pub struct AnyEvent {
    inner: alpm_event_any_t,
}

#[derive(Debug)]
pub enum PackageOperation<'a> {
    Install(Package<'a>),
    Upgrade(Package<'a>, Package<'a>),
    Reinstall(Package<'a>, Package<'a>),
    Downgrade(Package<'a>, Package<'a>),
    Remove(Package<'a>),
}

#[derive(Debug)]
pub struct PackageOperationEvent {
    handle: ManuallyDrop<Alpm>,
    inner: alpm_event_package_operation_t,
}

#[derive(Debug)]
pub struct OptDepRemovalEvent {
    handle: ManuallyDrop<Alpm>,
    inner: alpm_event_optdep_removal_t,
}

#[derive(Debug)]
pub struct ScriptletInfoEvent {
    inner: alpm_event_scriptlet_info_t,
}

#[derive(Debug)]
pub struct DatabaseMissingEvent {
    inner: alpm_event_database_missing_t,
}

#[derive(Debug)]
pub struct PkgDownloadEvent {
    inner: alpm_event_pkgdownload_t,
}

#[derive(Debug)]
pub struct PacnewCreatedEvent {
    handle: ManuallyDrop<Alpm>,
    inner: alpm_event_pacnew_created_t,
}

#[derive(Debug)]
pub struct PacsaveCreatedEvent {
    handle: ManuallyDrop<Alpm>,
    inner: alpm_event_pacsave_created_t,
}

#[derive(Debug)]
pub struct HookEvent {
    inner: alpm_event_hook_t,
}

#[derive(Debug)]
pub struct HookRunEvent {
    inner: alpm_event_hook_run_t,
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum HookWhen {
    PreTransaction = ALPM_HOOK_PRE_TRANSACTION as u32,
    PostTransaction = ALPM_HOOK_POST_TRANSACTION as u32,
}

#[cfg(feature = "git")]
#[derive(Debug)]
pub struct PkgRetrieveEvent {
    inner: alpm_event_pkg_retrieve_t,
}

#[derive(Debug)]
pub enum Event {
    PackageOperation(PackageOperationEvent),
    OptDepRemoval(OptDepRemovalEvent),
    ScriptletInfo(ScriptletInfoEvent),
    DatabaseMissing(DatabaseMissingEvent),
    #[cfg(not(feature = "git"))]
    PkgDownload(PkgDownloadEvent),
    PacnewCreated(PacnewCreatedEvent),
    PacsaveCreated(PacsaveCreatedEvent),
    Hook(HookEvent),
    HookRun(HookRunEvent),
    #[cfg(feature = "git")]
    PkgRetrieve(PkgRetrieveEvent),
    Other(EventType),
}

impl Event {
    /// This is an implementation detail and *should not* be called directly!
    #[doc(hidden)]
    pub unsafe fn new(handle: *mut alpm_handle_t, event: *const alpm_event_t) -> Event {
        let event_type = (*event).type_;
        let event_type = transmute::<alpm_event_type_t, EventType>(event_type);
        let handle = Alpm { handle };
        let handle = ManuallyDrop::new(handle);

        match &event_type {
            EventType::CheckDepsStart => Event::Other(event_type),
            EventType::CheckDepsDone => Event::Other(event_type),
            EventType::FileConflictsStart => Event::Other(event_type),
            EventType::FileConflictsDone => Event::Other(event_type),
            EventType::ResolveDepsStart => Event::Other(event_type),
            EventType::ResolveDepsDone => Event::Other(event_type),
            EventType::InterConflictsStart => Event::Other(event_type),
            EventType::InterConflictsDone => Event::Other(event_type),
            EventType::TransactionStart => Event::Other(event_type),
            EventType::TransactionDone => Event::Other(event_type),
            EventType::PackageOperationStart => Event::PackageOperation(PackageOperationEvent {
                handle,
                inner: (*event).package_operation,
            }),
            EventType::PackageOperationDone => Event::PackageOperation(PackageOperationEvent {
                handle,
                inner: (*event).package_operation,
            }),
            EventType::IntegrityStart => Event::Other(event_type),
            EventType::IntegrityDone => Event::Other(event_type),
            EventType::LoadStart => Event::Other(event_type),
            EventType::LoadDone => Event::Other(event_type),
            EventType::ScriptletInfo => Event::ScriptletInfo(ScriptletInfoEvent {
                inner: (*event).scriptlet_info,
            }),
            EventType::RetrieveStart => Event::Other(event_type),
            EventType::RetrieveDone => Event::Other(event_type),
            EventType::RetrieveFailed => Event::Other(event_type),
            #[cfg(not(feature = "git"))]
            EventType::PkgDownloadStart => Event::PkgDownload(PkgDownloadEvent {
                inner: (*event).pkgdownload,
            }),
            #[cfg(not(feature = "git"))]
            EventType::PkgDownloadDone => Event::PkgDownload(PkgDownloadEvent {
                inner: (*event).pkgdownload,
            }),
            #[cfg(not(feature = "git"))]
            EventType::PkgDownloadFailed => Event::PkgDownload(PkgDownloadEvent {
                inner: (*event).pkgdownload,
            }),
            EventType::DiskSpaceStart => Event::Other(event_type),
            EventType::DiskSpaceDone => Event::Other(event_type),
            EventType::OptDepRemoval => Event::OptDepRemoval(OptDepRemovalEvent {
                handle,
                inner: (*event).optdep_removal,
            }),
            EventType::DatabaseMissing => Event::DatabaseMissing(DatabaseMissingEvent {
                inner: (*event).database_missing,
            }),
            EventType::KeyringStart => Event::Other(event_type),
            EventType::KeyringDone => Event::Other(event_type),
            EventType::KeyDownloadStart => Event::Other(event_type),
            EventType::KeyDownloadDone => Event::Other(event_type),
            EventType::PacnewCreated => Event::PacnewCreated(PacnewCreatedEvent {
                handle,
                inner: (*event).pacnew_created,
            }),
            EventType::PacsaveCreated => Event::PacsaveCreated(PacsaveCreatedEvent {
                handle,
                inner: (*event).pacsave_created,
            }),
            EventType::HookStart => Event::Other(event_type),
            EventType::HookDone => Event::Other(event_type),
            EventType::HookRunStart => Event::Other(event_type),
            EventType::HookRunDone => Event::Other(event_type),
            #[cfg(feature = "git")]
            EventType::PkgDownloadStart => Event::PkgRetrieve(PkgRetrieveEvent {
                inner: (*event).pkg_retrieve,
            }),
            #[cfg(feature = "git")]
            EventType::PkgDownloadDone => Event::Other(event_type),
            #[cfg(feature = "git")]
            EventType::PkgDownloadFailed => Event::Other(event_type),
        }
    }

    pub fn any(self) -> AnyEvent {
        unsafe {
            let event = match self {
                Event::PackageOperation(x) => alpm_event_t {
                    package_operation: x.inner,
                },
                Event::OptDepRemoval(x) => alpm_event_t {
                    optdep_removal: x.inner,
                },
                Event::ScriptletInfo(x) => alpm_event_t {
                    scriptlet_info: x.inner,
                },
                Event::DatabaseMissing(x) => alpm_event_t {
                    database_missing: x.inner,
                },
                #[cfg(not(feature = "git"))]
                Event::PkgDownload(x) => alpm_event_t {
                    pkgdownload: x.inner,
                },
                Event::PacnewCreated(x) => alpm_event_t {
                    pacnew_created: x.inner,
                },
                Event::PacsaveCreated(x) => alpm_event_t {
                    pacsave_created: x.inner,
                },
                Event::Hook(x) => alpm_event_t { hook: x.inner },
                Event::HookRun(x) => alpm_event_t { hook_run: x.inner },
                #[cfg(feature = "git")]
                Event::PkgRetrieve(x) => alpm_event_t {
                    pkg_retrieve: x.inner,
                },
                Event::Other(x) => alpm_event_t {
                    type_: transmute::<EventType, alpm_event_type_t>(x),
                },
            };

            AnyEvent { inner: event.any }
        }
    }
}

impl Into<AnyEvent> for Event {
    fn into(self) -> AnyEvent {
        self.any()
    }
}

impl AnyEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }
}

impl PackageOperationEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn operation(&self) -> PackageOperation {
        let oldpkg = unsafe { Package::new(&self.handle, self.inner.oldpkg) };
        let newpkg = unsafe { Package::new(&self.handle, self.inner.newpkg) };

        match self.inner.operation {
            alpm_package_operation_t::ALPM_PACKAGE_INSTALL => PackageOperation::Install(newpkg),
            alpm_package_operation_t::ALPM_PACKAGE_UPGRADE => {
                PackageOperation::Upgrade(newpkg, oldpkg)
            }
            alpm_package_operation_t::ALPM_PACKAGE_REINSTALL => {
                PackageOperation::Reinstall(newpkg, oldpkg)
            }
            alpm_package_operation_t::ALPM_PACKAGE_DOWNGRADE => {
                PackageOperation::Downgrade(newpkg, oldpkg)
            }
            alpm_package_operation_t::ALPM_PACKAGE_REMOVE => PackageOperation::Remove(oldpkg),
        }
    }
}

impl OptDepRemovalEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn pkg(&self) -> Package {
        unsafe { Package::new(&self.handle, self.inner.pkg) }
    }

    pub fn optdep(&self) -> Dep {
        unsafe { Dep::from_ptr(self.inner.optdep) }
    }
}

impl ScriptletInfoEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn line(&self) -> &str {
        unsafe { from_cstr(self.inner.line) }
    }
}

impl DatabaseMissingEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn dbname(&self) -> &str {
        unsafe { from_cstr(self.inner.dbname) }
    }
}

#[cfg(not(feature = "git"))]
impl PkgDownloadEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr(self.inner.file) }
    }
}

impl PacnewCreatedEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_noupgrade(&self) -> bool {
        self.inner.from_noupgrade != 0
    }

    pub fn oldpkg(&self) -> Option<Package> {
        if self.inner.oldpkg.is_null() {
            None
        } else {
            unsafe { Some(Package::new(&self.handle, self.inner.oldpkg)) }
        }
    }

    pub fn newpkg(&self) -> Option<Package> {
        if self.inner.newpkg.is_null() {
            None
        } else {
            unsafe { Some(Package::new(&self.handle, self.inner.newpkg)) }
        }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr(self.inner.file) }
    }
}

impl PacsaveCreatedEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn oldpkg(&self) -> Option<Package> {
        if self.inner.oldpkg.is_null() {
            None
        } else {
            unsafe { Some(Package::new(&self.handle, self.inner.oldpkg)) }
        }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr(self.inner.file) }
    }
}

impl HookEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn when(&self) -> HookWhen {
        unsafe { transmute::<alpm_hook_when_t, HookWhen>(self.inner.when) }
    }
}

impl HookRunEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr(self.inner.name) }
    }

    pub fn desc(&self) -> &str {
        unsafe { from_cstr(self.inner.desc) }
    }

    pub fn position(&self) -> usize {
        self.inner.position as usize
    }

    pub fn total(&self) -> usize {
        self.inner.total as usize
    }
}

#[cfg(feature = "git")]
impl PkgRetrieveEvent {
    pub fn event_type(&self) -> EventType {
        unsafe { transmute::<alpm_event_type_t, EventType>(self.inner.type_) }
    }

    pub fn num(&self) -> usize {
        self.inner.num
    }

    pub fn total_size(&self) -> i64 {
        self.inner.total_size
    }
}

#[derive(Debug)]
pub struct AnyQuestion {
    inner: *mut alpm_question_any_t,
}

#[derive(Debug)]
pub struct InstallIgnorepkgQuestion {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_install_ignorepkg_t,
}

#[derive(Debug)]
pub struct ReplaceQuestion {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_replace_t,
}

#[derive(Debug)]
pub struct ConflictQuestion {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_conflict_t,
}

#[derive(Debug)]
pub struct CorruptedQuestion {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_corrupted_t,
}

#[derive(Debug)]
pub struct RemovePkgsQuestion {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_remove_pkgs_t,
}

#[derive(Debug)]
pub struct SelectProviderQuestion {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_select_provider_t,
}

#[derive(Debug)]
pub struct ImportKeyQuestion {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_import_key_t,
}

#[derive(Debug)]
pub enum Question {
    InstallIgnorepkg(InstallIgnorepkgQuestion),
    Replace(ReplaceQuestion),
    Conflict(ConflictQuestion),
    Corrupted(CorruptedQuestion),
    RemovePkgs(RemovePkgsQuestion),
    SelectProvider(SelectProviderQuestion),
    ImportKey(ImportKeyQuestion),
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum QuestionType {
    InstallIgnorepkg = ALPM_QUESTION_INSTALL_IGNOREPKG as u32,
    ReplacePkg = ALPM_QUESTION_REPLACE_PKG as u32,
    ConflictPkg = ALPM_QUESTION_CONFLICT_PKG as u32,
    CorruptedPkg = ALPM_QUESTION_CORRUPTED_PKG as u32,
    RemovePkgs = ALPM_QUESTION_REMOVE_PKGS as u32,
    SelectProvider = ALPM_QUESTION_SELECT_PROVIDER as u32,
    ImportKey = ALPM_QUESTION_IMPORT_KEY as u32,
}

impl Question {
    /// This is an implementation detail and *should not* be called directly!
    #[doc(hidden)]
    pub unsafe fn new(handle: *mut alpm_handle_t, question: *mut alpm_question_t) -> Question {
        let question_type = (*question).type_;
        let question_type = transmute::<alpm_question_type_t, QuestionType>(question_type);
        let handle = Alpm { handle };
        let handle = ManuallyDrop::new(handle);

        match &question_type {
            QuestionType::InstallIgnorepkg => {
                Question::InstallIgnorepkg(InstallIgnorepkgQuestion {
                    handle,
                    inner: &mut (*question).install_ignorepkg,
                })
            }
            QuestionType::ReplacePkg => Question::Replace(ReplaceQuestion {
                handle,
                inner: &mut (*question).replace,
            }),
            QuestionType::ConflictPkg => Question::Conflict(ConflictQuestion {
                handle,
                inner: &mut (*question).conflict,
            }),
            QuestionType::CorruptedPkg => Question::Corrupted(CorruptedQuestion {
                handle,
                inner: &mut (*question).corrupted,
            }),
            QuestionType::RemovePkgs => Question::RemovePkgs(RemovePkgsQuestion {
                handle,
                inner: &mut (*question).remove_pkgs,
            }),

            QuestionType::SelectProvider => Question::SelectProvider(SelectProviderQuestion {
                handle,
                inner: &mut (*question).select_provider,
            }),
            QuestionType::ImportKey => Question::ImportKey(ImportKeyQuestion {
                handle,
                inner: &mut (*question).import_key,
            }),
        }
    }

    pub fn any(self) -> AnyQuestion {
        unsafe {
            let question = match self {
                Question::InstallIgnorepkg(x) => x.inner as *mut alpm_question_t,
                Question::Replace(x) => x.inner as *mut alpm_question_t,
                Question::Conflict(x) => x.inner as *mut alpm_question_t,

                Question::Corrupted(x) => x.inner as *mut alpm_question_t,

                Question::RemovePkgs(x) => x.inner as *mut alpm_question_t,

                Question::SelectProvider(x) => x.inner as *mut alpm_question_t,

                Question::ImportKey(x) => x.inner as *mut alpm_question_t,
            };

            AnyQuestion {
                inner: &mut (*question).any,
            }
        }
    }
}

impl Into<AnyQuestion> for Question {
    fn into(self) -> AnyQuestion {
        self.any()
    }
}

impl AnyQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_answer(&mut self, answer: bool) {
        unsafe {
            if answer {
                (*self.inner).answer = 1;
            } else {
                (*self.inner).answer = 0;
            }
        }
    }

    pub fn answer(&mut self) -> bool {
        unsafe { (*self.inner).answer != 0 }
    }
}

impl InstallIgnorepkgQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_install(&mut self, install: bool) {
        unsafe {
            if install {
                (*self.inner).install = 1;
            } else {
                (*self.inner).install = 0;
            }
        }
    }

    pub fn install(&mut self) -> bool {
        unsafe { (*self.inner).install != 0 }
    }

    pub fn pkg(&self) -> Package {
        unsafe { Package::new(&self.handle, (*self.inner).pkg) }
    }
}

impl ReplaceQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_replace(&mut self, replace: bool) {
        unsafe {
            if replace {
                (*self.inner).replace = 1;
            } else {
                (*self.inner).replace = 0;
            }
        }
    }

    pub fn replace(&mut self) -> bool {
        unsafe { (*self.inner).replace != 0 }
    }

    pub fn newpkg(&self) -> Package {
        unsafe { Package::new(&self.handle, (*self.inner).newpkg) }
    }

    pub fn oldpkg(&self) -> Package {
        unsafe { Package::new(&self.handle, (*self.inner).oldpkg) }
    }

    pub fn newdb(&self) -> Db {
        unsafe {
            Db {
                db: (*self.inner).newdb,
                handle: &self.handle,
            }
        }
    }
}

impl ConflictQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_remove(&mut self, remove: bool) {
        unsafe {
            if remove {
                (*self.inner).remove = 1;
            } else {
                (*self.inner).remove = 0;
            }
        }
    }

    pub fn remove(&mut self) -> bool {
        unsafe { (*self.inner).remove != 0 }
    }

    pub fn conflict(&self) -> Conflict {
        unsafe { Conflict::from_ptr((*self.inner).conflict) }
    }
}

impl CorruptedQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_remove(&mut self, remove: bool) {
        unsafe {
            if remove {
                (*self.inner).remove = 1;
            } else {
                (*self.inner).remove = 0;
            }
        }
    }

    pub fn remove(&mut self) -> bool {
        unsafe { (*self.inner).remove != 0 }
    }

    pub fn filepath(&self) -> &str {
        unsafe { from_cstr((*self.inner).filepath) }
    }

    pub fn reason(&self) -> Error {
        unsafe { Error::new((*self.inner).reason) }
    }
}

impl RemovePkgsQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_skip(&mut self, skip: bool) {
        unsafe {
            if skip {
                (*self.inner).skip = 1;
            } else {
                (*self.inner).skip = 0;
            }
        }
    }

    pub fn skip(&mut self) -> bool {
        unsafe { (*self.inner).skip != 0 }
    }

    pub fn packages<'a>(&'a self) -> AlpmList<'a, Package> {
        let list = unsafe { (*self.inner).packages };
        AlpmList::from_parts(&self.handle, list)
    }
}

impl SelectProviderQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_index(&mut self, index: i32) {
        unsafe {
            (*self.inner).use_index = index;
        }
    }

    pub fn index(&mut self) -> i32 {
        unsafe { (*self.inner).use_index }
    }

    pub fn providers(&self) -> AlpmList<Package> {
        let list = unsafe { (*self.inner).providers };
        AlpmList::from_parts(&self.handle, list)
    }

    pub fn depend(&self) -> Dep {
        unsafe { Dep::from_ptr((*self.inner).depend) }
    }
}

impl ImportKeyQuestion {
    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute::<alpm_question_type_t, QuestionType>((*self.inner).type_) }
    }

    pub fn set_import(&mut self, import: bool) {
        unsafe {
            if import {
                (*self.inner).import = 1;
            } else {
                (*self.inner).import = 0;
            }
        }
    }

    pub fn import(&mut self) -> bool {
        unsafe { (*self.inner).import != 0 }
    }

    pub fn key(&self) -> PgpKey {
        let key = unsafe { *(*self.inner).key };
        PgpKey { inner: key }
    }
}

#[derive(Debug)]
pub struct Group<'a> {
    pub(crate) handle: &'a Alpm,
    pub(crate) inner: *mut alpm_group_t,
}

impl<'a> Group<'a> {
    pub fn name(&self) -> &'a str {
        unsafe { from_cstr((*self.inner).name) }
    }

    pub fn packages(&self) -> AlpmList<'a, Package<'a>> {
        let pkgs = unsafe { (*self.inner).packages };
        AlpmList::from_parts(self.handle, pkgs)
    }
}

#[cfg(feature = "mtree")]
pub struct MTree<'a> {
    pub(crate) pkg: &'a Package<'a>,
    pub(crate) archive: *mut archive,
}

#[cfg(feature = "mtree")]
impl<'a> Handle for MTree<'a> {
    unsafe fn handle(&self) -> *mut Struct_archive {
        self.archive as *mut Struct_archive
    }
}

#[cfg(feature = "mtree")]
impl<'a> Drop for MTree<'a> {
    fn drop(&mut self) {
        unsafe { alpm_pkg_mtree_close(self.pkg.pkg, self.archive) };
    }
}

#[cfg(feature = "mtree")]
impl<'a> Iterator for MTree<'a> {
    type Item = ReaderEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = ptr::null_mut();
        let ret = unsafe { alpm_pkg_mtree_next(self.pkg.pkg, self.archive, &mut entry) };

        if ret == ARCHIVE_OK {
            Some(ReaderEntry::new(entry as *mut Struct_archive_entry))
        } else {
            None
        }
    }
}

pub struct ChangeLog<'a> {
    pub(crate) pkg: &'a Package<'a>,
    pub(crate) stream: *mut c_void,
}

impl<'a> Drop for ChangeLog<'a> {
    fn drop(&mut self) {
        unsafe { alpm_pkg_changelog_close(self.pkg.pkg, self.stream) };
    }
}

impl<'a> Read for ChangeLog<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            alpm_pkg_changelog_read(
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                self.pkg.pkg,
                self.stream,
            )
        };
        Ok(ret)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum Match {
    No,
    Yes,
    Inverted,
}

#[derive(Debug)]
pub enum PrepareReturn<'a> {
    PkgInvalidArch(AlpmListMut<'a, Package<'a>>),
    UnsatisfiedDeps(AlpmListMut<'a, DependMissing>),
    ConflictingDeps(AlpmListMut<'a, OwnedConflict>),
    None,
}

#[derive(Debug)]
pub enum CommitReturn<'a> {
    FileConflict(AlpmListMut<'a, OwnedFileConflict>),
    PkgInvalid(AlpmListMut<'a, String>),
    None,
}

#[derive(Debug)]
pub struct Backup {
    pub(crate) inner: *mut alpm_backup_t,
}

impl Backup {
    pub fn hash(&self) -> &str {
        unsafe { from_cstr((*self.inner).hash) }
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.inner).name) }
    }
}

#[cfg(feature = "git")]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DownloadEvent {
    Init(DownloadEventInit),
    Progress(DownloadEventProgress),
    Completed(DownloadEventCompleted),
}

#[cfg(feature = "git")]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventInit {
    pub optional: bool,
}

#[cfg(feature = "git")]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventProgress {
    pub downloaded: i64,
    pub total: i64,
}

#[cfg(feature = "git")]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventCompleted {
    pub total: i64,
    pub result: DownloadResult,
}

#[cfg(feature = "git")]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DownloadResult {
    Success,
    UpToDate,
    Failed,
}
