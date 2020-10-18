use crate::win32 as Win32;

pub struct OffScreenBuffer {
    pub memory: *mut Win32::c_void,
    info: Win32::BITMAPINFO,
    pub width: i32,
    pub height: i32,
    pub bytes_per_pixel: usize,
}

impl Default for OffScreenBuffer {
    fn default() -> Self {
        OffScreenBuffer::new()
    }
}

impl OffScreenBuffer {
    pub fn new() -> Self {
        OffScreenBuffer {
            memory: core::ptr::null_mut(),
            info: Win32::BITMAPINFO::default(),
            width: 0,
            height: 0,
            bytes_per_pixel: core::mem::size_of::<u32>(),
        }
    }
    pub fn create_window(
        &self,
        window_class_name: &str,
        window_name: &str,
        window_proc: Win32::WNDPROC,
    ) -> Result<Win32::HWND, std::io::Error> {
        let window_class_name = Win32::c_str_a(window_class_name);
        let window_name = Win32::c_str_a(window_name);
        let window_instance =
            unsafe { Win32::GetModuleHandleA(core::ptr::null()) };
        let mut window_class = Win32::WNDCLASSA::default();

        window_class.style = Win32::CS_VREDRAW | Win32::CS_HREDRAW;
        window_class.lpfnWndProc = window_proc;
        window_class.hInstance = window_instance;
        window_class.lpszClassName = window_class_name.as_ptr();
        if unsafe { Win32::RegisterClassA(&window_class) } != 0 {
            let window = unsafe {
                Win32::CreateWindowExA(
                    0,
                    window_class_name.as_ptr(),
                    window_name.as_ptr(),
                    Win32::WS_OVERLAPPEDWINDOW | Win32::WS_VISIBLE,
                    Win32::CW_USEDEFAULT,
                    Win32::CW_USEDEFAULT,
                    Win32::CW_USEDEFAULT,
                    Win32::CW_USEDEFAULT,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    window_instance,
                    core::ptr::null_mut(),
                )
            };
            if !window.is_null() {
                Ok(window)
            } else {
                dbg!("window is null");
                Err(std::io::Error::last_os_error())
            }
        } else {
            dbg!("Class not registered");
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn update_window(
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

    pub fn resize_dib_section(&mut self, width: i32, height: i32) {
        if !self.memory.is_null() {
            unsafe { Win32::VirtualFree(self.memory, 0, Win32::MEM_RELEASE) };
        }

        self.height = height;
        self.width = width;

        self.info.bmiHeader.biSize =
            core::mem::size_of::<Win32::BITMAPINFOHEADER>() as u32;
        self.info.bmiHeader.biWidth = self.width;
        self.info.bmiHeader.biHeight = -self.height;
        self.info.bmiHeader.biPlanes = 1;
        self.info.bmiHeader.biBitCount = 32;
        self.info.bmiHeader.biCompression = Win32::BI_RGB;

        let size = self.bytes_per_pixel * (self.width * self.height) as usize;
        self.memory = unsafe {
            Win32::VirtualAlloc(
                core::ptr::null_mut::<Win32::c_void>(),
                size,
                Win32::MEM_COMMIT | Win32::MEM_RESERVE,
                Win32::PAGE_READWRITE,
            )
        };
    }
}
