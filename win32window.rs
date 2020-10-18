mod windows;

use windows as Win32Api;


use std::os::windows::ffi::OsStrExt;
fn c_str(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub struct Window<'window> {
    name: Vec::<u16>,
    title: Vec::<u16>,
    bit_memory
}

impl Window {
    pub fn new(self, name: &str, title: &str) -> Self{
        Window{
            name:c_str(name),
            title:c_str(title),
        }
    }
    pub create_window(name: &str, title: &str) -> Result<Win32Api::HWND, std::io::Error>

}
