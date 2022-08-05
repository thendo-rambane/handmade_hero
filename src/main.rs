mod utils;

use std::{mem::size_of_val, os::raw::c_void, ptr::null_mut};

use utils::win32_str;
use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
      BeginPaint, CreateCompatibleDC, CreateDIBSection, DeleteObject,
      EndPaint, StretchDIBits, BITMAPINFO, BI_RGB, DIB_RGB_COLORS, HBITMAP,
      HDC, PAINTSTRUCT, SRCCOPY,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
      GetMessageW, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
      CW_USEDEFAULT, MSG, WINDOW_EX_STYLE, WM_ACTIVATEAPP, WM_CLOSE,
      WM_DESTROY, WM_PAINT, WM_SIZE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
      WS_VISIBLE,
    },
  },
};
static mut RUNNING: bool = true;
static mut BITMAP_INFO: Option<BITMAPINFO> = None;
static mut BITMAP_HANDLE: HBITMAP = HBITMAP(0);
static mut BITMAP_DEVICE_HANDLE: HDC = HDC(0);
static mut BITMAP_MEMORY: *mut c_void = null_mut();
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
  if !BITMAP_HANDLE.is_invalid() {
    DeleteObject(BITMAP_HANDLE);
  }

  if BITMAP_DEVICE_HANDLE.is_invalid() {
    BITMAP_DEVICE_HANDLE = CreateCompatibleDC(HDC::default()).into();
  }
  if let Some(mut bitmap) = BITMAP_INFO {
    bitmap.bmiHeader.biSize =
      size_of_val(&bitmap.bmiHeader).try_into().unwrap();
    bitmap.bmiHeader.biWidth = width;
    bitmap.bmiHeader.biHeight = height;
    bitmap.bmiHeader.biPlanes = 1;
    bitmap.bmiHeader.biBitCount = 32;
    bitmap.bmiHeader.biCompression = BI_RGB as u32;
    BITMAP_INFO.replace(bitmap);

    if let Ok(bitmap_handle) = CreateDIBSection(
      BITMAP_DEVICE_HANDLE,
      &BITMAP_INFO.unwrap(),
      DIB_RGB_COLORS,
      &mut BITMAP_MEMORY,
      None,
      0,
    ) {
      BITMAP_HANDLE = bitmap_handle;
    }
  }
}
unsafe extern "system" fn window_proc(
  window_handle: HWND,
  msg: u32,
  w_param: WPARAM,
  l_param: LPARAM,
) -> LRESULT {
  match msg {
    WM_DESTROY => {
      RUNNING = false;
      LRESULT(0)
    }
    WM_CLOSE => {
      RUNNING = false;
      LRESULT(0)
    }
    WM_ACTIVATEAPP => {
      // OutputDebugStringW(win32_str("App Activated"));
      LRESULT(0)
    }
    WM_SIZE => {
      let client_rect = Box::leak(Box::new(RECT::default()));
      GetClientRect(window_handle, client_rect);
      let width = client_rect.right - client_rect.left;
      let height = client_rect.bottom - client_rect.top;
      resize_dib_section(width, height);
      LRESULT(0)
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
      LRESULT(0)
    }
    _ => DefWindowProcW(window_handle, msg, w_param, l_param),
  }
}

fn main() {
  unsafe {
    BITMAP_INFO.replace(BITMAPINFO::default());
  }
  // Window Name and Class Name
  let window_class_name = win32_str("ClassName");
  let window_name = win32_str("Window");

  // Create Window instance

  let window_handle =
    if let Ok(window_instance) = unsafe { GetModuleHandleW(None) } {
      let mut window_class = Box::leak(Box::new(WNDCLASSW::default()));
      window_class.style = CS_HREDRAW | CS_VREDRAW;
      window_class.lpfnWndProc = Some(window_proc);
      window_class.hInstance = window_instance;
      window_class.lpszClassName = PCWSTR(window_class_name);
      // dbg!(window_class);
      // Window Class Structure

      // Register Window Class
      let class_registered = unsafe { RegisterClassW(window_class) };
      if class_registered != 0 {
        // dbg!(class_registered);
        // Get Window Handle
        unsafe {
          CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(window_class_name),
            PCWSTR(window_name),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            window_instance,
            null_mut(),
          )
        }
      } else {
        HWND(0)
      }
    } else {
      HWND(0)
    };
  // dbg()

  if window_handle.0 != 0 {
    // Messaege Struce
    let message = Box::leak(Box::new(MSG::default()));

    // Message Loop
    'handle_messages: loop {
      unsafe {
        if RUNNING && GetMessageW(message, window_handle, 0, 0).as_bool() {
          TranslateMessage(message);
          DispatchMessageW(message);
        } else {
          break 'handle_messages;
        }
      }
    }
  }
}
