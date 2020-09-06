#[cfg(windows)]
extern crate winapi;
use std::io::Error as IoError;

#[cfg(windows)]
fn print_message(msg: &str) -> Result<i32, IoError> {
    /*NOTE: Extends `std::ffi::OsStr` with windows specific extend method
     * witch turns a string into an iterator that can be collected into a
     * Vec<u16> whose pointer can be passed to WinAPI methods
     */
    use std::os::windows::ffi::OsStrExt;

    /*NOTE: Collects `msg` into a Vec<u16> and adds a NULL at the end
     * (Required for c_strings)
     */
    let wide: Vec<u16> = std::ffi::OsStr::new(msg)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    /*NOTE: Calls the WinAPI MessageBoxW method and stores the result in ret
     */
    let ret = unsafe {
        winapi::um::winuser::MessageBoxW(
            std::ptr::null_mut(),
            wide.as_ptr(),
            wide.as_ptr(),
            winapi::um::winuser::MB_OK,
        )
    };
    if ret == 0 {
        return Err(IoError::last_os_error());
    }
    Ok(ret)
}

#[cfg(not(windows))]
fn print_message(msg: str) -> Result<(), IoError> {
    println!("Not Windows{}", msg);
    Ok(())
}

fn main() {
    print_message("Hello World").unwrap();
}
