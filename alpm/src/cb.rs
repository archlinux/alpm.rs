use crate::{free, Alpm, AnyDownloadEvent, AnyEvent, AnyQuestion, FetchResult, LogLevel, Progress};
use alpm_sys::*;
use std::cell::UnsafeCell;
use std::ffi::{c_void, CStr};
use std::mem::transmute;
use std::os::raw::{c_char, c_int};
use std::{fmt, panic, ptr};

extern "C" {
    fn vasprintf(str: *const *mut c_char, fmt: *const c_char, args: *mut __va_list_tag) -> c_int;
}

pub(crate) trait LogCbTrait {
    fn call(&mut self, level: LogLevel, s: &str);
}

pub(crate) trait DlCbTrait {
    fn call(&mut self, filename: &str, event: AnyDownloadEvent);
}

pub(crate) trait EventCbTrait {
    fn call(&mut self, event: AnyEvent);
    fn handle(&self) -> *mut alpm_handle_t;
}

pub(crate) trait ProgressCbTrait {
    fn call(
        &mut self,
        progress: Progress,
        pkgname: &str,
        percent: i32,
        howmany: usize,
        current: usize,
    );
}

pub(crate) trait QuestionCbTrait {
    fn call(&mut self, question: AnyQuestion);
    fn handle(&self) -> *mut alpm_handle_t;
}

pub(crate) trait FetchCbTrait {
    fn call(&mut self, url: &str, filename: &str, force: bool) -> FetchResult;
}

struct LogCbImpl<T, F> {
    cb: F,
    data: T,
}

impl<T, F: FnMut(LogLevel, &str, &mut T)> LogCbTrait for LogCbImpl<T, F> {
    fn call(&mut self, level: LogLevel, s: &str) {
        (self.cb)(level, s, &mut self.data)
    }
}

struct DlCbImpl<T, F> {
    cb: F,
    data: T,
}

impl<T, F: FnMut(&str, AnyDownloadEvent, &mut T)> DlCbTrait for DlCbImpl<T, F> {
    fn call(&mut self, s: &str, event: AnyDownloadEvent) {
        (self.cb)(s, event, &mut self.data)
    }
}

struct EventCbImpl<T, F> {
    cb: F,
    data: T,
    handle: *mut alpm_handle_t,
}

impl<T, F: FnMut(AnyEvent, &mut T)> EventCbTrait for EventCbImpl<T, F> {
    fn call(&mut self, event: AnyEvent) {
        (self.cb)(event, &mut self.data)
    }
    fn handle(&self) -> *mut alpm_handle_t {
        self.handle
    }
}

struct ProgressCbImpl<T, F> {
    cb: F,
    data: T,
}

impl<T, F: FnMut(Progress, &str, i32, usize, usize, &mut T)> ProgressCbTrait
    for ProgressCbImpl<T, F>
{
    fn call(
        &mut self,
        progress: Progress,
        pkgname: &str,
        percent: i32,
        howmany: usize,
        current: usize,
    ) {
        (self.cb)(progress, pkgname, percent, howmany, current, &mut self.data)
    }
}

struct QuestionCbImpl<T, F> {
    cb: F,
    data: T,
    handle: *mut alpm_handle_t,
}

impl<T, F: FnMut(AnyQuestion, &mut T)> QuestionCbTrait for QuestionCbImpl<T, F> {
    fn call(&mut self, question: AnyQuestion) {
        (self.cb)(question, &mut self.data)
    }
    fn handle(&self) -> *mut alpm_handle_t {
        self.handle
    }
}

struct FetchCbImpl<T, F> {
    cb: F,
    data: T,
}

impl<T, F: FnMut(&str, &str, bool, &mut T) -> FetchResult> FetchCbTrait for FetchCbImpl<T, F> {
    fn call(&mut self, url: &str, filename: &str, force: bool) -> FetchResult {
        (self.cb)(url, filename, force, &mut self.data)
    }
}

