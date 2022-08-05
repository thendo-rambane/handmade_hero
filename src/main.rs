mod utils;

use std::{mem::size_of_val, os::raw::c_void, ptr::null_mut};

use utils::win32_str;
use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
      BeginPaint, CreateCompatibleDC, EndPaint, GetDC, ReleaseDC,
      StretchDIBits, BITMAPINFO, BI_RGB, DIB_RGB_COLORS, HDC, PAINTSTRUCT,
      SRCCOPY,
    },
    System::{
      LibraryLoader::GetModuleHandleW,
      Memory::{
        VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE,
      },
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
      PeekMessageW, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
      CW_USEDEFAULT, MSG, PM_REMOVE, WINDOW_EX_STYLE, WM_ACTIVATEAPP,
      WM_CLOSE, WM_DESTROY, WM_PAINT, WM_QUIT, WM_SIZE, WNDCLASSW,
      WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
  },
};

//Consts and statics
static mut RUNNING: bool = false;
static mut BITMAP_INFO: Option<BITMAPINFO> = None;
// static mut BITMAP_HANDLE: HBITMAP = HBITMAP(0);
static mut BITMAP_DEVICE_HANDLE: HDC = HDC(0);
static mut BITMAP_MEMORY: *mut c_void = null_mut();
static mut BITMAP_WIDTH: i32 = 0;
static mut BITMAP_HEIGHT: i32 = 0;
const BYTES_PER_PIXEL: i32 = 4;

fn render_weird_gradient(blue_offset: i32, green_offset: i32) {
  let (width, height) = unsafe { (BITMAP_WIDTH, BITMAP_HEIGHT) };
  dbg!((width, height));
  let pitch = width * BYTES_PER_PIXEL;
  let mut row: *mut u8 = unsafe { BITMAP_MEMORY.cast() };
  (0..height).for_each(|y| {
    let mut pixel: *mut u32 = row.cast();
    (0..width).for_each(|x| {
      let blue: u32 = (x + blue_offset) as u32;
      let green: u32 = (y + green_offset) as u32;

      unsafe {
        *pixel = (green << 8) | blue;
        pixel = pixel.add(1);
      }
    });
    unsafe {
      row = row.add(pitch as usize);
    }
  });
}

pub fn update_window(device_context: HDC, width: i32, height: i32) {
  unsafe {
    StretchDIBits(
      device_context,
      0,
      0,
      BITMAP_WIDTH,
      BITMAP_HEIGHT,
      0,
      0,
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
  if BITMAP_MEMORY != null_mut() {
    VirtualFree(BITMAP_MEMORY, 0, MEM_RELEASE);
  }
  BITMAP_WIDTH = width;
  BITMAP_HEIGHT = height;

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

    let bitmap_memory_size = width * height * BYTES_PER_PIXEL;
    BITMAP_MEMORY = VirtualAlloc(
      null_mut(),
      bitmap_memory_size as usize,
      MEM_COMMIT,
      PAGE_READWRITE,
    )
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
      // let x = paint.rcPaint.left;
      // let y = paint.rcPaint.top;
      // let width = paint.rcPaint.right - x;
      // let height = paint.rcPaint.bottom - y;

      let client_rect = Box::leak(Box::new(RECT::default()));
      GetClientRect(window_handle, client_rect);
      let width = client_rect.right - client_rect.left;
      let height = client_rect.bottom - client_rect.top;

      update_window(device_context, width, height);

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

  if window_handle != HWND(0) {
    // Messaege Struce
    let message = Box::leak(Box::new(MSG::default()));
    unsafe {
      RUNNING = true;
    }
    let mut x_offset = 0;
    let mut y_offset = 0;

    // Message Loop
    unsafe {
      while RUNNING {
        while PeekMessageW(message, window_handle, 0, 0, PM_REMOVE).as_bool() {
          if message.message == WM_QUIT {
            RUNNING = false;
          }
          TranslateMessage(message);
          DispatchMessageW(message);
        }
        render_weird_gradient(x_offset, y_offset);
        let device_context = GetDC(window_handle);
        let client_rect = Box::leak(Box::new(RECT::default()));

        GetClientRect(window_handle, client_rect);
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;

        update_window(device_context, width, height);
        ReleaseDC(window_handle, device_context);
        y_offset += 1;
        x_offset += 1;
      }
    }
  }
}
