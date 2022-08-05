mod utils;

use std::{mem::size_of_val, ptr::null_mut};

use utils::win32_str;
use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::{HBITMAP, HDC, HWND, RECT},
    },
    um::{
        debugapi::OutputDebugStringW,
        libloaderapi::GetModuleHandleW,
        wingdi::{
            CreateCompatibleDC, CreateDIBSection, StretchDIBits, BITMAPINFO, BI_RGB,
            DIB_RGB_COLORS, SRCCOPY,
        },
        winuser::{
            BeginPaint, CreateWindowExW, DefWindowProcW, DispatchMessageW, EndPaint, GetClientRect,
            GetMessageW, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
            MSG, PAINTSTRUCT, WM_ACTIVATEAPP, WM_CLOSE, WM_DESTROY, WM_PAINT, WM_SIZE, WNDCLASSW,
            WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

fn main() {
    static mut RUNNING: bool = true;
    static mut BITMAP_INFO: Option<BITMAPINFO> = None;
    static mut BITMAP_HANDLE: HBITMAP = null_mut();
    static mut BITMAP_DEVICE_HANDLE: HDC = null_mut();
    static mut BITMAP_MEMORY: *mut c_void = null_mut();
    unsafe {
        BITMAP_INFO.replace(BITMAPINFO::default());
    }
    pub fn update(device_context: HDC, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            StretchDIBits(
                device_context,
                x,
                y,
                width,
                height,
                x,
                y,
                width,
                height,
                BITMAP_MEMORY,
                BITMAP_INFO.as_ref().unwrap(),
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    }
    unsafe fn resize_dib_section(width: i32, height: i32) {
        if BITMAP_HANDLE != null_mut() {
            BITMAP_HANDLE.drop_in_place()
        }

        if BITMAP_DEVICE_HANDLE == null_mut() {
            BITMAP_DEVICE_HANDLE = CreateCompatibleDC(null_mut())
        }
        if let Some(mut bitmap) = BITMAP_INFO {
            bitmap.bmiHeader.biSize = size_of_val(&bitmap.bmiHeader).try_into().unwrap();
            bitmap.bmiHeader.biWidth = width;
            bitmap.bmiHeader.biHeight = height;
            bitmap.bmiHeader.biPlanes = 1;
            bitmap.bmiHeader.biBitCount = 32;
            bitmap.bmiHeader.biCompression = BI_RGB;
            BITMAP_INFO.replace(bitmap);

            BITMAP_HANDLE = CreateDIBSection(
                BITMAP_DEVICE_HANDLE,
                &BITMAP_INFO.unwrap(),
                DIB_RGB_COLORS,
                &mut BITMAP_MEMORY,
                null_mut(),
                0,
            )
        }
    }
    unsafe extern "system" fn window_proc(
        window_handle: HWND,
        msg: UINT,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_DESTROY => {
                RUNNING = false;
                0
            }
            WM_CLOSE => {
                RUNNING = false;
                0
            }
            WM_ACTIVATEAPP => {
                OutputDebugStringW(win32_str("App Activated"));
                0
            }
            WM_SIZE => {
                let client_rect = Box::leak(Box::new(RECT::default()));
                GetClientRect(window_handle, client_rect);
                let width = client_rect.right - client_rect.left;
                let height = client_rect.bottom - client_rect.top;
                resize_dib_section(width, height);

                0
            }
            WM_PAINT => {
                let paint = Box::leak(Box::new(PAINTSTRUCT::default()));
                let device_context = BeginPaint(window_handle, paint);
                let x = paint.rcPaint.left;
                let y = paint.rcPaint.top;
                let width = paint.rcPaint.right - x;
                let height = paint.rcPaint.bottom - y;
                update(device_context, x, y, width, height);
                EndPaint(window_handle, paint);
                0
            }
            _ => DefWindowProcW(window_handle, msg, w_param, l_param),
        }
    }
    // Window Name and Class Name
    let window_class_name = win32_str("ClassName");
    let window_name = win32_str("Window");

    // Create Window instance
    let window_instance = unsafe { GetModuleHandleW(null_mut()) };

    // Window Class Structure
    let mut window_class = Box::leak(Box::new(WNDCLASSW::default()));
    window_class.style = CS_HREDRAW | CS_VREDRAW;
    window_class.lpfnWndProc = Some(window_proc);
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
                    if RUNNING && GetMessageW(message, window_handle, 0, 0) != 0 {
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