pub struct RawLogCb {
    pub(crate) raw: alpm_cb_log,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<UnsafeCell<dyn LogCbTrait>>>,
}

impl fmt::Debug for RawLogCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawLogCb")
    }
}

pub struct RawDlCb {
    pub(crate) raw: alpm_cb_download,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<UnsafeCell<dyn DlCbTrait>>>,
}

impl fmt::Debug for RawDlCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawDlCb")
    }
}

pub struct RawEventCb {
    pub(crate) raw: alpm_cb_event,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<UnsafeCell<dyn EventCbTrait>>>,
}

impl fmt::Debug for RawEventCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawEventCb")
    }
}

pub struct RawProgressCb {
    pub(crate) raw: alpm_cb_progress,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<UnsafeCell<dyn ProgressCbTrait>>>,
}
impl fmt::Debug for RawProgressCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawProgressCb")
    }
}

pub struct RawQuestionCb {
    pub(crate) raw: alpm_cb_question,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<UnsafeCell<dyn QuestionCbTrait>>>,
}

impl fmt::Debug for RawQuestionCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawQuestionCb")
    }
}

pub struct RawFetchCb {
    pub(crate) raw: alpm_cb_fetch,
    pub(crate) ctx: *mut c_void,
    pub(crate) cb: Option<Box<UnsafeCell<dyn FetchCbTrait>>>,
}
impl fmt::Debug for RawFetchCb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RawFetchCb")
    }
}

impl RawLogCb {
    pub fn none() -> Self {
        RawLogCb {
            raw: None,
            ctx: ptr::null_mut(),
            cb: None,
        }
    }
}

impl RawDlCb {
    pub fn none() -> Self {
        RawDlCb {
            raw: None,
            ctx: ptr::null_mut(),
            cb: None,
        }
    }
}

impl RawEventCb {
    pub fn none() -> Self {
        RawEventCb {
            raw: None,
            ctx: ptr::null_mut(),
            cb: None,
        }
    }
}

impl RawProgressCb {
    pub fn none() -> Self {
        RawProgressCb {
            raw: None,
            ctx: ptr::null_mut(),
            cb: None,
        }
    }
}

impl RawQuestionCb {
    pub fn none() -> Self {
        RawQuestionCb {
            raw: None,
            ctx: ptr::null_mut(),
            cb: None,
        }
    }
}

impl RawFetchCb {
    pub fn none() -> Self {
        RawFetchCb {
            raw: None,
            ctx: ptr::null_mut(),
            cb: None,
        }
    }
}

impl Alpm {
    pub fn set_log_cb<T: Send + 'static, F: FnMut(LogLevel, &str, &mut T) + Send + 'static>(
        &mut self,
        data: T,
        f: F,
    ) {
        let ctx = LogCbImpl { cb: f, data };
        let ctx = Box::new(UnsafeCell::new(ctx));
        let cb = logcb::<LogCbImpl<T, F>>;
        unsafe { alpm_option_set_logcb(self.handle, Some(cb), &*ctx as *const _ as *mut _) };
        self.cbs.get_mut().log = Some(ctx);
    }

    pub fn set_dl_cb<
        T: Send + 'static,
        F: FnMut(&str, AnyDownloadEvent, &mut T) + Send + 'static,
    >(
        &mut self,
        data: T,
        f: F,
    ) {
        let ctx = DlCbImpl { cb: f, data };
        let ctx = Box::new(UnsafeCell::new(ctx));
        let cb = dlcb::<DlCbImpl<T, F>>;
        unsafe { alpm_option_set_dlcb(self.handle, Some(cb), &*ctx as *const _ as *mut _) };
        self.cbs.get_mut().dl = Some(ctx);
    }

