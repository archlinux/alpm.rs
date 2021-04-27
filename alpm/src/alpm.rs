use crate::utils::*;
use crate::{
    DlCbTrait, Error, EventCbTrait, FetchCbTrait, LogCbTrait, ProgressCbTrait, QuestionCbTrait,
    Result,
};

use std::ffi::{c_void, CString};
use std::os::raw::c_int;

use alpm_sys::*;
use bitflags::bitflags;

extern "C" {
    pub(crate) fn free(ptr: *mut c_void);
}

pub struct Alpm {
    pub(crate) handle: *mut alpm_handle_t,
    pub(crate) logcb: Option<*mut dyn LogCbTrait>,
    pub(crate) dlcb: Option<*mut dyn DlCbTrait>,
    pub(crate) eventcb: Option<*mut dyn EventCbTrait>,
    pub(crate) progresscb: Option<*mut dyn ProgressCbTrait>,
    pub(crate) questioncb: Option<*mut dyn QuestionCbTrait>,
    pub(crate) fetchcb: Option<*mut dyn FetchCbTrait>,
}

impl std::fmt::Debug for Alpm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Alpm")
    }
}

unsafe impl Send for Alpm {}

impl Drop for Alpm {
    fn drop(&mut self) {
        unsafe { alpm_trans_release(self.handle) };
        unsafe { alpm_release(self.handle) };

        unsafe { self.logcb.map(|cb| Box::from_raw(cb)) };
        unsafe { self.dlcb.map(|cb| Box::from_raw(cb)) };
        unsafe { self.eventcb.map(|cb| Box::from_raw(cb)) };
        unsafe { self.progresscb.map(|cb| Box::from_raw(cb)) };
        unsafe { self.questioncb.map(|cb| Box::from_raw(cb)) };
        unsafe { self.fetchcb.map(|cb| Box::from_raw(cb)) };
    }
}

impl Alpm {
    pub fn new<S: Into<Vec<u8>>>(root: S, db_path: S) -> Result<Alpm> {
        let mut err = alpm_errno_t::ALPM_ERR_OK;
        let root = CString::new(root).unwrap();
        let db_path = CString::new(db_path).unwrap();

        let handle = unsafe { alpm_initialize(root.as_ptr(), db_path.as_ptr(), &mut err) };

        if handle.is_null() {
            unsafe { return Err(Error::new(err)) };
        }

        Ok(Alpm {
            handle,
            logcb: None,
            dlcb: None,
            eventcb: None,
            progresscb: None,
            questioncb: None,
            fetchcb: None,
        })
    }

    pub(crate) unsafe fn from_ptr(handle: *mut alpm_handle_t) -> Alpm {
        Alpm {
            handle,
            logcb: None,
            dlcb: None,
            eventcb: None,
            progresscb: None,
            questioncb: None,
            fetchcb: None,
        }
    }

    pub(crate) fn check_ret(&self, int: c_int) -> Result<()> {
        if int != 0 {
            Err(self.last_error())
        } else {
            Ok(())
        }
    }

    pub(crate) fn check_null<T>(&self, ptr: *const T) -> Result<()> {
        if ptr.is_null() {
            Err(self.last_error())
        } else {
            Ok(())
        }
    }
}

pub fn version() -> &'static str {
    unsafe { from_cstr(alpm_version()) }
}

bitflags! {
    pub struct Capabilities: u32 {
        const NLS = alpm_caps::ALPM_CAPABILITY_NLS;
        const DOWNLOADER = alpm_caps::ALPM_CAPABILITY_DOWNLOADER;
        const SIGNATURES = alpm_caps::ALPM_CAPABILITY_SIGNATURES;
    }
}

impl Default for Capabilities {
    fn default() -> Capabilities {
        Capabilities::new()
    }
}

impl Capabilities {
    pub fn new() -> Capabilities {
        Capabilities::from_bits(unsafe { alpm_capabilities() as u32 }).unwrap()
    }

    pub fn nls(self) -> bool {
        self.intersects(Capabilities::NLS)
    }

    pub fn downloader(self) -> bool {
        self.intersects(Capabilities::DOWNLOADER)
    }

    pub fn signatures(self) -> bool {
        self.intersects(Capabilities::SIGNATURES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        log_action, AnyDownloadEvent, AnyEvent, AnyQuestion, DownloadEvent, Event, FetchResult,
        LogLevel, Progress, Question, SigLevel,
    };

    fn logcb(_level: LogLevel, msg: &str, data: &mut u32) {
        print!("log {} {}", data, msg);
        *data += 1;
    }

    fn eventcb(event: AnyEvent, _: &mut ()) {
        match event.event() {
            Event::DatabaseMissing(x) => println!("missing database: {}", x.dbname()),
            _ => println!("event: {:?}", event),
        }
    }

    fn fetchcb(_url: &str, _path: &str, _force: bool, _: &mut ()) -> FetchResult {
        FetchResult::Ok
    }

    fn questioncb(question: AnyQuestion, _: &mut ()) {
        println!("question {:?}", question);
        match question.question() {
            Question::Conflict(x) => {
                let c = x.conflict();
                println!("CONFLICT BETWEEN {} AND {}", c.package1(), c.package2(),);
                println!("conflict: {}", c.reason());
            }
            _ => (),
        }
    }

    fn downloadcb(filename: &str, download: AnyDownloadEvent, _: &mut ()) {
        match download.event() {
            DownloadEvent::Init(init) => {
                println!("init: file={} optional={}", filename, init.optional)
            }
            DownloadEvent::Completed(comp) => println!(
                "complete: file={} total={} result={:?}",
                filename, comp.total, comp.result
            ),
            _ => (),
        }
    }

    fn progresscb(
        progress: Progress,
        pkgname: &str,
        percent: i32,
        howmany: usize,
        current: usize,
        _: &mut (),
    ) {
        println!(
            "progress {:?}, {} {} {} {}",
            progress, pkgname, percent, howmany, current
        );
    }

    #[test]
    fn test_capabilities() {
        let _caps = Capabilities::new();
    }

    #[test]
    fn test_init() {
        let _handle = Alpm::new("/", "tests/db").unwrap();
    }

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_cb() {
        let mut handle = Alpm::new("/", "tests/db").unwrap();

        handle.set_use_syslog(true);
        handle.set_logfile("tests/log").unwrap();
        handle.set_logcb(0, logcb);
        handle.set_eventcb((), eventcb);
        handle.set_fetchcb((), fetchcb);
        handle.set_questioncb((), questioncb);
        handle.set_dlcb((), downloadcb);
        handle.set_progresscb((), progresscb);

        log_action!(handle, "me", "look i am logging an action {}", ":D").unwrap();

        let db = handle.register_syncdb_mut("core", SigLevel::NONE).unwrap();
        db.add_server("https://ftp.rnl.tecnico.ulisboa.pt/pub/archlinux/core/os/x86_64")
            .unwrap();
        db.pkg("filesystem").unwrap();
    }

    #[test]
    fn test_lifetime() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let name = pkg.name();

        drop(pkg);
        drop(db);
        assert_eq!(name, "linux");
    }

    #[test]
    fn test_list_lifetime() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkgs = db.pkgs();

        drop(db);
        assert!(pkgs.len() > 10);
    }
}
