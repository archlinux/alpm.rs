use crate::{Alpm, AlpmList, FreeMethod, Package, Result};

use alpm_sys::_alpm_transflag_t::*;
use alpm_sys::*;

use std::marker::PhantomData;
use std::ptr;

use bitflags::bitflags;

bitflags! {
    pub struct TransFlag: u32 {
        const NONE = 0;
        const NO_DEPS = ALPM_TRANS_FLAG_NODEPS;
        const NO_SAVE = ALPM_TRANS_FLAG_NOSAVE;
        const NO_DEP_VERSION = ALPM_TRANS_FLAG_NODEPVERSION;
        const CASCADE = ALPM_TRANS_FLAG_CASCADE;
        const RECURSE = ALPM_TRANS_FLAG_RECURSE;
        const DB_ONLY = ALPM_TRANS_FLAG_DBONLY;
        const ALL_DEPS = ALPM_TRANS_FLAG_ALLDEPS;
        const DOWNLOAD_ONLY = ALPM_TRANS_FLAG_DOWNLOADONLY;
        const NO_SCRIPTLET = ALPM_TRANS_FLAG_NOSCRIPTLET;
        const NO_CONFLICTS = ALPM_TRANS_FLAG_NOCONFLICTS;
        const NEEDED = ALPM_TRANS_FLAG_NEEDED;
        const ALL_EXPLICIT = ALPM_TRANS_FLAG_ALLEXPLICIT;
        const UNNEEDED = ALPM_TRANS_FLAG_UNNEEDED;
        const RECURSE_ALL = ALPM_TRANS_FLAG_RECURSEALL;
        const NO_LOCK = ALPM_TRANS_FLAG_NOLOCK;
    }
}

pub struct Trans<'a> {
    pub(crate) handle: &'a Alpm,
}

impl<'a> Drop for Trans<'a> {
    fn drop(&mut self) {
        unsafe { alpm_trans_release(self.handle.handle) };
    }
}

impl<'a> Trans<'a> {
    pub fn flags(self) -> TransFlag {
        let flags = unsafe { alpm_trans_get_flags(self.handle.handle) };
        TransFlag::from_bits(flags as u32).unwrap()
    }

    pub fn prepare(&mut self) -> Result<()> {
        //TODO: handle return
        let mut list = ptr::null_mut();
        let ret = unsafe { alpm_trans_prepare(self.handle.handle, &mut list) };
        self.handle.check_ret(ret)
    }

    pub fn commit(&mut self) -> Result<()> {
        //TODO: handle return
        let mut list = ptr::null_mut();
        let ret = unsafe { alpm_trans_commit(self.handle.handle, &mut list) };
        self.handle.check_ret(ret)
    }

    pub fn interrupt(&mut self) -> Result<()> {
        let ret = unsafe { alpm_trans_interrupt(self.handle.handle) };
        self.handle.check_ret(ret)
    }

    pub fn add(&self) -> Result<AlpmList<Package>> {
        let list = unsafe { alpm_trans_get_add(self.handle.handle) };
        self.handle.check_null(list)?;

        let list = AlpmList {
            handle: self.handle,
            item: list,
            free: FreeMethod::None,
            _marker: PhantomData,
        };

        Ok(list)
    }

    pub fn remove(&self) -> Result<AlpmList<Package>> {
        let list = unsafe { alpm_trans_get_remove(self.handle.handle) };
        self.handle.check_null(list)?;

        let list = AlpmList {
            handle: self.handle,
            item: list,
            free: FreeMethod::None,
            _marker: PhantomData,
        };

        Ok(list)
    }
}

impl<'a> Alpm {
    pub fn trans(&'a self, flags: TransFlag) -> Result<Trans<'a>> {
        let ret = unsafe { alpm_trans_init(self.handle, flags.bits() as i32) };
        self.check_ret(ret)?;
        Ok(Trans { handle: self })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{set_eventcb, set_logcb, Event, LogLevel, SigLevel};

    fn logcb(_level: LogLevel, msg: &str) {
        print!("{}", msg);
    }

    fn eventcb(event: Event) {
        match event {
            Event::DatabaseMissing(x) => println!("missing database: {}", x.dbname()),
            _ => println!("event: {:?}", event),
        }
    }

    #[test]
    fn test_trans() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let flags = TransFlag::DB_ONLY;

        set_logcb!(handle, logcb);
        set_eventcb!(handle, eventcb);

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
