use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

pub(crate) fn sanitize_string(value: &(impl AsRef<OsStr> + Sized)) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
