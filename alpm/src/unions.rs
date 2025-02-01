use crate::utils::*;

use crate::{AlpmList, Conflict, Db, Dep, Error, Package};

use std::fmt;
use std::marker::PhantomData;
use std::mem::transmute;

use _alpm_event_type_t::*;
use _alpm_hook_when_t::*;
use _alpm_question_type_t::*;
use alpm_sys::*;

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
    Install(&'a Package),
    Upgrade(&'a Package, &'a Package),
    Reinstall(&'a Package, &'a Package),
    Downgrade(&'a Package, &'a Package),
    Remove(&'a Package),
}

pub struct PackageOperationEvent<'a> {
    inner: *const alpm_event_package_operation_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for PackageOperationEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageOperationEvent")
            .field("operation", &self.operation())
            .finish()
    }
}

pub struct OptDepRemovalEvent<'a> {
    inner: *const alpm_event_optdep_removal_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for OptDepRemovalEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OptDepRemovalEvent")
            .field("pkg", &self.pkg())
            .field("optdep", &self.optdep())
            .finish()
    }
}

pub struct ScriptletInfoEvent<'a> {
    inner: *const alpm_event_scriptlet_info_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for ScriptletInfoEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptletInfoEvent")
            .field("line", &self.line())
            .finish()
    }
}

pub struct DatabaseMissingEvent<'a> {
    inner: *const alpm_event_database_missing_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for DatabaseMissingEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseMissingEvent")
            .field("dbname", &self.dbname())
            .finish()
    }
}

pub struct PkgDownloadEvent<'a> {
    inner: *const alpm_event_pkgdownload_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for PkgDownloadEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PkgDownloadEvent")
            .field("file", &self.file())
            .finish()
    }
}

pub struct PacnewCreatedEvent<'a> {
    inner: *const alpm_event_pacnew_created_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for PacnewCreatedEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PacnewCreatedEvent")
            .field("from_noupgrade", &self.from_noupgrade())
            .field("oldpkg", &self.oldpkg())
            .field("oldnew", &self.oldpkg())
            .field("file", &self.file())
            .finish()
    }
}

pub struct PacsaveCreatedEvent<'a> {
    inner: *const alpm_event_pacsave_created_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for PacsaveCreatedEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PacsaveCreatedEvent")
            .field("oldpkg", &self.oldpkg())
            .field("file", &self.file())
            .finish()
    }
}

pub struct HookEvent<'a> {
    inner: *const alpm_event_hook_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for HookEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HookEvent")
            .field("when", &self.when())
            .finish()
    }
}

pub struct HookRunEvent<'a> {
    inner: *const alpm_event_hook_run_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for HookRunEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HookRunEvent")
            .field("name", &self.name())
            .field("desc", &self.desc())
            .field("position", &self.position())
            .field("total", &self.total())
            .finish()
    }
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum HookWhen {
    PreTransaction = ALPM_HOOK_PRE_TRANSACTION as u32,
    PostTransaction = ALPM_HOOK_POST_TRANSACTION as u32,
}

pub struct PkgRetrieveEvent<'a> {
    inner: *const alpm_event_pkg_retrieve_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for PkgRetrieveEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PkgRetrieveEvent")
            .field("num", &self.num())
            .field("total_size", &self.total_size())
            .finish()
    }
}

pub struct AnyEvent<'a> {
    inner: *const alpm_event_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for AnyEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnyEvent")
            .field("event", &self.event())
            .finish()
    }
}

