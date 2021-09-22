fn main() {
    use std::env;
    use std::path::Path;

    if cfg!(feature = "docs-rs") {
        return;
    }

    #[cfg(feature = "static")]
    println!("cargo:rerun-if-changed=/usr/lib/pacman/lib/pkgconfig");
    #[cfg(feature = "static")]
    println!("cargo:rustc-link-search=/usr/lib/pacman/lib/");
    println!("cargo:rerun-if-env-changed=ALPM_LIB_DIR");

    if cfg!(feature = "static") && Path::new("/usr/lib/pacman/lib/pkgconfig").exists() {
        env::set_var("PKG_CONFIG_LIBDIR", "/usr/lib/pacman/lib/pkgconfig");
    }

    if let Ok(dir) = env::var("ALPM_LIB_DIR") {
        println!("cargo:rustc-link-search={}", dir);
    }

    pkg_config::Config::new()
        .atleast_version("13.0.0")
        .statik(cfg!(feature = "static"))
        .probe("libalpm")
        .unwrap();

    #[cfg(feature = "generate")]
    {

        println!("cargo:rerun-if-env-changed=ALPM_INCLUDE_DIR");

        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("ffi_generated.rs");

        let alpm_dir = env::var("ALPM_INCLUDE_DIR");
        let alpm_dir = match alpm_dir {
            Ok(ref dir) => Path::new(dir),
            Err(_) => Path::new("/usr/include"),
        };

        let header = alpm_dir.join("alpm.h").to_str().unwrap().to_string();

        let bindings = bindgen::builder()
            .header(header)
            .whitelist_type("(alpm|ALPM).*")
            .whitelist_function("(alpm|ALPM).*")
            .rustified_enum("_alpm_[a-z_]+_t")
            .rustified_enum("alpm_download_event_type_t")
            .constified_enum_module("_alpm_siglevel_t")
            .constified_enum_module("_alpm_pkgvalidation_t")
            .constified_enum_module("_alpm_loglevel_t")
            .constified_enum_module("_alpm_question_type_t")
            .constified_enum_module("_alpm_transflag_t")
            .constified_enum_module("_alpm_db_usage_")
            .constified_enum_module("_alpm_db_usage_t")
            .constified_enum_module("alpm_caps")
            .opaque_type("alpm_handle_t")
            .opaque_type("alpm_db_t")
            .opaque_type("alpm_pkg_t")
            .opaque_type("alpm_trans_t")
            .size_t_is_usize(true)
            .generate()
            .unwrap();

        bindings.write_to_file(dest_path).unwrap();
    }
}
