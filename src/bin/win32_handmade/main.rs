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
static mut bitmap_device_context: Win32::HDC = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut bitmap_handle: Win32::HBITMAP = std::ptr::null_mut();

unsafe fn resize_dib_section(width: i32, height: i32) {
    if !bitmap_handle.is_null() {
        Win32::DeleteObject(bitmap_handle.cast());
    }
    if bitmap_device_context.is_null() {
        bitmap_device_context =
            Win32::CreateCompatibleDC(std::ptr::null_mut());
    }
    bitmap_info =
        Box::into_raw(Box::new(std::mem::zeroed::<Win32::BITMAPINFO>()));
    (*bitmap_info).bmiHeader.biSize =
        std::mem::size_of::<Win32::BITMAPINFOHEADER>() as u32;
    (*bitmap_info).bmiHeader.biWidth = width;
    (*bitmap_info).bmiHeader.biHeight = height;
    (*bitmap_info).bmiHeader.biPlanes = 1;
    (*bitmap_info).bmiHeader.biBitCount = 32;
    (*bitmap_info).bmiHeader.biCompression = Win32::BI_RGB;
    bitmap_handle = Win32::CreateDIBSection(
        bitmap_device_context,
        bitmap_info,
        Win32::DIB_RGB_COLORS,
        &mut bitmap_memory,
        std::ptr::null_mut(),
        0,
    );
    dbg!((*bitmap_info).bmiHeader.biBitCount);
    dbg!(bitmap_handle);
    dbg!(bitmap_memory);
    dbg!(std::io::Error::last_os_error());
    (0..(width * height)).for_each(|index| {
        bitmap_memory
            .cast::<u32>()
            .add(index as usize)
            .write(0x00_00_00_00);
    });
}

fn update_window(
    device_context: Win32::HDC,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    unsafe {
        Win32::StretchDIBits(
            device_context,
            x,
            y,
            width,
            height,
            x,
            y,
            width,
            height,
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
            unsafe {
                while running {
                    let mut msg: Win32::MSG = std::mem::zeroed();
                    let message_result =
                        Win32::GetMessageW(&mut msg, window, 0, 0);
                    if message_result > 0 {
                        Win32::TranslateMessage(&msg);
                        Win32::DispatchMessageW(&msg);
                    } else {
                        running = false;
                    }
                }
            }
        } else {
            dbg!("WINDOW_IS_NULL"); //TODO:{Thendo} LOGGING
        }
    } else {
        dbg!("FAILED TO REGISTER CLASS"); //TODO:{Thendo} LOGGING
    }
}
