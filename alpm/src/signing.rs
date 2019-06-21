use crate::utils::*;
use crate::{free, Alpm, AlpmList, Db, FreeMethod, Package, Result};

use alpm_sys::_alpm_sigstatus_t::*;
use alpm_sys::_alpm_sigvalidity_t::*;
use alpm_sys::*;

use std::ffi::{c_void, CString};
use std::mem::transmute;
use std::{ptr, slice};

pub fn decode_signature<S: Into<String>>(b64: S) -> std::result::Result<Vec<u8>, ()> {
    let b64 = CString::new(b64.into()).unwrap();
    let mut data = ptr::null_mut();
    let mut len = 0;
    let ret = unsafe { alpm_decode_signature(b64.as_ptr(), &mut data, &mut len) };
    if ret != 0 {
        return Err(());
    }

    let buff = unsafe { slice::from_raw_parts(data, len) };
    let v = buff.to_owned();

    unsafe { free(data as *mut c_void) };
    Ok(v)
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum SigStatus {
    Valid = ALPM_SIGSTATUS_VALID as u32,
    KeyExpired = ALPM_SIGSTATUS_KEY_EXPIRED as u32,
    SigExpired = ALPM_SIGSTATUS_SIG_EXPIRED as u32,
    KeyUnknown = ALPM_SIGSTATUS_KEY_UNKNOWN as u32,
    KeyDisabled = ALPM_SIGSTATUS_KEY_DISABLED as u32,
    Invalid = ALPM_SIGSTATUS_INVALID as u32,
}

#[repr(u32)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum SigValidity {
    Full = ALPM_SIGVALIDITY_FULL as u32,
    Marginal = ALPM_SIGVALIDITY_MARGINAL as u32,
    Never = ALPM_SIGVALIDITY_NEVER as u32,
    Unknown = ALPM_SIGVALIDITY_UNKNOWN as u32,
}

#[derive(Debug)]
pub struct PgpKey {
    pub(crate) inner: alpm_pgpkey_t,
}

impl PgpKey {
    //TODO: what is data?

    pub fn fingerprint(&self) -> &str {
        unsafe { from_cstr(self.inner.fingerprint) }
    }

    pub fn uid(&self) -> &str {
        unsafe { from_cstr(self.inner.uid) }
    }

    pub fn name(&self) -> &str {
        unsafe { from_cstr(self.inner.name) }
    }

    pub fn email(&self) -> &str {
        unsafe { from_cstr(self.inner.email) }
    }

    pub fn created(&self) -> i64 {
        self.inner.created
    }

    pub fn expires(&self) -> i64 {
        self.inner.expires
    }

    pub fn length(&self) -> u32 {
        self.inner.length
    }

    pub fn revoked(&self) -> u32 {
        self.inner.revoked
    }

    pub fn pubkey_algo(&self) -> i8 {
        self.inner.pubkey_algo
    }
}

#[derive(Debug)]
pub struct SigResult {
    inner: alpm_sigresult_t,
}

impl SigResult {
    pub fn key(&self) -> PgpKey {
        PgpKey {
            inner: self.inner.key,
        }
    }

    pub fn status(&self) -> SigStatus {
        unsafe { transmute::<alpm_sigstatus_t, SigStatus>(self.inner.status) }
    }

    pub fn validity(&self) -> SigValidity {
        unsafe { transmute::<alpm_sigvalidity_t, SigValidity>(self.inner.validity) }
    }
}

#[derive(Debug)]
pub struct SigList {
    inner: alpm_siglist_t,
}

impl Drop for SigList {
    fn drop(&mut self) {
        unsafe { alpm_siglist_cleanup(&mut self.inner) };
    }
}

impl Default for SigList {
    fn default() -> SigList {
        Self::new()
    }
}

impl SigList {
    pub fn new() -> SigList {
        SigList {
            inner: alpm_siglist_t {
                count: 0,
                results: ptr::null_mut(),
            },
        }
    }

    pub fn results(&self) -> &[SigResult] {
        unsafe { slice::from_raw_parts(self.inner.results as *const SigResult, self.inner.count) }
    }
}

impl<'a> Package<'a> {
    pub fn check_signature(&self) -> Result<(bool, SigList)> {
        let mut siglist = SigList::new();
        let ret = unsafe { alpm_pkg_check_pgp_signature(self.pkg, &mut siglist.inner) };
        let valid = match ret {
            0 => true,
            1 => false,
            _ => return Err(self.handle.last_error()),
        };

        Ok((valid, siglist))
    }
}

impl<'a> Db<'a> {
    pub fn check_signature(&self) -> Result<(bool, SigList)> {
        let mut siglist = SigList::new();
        let ret = unsafe { alpm_db_check_pgp_signature(self.db, &mut siglist.inner) };
        let valid = match ret {
            0 => true,
            1 => false,
            _ => return Err(self.handle.last_error()),
        };

        Ok((valid, siglist))
    }
}

impl Alpm {
    pub fn extract_keyid<'a, S: Into<String>>(
        &'a self,
        ident: S,
        sig: &[u8],
    ) -> Result<AlpmList<'a, String>> {
        let ident = CString::new(ident.into()).unwrap();
        let mut keys = ptr::null_mut();

        let ret = unsafe {
            alpm_extract_keyid(
                self.handle,
                ident.as_ptr(),
                sig.as_ptr(),
                sig.len(),
                &mut keys,
            )
        };

        self.check_ret(ret)?;;
        Ok(AlpmList::new(self, keys, FreeMethod::FreeInner))
    }
}
