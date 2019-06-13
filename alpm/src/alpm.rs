use crate::utils::*;
use crate::{Event, FetchCBReturn, LogLevel, Progress, Question, Result};

use std::ffi::{c_void, CString};
use std::os::raw::c_int;

use alpm_sys::*;
use bitflags::bitflags;

pub type LogCB = fn(level: LogLevel, s: &str);
pub type DownloadCB = fn(filename: &str, xfered: u64, total: u64);
pub type FetchCB = fn(url: &str, filename: &str, force: bool) -> FetchCBReturn;
pub type TotalDownloadCB = fn(total: u64);
pub type EventCB = fn(event: Event);
pub type QuestionCB = fn(question: Question);
pub type ProgressCB =
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
            unsafe { alpm_release(self.handle) };
        }
    }
}

impl Alpm {
    pub fn new<S: Into<String>>(root: S, db_path: S) -> Result<Alpm> {
        let mut err = alpm_errno_t::ALPM_ERR_OK;
        let root = CString::new(root.into())?;
        let db_path = CString::new(db_path.into())?;

        let handle = unsafe { alpm_initialize(root.as_ptr(), db_path.as_ptr(), &mut err) };

        if handle.is_null() {
            Err(err)?;
        }

        Ok(Alpm { handle, drop: true })
    }

    pub fn release(self) -> Result<()> {
        self.check_ret(unsafe { alpm_release(self.handle) })
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
        TransFlag,
    };

    fn logcb(level: LogLevel, msg: &str) {
        if level == LogLevel::ERROR {
            print!("log {}", msg);
        }
    }

    fn eventcb(event: Event) {
        match event {
            Event::DatabaseMissing(x) => println!("missing database: {}", x.dbname()),
            _ => println!("event: {:?}", event),
        }
    }

    fn fetchcb(_url: &str, _path: &str, _force: bool) -> FetchCBReturn {
        FetchCBReturn::Ok
    }

    fn questioncb(question: Question) {
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
        let handle = Alpm::new("/", "tests/db").unwrap();
        let flags = TransFlag::DB_ONLY;
        set_logcb!(handle, logcb);
        set_eventcb!(handle, eventcb);
        set_fetchcb!(handle, fetchcb);
        set_questioncb!(handle, questioncb);
        set_progresscb!(handle, progresscb);

        handle.set_use_syslog(true);
        handle.set_logfile("tests/log").unwrap();

        log_action!(handle, "me", "look i am logging an action {}", ":D").unwrap();

        let mut db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        db.add_server("https://ftp.rnl.tecnico.ulisboa.pt/pub/archlinux/core/os/x86_64")
            .unwrap();
        let pkg = db.pkg("filesystem").unwrap();

        let mut trans = handle.trans(flags).unwrap();
        trans.add_pkg(pkg).unwrap();
        trans.prepare().unwrap();
        trans.commit().unwrap();
    }
}
