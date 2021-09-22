use alpm::{Alpm, SigLevel, Usage};

fn main() {
    let mut handle = Alpm::new("/", "tests/db").unwrap();

    let core = handle
        .register_syncdb_mut("core", SigLevel::USE_DEFAULT)
        .unwrap();

    core.add_server("https://example.com/core").unwrap();
    core.set_usage(Usage::SYNC | Usage::SEARCH).unwrap();

    // update the databases
    // (will fail because of bogus mirror)
    handle.syncdbs_mut().update(false).unwrap_err();
}
