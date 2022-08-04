use std::{ffi::OsStr, iter::once, os::windows::prelude::OsStrExt};

pub fn win32_str(string: &str) -> *const u16 {
    let win32_string: Vec<u16> = OsStr::new(string).encode_wide().chain(once(0)).collect();
    win32_string.as_ptr()
}
