use crate::utils::*;
use crate::{
    Alpm, AlpmList, AlpmListMut, Conflict, Db, Dep, DependMissing, Error, OwnedConflict,
    OwnedFileConflict, Package, PgpKey,
};

use std::cmp::Ordering;
use std::ffi::c_void;
use std::io::{self, Read};
use std::marker::PhantomData;
use std::mem::{transmute, ManuallyDrop};
#[cfg(feature = "git")]
use std::os::raw::c_uchar;
#[cfg(feature = "mtree")]
use std::ptr;
#[cfg(feature = "git")]
use std::slice;

use _alpm_db_usage_t::*;
use _alpm_download_event_type_t::*;
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
    RetrieveStart = ALPM_EVENT_DB_RETRIEVE_START as u32,
    RetrieveDone = ALPM_EVENT_DB_RETRIEVE_DONE as u32,
    RetrieveFailed = ALPM_EVENT_DB_RETRIEVE_FAILED as u32,
    PkgRetrieveStart = ALPM_EVENT_PKG_RETRIEVE_START as u32,
    PkgRetrieveDone = ALPM_EVENT_PKG_RETRIEVE_DONE as u32,
    PkgRetrieveFailed = ALPM_EVENT_PKG_RETRIEVE_FAILED as u32,
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
pub enum PackageOperation<'a> {
    Install(Package<'a>),
    Upgrade(Package<'a>, Package<'a>),
    Reinstall(Package<'a>, Package<'a>),
    Downgrade(Package<'a>, Package<'a>),
    Remove(Package<'a>),
}

#[derive(Debug)]
pub struct PackageOperationEvent<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *const alpm_event_package_operation_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct OptDepRemovalEvent<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *const alpm_event_optdep_removal_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct ScriptletInfoEvent<'a> {
    inner: *const alpm_event_scriptlet_info_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct DatabaseMissingEvent<'a> {
    inner: *const alpm_event_database_missing_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct PkgDownloadEvent<'a> {
    inner: *const alpm_event_pkgdownload_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct PacnewCreatedEvent<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *const alpm_event_pacnew_created_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct PacsaveCreatedEvent<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *const alpm_event_pacsave_created_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct HookEvent<'a> {
    inner: *const alpm_event_hook_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct HookRunEvent<'a> {
    inner: *const alpm_event_hook_run_t,
    marker: PhantomData<&'a ()>,
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum HookWhen {
    PreTransaction = ALPM_HOOK_PRE_TRANSACTION as u32,
    PostTransaction = ALPM_HOOK_POST_TRANSACTION as u32,
}

#[derive(Debug)]
pub struct PkgRetrieveStartEvent<'a> {
    inner: *const alpm_event_pkg_retrieve_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct AnyEvent {
    inner: *const alpm_event_t,
    handle: *mut alpm_handle_t,
}

#[derive(Debug)]
pub enum Event<'a> {
    PackageOperation(PackageOperationEvent<'a>),
    OptDepRemoval(OptDepRemovalEvent<'a>),
    ScriptletInfo(ScriptletInfoEvent<'a>),
    DatabaseMissing(DatabaseMissingEvent<'a>),
    PacnewCreated(PacnewCreatedEvent<'a>),
    PacsaveCreated(PacsaveCreatedEvent<'a>),
    Hook(HookEvent<'a>),
    HookRun(HookRunEvent<'a>),
    PkgRetrieveStart(PkgRetrieveStartEvent<'a>),
    PkgRetrieveDone,
    PkgRetrieveFailed,
    CheckDepsStart,
    CheckDepsDone,
    FileConflictsStart,
    FileConflictsDone,
    ResolveDepsStart,
    ResolveDepsDone,
    InterConflictsStart,
    InterConflictsDone,
    TransactionStart,
    TransactionDone,
    IntegrityStart,
    IntegrityDone,
    LoadStart,
    LoadDone,
    RetrieveStart,
    RetrieveDone,
    RetrieveFailed,
    DiskSpaceStart,
    DiskSpaceDone,
    KeyringStart,
    KeyringDone,
    KeyDownloadStart,
    KeyDownloadDone,
    HookStart,
    HookDone,
    HookRunStart,
    HookRunDone,
}

impl AnyEvent {
    /// This is an implementation detail and *should not* be called directly!
    #[doc(hidden)]
    pub unsafe fn new(handle: *mut alpm_handle_t, inner: *const alpm_event_t) -> AnyEvent {
        AnyEvent { handle, inner }
    }

    pub fn event(&self) -> Event {
        let event = self.inner;
        let event_type = self.event_type();
        let handle = Alpm {
            handle: self.handle,
        };
        let handle = ManuallyDrop::new(handle);

        match &event_type {
            EventType::CheckDepsStart => Event::CheckDepsStart,
            EventType::CheckDepsDone => Event::CheckDepsDone,
            EventType::FileConflictsStart => Event::FileConflictsStart,
            EventType::FileConflictsDone => Event::FileConflictsDone,
            EventType::ResolveDepsStart => Event::ResolveDepsStart,
            EventType::ResolveDepsDone => Event::ResolveDepsDone,
            EventType::InterConflictsStart => Event::InterConflictsStart,
            EventType::InterConflictsDone => Event::InterConflictsDone,
            EventType::TransactionStart => Event::TransactionStart,
            EventType::TransactionDone => Event::TransactionDone,
            EventType::PackageOperationStart => Event::PackageOperation(PackageOperationEvent {
                handle,
                inner: unsafe { &(*event).package_operation },

                marker: PhantomData,
            }),
            EventType::PackageOperationDone => Event::PackageOperation(PackageOperationEvent {
                handle,
                inner: unsafe { &(*event).package_operation },
                marker: PhantomData,
            }),
            EventType::IntegrityStart => Event::IntegrityStart,
            EventType::IntegrityDone => Event::InterConflictsDone,
            EventType::LoadStart => Event::LoadStart,
            EventType::LoadDone => Event::LoadDone,
            EventType::ScriptletInfo => Event::ScriptletInfo(ScriptletInfoEvent {
                inner: unsafe { &(*event).scriptlet_info },
                marker: PhantomData,
            }),
            EventType::RetrieveStart => Event::RetrieveStart,
            EventType::RetrieveDone => Event::RetrieveDone,
            EventType::RetrieveFailed => Event::RetrieveFailed,
            EventType::DiskSpaceStart => Event::DiskSpaceStart,
            EventType::DiskSpaceDone => Event::DiskSpaceDone,
            EventType::OptDepRemoval => Event::OptDepRemoval(OptDepRemovalEvent {
                handle,
                inner: unsafe { &(*event).optdep_removal },
                marker: PhantomData,
            }),
            EventType::DatabaseMissing => Event::DatabaseMissing(DatabaseMissingEvent {
                inner: unsafe { &(*event).database_missing },
                marker: PhantomData,
            }),
            EventType::KeyringStart => Event::KeyringStart,
            EventType::KeyringDone => Event::KeyringDone,
            EventType::KeyDownloadStart => Event::KeyDownloadStart,
            EventType::KeyDownloadDone => Event::KeyringDone,
            EventType::PacnewCreated => Event::PacnewCreated(PacnewCreatedEvent {
                handle,
                inner: unsafe { &(*event).pacnew_created },
                marker: PhantomData,
            }),
            EventType::PacsaveCreated => Event::PacsaveCreated(PacsaveCreatedEvent {
                handle,
                inner: unsafe { &(*event).pacsave_created },
                marker: PhantomData,
            }),
            EventType::HookStart => Event::HookStart,
            EventType::HookDone => Event::HookDone,
            EventType::HookRunStart => Event::HookRunStart,
            EventType::HookRunDone => Event::HookRunDone,
            EventType::PkgRetrieveStart => Event::PkgRetrieveStart(PkgRetrieveStartEvent {
                inner: unsafe { &(*event).pkg_retrieve },
                marker: PhantomData,
            }),
            EventType::PkgRetrieveDone => Event::PkgRetrieveDone,
            EventType::PkgRetrieveFailed => Event::PkgRetrieveFailed,
        }
    }

    pub fn event_type(&self) -> EventType {
        unsafe { transmute((*self.inner).type_) }
    }
}

impl<'a> PackageOperationEvent<'a> {
    pub fn operation(&self) -> PackageOperation {
        let oldpkg = unsafe { Package::new(&self.handle, (*self.inner).oldpkg) };
        let newpkg = unsafe { Package::new(&self.handle, (*self.inner).newpkg) };

        let op = unsafe { (*self.inner).operation };
        match op {
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

impl<'a> OptDepRemovalEvent<'a> {
    pub fn pkg(&self) -> Package {
        unsafe { Package::new(&self.handle, (*self.inner).pkg) }
    }

    pub fn optdep(&self) -> Dep {
        unsafe { Dep::from_ptr((*self.inner).optdep) }
    }
}

impl<'a> ScriptletInfoEvent<'a> {
    pub fn line(&self) -> &str {
        unsafe { from_cstr((*self.inner).line) }
    }
}

impl<'a> DatabaseMissingEvent<'a> {
    pub fn dbname(&self) -> &str {
        unsafe { from_cstr((*self.inner).dbname) }
    }
}

impl<'a> PacnewCreatedEvent<'a> {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_noupgrade(&self) -> bool {
        unsafe { (*self.inner).from_noupgrade != 0 }
    }

    pub fn oldpkg(&self) -> Option<Package> {
        unsafe {
            (*self.inner).oldpkg.as_ref()?;
            Some(Package::new(&self.handle, (*self.inner).oldpkg))
        }
    }

    pub fn newpkg(&self) -> Option<Package> {
        unsafe {
            (*self.inner).newpkg.as_ref()?;
            Some(Package::new(&self.handle, (*self.inner).newpkg))
        }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr((*self.inner).file) }
    }
}

impl<'a> PacsaveCreatedEvent<'a> {
    pub fn oldpkg(&self) -> Option<Package> {
        unsafe {
            (*self.inner).oldpkg.as_ref()?;
            Some(Package::new(&self.handle, (*self.inner).oldpkg))
        }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr((*self.inner).file) }
    }
}

impl<'a> HookEvent<'a> {
    pub fn when(&self) -> HookWhen {
        unsafe { transmute::<alpm_hook_when_t, HookWhen>((*self.inner).when) }
    }
}

impl<'a> HookRunEvent<'a> {
    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.inner).name) }
    }

    pub fn desc(&self) -> &str {
        unsafe { from_cstr((*self.inner).desc) }
    }

    pub fn position(&self) -> usize {
        unsafe { (*self.inner).position as usize }
    }

    pub fn total(&self) -> usize {
        unsafe { (*self.inner).total as usize }
    }
}

impl<'a> PkgRetrieveStartEvent<'a> {
    pub fn num(&self) -> usize {
        unsafe { (*self.inner).num }
    }

    pub fn total_size(&self) -> i64 {
        unsafe { (*self.inner).total_size }
    }
}

#[derive(Debug)]
pub struct InstallIgnorepkgQuestion<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_install_ignorepkg_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct ReplaceQuestion<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_replace_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct ConflictQuestion<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_conflict_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct CorruptedQuestion<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_corrupted_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct RemovePkgsQuestion<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_remove_pkgs_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct SelectProviderQuestion<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_select_provider_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct ImportKeyQuestion<'a> {
    handle: ManuallyDrop<Alpm>,
    inner: *mut alpm_question_import_key_t,
    marker: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct AnyQuestion {
    handle: *mut alpm_handle_t,
    inner: *mut alpm_question_t,
}

#[derive(Debug)]
pub enum Question<'a> {
    InstallIgnorepkg(InstallIgnorepkgQuestion<'a>),
    Replace(ReplaceQuestion<'a>),
    Conflict(ConflictQuestion<'a>),
    Corrupted(CorruptedQuestion<'a>),
    RemovePkgs(RemovePkgsQuestion<'a>),
    SelectProvider(SelectProviderQuestion<'a>),
    ImportKey(ImportKeyQuestion<'a>),
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

impl AnyQuestion {
    /// This is an implementation detail and *should not* be called directly!
    #[doc(hidden)]
    pub unsafe fn new(handle: *mut alpm_handle_t, question: *mut alpm_question_t) -> AnyQuestion {
        AnyQuestion {
            inner: question,
            handle,
        }
    }

    pub fn question(&self) -> Question {
        let question_type = self.question_type();
        let handle = Alpm {
            handle: self.handle,
        };
        let handle = ManuallyDrop::new(handle);

        match &question_type {
            QuestionType::InstallIgnorepkg => {
                Question::InstallIgnorepkg(InstallIgnorepkgQuestion {
                    handle,
                    inner: &mut unsafe { (*self.inner).install_ignorepkg },
                    marker: PhantomData,
                })
            }
            QuestionType::ReplacePkg => Question::Replace(ReplaceQuestion {
                handle,
                inner: &mut unsafe { (*self.inner).replace },
                marker: PhantomData,
            }),
            QuestionType::ConflictPkg => Question::Conflict(ConflictQuestion {
                handle,
                inner: &mut unsafe { (*self.inner).conflict },
                marker: PhantomData,
            }),
            QuestionType::CorruptedPkg => Question::Corrupted(CorruptedQuestion {
                handle,
                inner: &mut unsafe { (*self.inner).corrupted },
                marker: PhantomData,
            }),
            QuestionType::RemovePkgs => Question::RemovePkgs(RemovePkgsQuestion {
                handle,
                inner: &mut unsafe { (*self.inner).remove_pkgs },
                marker: PhantomData,
            }),

            QuestionType::SelectProvider => Question::SelectProvider(SelectProviderQuestion {
                handle,
                inner: &mut unsafe { (*self.inner).select_provider },
                marker: PhantomData,
            }),
            QuestionType::ImportKey => Question::ImportKey(ImportKeyQuestion {
                handle,
                inner: &mut unsafe { (*self.inner).import_key },
                marker: PhantomData,
            }),
        }
    }

    pub fn set_answer(&mut self, answer: bool) {
        unsafe { (*self.inner).any.answer = answer as _ }
    }

    pub fn question_type(&self) -> QuestionType {
        unsafe { transmute((*self.inner).type_) }
    }
}

impl<'a> InstallIgnorepkgQuestion<'a> {
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

impl<'a> ReplaceQuestion<'a> {
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

impl<'a> ConflictQuestion<'a> {
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

impl<'a> CorruptedQuestion<'a> {
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

impl<'a> RemovePkgsQuestion<'a> {
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

    pub fn packages(&'a self) -> AlpmList<'a, Package> {
        let list = unsafe { (*self.inner).packages };
        AlpmList::from_parts(&self.handle, list)
    }
}

impl<'a> SelectProviderQuestion<'a> {
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

impl<'a> ImportKeyQuestion<'a> {
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

pub struct AnyDownloadEvent {
    event: alpm_download_event_type_t,
    data: *mut c_void,
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DownloadEventType {
    Init = ALPM_DOWNLOAD_INIT as u32,
    Progress = ALPM_DOWNLOAD_PROGRESS as u32,
    Completed = ALPM_DOWNLOAD_COMPLETED as u32,
}

impl AnyDownloadEvent {
    /// This is an implementation detail and *should not* be called directly!
    #[doc(hidden)]
    pub unsafe fn new(event: alpm_download_event_type_t, data: *mut c_void) -> AnyDownloadEvent {
        AnyDownloadEvent { event, data }
    }

    pub fn event(&self) -> DownloadEvent {
        let event = unsafe { transmute(self.event) };
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
                    downloaded: unsafe { (*data).downloaded },
                    total: unsafe { (*data).total },
                };
                DownloadEvent::Progress(event)
            }
            DownloadEventType::Completed => {
                let data = self.data as *mut alpm_download_event_completed_t;
                let result = match unsafe { (*data).result.cmp(&0) } {
                    Ordering::Equal => DownloadResult::Success,
                    Ordering::Greater => DownloadResult::UpToDate,
                    Ordering::Less => DownloadResult::Failed,
                };
                let event = DownloadEventCompleted {
                    total: unsafe { (*data).total },
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

#[cfg(feature = "git")]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct DownloadEventCompleted {
    pub total: i64,
    pub result: DownloadResult,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum DownloadResult {
    Success,
    UpToDate,
    Failed,
}

#[cfg(feature = "git")]
#[derive(Debug)]
pub struct Signature {
    pub(crate) sig: *mut c_uchar,
    pub(crate) len: usize,
}

#[cfg(feature = "git")]
impl Signature {
    pub fn sig(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.sig, self.len) }
    }
}

#[cfg(feature = "git")]
impl Drop for Signature {
    fn drop(&mut self) {
        unsafe { crate::free(self.sig as _) }
    }
}
