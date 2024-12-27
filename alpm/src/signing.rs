use crate::{free, Alpm, AlpmListMut, Db, Result};
use crate::{utils::*, Pkg};

use alpm_sys::_alpm_sigstatus_t::*;
use alpm_sys::_alpm_sigvalidity_t::*;
use alpm_sys::*;

use std::ffi::{c_void, CString};
use std::mem::transmute;
use std::{fmt, ptr, slice};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct SignatureDecodeError;

impl fmt::Display for SignatureDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to decode signature")
    }
}

impl std::error::Error for SignatureDecodeError {}

pub fn decode_signature<S: Into<Vec<u8>>>(
    b64: S,
) -> std::result::Result<Vec<u8>, SignatureDecodeError> {
    let b64 = CString::new(b64).unwrap();
    let mut data = ptr::null_mut();
    let mut len = 0;
    let ret = unsafe { alpm_decode_signature(b64.as_ptr(), &mut data, &mut len) };
    if ret != 0 {
        return Err(SignatureDecodeError);
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

pub struct PgpKey {
    pub(crate) inner: alpm_pgpkey_t,
}

impl fmt::Debug for PgpKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgpKey")
            .field("name", &self.name())
            .field("email", &self.email())
            .field("uid", &self.uid())
            .field("fingerprint", &self.fingerprint())
            .field("created", &self.created())
            .field("expires", &self.expires())
            .field("length", &self.length())
            .field("revoked", &self.revoked())
            .finish()
    }
}

impl PgpKey {
    pub fn fingerprint(&self) -> &str {
        unsafe { from_cstr(self.inner.fingerprint) }
    }

    pub fn uid(&self) -> &str {
        unsafe { from_cstr(self.inner.uid) }
    }

    pub fn name(&self) -> Option<&str> {
        unsafe { from_cstr_optional(self.inner.name) }
    }

    pub fn email(&self) -> Option<&str> {
        unsafe { from_cstr_optional(self.inner.email) }
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

    #[cfg(not(feature = "git"))]
    pub fn pubkey_algo(&self) -> u8 {
        self.inner.pubkey_algo as u8
    }
}

#[repr(transparent)]
pub struct SigResult {
    inner: alpm_sigresult_t,
}

impl fmt::Debug for SigResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SigResult")
            .field("key", &self.key())
            .field("status", &self.status())
            .field("validity", &self.validity())
            .finish()
    }
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

pub struct SigList {
    inner: alpm_siglist_t,
}

impl fmt::Debug for SigList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.results()).finish()
    }
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
        if self.inner.results.is_null() {
            unsafe { slice::from_raw_parts(1 as *const SigResult, 0) }
        } else {
            unsafe {
                slice::from_raw_parts(self.inner.results as *const SigResult, self.inner.count)
            }
        }
    }
}

impl Pkg {
    pub fn check_signature(&self, siglist: &mut SigList) -> Result<()> {
        let ret = unsafe { alpm_pkg_check_pgp_signature(self.as_ptr(), &mut siglist.inner) };
        match ret {
            0 => Ok(()),
            _ => Err(self.last_error()),
        }
    }
}

impl Db {
    pub fn check_signature(&self, siglist: &mut SigList) -> Result<()> {
        let ret = unsafe { alpm_db_check_pgp_signature(self.as_ptr(), &mut siglist.inner) };
        match ret {
            0 => Ok(()),
            _ => Err(self.last_error()),
        }
    }
}

impl Alpm {
    pub fn extract_keyid<S: Into<Vec<u8>>>(
        &self,
        ident: S,
        sig: &[u8],
    ) -> Result<AlpmListMut<String>> {
        let ident = CString::new(ident).unwrap();
        let mut keys = ptr::null_mut();

        let ret = unsafe {
            alpm_extract_keyid(
                self.as_ptr(),
                ident.as_ptr(),
                sig.as_ptr(),
                sig.len(),
                &mut keys,
            )
        };

        self.check_ret(ret)?;
        unsafe { Ok(AlpmListMut::from_ptr(keys)) }
    }
}
