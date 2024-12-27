use crate::{free, Alpm, AnyDownloadEvent, AnyEvent, AnyQuestion, FetchResult, LogLevel, Progress};
use alpm_sys::*;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::mem::transmute;
use std::os::raw::{c_char, c_int};
use std::{fmt, panic, ptr};

extern "C" {
    fn vasprintf(str: *const *mut c_char, fmt: *const c_char, args: VaList) -> c_int;
}

#[cfg(not(all(
    feature = "generate",
    any(target_arch = "arm", target_arch = "aarch64")
)))]
pub type VaList = *mut __va_list_tag;
#[cfg(all(
    feature = "generate",
    any(target_arch = "arm", target_arch = "aarch64")
))]
pub type VaList = va_list;

type Cb<T> = UnsafeCell<Option<Box<T>>>;

#[derive(Default)]
pub(crate) struct Callbacks {
    pub(crate) log: Cb<dyn LogCbTrait>,
    pub(crate) dl: Cb<dyn DlCbTrait>,
    pub(crate) event: Cb<dyn EventCbTrait>,
    pub(crate) progress: Cb<dyn ProgressCbTrait>,
    pub(crate) question: Cb<dyn QuestionCbTrait>,
    pub(crate) fetch: Cb<dyn FetchCbTrait>,
}

pub(crate) trait LogCbTrait {
    fn call(&self, level: LogLevel, s: &str);
    fn assert_unlocked(&self);
}

pub(crate) trait DlCbTrait {
    fn call(&self, filename: &str, event: AnyDownloadEvent);
    fn assert_unlocked(&self);
}

pub(crate) trait EventCbTrait {
    fn call(&self, event: AnyEvent);
    fn assert_unlocked(&self);
}

pub(crate) trait ProgressCbTrait {
    fn call(&self, progress: Progress, pkgname: &str, percent: i32, howmany: usize, current: usize);
    fn assert_unlocked(&self);
}

pub(crate) trait QuestionCbTrait {
    fn call(&self, question: AnyQuestion);
    fn assert_unlocked(&self);
}

pub(crate) trait FetchCbTrait {
    fn call(&self, url: &str, filename: &str, force: bool) -> FetchResult;
    fn assert_unlocked(&self);
}

struct LogCbImpl<T, F>(RefCell<(F, T)>);

impl<T, F: FnMut(LogLevel, &str, &mut T)> LogCbTrait for LogCbImpl<T, F> {
    fn call(&self, level: LogLevel, s: &str) {
        let mut cb = self.0.borrow_mut();
        let cb = &mut *cb;
        (cb.0)(level, s, &mut cb.1)
    }
    fn assert_unlocked(&self) {
        self.0.try_borrow_mut().expect("callback is in use");
    }
}

struct DlCbImpl<T, F>(RefCell<(F, T)>);

impl<T, F: FnMut(&str, AnyDownloadEvent, &mut T)> DlCbTrait for DlCbImpl<T, F> {
    fn call(&self, s: &str, event: AnyDownloadEvent) {
        let mut cb = self.0.borrow_mut();
        let cb = &mut *cb;
        (cb.0)(s, event, &mut cb.1)
    }
    fn assert_unlocked(&self) {
        self.0.try_borrow_mut().expect("callback is in use");
    }
}

struct EventCbImpl<T, F>(RefCell<(F, T)>);

impl<T, F: FnMut(AnyEvent, &mut T)> EventCbTrait for EventCbImpl<T, F> {
    fn call(&self, event: AnyEvent) {
        let mut cb = self.0.borrow_mut();
        let cb = &mut *cb;
        (cb.0)(event, &mut cb.1)
    }

    fn assert_unlocked(&self) {
        self.0.try_borrow_mut().expect("callback is in use");
    }
}

struct ProgressCbImpl<T, F>(RefCell<(F, T)>);

impl<T, F: FnMut(Progress, &str, i32, usize, usize, &mut T)> ProgressCbTrait
    for ProgressCbImpl<T, F>
{
    fn call(
        &self,
        progress: Progress,
        pkgname: &str,
        percent: i32,
        howmany: usize,
        current: usize,
    ) {
        let mut cb = self.0.borrow_mut();
        let cb = &mut *cb;
        (cb.0)(progress, pkgname, percent, howmany, current, &mut cb.1)
    }
    fn assert_unlocked(&self) {
        self.0.try_borrow_mut().expect("callback is in use");
    }
}

