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

        let current = parts[0];
        let revision = parts[1];
        let age = parts[2];

        let supported_current = 13;
        let supported_revision = 0;

        assert!(
            supported_current == current
                && (revision - age..=revision).contains(&supported_revision),
            "this version of alpm.rs does not support libalpm v{} only v{}.{}.0 is supported",
            ver,
            supported_current,
            supported_revision,
        );
    }
}

#[cfg(not(feature = "checkver"))]
fn main() {}
