#[macro_export]
macro_rules! set_logcb {
    ( $handle:tt, $f:tt ) => {{
        use std::ffi::{c_void, CStr};
        use std::os::raw::{c_char, c_int};
        use std::ptr;
        use $crate::alpm_sys::*;
        use $crate::LogLevel;

        extern "C" {
            fn vasprintf(
                str: *const *mut c_char,
                fmt: *const c_char,
                args: *mut __va_list_tag,
            ) -> c_int;
            fn free(ptr: *mut c_void);
        }

        unsafe extern "C" fn c_logcb(
            #[cfg(feature = "git")] _ctx: *mut c_void,
            level: alpm_loglevel_t,
            fmt: *const c_char,
            args: *mut __va_list_tag,
        ) {
            let buff = ptr::null_mut();
            let n = vasprintf(&buff, fmt, args);
            if n != -1 {
                let s = CStr::from_ptr(buff);
                let level = LogLevel::from_bits(level).unwrap();
                $f(level, &s.to_string_lossy());
                free(buff as *mut c_void);
            }
        }

        unsafe {
            #[cfg(not(feature = "git"))]
            alpm_option_set_logcb($handle.as_alpm_handle_t(), Some(c_logcb));
            #[cfg(feature = "git")]
            alpm_option_set_logcb($handle.as_alpm_handle_t(), Some(c_logcb), ptr::null_mut());
        }
    }};
}

#[macro_export]
macro_rules! set_dlcb {
    ( $handle:tt, $f:tt ) => {{
        use std::cmp::Ordering;
        use std::ffi::{c_void, CStr};
        use std::os::raw::c_char;
        use std::ptr;
        use $crate::alpm_sys::alpm_download_event_type_t::*;
        use $crate::alpm_sys::*;
        use $crate::{
            DownloadEvent, DownloadEventCompleted, DownloadEventInit, DownloadEventProgress,
            DownloadEventRetry, DownloadResult,
        };

        unsafe extern "C" fn c_dlcb(
            _ctx: *mut c_void,
            filename: *const c_char,
            event: alpm_download_event_type_t,
            data: *mut c_void,
        ) {
            let filename = CStr::from_ptr(filename);
            let filename = filename.to_str().unwrap();

            let event = match event {
                ALPM_DOWNLOAD_INIT => {
                    let data = data as *const alpm_download_event_init_t;
                    let event = DownloadEventInit {
                        optional: (*data).optional != 0,
                    };
                    DownloadEvent::Init(event)
                }
                ALPM_DOWNLOAD_RETRY => {
                    let data = data as *const alpm_download_event_retry_t;
                    let event = DownloadEventRetry {
                        resume: (*data).resume != 0,
                    };
                    DownloadEvent::Retry(event)
                }
                ALPM_DOWNLOAD_PROGRESS => {
                    let data = data as *const alpm_download_event_progress_t;
                    let event = DownloadEventProgress {
                        downloaded: (*data).downloaded,
                        total: (*data).total,
                    };
                    DownloadEvent::Progress(event)
                }
                ALPM_DOWNLOAD_COMPLETED => {
                    let data = data as *mut alpm_download_event_completed_t;
                    let result = match (*data).result.cmp(&0) {
                        Ordering::Equal => DownloadResult::Success,
                        Ordering::Greater => DownloadResult::UpToDate,
                        Ordering::Less => DownloadResult::Failed,
                    };
                    let event = DownloadEventCompleted {
                        total: (*data).total,
                        result,
                    };
                    DownloadEvent::Completed(event)
                }
            };
            $f(&filename, event);
        }

        unsafe { alpm_option_set_dlcb($handle.as_alpm_handle_t(), Some(c_dlcb), ptr::null_mut()) };
    }};
}

#[macro_export]
macro_rules! set_fetchcb {
    ( $handle:tt, $f:tt ) => {{
        use std::ffi::CStr;
        #[cfg(feature = "git")]
        use std::os::raw::c_void;
        use std::os::raw::{c_char, c_int};
        use $crate::alpm_sys::*;
        use $crate::FetchCbReturn;

        unsafe extern "C" fn c_fetchcb(
            #[cfg(feature = "git")] _ctx: *mut c_void,
            url: *const c_char,
            localpath: *const c_char,
            force: c_int,
        ) -> c_int {
            let url = CStr::from_ptr(url).to_str().unwrap();
            let localpath = CStr::from_ptr(localpath).to_str().unwrap();
            let ret = $f(url, localpath, force != 0);

            match ret {
                FetchCbReturn::Ok => 0,
                FetchCbReturn::Err => -1,
                FetchCbReturn::FileExists => 1,
            }
        }

        unsafe {
            #[cfg(not(feature = "git"))]
            alpm_option_set_fetchcb($handle.as_alpm_handle_t(), Some(c_fetchcb));
            #[cfg(feature = "git")]
            alpm_option_set_fetchcb(
                $handle.as_alpm_handle_t(),
                Some(c_fetchcb),
                std::ptr::null_mut(),
            );
        }
    }};
}

