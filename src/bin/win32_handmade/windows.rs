extern crate winapi;

pub use winapi::ctypes::*;
pub use winapi::shared::minwindef::*;
pub use winapi::shared::windef::*;
pub use winapi::um::libloaderapi::*;
pub use winapi::um::memoryapi::*;
pub use winapi::um::wingdi::*;
pub use winapi::um::winnt::*;
pub use winapi::um::winuser::*;


pub fn c_str(string: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(string)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