    pub fn set_event_cb<T: Send + 'static, F: FnMut(AnyEvent, &mut T) + Send + 'static>(
        &mut self,
        data: T,
        f: F,
    ) {
        let ctx = EventCbImpl {
            cb: f,
            data,
            handle: self.handle,
        };
        let ctx = Box::new(UnsafeCell::new(ctx));
        let cb = eventcb::<EventCbImpl<T, F>>;
        unsafe { alpm_option_set_eventcb(self.handle, Some(cb), &*ctx as *const _ as *mut _) };
        self.cbs.get_mut().event = Some(ctx);
    }

    pub fn set_progress_cb<
        T: Send + 'static,
        F: FnMut(Progress, &str, i32, usize, usize, &mut T) + Send + 'static,
    >(
        &mut self,
        data: T,
        f: F,
    ) {
        let ctx = ProgressCbImpl { cb: f, data };
        let ctx = Box::new(UnsafeCell::new(ctx));
        let cb = progresscb::<ProgressCbImpl<T, F>>;
        unsafe { alpm_option_set_progresscb(self.handle, Some(cb), &*ctx as *const _ as *mut _) };
        self.cbs.get_mut().progress = Some(ctx);
    }

    pub fn set_question_cb<T: Send + 'static, F: FnMut(AnyQuestion, &mut T) + Send + 'static>(
        &mut self,
        data: T,
        f: F,
    ) {
        let ctx = QuestionCbImpl {
            cb: f,
            data,
            handle: self.handle,
        };
        let ctx = Box::new(UnsafeCell::new(ctx));
        let cb = questioncb::<QuestionCbImpl<T, F>>;
        unsafe { alpm_option_set_questioncb(self.handle, Some(cb), &*ctx as *const _ as *mut _) };
        self.cbs.get_mut().question = Some(ctx);
    }

    pub fn set_fetch_cb<
        T: Send + 'static,
        F: FnMut(&str, &str, bool, &mut T) -> FetchResult + Send + 'static,
    >(
        &mut self,
        data: T,
        f: F,
    ) {
        let ctx = FetchCbImpl { cb: f, data };
        let ctx = Box::new(UnsafeCell::new(ctx));
        let cb = fetchcb::<FetchCbImpl<T, F>>;
        unsafe { alpm_option_set_fetchcb(self.handle, Some(cb), &*ctx as *const _ as *mut _) };
        self.cbs.get_mut().fetch = Some(ctx);
    }

