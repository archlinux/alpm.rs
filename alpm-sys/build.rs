fn main() {
    use std::env;
    use std::path::Path;

    if cfg!(feature = "docs-rs") {
        return;
    }

    #[cfg(feature = "static")]
    println!("cargo:rerun-if-changed=/usr/lib/pacman/lib/pkgconfig");
    println!("cargo:rerun-if-env-changed=ALPM_LIB_DIR");

    if cfg!(feature = "static") && Path::new("/usr/lib/pacman/lib/pkgconfig").exists() {
        env::set_var("PKG_CONFIG_LIBDIR", "/usr/lib/pacman/lib/pkgconfig");
    }

    if let Ok(dir) = env::var("ALPM_LIB_DIR") {
        println!("cargo:rustc-link-search={}", dir);
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    let lib = pkg_config::Config::new()
        .atleast_version("13.0.0")
        .statik(cfg!(feature = "static"))
        .probe("libalpm")
        .unwrap();

    #[cfg(feature = "generate")]
    {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("ffi_generated.rs");

        let header = lib
            .include_paths
            .iter()
            .map(|i| i.join("alpm.h"))
            .find(|i| i.exists())
            .expect("could not find alpm.h");
        let mut include = lib
            .include_paths
            .iter()
            .map(|i| format!("-I{}", i.display().to_string()))
            .collect::<Vec<_>>();

        println!("cargo:rerun-if-env-changed=ALPM_INCLUDE_DIR");
        if let Ok(path) = env::var("ALPM_INCLUDE_DIR") {
            include.clear();
            include.insert(0, path);
        }

        let bindings = bindgen::builder()
            .clang_args(&include)
            .header(header.display().to_string())
            .allowlist_type("(alpm|ALPM).*")
            .allowlist_function("(alpm|ALPM).*")
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
            .derive_eq(true)
            .derive_ord(true)
            .derive_copy(true)
            .derive_hash(true)
            .derive_debug(true)
            .derive_partialeq(true)
            .derive_debug(true)
            .generate()
            .unwrap();

        bindings.write_to_file(dest_path).unwrap();
    }
}