#[derive(Debug)]
pub enum Event<'a> {
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
    PackageOperationStart(PackageOperationEvent<'a>),
    PackageOperationDone(PackageOperationEvent<'a>),
    IntegrityStart,
    IntegrityDone,
    LoadStart,
    LoadDone,
    ScriptletInfo(ScriptletInfoEvent<'a>),
    RetrieveStart,
    RetrieveDone,
    RetrieveFailed,
    PkgRetrieveStart(PkgRetrieveEvent<'a>),
    PkgRetrieveDone(PkgRetrieveEvent<'a>),
    PkgRetrieveFailed(PkgRetrieveEvent<'a>),
    DiskSpaceStart,
    DiskSpaceDone,
    OptDepRemoval(OptDepRemovalEvent<'a>),
    DatabaseMissing(DatabaseMissingEvent<'a>),
    KeyringStart,
    KeyringDone,
    KeyDownloadStart,
    KeyDownloadDone,
    PacnewCreated(PacnewCreatedEvent<'a>),
    PacsaveCreated(PacsaveCreatedEvent<'a>),
    HookStart(HookEvent<'a>),
    HookDone(HookEvent<'a>),
    HookRunStart(HookRunEvent<'a>),
    HookRunDone(HookRunEvent<'a>),
}

impl<'a> AnyEvent<'a> {
    pub(crate) unsafe fn new(inner: *const alpm_event_t) -> AnyEvent<'a> {
        AnyEvent {
            inner,
            marker: PhantomData,
        }
    }

    pub fn event(&self) -> Event<'a> {
        let event = self.inner;
        let event_type = self.event_type();

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
            EventType::PackageOperationStart => {
                Event::PackageOperationStart(PackageOperationEvent {
                    inner: unsafe { &(*event).package_operation },

                    marker: PhantomData,
                })
            }
            EventType::PackageOperationDone => Event::PackageOperationDone(PackageOperationEvent {
                inner: unsafe { &(*event).package_operation },
                marker: PhantomData,
            }),
            EventType::IntegrityStart => Event::IntegrityStart,
            EventType::IntegrityDone => Event::IntegrityDone,
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
                inner: unsafe { &(*event).pacnew_created },
                marker: PhantomData,
            }),
            EventType::PacsaveCreated => Event::PacsaveCreated(PacsaveCreatedEvent {
                inner: unsafe { &(*event).pacsave_created },
                marker: PhantomData,
            }),
            EventType::HookStart => Event::HookStart(HookEvent {
                inner: unsafe { &(*event).hook },
                marker: PhantomData,
            }),
            EventType::HookDone => Event::HookDone(HookEvent {
                inner: unsafe { &(*event).hook },
                marker: PhantomData,
            }),
            EventType::HookRunStart => Event::HookRunStart(HookRunEvent {
                inner: unsafe { &(*event).hook_run },
                marker: PhantomData,
            }),
            EventType::HookRunDone => Event::HookRunDone(HookRunEvent {
                inner: unsafe { &(*event).hook_run },
                marker: PhantomData,
            }),
            EventType::PkgRetrieveStart => Event::PkgRetrieveStart(PkgRetrieveEvent {
                inner: unsafe { &(*event).pkg_retrieve },
                marker: PhantomData,
            }),
            EventType::PkgRetrieveDone => Event::PkgRetrieveDone(PkgRetrieveEvent {
                inner: unsafe { &(*event).pkg_retrieve },
                marker: PhantomData,
            }),
            EventType::PkgRetrieveFailed => Event::PkgRetrieveFailed(PkgRetrieveEvent {
                inner: unsafe { &(*event).pkg_retrieve },
                marker: PhantomData,
            }),
        }
    }

    pub fn event_type(&self) -> EventType {
        unsafe { transmute((*self.inner).type_) }
    }
}

impl PackageOperationEvent<'_> {
    pub fn operation(&self) -> PackageOperation {
        let oldpkg = unsafe { Package::from_ptr((*self.inner).oldpkg) };
        let newpkg = unsafe { Package::from_ptr((*self.inner).newpkg) };

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

impl OptDepRemovalEvent<'_> {
    pub fn pkg(&self) -> &Package {
        unsafe { Package::from_ptr((*self.inner).pkg) }
    }

    pub fn optdep(&self) -> &Dep {
        unsafe { Dep::from_ptr((*self.inner).optdep) }
    }
}

impl ScriptletInfoEvent<'_> {
    pub fn line(&self) -> &str {
        unsafe { from_cstr((*self.inner).line) }
    }
}

impl DatabaseMissingEvent<'_> {
    pub fn dbname(&self) -> &str {
        unsafe { from_cstr((*self.inner).dbname) }
    }
}

impl PkgDownloadEvent<'_> {
    pub fn file(&self) -> &str {
        unsafe { from_cstr((*self.inner).file) }
    }
}

impl PacnewCreatedEvent<'_> {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_noupgrade(&self) -> bool {
        unsafe { (*self.inner).from_noupgrade != 0 }
    }

    pub fn oldpkg(&self) -> Option<&Package> {
        unsafe {
            (*self.inner).oldpkg.as_ref()?;
            Some(Package::from_ptr((*self.inner).oldpkg))
        }
    }

    pub fn newpkg(&self) -> Option<&Package> {
        unsafe {
            (*self.inner).newpkg.as_ref()?;
            Some(Package::from_ptr((*self.inner).newpkg))
        }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr((*self.inner).file) }
    }
}

impl PacsaveCreatedEvent<'_> {
    pub fn oldpkg(&self) -> Option<&Package> {
        unsafe {
            (*self.inner).oldpkg.as_ref()?;
            Some(Package::from_ptr((*self.inner).oldpkg))
        }
    }

    pub fn file(&self) -> &str {
        unsafe { from_cstr((*self.inner).file) }
    }
}

impl HookEvent<'_> {
    pub fn when(&self) -> HookWhen {
        unsafe { transmute::<alpm_hook_when_t, HookWhen>((*self.inner).when) }
    }
}

