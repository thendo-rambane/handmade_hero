#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows as Win32;

#[allow(non_upper_case_globals)]
static mut running: bool = true;
#[allow(non_upper_case_globals)]
static mut bitmap_memory: *mut Win32::VOID = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut bitmap_info: *mut Win32::BITMAPINFO = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut bitmap_width: i32 = 0;
#[allow(non_upper_case_globals)]
static mut bitmap_height: i32 = 0;
#[allow(non_upper_case_globals)]
static bytes_per_pixel: usize = std::mem::size_of::<u32>();

unsafe fn render_weird_gradient(x_offset: i32, y_offset: i32) {
    (0..bitmap_width).for_each(|x| {
        (0..bitmap_height).for_each(|y| {
            let index = (x + bitmap_width * y) as usize;
            let pixel = bitmap_memory.cast::<i32>().add(index);
            let green = x + x_offset;
            let blue = y + y_offset;
            pixel.write((green << 8) | blue);
        })
    })
}

unsafe fn resize_dib_section(width: i32, height: i32) {
    if !bitmap_memory.is_null() {
        Win32::VirtualFree(bitmap_memory, 0, Win32::MEM_RELEASE);
    }

    bitmap_height = height;
    bitmap_width = width;

    bitmap_info =
        Box::into_raw(Box::new(std::mem::zeroed::<Win32::BITMAPINFO>()));
    (*bitmap_info).bmiHeader.biSize =
        std::mem::size_of::<Win32::BITMAPINFOHEADER>() as u32;
    (*bitmap_info).bmiHeader.biWidth = width;
    (*bitmap_info).bmiHeader.biHeight = -height;
    (*bitmap_info).bmiHeader.biPlanes = 1;
    (*bitmap_info).bmiHeader.biBitCount = 32;
    (*bitmap_info).bmiHeader.biCompression = Win32::BI_RGB;


    let bitmap_size = bytes_per_pixel * (width * height) as usize;
    bitmap_memory = Win32::VirtualAlloc(
        std::ptr::null_mut(),
        bitmap_size,
        Win32::MEM_COMMIT,
        Win32::PAGE_READWRITE,
    );
}

fn update_window(
    device_context: Win32::HDC,
    x: i32,
    y: i32,
    _width: i32,
    _height: i32,
) {
    unsafe {
        Win32::StretchDIBits(
            device_context,
            x,
            y,
            bitmap_width,
            bitmap_height,
            x,
            y,
            bitmap_width,
            bitmap_height,
            bitmap_memory,
            bitmap_info,
            Win32::DIB_RGB_COLORS,
            Win32::SRCCOPY,
        )
    };
}

unsafe extern "system" fn main_window_callback(
    window: Win32::HWND,
    message: Win32::UINT,
    wparam: Win32::WPARAM,
    lparam: Win32::LPARAM,
) -> Win32::LRESULT {
    let mut result: Win32::LRESULT = 0;
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
            let x = paint_struct.rcPaint.left;
            let y = paint_struct.rcPaint.top;
            let width = paint_struct.rcPaint.right - paint_struct.rcPaint.left;
            let height =
                paint_struct.rcPaint.bottom - paint_struct.rcPaint.top;
            update_window(device_context, x, y, width, height);
            Win32::EndPaint(window, &paint_struct);
            dbg!("WM_PAINT");
        }
        Win32::WM_SIZE => {
            let mut client_rect: Win32::RECT = std::mem::zeroed();
            Win32::GetClientRect(window, &mut client_rect);
            let height = client_rect.bottom - client_rect.top;
            let width = client_rect.right - client_rect.left;
            resize_dib_section(width, height);
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
                while running {
                    let mut msg: Win32::MSG = std::mem::zeroed();
                    while Win32::PeekMessageW(
                        &mut msg,
                        window,
                        0,
                        0,
                        Win32::PM_REMOVE,
                    ) > 0
                    {
                        Win32::TranslateMessage(&msg);
                        Win32::DispatchMessageW(&msg);
                    }
                    render_weird_gradient(x_offset, y_offset);
                    let device_context = Win32::GetDC(window);
                    update_window(device_context, 0, 0, 0, 0);
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
