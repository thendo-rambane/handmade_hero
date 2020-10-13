extern crate winapi;

pub use winapi::ctypes::*;
pub use winapi::shared::guiddef::*;
pub use winapi::shared::minwindef::*;
pub use winapi::shared::mmreg::*;
pub use winapi::shared::windef::*;
pub use winapi::shared::winerror::*;
pub use winapi::um::dsound::*;
pub use winapi::um::libloaderapi::*;
pub use winapi::um::memoryapi::*;
pub use winapi::um::unknwnbase::*;
pub use winapi::um::wingdi::*;
pub use winapi::um::winnt::HRESULT;
pub use winapi::um::winnt::*;
pub use winapi::um::winuser::*;
pub use winapi::um::xinput::*;

use std::{ffi, os};

pub fn c_str_w(string: &str) -> Vec<u16> {
    use os::windows::ffi::OsStrExt;
    ffi::OsStr::new(string)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn c_str_a(string: &str) -> ffi::CString {
    ffi::CString::new(string).unwrap()
}
