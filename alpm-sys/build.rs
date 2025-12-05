fn main() {
    use std::env;
    

    if cfg!(feature = "docs-rs") {
        return;
    }

    let mut includes = Vec::new();

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=ALPM_INCLUDE_DIR");
    if let Ok(dirs) = env::var("ALPM_INCLUDE_DIR") {
        includes.extend(dirs.split(':').map(|s| s.to_string()))
    } else if !cfg!(feature = "pkg-config") {
        includes.push("/usr/include".into());
    }

    println!("cargo:rerun-if-env-changed=ALPM_LIB_DIR");
    if let Ok(dirs) = env::var("ALPM_LIB_DIR") {
        println!("cargo::rustc-link-lib=libalpm");
        for dir in dirs.split(':') {
            println!("cargo::rustc-link-search=native={}", dir);
        }
    } else if !cfg!(feature = "pkg-config") {
        println!("cargo::rustc-link-lib=alpm");
        println!("cargo::rustc-link-search=native=/usr/lib");
    }

    #[cfg(feature = "pkg-config")]
    {
        let pkgconf = pkg_config::Config::new()
            .atleast_version("16.0.0")
            .statik(cfg!(feature = "static"))
            .probe("libalpm")
            .expect("failed to run pkgconf for libalpm");

        includes.extend(
            pkgconf
                .include_paths
                .iter()
                .map(|d| d.display().to_string()),
        );
    }

    #[cfg(feature = "generate")]
    {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("ffi_generated.rs");

        let header = includes
            .iter()
            .map(|s| Path::new(s).join("alpm.h"))
            .find(|p| p.exists())
            .expect("failed to find alpm.h");

        let includes = includes
            .iter()
            .map(|i| format!("-I{i}"))
            .collect::<Vec<_>>();

        let bindings = bindgen::builder()
            .clang_args(&includes)
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