    pub fn take_raw_log_cb(&self) -> RawLogCb {
        let cb = RawLogCb {
            ctx: unsafe { alpm_option_get_logcb_ctx(self.handle) },
            raw: unsafe { alpm_option_get_logcb(self.handle) },
            cb: self.cbs.borrow_mut().log.take(),
        };
        unsafe { alpm_option_set_logcb(self.handle, None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_log_cb(&self, cb: RawLogCb) {
        unsafe { alpm_option_set_logcb(self.handle, cb.raw, cb.ctx) };
        self.cbs.borrow_mut().log = cb.cb;
    }

    pub fn take_raw_dl_cb(&self) -> RawDlCb {
        let cb = RawDlCb {
            ctx: unsafe { alpm_option_get_dlcb_ctx(self.handle) },
            raw: unsafe { alpm_option_get_dlcb(self.handle) },
            cb: self.cbs.borrow_mut().dl.take(),
        };
        unsafe { alpm_option_set_dlcb(self.handle, None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_dl_cb(&self, cb: RawDlCb) {
        unsafe { alpm_option_set_dlcb(self.handle, cb.raw, cb.ctx) };
        self.cbs.borrow_mut().dl = cb.cb;
    }

    pub fn take_raw_event_cb(&self) -> RawEventCb {
        let cb = RawEventCb {
            ctx: unsafe { alpm_option_get_eventcb_ctx(self.handle) },
            raw: unsafe { alpm_option_get_eventcb(self.handle) },
            cb: self.cbs.borrow_mut().event.take(),
        };
        unsafe { alpm_option_set_eventcb(self.handle, None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_event_cb(&self, cb: RawEventCb) {
        unsafe { alpm_option_set_eventcb(self.handle, cb.raw, cb.ctx) };
        self.cbs.borrow_mut().event = cb.cb;
    }

    pub fn take_raw_progress_cb(&self) -> RawProgressCb {
        let cb = RawProgressCb {
            ctx: unsafe { alpm_option_get_progresscb_ctx(self.handle) },
            raw: unsafe { alpm_option_get_progresscb(self.handle) },
            cb: self.cbs.borrow_mut().progress.take(),
        };
        unsafe { alpm_option_set_progresscb(self.handle, None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_progress_cb(&self, cb: RawProgressCb) {
        unsafe { alpm_option_set_progresscb(self.handle, cb.raw, cb.ctx) };
        self.cbs.borrow_mut().progress = cb.cb;
    }

    pub fn take_raw_question_cb(&self) -> RawQuestionCb {
        let cb = RawQuestionCb {
            ctx: unsafe { alpm_option_get_questioncb_ctx(self.handle) },
            raw: unsafe { alpm_option_get_questioncb(self.handle) },
            cb: self.cbs.borrow_mut().question.take(),
        };
        unsafe { alpm_option_set_questioncb(self.handle, None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_question_cb(&self, cb: RawQuestionCb) {
        unsafe { alpm_option_set_questioncb(self.handle, cb.raw, cb.ctx) };
        self.cbs.borrow_mut().question = cb.cb;
    }

    pub fn take_raw_fetch_cb(&self) -> RawFetchCb {
        let cb = RawFetchCb {
            ctx: unsafe { alpm_option_get_fetchcb_ctx(self.handle) },
            raw: unsafe { alpm_option_get_fetchcb(self.handle) },
            cb: self.cbs.borrow_mut().fetch.take(),
        };

        unsafe { alpm_option_set_fetchcb(self.handle, None, ptr::null_mut()) };
        cb
    }

    pub fn set_raw_fetch_cb(&self, cb: RawFetchCb) {
        unsafe { alpm_option_set_fetchcb(self.handle, cb.raw, cb.ctx) };
        self.cbs.borrow_mut().fetch = cb.cb;
    }
}

extern "C" fn logcb<C: LogCbTrait>(
    ctx: *mut c_void,
    level: alpm_loglevel_t,
    fmt: *const c_char,
    args: *mut __va_list_tag,
) {
    let _ = panic::catch_unwind(|| {
        let buff = ptr::null_mut();
        let n = unsafe { vasprintf(&buff, fmt, args) };
        if n != -1 {
            let s = unsafe { CStr::from_ptr(buff) };
            let level = LogLevel::from_bits(level).unwrap();
            let cb = unsafe { &*(ctx as *const UnsafeCell<C>) };

            let cb = unsafe { &mut *cb.get() };
            cb.call(level, &s.to_string_lossy());
            unsafe { free(buff as *mut c_void) };
        }
    });
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
        let cb = unsafe { &*(ctx as *const UnsafeCell<C>) };
        let cb = unsafe { &mut *cb.get() };
        cb.call(&filename, event);
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
        let cb = unsafe { &*(ctx as *const UnsafeCell<C>) };
        let cb = unsafe { &mut *cb.get() };
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
        let cb = unsafe { &*(ctx as *const UnsafeCell<C>) };
        let cb = unsafe { &mut *cb.get() };

        let event = unsafe { AnyEvent::new(cb.handle(), event) };
        cb.call(event);
    });
}

extern "C" fn questioncb<C: QuestionCbTrait>(ctx: *mut c_void, question: *mut alpm_question_t) {
    let _ = panic::catch_unwind(|| {
        let cb = unsafe { &*(ctx as *const UnsafeCell<C>) };
        let cb = unsafe { &mut *cb.get() };
        let question = unsafe { AnyQuestion::new(cb.handle(), question) };
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
        let cb = unsafe { &*(ctx as *const UnsafeCell<C>) };
        let cb = unsafe { &mut *cb.get() };
        cb.call(progress, &pkgname, percent as i32, howmany, current);
    });
}
