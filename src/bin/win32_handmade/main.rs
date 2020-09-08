#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows as Win32;

fn c_str(string: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(string)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn message_box(message: &str) {
    let msg = c_str(message);
    unsafe {
        Win32::MessageBoxW(
            std::ptr::null_mut(),
            msg.as_ptr(),
            msg.as_ptr(),
            Win32::MB_OK,
        )
    };
}

fn main() {
    message_box("Handmade Hero");
}