struct QuestionCbImpl<T, F>(RefCell<(F, T)>);

impl<T, F: FnMut(AnyQuestion, &mut T)> QuestionCbTrait for QuestionCbImpl<T, F> {
    fn call(&self, question: AnyQuestion) {
        let mut cb = self.0.borrow_mut();
        let cb = &mut *cb;
        (cb.0)(question, &mut cb.1)
    }
    fn assert_unlocked(&self) {
        self.0.try_borrow_mut().expect("callback is in use");
    }
}

struct FetchCbImpl<T, F>(RefCell<(F, T)>);

impl<T, F: FnMut(&str, &str, bool, &mut T) -> FetchResult> FetchCbTrait for FetchCbImpl<T, F> {
    fn call(&self, url: &str, filename: &str, force: bool) -> FetchResult {
        let mut cb = self.0.borrow_mut();
        let cb = &mut *cb;
        (cb.0)(url, filename, force, &mut cb.1)
    }

    fn assert_unlocked(&self) {
        self.0.try_borrow_mut().expect("callback is in use");
    }
}

pub struct RawLogCb {
    pub(crate) raw: alpm_cb_log,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<dyn LogCbTrait>>,
}

impl fmt::Debug for RawLogCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawLogCb")
    }
}

pub struct RawDlCb {
    pub(crate) raw: alpm_cb_download,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<dyn DlCbTrait>>,
}

impl fmt::Debug for RawDlCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawDlCb")
    }
}

pub struct RawEventCb {
    pub(crate) raw: alpm_cb_event,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<dyn EventCbTrait>>,
}

impl fmt::Debug for RawEventCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawEventCb")
    }
}

pub struct RawProgressCb {
    pub(crate) raw: alpm_cb_progress,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<dyn ProgressCbTrait>>,
}

impl fmt::Debug for RawProgressCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawProgressCb")
    }
}

pub struct RawQuestionCb {
    pub(crate) raw: alpm_cb_question,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<dyn QuestionCbTrait>>,
}

impl fmt::Debug for RawQuestionCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawQuestionCb")
    }
}

pub struct RawFetchCb {
    pub(crate) raw: alpm_cb_fetch,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<dyn FetchCbTrait>>,
}

impl fmt::Debug for RawFetchCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawFetchCb")
    }
}

