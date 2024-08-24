fn main() {
    #[cfg(feature = "checkver")]
    {
        #[cfg(all(not(feature = "git"), not(feature = "docs-rs")))]
        {
            use alpm_sys::alpm_version;
            use std::ffi::CStr;

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
            let age = parts[2];

            let supported_current = 15;

            assert!(
                supported_current == current
                    && (current - age..=current).contains(&supported_current),
                "this version of alpm.rs does not support libalpm v{} only v{}.x.x is supported",
                ver,
                supported_current,
            );
        }
    }
}
