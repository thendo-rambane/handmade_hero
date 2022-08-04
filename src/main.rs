mod utils;

use std::ptr::null_mut;

use utils::win32_str;
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    winuser::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
        TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, WNDCLASSW,
        WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
};

fn main() {
    // Window Name and Class Name
    let window_class_name = win32_str("ClassName");
    let window_name = win32_str("Window");

    // Create Window instance
    let window_instance = unsafe { GetModuleHandleW(null_mut()) };

    // Window Class Structure
    let mut window_class = Box::leak(Box::new(WNDCLASSW::default()));
    window_class.style = CS_HREDRAW | CS_VREDRAW;
    window_class.lpfnWndProc = Some(DefWindowProcW);
    window_class.hInstance = window_instance;
    window_class.lpszClassName = window_class_name;

    // Register Window Class
    let class_registered = unsafe { RegisterClassW(window_class) };
    if class_registered != 0 {
        // Get Window Handle
        let window_handle = unsafe {
            CreateWindowExW(
                0,
                window_class_name,
                window_name,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                null_mut(),
                null_mut(),
                window_instance,
                null_mut(),
            )
        };

        if window_handle != null_mut() {
            // Messaege Struce
            let message = Box::leak(Box::new(MSG::default()));

            // Message Loop
            'handle_messages: loop {
                unsafe {
                    if GetMessageW(message, window_handle, 0, 0) != 0 {
                        TranslateMessage(message);
                        DispatchMessageW(message);
                    } else {
                        break 'handle_messages;
                    }
                }
            }
        }
    }
}
