mod utils;

use std::{mem::size_of_val, os::raw::c_void, ptr::null_mut};

use utils::win32_str;
use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
      BeginPaint, EndPaint, GetDC, StretchDIBits, BITMAPINFO, BI_RGB,
      DIB_RGB_COLORS, HDC, PAINTSTRUCT, SRCCOPY,
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
const BYTES_PER_PIXEL: i32 = 4;

static mut OFF_SCREEN_BUFFER: OffScreenBuffer = OffScreenBuffer::new();

#[derive(Debug)]
struct OffScreenBuffer {
  info: Option<BITMAPINFO>,
  memory: *mut c_void,
  width: i32,
  height: i32,
  pitch: usize,
}

impl OffScreenBuffer {
  const fn new() -> Self {
    Self {
      info: None,
      memory: null_mut(),
      width: 0,
      height: 0,
      pitch: 0,
    }
  }

  fn init(&mut self, window_handle: HWND) {
    let window_dimensions = WindowDimensions::new_for(window_handle);
    self.height = window_dimensions.height;
    self.width = window_dimensions.width;
    self.pitch = (self.width * BYTES_PER_PIXEL) as usize;
    self.info = Some(BITMAPINFO::default());
  }

  fn render_weird_gradient(&mut self, blue_offset: i32, green_offset: i32) {
    let mut row: *mut u8 = self.memory.cast();
    (0..self.height).for_each(|y| {
      let mut pixel: *mut u32 = row.cast();
      (0..self.width).for_each(|x| {
        let blue: u32 = (x + blue_offset) as u32;
        let green: u32 = (y + green_offset) as u32;
        unsafe {
          *pixel = (green << 8) | blue;
          pixel = pixel.add(1);
        }
      });
      unsafe {
        row = row.add(self.pitch);
      }
    });
  }

  fn update(
    &mut self,
    device_context: HDC,
    window_width: i32,
    window_height: i32,
  ) {
    unsafe {
      StretchDIBits(
        device_context,
        0,
        0,
        window_width,
        window_height,
        0,
        0,
        self.width,
        self.height,
        self.memory,
        self.info.as_ref().unwrap(),
        DIB_RGB_COLORS,
        SRCCOPY,
      );
    }
  }
  fn resize_dib_section(&mut self, width: i32, height: i32) {
    if self.memory != null_mut() {
      unsafe {
        let free = VirtualFree(self.memory, 0, MEM_RELEASE);
        self.memory = null_mut();
        assert!(free.as_bool());
      }
    }
    self.height = height;
    self.width = width;

    if let Some(mut bitmap) = self.info {
      bitmap.bmiHeader.biSize =
        size_of_val(&bitmap.bmiHeader).try_into().unwrap();
      bitmap.bmiHeader.biWidth = width;
      bitmap.bmiHeader.biHeight = height;
      bitmap.bmiHeader.biPlanes = 1;
      bitmap.bmiHeader.biBitCount = 32;
      bitmap.bmiHeader.biCompression = BI_RGB as u32;
      self.info.replace(bitmap);

      let bitmap_memory_size = width * height * BYTES_PER_PIXEL;
      unsafe {
        self.memory = VirtualAlloc(
          null_mut(),
          bitmap_memory_size as usize,
          MEM_COMMIT,
          PAGE_READWRITE,
        );
        assert_ne!(self.memory, null_mut());
      }
    }
    self.pitch = (width * BYTES_PER_PIXEL) as usize;
  }
}

struct WindowDimensions {
  width: i32,
  height: i32,
}

impl WindowDimensions {
  pub fn new_for(window_handle: HWND) -> Self {
    let mut client_rect = Box::new(RECT::default());
    unsafe {
      GetClientRect(window_handle, client_rect.as_mut());
    }
    Self {
      width: client_rect.right - client_rect.left,
      height: client_rect.bottom - client_rect.top,
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
      // let window_dimensions = WindowDimensions::new_for(window_handle);
      // OFF_SCREEN_BUFFER
      //   .resize_dib_section(window_dimensions.width, window_dimensions.height);
      LRESULT(0)
    }
    WM_PAINT => {
      let paint = Box::leak(Box::new(PAINTSTRUCT::default()));
      let device_context = BeginPaint(window_handle, paint);
      let window_dimensions = WindowDimensions::new_for(window_handle);

      OFF_SCREEN_BUFFER.update(
        device_context,
        window_dimensions.width,
        window_dimensions.height,
      );

      EndPaint(window_handle, paint);
      LRESULT(0)
    }
    _ => DefWindowProcW(window_handle, msg, w_param, l_param),
  }
}

fn main() {
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
      // Window Class Structure

      // Register Window Class
      let class_registered = unsafe { RegisterClassW(window_class) };
      if class_registered != 0 {
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

  if window_handle != HWND(0) {
    // Messaege Struce
    let message = Box::leak(Box::new(MSG::default()));
    let mut x_offset = 0;
    let mut y_offset = 0;

    // Message Loop
    unsafe {
      RUNNING = true;
      OFF_SCREEN_BUFFER.init(window_handle);
      OFF_SCREEN_BUFFER.resize_dib_section(1920, 1080);
      let device_context = GetDC(window_handle);
      while RUNNING {
        while PeekMessageW(message, window_handle, 0, 0, PM_REMOVE).as_bool() {
          if message.message == WM_QUIT {
            RUNNING = false;
          }
          TranslateMessage(message);
          DispatchMessageW(message);
        }
        OFF_SCREEN_BUFFER.render_weird_gradient(x_offset, y_offset);
        let window_dimensions = WindowDimensions::new_for(window_handle);
        // dbg!(&OFF_SCREEN_BUFFER.memory);
        OFF_SCREEN_BUFFER.update(
          device_context,
          window_dimensions.width,
          window_dimensions.height,
        );
        y_offset += 1;
        x_offset += 2;
        // ReleaseDC(window_handle, device_context);
      }
    }
  }
}