impl Alpm {
    pub fn set_log_cb<T: 'static, F: FnMut(LogLevel, &str, &mut T) + 'static>(
        &self,
        data: T,
        f: F,
    ) {
        let c = unsafe { &mut *self.cbs.log.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let ctx = LogCbImpl(RefCell::new((f, data)));
        let ctx = Box::new(ctx);
        let cb = logcb::<LogCbImpl<T, F>>;
        unsafe { alpm_option_set_logcb(self.as_ptr(), Some(cb), &*ctx as *const _ as *mut _) };
        c.replace(ctx);
    }

    pub fn set_dl_cb<T: 'static, F: FnMut(&str, AnyDownloadEvent, &mut T) + 'static>(
        &self,
        data: T,
        f: F,
    ) {
        let c = unsafe { &mut *self.cbs.dl.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let ctx = DlCbImpl(RefCell::new((f, data)));
        let ctx = Box::new(ctx);
        let cb = dlcb::<DlCbImpl<T, F>>;
        unsafe { alpm_option_set_dlcb(self.as_ptr(), Some(cb), &*ctx as *const _ as *mut _) };
        c.replace(ctx);
    }

    pub fn set_event_cb<T: 'static, F: FnMut(AnyEvent, &mut T) + 'static>(&self, data: T, f: F) {
        let c = unsafe { &mut *self.cbs.event.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let ctx = EventCbImpl(RefCell::new((f, data)));
        let ctx = Box::new(ctx);
        let cb = eventcb::<EventCbImpl<T, F>>;
        unsafe { alpm_option_set_eventcb(self.as_ptr(), Some(cb), &*ctx as *const _ as *mut _) };
        c.replace(ctx);
    }

    pub fn set_progress_cb<
        T: 'static,
        F: FnMut(Progress, &str, i32, usize, usize, &mut T) + 'static,
    >(
        &self,
        data: T,
        f: F,
    ) {
        let c = unsafe { &mut *self.cbs.progress.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let ctx = ProgressCbImpl(RefCell::new((f, data)));
        let ctx = Box::new(ctx);
        let cb = progresscb::<ProgressCbImpl<T, F>>;
        unsafe { alpm_option_set_progresscb(self.as_ptr(), Some(cb), &*ctx as *const _ as *mut _) };
        c.replace(ctx);
    }

    pub fn set_question_cb<T: 'static, F: FnMut(AnyQuestion, &mut T) + 'static>(
        &self,
        data: T,
        f: F,
    ) {
        let c = unsafe { &mut *self.cbs.question.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let ctx = QuestionCbImpl(RefCell::new((f, data)));
        let ctx = Box::new(ctx);
        let cb = questioncb::<QuestionCbImpl<T, F>>;
        unsafe { alpm_option_set_questioncb(self.as_ptr(), Some(cb), &*ctx as *const _ as *mut _) };
        c.replace(ctx);
    }

    pub fn set_fetch_cb<T: 'static, F: FnMut(&str, &str, bool, &mut T) -> FetchResult + 'static>(
        &self,
        data: T,
        f: F,
    ) {
        let c = unsafe { &mut *self.cbs.fetch.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let ctx = FetchCbImpl(RefCell::new((f, data)));
        let ctx = Box::new(ctx);
        let cb = fetchcb::<FetchCbImpl<T, F>>;
        unsafe { alpm_option_set_fetchcb(self.as_ptr(), Some(cb), &*ctx as *const _ as *mut _) };
        c.replace(ctx);
    }

    pub fn take_raw_log_cb(&self) -> RawLogCb {
        let c = unsafe { &mut *self.cbs.log.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        let cb = RawLogCb {
            ctx: unsafe { alpm_option_get_logcb_ctx(self.as_ptr()) },
            raw: unsafe { alpm_option_get_logcb(self.as_ptr()) },
            cb: c.take(),
        };
        unsafe { alpm_option_set_logcb(self.as_ptr(), None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_log_cb(&self, cb: RawLogCb) {
        let c = unsafe { &mut *self.cbs.log.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        unsafe { alpm_option_set_logcb(self.as_ptr(), cb.raw, cb.ctx) };
        *c = cb.cb
    }

    pub fn take_raw_dl_cb(&self) -> RawDlCb {
        let c = unsafe { &mut *self.cbs.dl.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let cb = RawDlCb {
            ctx: unsafe { alpm_option_get_dlcb_ctx(self.as_ptr()) },
            raw: unsafe { alpm_option_get_dlcb(self.as_ptr()) },
            cb: c.take(),
        };
        unsafe { alpm_option_set_dlcb(self.as_ptr(), None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_dl_cb(&self, cb: RawDlCb) {
        let c = unsafe { &mut *self.cbs.dl.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        unsafe { alpm_option_set_dlcb(self.as_ptr(), cb.raw, cb.ctx) };
        *c = cb.cb
    }

    pub fn take_raw_event_cb(&self) -> RawEventCb {
        let c = unsafe { &mut *self.cbs.event.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }
        let cb = RawEventCb {
            ctx: unsafe { alpm_option_get_eventcb_ctx(self.as_ptr()) },
            raw: unsafe { alpm_option_get_eventcb(self.as_ptr()) },
            cb: c.take(),
        };
        unsafe { alpm_option_set_eventcb(self.as_ptr(), None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_event_cb(&self, cb: RawEventCb) {
        let c = unsafe { &mut *self.cbs.event.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        unsafe { alpm_option_set_eventcb(self.as_ptr(), cb.raw, cb.ctx) };
        *c = cb.cb
    }

    pub fn take_raw_progress_cb(&self) -> RawProgressCb {
        let c = unsafe { &mut *self.cbs.progress.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        let cb = RawProgressCb {
            ctx: unsafe { alpm_option_get_progresscb_ctx(self.as_ptr()) },
            raw: unsafe { alpm_option_get_progresscb(self.as_ptr()) },
            cb: c.take(),
        };
        unsafe { alpm_option_set_progresscb(self.as_ptr(), None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_progress_cb(&self, cb: RawProgressCb) {
        let c = unsafe { &mut *self.cbs.progress.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        unsafe { alpm_option_set_progresscb(self.as_ptr(), cb.raw, cb.ctx) };
        *c = cb.cb;
    }

    pub fn take_raw_question_cb(&self) -> RawQuestionCb {
        let c = unsafe { &mut *self.cbs.question.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        let cb = RawQuestionCb {
            ctx: unsafe { alpm_option_get_questioncb_ctx(self.as_ptr()) },
            raw: unsafe { alpm_option_get_questioncb(self.as_ptr()) },
            cb: c.take(),
        };
        unsafe { alpm_option_set_questioncb(self.as_ptr(), None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_question_cb(&self, cb: RawQuestionCb) {
        let c = unsafe { &mut *self.cbs.question.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        unsafe { alpm_option_set_questioncb(self.as_ptr(), cb.raw, cb.ctx) };
        *c = cb.cb;
    }

    pub fn take_raw_fetch_cb(&self) -> RawFetchCb {
        let c = unsafe { &mut *self.cbs.fetch.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        let cb = RawFetchCb {
            ctx: unsafe { alpm_option_get_fetchcb_ctx(self.as_ptr()) },
            raw: unsafe { alpm_option_get_fetchcb(self.as_ptr()) },
            cb: c.take(),
        };

        unsafe { alpm_option_set_fetchcb(self.as_ptr(), None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_fetch_cb(&self, cb: RawFetchCb) {
        let c = unsafe { &mut *self.cbs.fetch.get() };
        if let Some(cb) = c.as_ref() {
            cb.assert_unlocked()
        }

        unsafe { alpm_option_set_fetchcb(self.as_ptr(), cb.raw, cb.ctx) };
        *c = cb.cb;
    }
}

extern "C" fn logcb<C: LogCbTrait>(
    ctx: *mut c_void,
    level: alpm_loglevel_t,
    fmt: *const c_char,
    args: VaList,
) {
    let buff = ptr::null_mut();
    let n = unsafe { vasprintf(&buff, fmt, args) };
    if n != -1 {
        let _ = panic::catch_unwind(|| {
            let s = unsafe { CStr::from_ptr(buff) };
            let level = LogLevel::from_bits(level).unwrap();
            let cb = unsafe { &*(ctx as *const C) };
            cb.call(level, &s.to_string_lossy());
        });

        unsafe { free(buff as *mut c_void) };
    }
}

extern "C" fn dlcb<C: DlCbTrait>(
    ctx: *mut c_void,
    filename: *const c_char,
    event: alpm_download_event_type_t,
    data: *mut c_void,
) {
    let _ = panic::catch_unwind(|| {
        let filename = unsafe { CStr::from_ptr(filename) };
        let filename = filename.to_str().unwrap();
        let event = unsafe { AnyDownloadEvent::new(event, data) };
        let cb = unsafe { &*(ctx as *const C) };
        cb.call(filename, event);
    });
}

extern "C" fn fetchcb<C: FetchCbTrait>(
    ctx: *mut c_void,
    url: *const c_char,
    localpath: *const c_char,
    force: c_int,
) -> c_int {
    let ret = panic::catch_unwind(|| {
        let url = unsafe { CStr::from_ptr(url).to_str().unwrap() };
        let localpath = unsafe { CStr::from_ptr(localpath).to_str().unwrap() };
        let cb = unsafe { &*(ctx as *const C) };
        let ret = cb.call(url, localpath, force != 0);

        match ret {
            FetchResult::Ok => 0,
            FetchResult::Err => -1,
            FetchResult::FileExists => 1,
        }
    });

    ret.unwrap_or(-1)
}

extern "C" fn eventcb<C: EventCbTrait>(ctx: *mut c_void, event: *mut alpm_event_t) {
    let _ = panic::catch_unwind(|| {
        let cb = unsafe { &*(ctx as *const C) };

        let event = unsafe { AnyEvent::new(event) };
        cb.call(event);
    });
}

extern "C" fn questioncb<C: QuestionCbTrait>(ctx: *mut c_void, question: *mut alpm_question_t) {
    let _ = panic::catch_unwind(|| {
        let cb = unsafe { &*(ctx as *const C) };
        let question = unsafe { AnyQuestion::new(question) };
        cb.call(question);
    });
}

extern "C" fn progresscb<C: ProgressCbTrait>(
    ctx: *mut c_void,
    progress: alpm_progress_t,
    pkgname: *const c_char,
    percent: c_int,
    howmany: usize,
    current: usize,
) {
    let _ = panic::catch_unwind(|| {
        let pkgname = unsafe { CStr::from_ptr(pkgname) };
        let pkgname = pkgname.to_str().unwrap();
        let progress = unsafe { transmute::<alpm_progress_t, Progress>(progress) };
        let cb = unsafe { &*(ctx as *const C) };
        #[allow(clippy::unnecessary_cast)]
        cb.call(progress, pkgname, percent as i32, howmany, current);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        log_action, version, AnyDownloadEvent, AnyEvent, AnyQuestion, Capabilities, DownloadEvent,
        Event, FetchResult, Progress, Question, SigLevel,
    };
    use std::cell::Cell;
    use std::rc::Rc;

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
                println!(
                    "CONFLICT BETWEEN {} AND {}",
                    c.package1().name(),
                    c.package2().name()
                );
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
        handle.set_log_cb(0, |_, msg, data| {
            print!("log {} {}", data, msg);
            *data += 1;
        });
        handle.set_event_cb((), eventcb);
        handle.set_fetch_cb((), fetchcb);
        handle.set_question_cb((), questioncb);
        handle.set_dl_cb((), downloadcb);
        handle.set_progress_cb((), progresscb);

        log_action!(handle, "me", "look i am logging an action {}", ":D").unwrap();
        handle
            .log_action("me", "look i am logging an action 2")
            .unwrap();
        handle
            .log_action("me", "look i am logging an action 2")
            .unwrap();
        handle
            .log_action("me", "look i am logging an action 2")
            .unwrap();
        handle
            .log_action("me", "look i am logging an action 2")
            .unwrap();
        handle
            .log_action("me", "look i am logging an action 2")
            .unwrap();

        let db = handle.register_syncdb_mut("core", SigLevel::NONE).unwrap();
        db.add_server("https://ftp.rnl.tecnico.ulisboa.pt/pub/archlinux/core/os/x86_64")
            .unwrap();
        db.pkg("filesystem").unwrap();
    }

    #[test]
    fn test_cb_data() {
        let handle = Alpm::new("/", "tests/db").unwrap();

        let data = Rc::new(Cell::new(0));

        handle.set_log_cb(data.clone(), |_, _, data| data.set(7));
        handle.register_syncdb("core", SigLevel::NONE).unwrap();

        assert_eq!(data.get(), 7);
    }

    #[test]
    fn test_cb_refcell1() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let handle = Rc::new(handle);

        handle.set_log_cb(Rc::downgrade(&handle), |_, msg, data| {
            let handle = data.upgrade().unwrap();
            println!("{} {:?}", msg, handle);
            handle.take_raw_log_cb();
        });
        handle.register_syncdb("core", SigLevel::NONE).unwrap();
    }

    #[test]
    fn test_cb_refcell2() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let handle = Rc::new(handle);

        handle.set_log_cb(Rc::downgrade(&handle), |_, msg, data| {
            let handle = data.upgrade().unwrap();
            println!("{} {:?}", msg, handle);
            handle.set_log_cb((), |_, _, _| {});
        });
        handle.register_syncdb("core", SigLevel::NONE).unwrap();
    }

    #[ignore]
    #[test]
    fn test_cb_refcell_mut() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let handle = Rc::new(RefCell::new(handle));
        let borrow = handle.borrow();
        let db = borrow.register_syncdb("core", SigLevel::NONE).unwrap();

        handle
            .borrow()
            .set_log_cb(Rc::clone(&handle), |_, msg, data| {
                let handle = data;
                println!("{} {:?}", msg, handle);
                handle.borrow_mut().unregister_all_syncdbs().unwrap();
                println!("Done");
            });

        println!("{:?}", db.pkg("linux"));
        assert_eq!(handle.borrow().syncdbs().len(), 1);
    }

    #[test]
    fn test_cb_drop() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let mut val = Rc::new(42);
        handle.set_log_cb(Rc::clone(&val), |_, _, _| ());
        assert!(Rc::get_mut(&mut val).is_none());
        let cb = handle.take_raw_log_cb();
        assert!(Rc::get_mut(&mut val).is_none());
        drop(cb);
        Rc::get_mut(&mut val).unwrap();
        drop(handle);
    }
}
