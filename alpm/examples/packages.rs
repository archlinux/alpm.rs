use alpm::{Alpm, PackageReason, SigLevel};

fn main() {
    let handle = Alpm::new("/", "tests/db").unwrap();

    handle
        .register_syncdb("core", SigLevel::USE_DEFAULT)
        .unwrap();
    handle
        .register_syncdb("extra", SigLevel::USE_DEFAULT)
        .unwrap();
    handle
        .register_syncdb("community", SigLevel::USE_DEFAULT)
        .unwrap();

    // iterate through each database
    for db in handle.syncdbs() {
        // search each database for packages matching the regex "linux-[a-z]" AND "headers"
        for pkg in db.search(["linux-[a-z]", "headers"].iter()).unwrap() {
            println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
        }
    }

    // iterate through each database
    for db in handle.syncdbs() {
        // look for a package named "pacman" in each databse
        // the database is implemented as a hashmap so this is faster than iterating
        if let Ok(pkg) = db.pkg("pacman") {
            println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
        }
    }

    // iterate through each database
    for db in handle.syncdbs() {
        // iterate through every package in the databse
        for pkg in db.pkgs() {
            // print only explititly intalled packages
            if pkg.reason() == PackageReason::Explicit {
                println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
            }
        }
    }

    // iterate through each database
    for db in handle.syncdbs() {
        // look for the base-devel group
        if let Ok(group) = db.group("base-devel") {
            // print each package in the group
            for pkg in group.packages() {
                println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
            }
        }
    }

    // find a package matching a dep
    let pkg = handle.syncdbs().find_satisfier("linux>3").unwrap();
    println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));

    // load the pacman package from disk instead of from database
    let pkg = handle
        .pkg_load(
            "tests/pacman-5.1.3-1-x86_64.pkg.tar.xz",
            true,
            SigLevel::USE_DEFAULT,
        )
        .unwrap();
    println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
}
