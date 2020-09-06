#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows as Win32Api;

macro_rules! global {
    ($global_name:ident:$global_type:ty = $init_value:literal ) => {
        static mut $global_name: $global_type = $init_value;
    };
    ($global_name:ident:$global_type:ty = $init_value:expr ) => {
        static mut $global_name: $global_type = $init_value;
    };
}

global!(RUNNING: bool = true);
global!(BUFFER: *mut WindowBuffer = std::ptr::null_mut());

use std::os::windows::ffi::OsStrExt;
fn c_str(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

struct WindowBuffer {
    bitmap_memory: *mut Win32Api::c_void,
    bitmap_info: Win32Api::BITMAPINFO,
    height: i32,
    width: i32,
    alloc_size: usize,
}

impl WindowBuffer {
    unsafe fn new() -> Self {
        WindowBuffer {
            bitmap_memory: std::ptr::null_mut(),
            bitmap_info: std::mem::zeroed(),
            height: 0,
            width: 0,
            alloc_size: 0,
        }
    }

    unsafe fn resize_dibsection(&mut self, width: i32, height: i32) {
        if !self.bitmap_memory.is_null() {
            Win32Api::VirtualFree(
                self.bitmap_memory,
                0,
                Win32Api::MEM_RELEASE,
            );
        }

        self.width = width;
        self.height = height;
        self.bitmap_info = std::mem::zeroed::<Win32Api::BITMAPINFO>();
        self.bitmap_info.bmiHeader.biSize =
            std::mem::size_of::<Win32Api::BITMAPINFOHEADER>() as u32;
        self.bitmap_info.bmiHeader.biWidth = self.width;
        self.bitmap_info.bmiHeader.biHeight = -self.height;
        self.bitmap_info.bmiHeader.biPlanes = 1;
        self.bitmap_info.bmiHeader.biBitCount = 32;
        self.bitmap_info.bmiHeader.biCompression = Win32Api::BI_RGB;

        self.alloc_size =
            std::mem::size_of::<u32>() * (height * width) as usize;

        self.bitmap_memory = Win32Api::VirtualAlloc(
            std::ptr::null_mut::<Win32Api::c_void>(),
            self.alloc_size,
            Win32Api::MEM_COMMIT,
            Win32Api::PAGE_READWRITE,
        );
    }

    unsafe fn create_window(
        &self,
        name: &str,
        title: &str,
    ) -> Result<Win32Api::HWND, std::io::Error> {
        let name = c_str(name);
        let title = c_str(title);

        let hinstance = Win32Api::GetModuleHandleW(std::ptr::null_mut());

        let mut window_class = std::mem::zeroed::<Win32Api::WNDCLASSW>();
        window_class.lpfnWndProc = Some(window_proc);
        window_class.hInstance = hinstance;
        window_class.lpszClassName = name.as_ptr();

        if Win32Api::RegisterClassW(&window_class) != 0 {
            let handle = Win32Api::CreateWindowExW(
                0,
                name.as_ptr(),
                title.as_ptr(),
                Win32Api::WS_OVERLAPPEDWINDOW | Win32Api::WS_VISIBLE,
                Win32Api::CW_USEDEFAULT,
                Win32Api::CW_USEDEFAULT,
                Win32Api::CW_USEDEFAULT,
                Win32Api::CW_USEDEFAULT,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );
            if !handle.is_null() {
                return Ok(handle);
            } else {
                return Err(std::io::Error::last_os_error());
            }
        } else {
            return Err(std::io::Error::last_os_error());
        }
    }
    unsafe fn update_window(
        &self,
        device_context: Win32Api::HDC,
        client_rect: Win32Api::RECT,
    ) {
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;

        Win32Api::StretchDIBits(
            device_context,
            0,
            0,
            self.width,
            self.height,
            0,
            0,
            width,
            height,
            self.bitmap_memory,
            &self.bitmap_info,
            Win32Api::DIB_RGB_COLORS,
            Win32Api::SRCCOPY,
        );
    }
}

unsafe fn render_weird_gradient(
    buffer: &WindowBuffer,
    x_offset: i32,
    y_offset: i32,
) {
    let pixel = buffer.bitmap_memory.cast::<u32>();
    for x in 0..buffer.width {
        for y in 0..buffer.height {
            let index = (y * buffer.width + x) as usize;
            let pixel_red = pixel.add(index).cast::<u8>().add(2);
            let pixel_green = pixel.add(index).cast::<u8>().add(1);
            let pixel_blue = pixel.add(index).cast::<u8>().add(0);
            pixel_blue.write((x + x_offset) as u8);
            pixel_green.write((y + y_offset) as u8);
            pixel_red.write(0)
        }
    }
}
struct WinDimensions {
    width: i32,
    height: i32,
}
fn get_win_dimensions(client_rect: Win32Api::RECT) -> WinDimensions {
    let width = client_rect.right - client_rect.left;
    let height = client_rect.bottom - client_rect.top;
    WinDimensions { width, height }
}
unsafe extern "system" fn window_proc(
    window: Win32Api::HWND,
    message: Win32Api::UINT,
    w_param: Win32Api::WPARAM,
    l_param: Win32Api::LPARAM,
) -> Win32Api::LRESULT {
    let buffer = BUFFER.as_mut().unwrap();
    match message {
        Win32Api::WM_CLOSE => {
            RUNNING = false;
            0
        }
        Win32Api::WM_DESTROY => {
            RUNNING = false;
            0
        }
        Win32Api::WM_SIZE => {
            let mut client_rect: Win32Api::RECT = std::mem::zeroed();
            Win32Api::GetClientRect(window, &mut client_rect);
            let win_dimensions = get_win_dimensions(client_rect);
            buffer.resize_dibsection(
                win_dimensions.width,
                win_dimensions.height,
            );

            0
        }
        Win32Api::WM_PAINT => {
            let mut paint_struct: Win32Api::PAINTSTRUCT = std::mem::zeroed();
            let device_context =
                Win32Api::BeginPaint(window, &mut paint_struct);

            let window_rect: Win32Api::RECT = paint_struct.rcPaint;
            let win_dimensions = get_win_dimensions(window_rect);
            buffer.resize_dibsection(
                win_dimensions.width,
                win_dimensions.height,
            );
            buffer.update_window(device_context, window_rect);
            Win32Api::EndPaint(window, &paint_struct) as isize
        }
        _ => Win32Api::DefWindowProcW(window, message, w_param, l_param),
    }
}
fn main() {
    unsafe {
        /* Initialise global BUFFER
         * This only works because the box is in main's stack
         * and will live as long as main does which is essentially
         * the life time of the window
         */
        BUFFER = Box::into_raw(Box::new(WindowBuffer::new()));
        let buffer = BUFFER.as_mut().unwrap();

        let handle_result = buffer.create_window("Window", "Window Title");
        if let Ok(handle) = handle_result {
            let mut x_offset = 0;
            let y_offset = 0;
            while RUNNING {
                let mut message = std::mem::zeroed::<Win32Api::MSG>();
                while Win32Api::PeekMessageW(
                    &mut message,
                    handle,
                    0,
                    0,
                    Win32Api::PM_REMOVE,
                ) != 0
                {
                    if message.message == Win32Api::WM_QUIT {
                        RUNNING = false;
                    }
                    Win32Api::TranslateMessage(&message);
                    Win32Api::DispatchMessageW(&message);
                }
                render_weird_gradient(&buffer, x_offset, y_offset);
                x_offset += 1;

                let device_context = Win32Api::GetDC(handle);

                let mut client_rect: Win32Api::RECT = std::mem::zeroed();
                Win32Api::GetClientRect(handle, &mut client_rect);
                buffer.update_window(device_context, client_rect);
                Win32Api::ReleaseDC(handle, device_context);
            }
        } else {
            eprint!("{:#?}", handle_result.err().unwrap());
        }
    }
}
