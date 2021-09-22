use std::cmp::Ordering;

use alpm::{Alpm, SigLevel, Version};

fn main() {
    let handle = Alpm::new("/", "tests/db").unwrap();

    let core = handle
        .register_syncdb("core", SigLevel::USE_DEFAULT)
        .unwrap();

    // compare two versions
    if Version::new("2.2.3") > Version::new("2.0.0") {
        println!("2.2.3 is bigger");
    }

    // compare with a package
    let linux = core.pkg("linux").unwrap();
    if linux.version() > Version::new("1.0.0") {
        println!("linux is bigger with ver {}", linux.version());
    }

    // compare in a match
    match linux.version().vercmp(Version::new("1.0.0")) {
        Ordering::Less => println!("less"),
        Ordering::Equal => println!("equal"),
        Ordering::Greater => println!("greater"),
    }

    // sorting packages by version
    let mut pkgs = core.pkgs().iter().collect::<Vec<_>>();
    pkgs.sort_by(|a, b| a.version().vercmp(b.version()));
}