#[macro_export]
macro_rules! set_eventcb {
    ( $handle:tt, $f:tt ) => {{
        #[cfg(feature = "git")]
        use std::os::raw::c_void;
        use std::ptr;
        use $crate::alpm_sys::*;
        use $crate::Event;

        static mut C_ALPM_HANDLE: *mut alpm_handle_t = ptr::null_mut();
        unsafe {
            C_ALPM_HANDLE = $handle.as_alpm_handle_t();
        }

        unsafe extern "C" fn c_eventcb(
            #[cfg(feature = "git")] _ctx: *mut c_void,
            event: *mut alpm_event_t,
        ) {
            let event = Event::new(C_ALPM_HANDLE, event);
            $f(&event);
        }

        unsafe {
            #[cfg(not(feature = "git"))]
            alpm_option_set_eventcb($handle.as_alpm_handle_t(), Some(c_eventcb));
            #[cfg(feature = "git")]
            alpm_option_set_eventcb(
                $handle.as_alpm_handle_t(),
                Some(c_eventcb),
                std::ptr::null_mut(),
            );
        }
    }};
}

#[macro_export]
macro_rules! set_questioncb {
    ( $handle:tt, $f:tt ) => {{
        #[cfg(feature = "git")]
        use std::os::raw::c_void;
        use std::ptr;
        use $crate::alpm_sys::*;
        use $crate::Question;

        static mut C_ALPM_HANDLE: *mut alpm_handle_t = ptr::null_mut();
        unsafe {
            C_ALPM_HANDLE = $handle.as_alpm_handle_t();
        }

        unsafe extern "C" fn c_questioncb(
            #[cfg(feature = "git")] _ctx: *mut c_void,
            question: *mut alpm_question_t,
        ) {
            let mut question = Question::new(C_ALPM_HANDLE, question);
            $f(&mut question);
        }

        unsafe {
            #[cfg(not(feature = "git"))]
            alpm_option_set_questioncb($handle.as_alpm_handle_t(), Some(c_questioncb));

            #[cfg(feature = "git")]
            alpm_option_set_questioncb(
                $handle.as_alpm_handle_t(),
                Some(c_questioncb),
                ptr::null_mut(),
            );
        }
    }};
}

#[macro_export]
macro_rules! set_progresscb {
    ( $handle:tt, $f:tt ) => {{
        use std::ffi::CStr;
        use std::mem::transmute;
        #[cfg(feature = "git")]
        use std::os::raw::c_void;
        use std::os::raw::{c_char, c_int};
        use $crate::alpm_sys::*;
        use $crate::Progress;

        unsafe extern "C" fn c_progresscb(
            #[cfg(feature = "git")] _ctx: *mut c_void,
            progress: alpm_progress_t,
            pkgname: *const c_char,
            percent: c_int,
            howmany: usize,
            current: usize,
        ) {
            let pkgname = CStr::from_ptr(pkgname);
            let pkgname = pkgname.to_str().unwrap();
            let progress = transmute::<alpm_progress_t, Progress>(progress);
            $f(progress, &pkgname, percent as i32, howmany, current);
        }

        unsafe {
            #[cfg(not(feature = "git"))]
            alpm_option_set_progresscb($handle.as_alpm_handle_t(), Some(c_progresscb));
            #[cfg(feature = "git")]
            alpm_option_set_progresscb(
                $handle.as_alpm_handle_t(),
                Some(c_progresscb),
                std::ptr::null_mut(),
            );
        }
    }};
}

#[macro_export]
macro_rules! log_action {
    ($handle:tt, $prefix:tt, $($arg:tt)*) => ({
        use $crate::alpm_sys::*;
        use ::std::ffi::CString;

        let mut s = format!($($arg)*);
        s.push('\n');
        let s = CString::new(s).unwrap();
        let p = CString::new($prefix).unwrap();

        let ret = unsafe { alpm_logaction($handle.as_alpm_handle_t(), p.as_ptr(), s.as_ptr()) };
        if ret != 0 {
            Err($handle.last_error())
        } else {
            Ok(())
        }
    })
}