impl HookRunEvent<'_> {
    pub fn name(&self) -> &str {
        unsafe { from_cstr((*self.inner).name) }
    }

    pub fn desc(&self) -> Option<&str> {
        unsafe { from_cstr_optional((*self.inner).desc) }
    }

    pub fn position(&self) -> usize {
        #[allow(clippy::unnecessary_cast)]
        unsafe {
            (*self.inner).position as usize
        }
    }

    pub fn total(&self) -> usize {
        #[allow(clippy::unnecessary_cast)]
        unsafe {
            (*self.inner).total as usize
        }
    }
}

impl PkgRetrieveEvent<'_> {
    pub fn num(&self) -> usize {
        unsafe { (*self.inner).num }
    }

    #[allow(clippy::useless_conversion)]
    pub fn total_size(&self) -> i64 {
        unsafe { (*self.inner).total_size.into() }
    }
}

pub struct InstallIgnorepkgQuestion<'a> {
    inner: *mut alpm_question_install_ignorepkg_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for InstallIgnorepkgQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstallIgnorepkgQuestion")
            .field("install", &self.install())
            .field("pkg", &self.pkg())
            .finish()
    }
}

pub struct ReplaceQuestion<'a> {
    inner: *mut alpm_question_replace_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for ReplaceQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReplaceQuestion")
            .field("replace", &self.replace())
            .field("oldpkg", &self.oldpkg())
            .field("newpkg", &self.newpkg())
            .field("newdb", &self.newdb())
            .finish()
    }
}

pub struct ConflictQuestion<'a> {
    inner: *mut alpm_question_conflict_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for ConflictQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConflictQuestion")
            .field("remove", &self.remove())
            .field("conflict", &self.conflict())
            .finish()
    }
}

pub struct CorruptedQuestion<'a> {
    inner: *mut alpm_question_corrupted_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for CorruptedQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorruptedQuestion")
            .field("remove", &self.remove())
            .field("filepath", &self.filepath())
            .field("reason", &self.reason())
            .finish()
    }
}

pub struct RemovePkgsQuestion<'a> {
    inner: *mut alpm_question_remove_pkgs_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for RemovePkgsQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemovePkgsQuestion")
            .field("skip", &self.skip())
            .field("packages", &self.packages())
            .finish()
    }
}

pub struct SelectProviderQuestion<'a> {
    inner: *mut alpm_question_select_provider_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for SelectProviderQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectProviderQuestion")
            .field("index", &self.index())
            .field("providers", &self.providers())
            .field("depend", &self.depend())
            .finish()
    }
}

pub struct ImportKeyQuestion<'a> {
    inner: *mut alpm_question_import_key_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for ImportKeyQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportKeyQuestion")
            .field("import", &self.import())
            .field("uid", &self.uid())
            .field("fingerprint", &self.fingerprint())
            .finish()
    }
}

pub struct AnyQuestion<'a> {
    inner: *mut alpm_question_t,
    marker: PhantomData<&'a ()>,
}

impl fmt::Debug for AnyQuestion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnyQuestion")
            .field("question", &self.question())
            .finish()
    }
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
#[allow(clippy::unnecessary_cast)]
pub enum QuestionType {
    InstallIgnorepkg = ALPM_QUESTION_INSTALL_IGNOREPKG as u32,
    ReplacePkg = ALPM_QUESTION_REPLACE_PKG as u32,
    ConflictPkg = ALPM_QUESTION_CONFLICT_PKG as u32,
    CorruptedPkg = ALPM_QUESTION_CORRUPTED_PKG as u32,
    RemovePkgs = ALPM_QUESTION_REMOVE_PKGS as u32,
    SelectProvider = ALPM_QUESTION_SELECT_PROVIDER as u32,
    ImportKey = ALPM_QUESTION_IMPORT_KEY as u32,
}

