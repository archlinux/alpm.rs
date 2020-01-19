use crate::utils::*;
use crate::{Error, Event, FetchCbReturn, LogLevel, Progress, Question, Result};

use std::ffi::{c_void, CString};
use std::os::raw::c_int;

use alpm_sys::*;
use bitflags::bitflags;

pub type LogCb = fn(level: LogLevel, s: &str);
pub type DownloadCb = fn(filename: &str, xfered: u64, total: u64);
pub type FetchCb = fn(url: &str, filename: &str, force: bool) -> FetchCbReturn;
pub type TotalDownloadCb = fn(total: u64);
pub type EventCb = fn(event: &Event);
pub type QuestionCb = fn(question: &Question);
pub type ProgressCb =
    fn(progress: Progress, pkgname: &str, percent: i32, howmany: usize, current: usize);

extern "C" {
    pub(crate) fn free(ptr: *mut c_void);
}

#[derive(Debug)]
pub struct Alpm {
    pub(crate) handle: *mut alpm_handle_t,
    pub(crate) drop: bool,
}

impl Drop for Alpm {
    fn drop(&mut self) {
        if self.drop {
            // alpm should do this for us, but is bugged
            unsafe { alpm_trans_release(self.handle) };
            unsafe { alpm_release(self.handle) };
        }
    }
}

impl Alpm {
    pub fn new<S: Into<String>>(root: S, db_path: S) -> Result<Alpm> {
        let mut err = alpm_errno_t::ALPM_ERR_OK;
        let root = CString::new(root.into()).unwrap();
        let db_path = CString::new(db_path.into()).unwrap();

        let handle = unsafe { alpm_initialize(root.as_ptr(), db_path.as_ptr(), &mut err) };

        if handle.is_null() {
            unsafe { return Err(Error::new(err)) };
        }

        Ok(Alpm { handle, drop: true })
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
        log_action, set_eventcb, set_fetchcb, set_logcb, set_progresscb, set_questioncb, SigLevel,
    };

    fn logcb(level: LogLevel, msg: &str) {
        if level == LogLevel::ERROR {
            print!("log {}", msg);
        }
    }

    fn eventcb(event: &Event) {
        match event {
            Event::DatabaseMissing(x) => println!("missing database: {}", x.dbname()),
            _ => println!("event: {:?}", event),
        }
    }

    fn fetchcb(_url: &str, _path: &str, _force: bool) -> FetchCbReturn {
        FetchCbReturn::Ok
    }

    fn questioncb(question: &Question) {
        println!("question {:?}", question);
        match question {
            Question::Conflict(x) => {
                let c = x.conflict();
                println!("CONFLICT BETWEEN {} AND {}", c.package1(), c.package2(),);
                println!("conflict: {}", c.reason());
            }
            _ => (),
        }
    }

    fn progresscb(progress: Progress, pkgname: &str, percent: i32, howmany: usize, current: usize) {
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
        set_logcb!(handle, logcb);
        set_eventcb!(handle, eventcb);
        set_fetchcb!(handle, fetchcb);
        set_questioncb!(handle, questioncb);
        set_progresscb!(handle, progresscb);

        handle.set_use_syslog(true);
        handle.set_logfile("tests/log").unwrap();

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
        let pkgs = db.pkgs().unwrap();

        drop(db);
        assert!(pkgs.count() > 10);
    }
}
