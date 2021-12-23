use alpm::{Alpm, Error, SigLevel, TransFlag};

fn main() {
    let mut handle = Alpm::new("/", "tests/db").unwrap();

    let db = handle.register_syncdb_mut("core", SigLevel::NONE).unwrap();

    db.add_server("https://ftp.rnl.tecnico.ulisboa.pt/pub/archlinux/core/os/x86_64")
        .unwrap();

    let core = handle
        .syncdbs()
        .iter()
        .find(|db| db.name() == "core")
        .unwrap();
    let pkg = core.pkg("filesystem").unwrap();

    // set what flags we want to enable for the transaction;
    let flags = TransFlag::DB_ONLY | TransFlag::NO_DEPS;

    // initialise the transaction
    handle.trans_init(flags).unwrap();
    // add the packages we want to install
    // we could also remove packages with .trans_remove_pkg
    handle.trans_add_pkg(pkg).unwrap();
    // do a full sysupgrade
    handle.sync_sysupgrade(false).unwrap();

    // prepare the transaction
    handle.trans_prepare().unwrap();

    // fetch the list of packages we are going to install when we commit
    let toinstall = handle.trans_add();
    println!("{:#?}", toinstall);

    // commit the transaction
    // due to age the mirror now returns 404 for the package
    assert!(handle.trans_commit().unwrap_err().error() == Error::Retrieve);
}