impl<'a> AnyQuestion<'a> {
    pub(crate) unsafe fn new(question: *mut alpm_question_t) -> AnyQuestion<'a> {
        AnyQuestion {
            inner: question,
            marker: PhantomData,
        }
    }

    pub fn question(&self) -> Question<'a> {
        let question_type = self.question_type();

        match &question_type {
            QuestionType::InstallIgnorepkg => {
                Question::InstallIgnorepkg(InstallIgnorepkgQuestion {
                    inner: unsafe { &mut (*self.inner).install_ignorepkg },
                    marker: PhantomData,
                })
            }
            QuestionType::ReplacePkg => Question::Replace(ReplaceQuestion {
                inner: unsafe { &mut (*self.inner).replace },
                marker: PhantomData,
            }),
            QuestionType::ConflictPkg => Question::Conflict(ConflictQuestion {
                inner: unsafe { &mut (*self.inner).conflict },
                marker: PhantomData,
            }),
            QuestionType::CorruptedPkg => Question::Corrupted(CorruptedQuestion {
                inner: unsafe { &mut (*self.inner).corrupted },
                marker: PhantomData,
            }),
            QuestionType::RemovePkgs => Question::RemovePkgs(RemovePkgsQuestion {
                inner: unsafe { &mut (*self.inner).remove_pkgs },
                marker: PhantomData,
            }),

            QuestionType::SelectProvider => Question::SelectProvider(SelectProviderQuestion {
                inner: unsafe { &mut (*self.inner).select_provider },
                marker: PhantomData,
            }),
            QuestionType::ImportKey => Question::ImportKey(ImportKeyQuestion {
                inner: unsafe { &mut (*self.inner).import_key },
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

impl InstallIgnorepkgQuestion<'_> {
    pub fn set_install(&mut self, install: bool) {
        unsafe {
            if install {
                (*self.inner).install = 1;
            } else {
                (*self.inner).install = 0;
            }
        }
    }

    pub fn install(&self) -> bool {
        unsafe { (*self.inner).install != 0 }
    }

    pub fn pkg(&self) -> &Package {
        unsafe { Package::from_ptr((*self.inner).pkg) }
    }
}

impl ReplaceQuestion<'_> {
    pub fn set_replace(&self, replace: bool) {
        unsafe {
            if replace {
                (*self.inner).replace = 1;
            } else {
                (*self.inner).replace = 0;
            }
        }
    }

    pub fn replace(&self) -> bool {
        unsafe { (*self.inner).replace != 0 }
    }

    pub fn newpkg(&self) -> &Package {
        unsafe { Package::from_ptr((*self.inner).newpkg) }
    }

    pub fn oldpkg(&self) -> &Package {
        unsafe { Package::from_ptr((*self.inner).oldpkg) }
    }

    pub fn newdb(&self) -> &Db {
        unsafe { Db::from_ptr((*self.inner).newdb) }
    }
}

impl ConflictQuestion<'_> {
    pub fn set_remove(&mut self, remove: bool) {
        unsafe {
            if remove {
                (*self.inner).remove = remove as _;
            } else {
                (*self.inner).remove = 0;
            }
        }
    }

    pub fn remove(&self) -> bool {
        unsafe { (*self.inner).remove != 0 }
    }

    pub fn conflict(&self) -> &Conflict {
        unsafe { Conflict::from_ptr((*self.inner).conflict) }
    }
}

impl CorruptedQuestion<'_> {
    pub fn set_remove(&mut self, remove: bool) {
        unsafe {
            if remove {
                (*self.inner).remove = 1;
            } else {
                (*self.inner).remove = 0;
            }
        }
    }

    pub fn remove(&self) -> bool {
        unsafe { (*self.inner).remove != 0 }
    }

    pub fn filepath(&self) -> &str {
        unsafe { from_cstr((*self.inner).filepath) }
    }

    pub fn reason(&self) -> Error {
        unsafe { Error::new((*self.inner).reason) }
    }
}

impl RemovePkgsQuestion<'_> {
    pub fn set_skip(&mut self, skip: bool) {
        unsafe {
            if skip {
                (*self.inner).skip = 1;
            } else {
                (*self.inner).skip = 0;
            }
        }
    }

    pub fn skip(&self) -> bool {
        unsafe { (*self.inner).skip != 0 }
    }

    pub fn packages(&self) -> AlpmList<&Package> {
        let list = unsafe { (*self.inner).packages };
        unsafe { AlpmList::from_ptr(list) }
    }
}

impl SelectProviderQuestion<'_> {
    pub fn set_index(&mut self, index: i32) {
        unsafe {
            (*self.inner).use_index = index;
        }
    }

    pub fn index(&self) -> i32 {
        unsafe { (*self.inner).use_index }
    }

    pub fn providers(&self) -> AlpmList<&Package> {
        let list = unsafe { (*self.inner).providers };
        unsafe { AlpmList::from_ptr(list) }
    }

    pub fn depend(&self) -> &Dep {
        unsafe { Dep::from_ptr((*self.inner).depend) }
    }
}

impl ImportKeyQuestion<'_> {
    pub fn set_import(&mut self, import: bool) {
        unsafe {
            if import {
                (*self.inner).import = 1;
            } else {
                (*self.inner).import = 0;
            }
        }
    }

    pub fn import(&self) -> bool {
        unsafe { (*self.inner).import != 0 }
    }

    pub fn uid(&self) -> &str {
        unsafe { from_cstr((*self.inner).uid) }
    }

    pub fn fingerprint(&self) -> &str {
        unsafe { from_cstr((*self.inner).fingerprint) }
    }
}
