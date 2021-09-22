use alpm::{Alpm, SigLevel};

fn main() {
    // initialise the handle
    let mut handle = Alpm::new("/", "tests/db").unwrap();

    // configure any settings
    handle.set_ignorepkgs(["a", "b", "c"].iter()).unwrap();
    handle.add_cachedir("/var/lib/pacman").unwrap();
    handle.set_check_space(true);

    // register any databases you wish to use
    handle
        .register_syncdb("core", SigLevel::USE_DEFAULT)
        .unwrap();
    handle
        .register_syncdb("extra", SigLevel::USE_DEFAULT)
        .unwrap();
    handle
        .register_syncdb("community", SigLevel::USE_DEFAULT)
        .unwrap();
}
