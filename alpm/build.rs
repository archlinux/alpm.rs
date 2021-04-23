#[cfg(feature = "checkver")]
fn main() {
    use alpm_sys::alpm_version;
    use std::ffi::CStr;

    if cfg!(feature = "checkver") && !cfg!(feature = "git") {
        let ver = unsafe { alpm_version() };
        assert!(!ver.is_null());
        let ver = unsafe { CStr::from_ptr(ver) };
        let ver = ver.to_str().unwrap();

        let parts = ver.split('.').collect::<Vec<_>>();
        let parts = parts
            .iter()
            .map(|v| v.parse::<i32>().unwrap())
            .collect::<Vec<_>>();

        assert!(
            parts[0] == 12 && parts[2] <= 2,
            "this version of alpm.rs does not support libalpm v{} only v12.x.0 - v12.x.2 is supported",
            ver,
        );
    }
}

#[cfg(not(feature = "checkver"))]
fn main() {}
