#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows as Win32;

#[allow(non_upper_case_globals)]
static mut running: bool = true;

#[allow(non_upper_case_globals)]
static mut global_buffer: *mut OffScreenBuffer = std::ptr::null_mut();

struct WindowDimentions {
    width: i32,
    height: i32,
}

fn get_window_dimensions(window: Win32::HWND) -> WindowDimentions {
    let mut client_rect: Win32::RECT = unsafe { std::mem::zeroed() };
    unsafe { Win32::GetClientRect(window, &mut client_rect) };
    WindowDimentions {
        width: client_rect.right - client_rect.left,
        height: client_rect.bottom - client_rect.top,
    }
}

struct OffScreenBuffer {
    memory: *mut Win32::VOID,
    info: Win32::BITMAPINFO,
    width: i32,
    height: i32,
    bytes_per_pixel: usize,
}

impl OffScreenBuffer {
    fn new() -> Self {
        OffScreenBuffer {
            memory: std::ptr::null_mut(),
            info: unsafe { std::mem::zeroed() },
            width: 0,
            height: 0,
            bytes_per_pixel: std::mem::size_of::<u32>(),
        }
    }
    fn update_window(
        &self,
        device_context: Win32::HDC,
        window_width: i32,
        window_height: i32,
    ) {
        unsafe {
            Win32::StretchDIBits(
                device_context,
                0,             //dst
                0,             //dst
                window_width,  //dst
                window_height, //dst
                0,
                0,
                self.width,
                self.height,
                self.memory,
                &self.info,
                Win32::DIB_RGB_COLORS,
                Win32::SRCCOPY,
            )
        };
    }

    unsafe fn resize_dib_section(&mut self, width: i32, height: i32) {
        if !self.memory.is_null() {
            Win32::VirtualFree(self.memory, 0, Win32::MEM_RELEASE);
        }

        self.height = height;
        self.width = width;

        self.info.bmiHeader.biSize =
            std::mem::size_of::<Win32::BITMAPINFOHEADER>() as u32;
        self.info.bmiHeader.biWidth = self.width;
        self.info.bmiHeader.biHeight = -self.height;
        self.info.bmiHeader.biPlanes = 1;
        self.info.bmiHeader.biBitCount = 32;
        self.info.bmiHeader.biCompression = Win32::BI_RGB;

        let size = self.bytes_per_pixel * (self.width * self.height) as usize;
        self.memory = Win32::VirtualAlloc(
            std::ptr::null_mut(),
            size,
            Win32::MEM_COMMIT,
            Win32::PAGE_READWRITE,
        );
    }
}

unsafe fn render_weird_gradient(
    buffer: &mut OffScreenBuffer,
    x_offset: i32,
    y_offset: i32,
) {
    (0..buffer.width).for_each(|x| {
        (0..buffer.height).for_each(|y| {
            let index = (x + buffer.width * y) as usize;
            let pixel = buffer.memory.cast::<u32>().add(index);
            let green = (x + x_offset) as u32;
            let blue = (y + y_offset) as u32;
            pixel.write(0 | (green << 8) | blue);
        })
    })
}
unsafe extern "system" fn main_window_callback(
    window: Win32::HWND,
    message: Win32::UINT,
    wparam: Win32::WPARAM,
    lparam: Win32::LPARAM,
) -> Win32::LRESULT {
    let mut result: Win32::LRESULT = 0;
    let buffer = if let Some(buffer) = global_buffer.as_mut() {
        buffer
    } else {
        panic!("COULD NOT UNWRAP BUFFER")
    };
    match message {
        Win32::WM_ACTIVATEAPP => {
            dbg!("WM_ACTIVATEAPP");
        }
        Win32::WM_CLOSE => {
            running = false;
            dbg!("WM_CLOSE");
        }
        Win32::WM_DESTROY => {
            dbg!("WM_DESTROY");
        }
        Win32::WM_PAINT => {
            let mut paint_struct: Win32::PAINTSTRUCT = std::mem::zeroed();
            let device_context: Win32::HDC =
                Win32::BeginPaint(window, &mut paint_struct);
            let window_dimensions = get_window_dimensions(window);
            buffer.update_window(
                device_context,
                window_dimensions.width,
                window_dimensions.height,
            );
            Win32::EndPaint(window, &paint_struct);
            dbg!("WM_PAINT");
        }
        Win32::WM_SIZE => {
            dbg!("WM_SIZE");
        }
        _ => {
            result = Win32::DefWindowProcW(window, message, wparam, lparam);
        }
    };
    result
}

fn main() {
    let instance = unsafe { Win32::GetModuleHandleW(std::ptr::null()) };
    let window_class_name = Win32::c_str("HandmadeHeroWindowClass");
    let window_name = Win32::c_str("Handmade Hero");
    let buffer = unsafe {
        global_buffer = Box::into_raw(Box::new(OffScreenBuffer::new()));
        global_buffer.as_mut().unwrap()
    };

    let mut window_class: Win32::WNDCLASSW = unsafe { std::mem::zeroed() };

    window_class.style = Win32::CS_VREDRAW | Win32::CS_HREDRAW;
    window_class.lpfnWndProc = Some(main_window_callback);
    window_class.hInstance = instance;
    window_class.lpszClassName = window_class_name.as_ptr();

    if unsafe { Win32::RegisterClassW(&window_class) } != 0 {
        let window = unsafe {
            Win32::CreateWindowExW(
                0,
                window_class_name.as_ptr(),
                window_name.as_ptr(),
                Win32::WS_OVERLAPPEDWINDOW | Win32::WS_VISIBLE,
                Win32::CW_USEDEFAULT,
                Win32::CW_USEDEFAULT,
                Win32::CW_USEDEFAULT,
                Win32::CW_USEDEFAULT,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            )
        };

        if !window.is_null() {
            let mut x_offset = 0;
            let mut y_offset = 0;
            unsafe {
                buffer.resize_dib_section(1280, 720);
                while running {
                    let mut msg: Win32::MSG = std::mem::zeroed();
                    while Win32::PeekMessageW(
                        &mut msg,
                        window,
                        0,
                        0,
                        Win32::PM_REMOVE,
                    ) != 0
                    {
                        Win32::TranslateMessage(&msg);
                        Win32::DispatchMessageW(&msg);
                    }
                    render_weird_gradient(buffer, x_offset, y_offset);
                    let device_context = Win32::GetDC(window);
                    let window_dimensions = get_window_dimensions(window);
                    buffer.update_window(
                        device_context,
                        window_dimensions.width,
                        window_dimensions.height,
                    );
                    Win32::ReleaseDC(window, device_context);
                    x_offset += 1;
                    y_offset += 2;
                }
            }
        } else {
            dbg!("WINDOW_IS_NULL"); //TODO:{Thendo} LOGGING
        }
    } else {
        dbg!("FAILED TO REGISTER CLASS"); //TODO:{Thendo} LOGGING
    }
}
