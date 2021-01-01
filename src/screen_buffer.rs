use crate::win32;

pub struct OffScreenBuffer {
    pub memory: *mut win32::c_void,
    info: win32::BITMAPINFO,
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
            info: win32::BITMAPINFO::default(),
            width: 0,
            height: 0,
            bytes_per_pixel: core::mem::size_of::<u32>(),
        }
    }
    pub fn create_window(
        &self,
        window_class_name: &str,
        window_name: &str,
        window_proc: win32::WNDPROC,
    ) -> Result<win32::HWND, std::io::Error> {
        let window_class_name = win32::c_str_a(window_class_name);
        let window_name = win32::c_str_a(window_name);
        let window_instance =
            unsafe { win32::GetModuleHandleA(core::ptr::null()) };
        let mut window_class = win32::WNDCLASSA::default();

        window_class.style = win32::CS_VREDRAW | win32::CS_HREDRAW;
        window_class.lpfnWndProc = window_proc;
        window_class.hInstance = window_instance;
        window_class.lpszClassName = window_class_name.as_ptr();
        if unsafe { win32::RegisterClassA(&window_class) } != 0 {
            let window = unsafe {
                win32::CreateWindowExA(
                    0,
                    window_class_name.as_ptr(),
                    window_name.as_ptr(),
                    win32::WS_OVERLAPPEDWINDOW | win32::WS_VISIBLE,
                    win32::CW_USEDEFAULT,
                    win32::CW_USEDEFAULT,
                    win32::CW_USEDEFAULT,
                    win32::CW_USEDEFAULT,
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
        device_context: win32::HDC,
        window_width: i32,
        window_height: i32,
    ) {
        unsafe {
            win32::StretchDIBits(
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
                win32::DIB_RGB_COLORS,
                win32::SRCCOPY,
            )
        };
    }

    pub fn resize_dib_section(&mut self, width: i32, height: i32) {
        if !self.memory.is_null() {
            unsafe { win32::VirtualFree(self.memory, 0, win32::MEM_RELEASE) };
        }

        self.height = height;
        self.width = width;

        self.info.bmiHeader.biSize =
            core::mem::size_of::<win32::BITMAPINFOHEADER>() as u32;
        self.info.bmiHeader.biWidth = self.width;
        self.info.bmiHeader.biHeight = -self.height;
        self.info.bmiHeader.biPlanes = 1;
        self.info.bmiHeader.biBitCount = 32;
        self.info.bmiHeader.biCompression = win32::BI_RGB;

        let size = self.bytes_per_pixel * (self.width * self.height) as usize;
        self.memory = unsafe {
            win32::VirtualAlloc(
                core::ptr::null_mut::<win32::c_void>(),
                size,
                win32::MEM_COMMIT | win32::MEM_RESERVE,
                win32::PAGE_READWRITE,
            )
        };
    }
}
