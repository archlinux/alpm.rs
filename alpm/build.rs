use std::env;

fn main() {
    let version = env::var("DEP_ALPM_LIBALPM_VERSION").unwrap();
    println!("cargo::rustc-check-cfg=cfg(alpm15,alpm16)");
    if version.starts_with("15.") {
        println!("cargo::rustc-cfg=alpm15");
    }
    if version.starts_with("16.") {
        println!("cargo::rustc-cfg=alpm15");
        println!("cargo::rustc-cfg=alpm16");
    }
}
