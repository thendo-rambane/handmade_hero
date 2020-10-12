#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows as Win32;

static mut Y_OFFSET: i32 = 0;
static mut X_OFFSET: i32 = 0;

#[allow(non_upper_case_globals)]
static mut running: bool = true;
#[allow(non_upper_case_globals)]
static mut global_buffer: *mut OffScreenBuffer = std::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut controller: *mut Controller = std::ptr::null_mut();

type GetXInputState = fn(u32, *mut Win32::XINPUT_STATE) -> u32;
type SetXInputState = fn(u32, *mut Win32::XINPUT_VIBRATION) -> u32;
type SomeXInputFunction = Win32::FARPROC;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref GET_X_INPUT_THUNK: GetXInputState = |_, _| 0;
    static ref SET_X_INPUT_THUNK: SetXInputState = |_, _| 0;
}

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

struct Controller {
    get_x_input_state: GetXInputState,
    set_x_input_state: SetXInputState,
}

impl Controller {
    fn new(get: GetXInputState, set: SetXInputState) -> Self {
        Self {
            get_x_input_state: get,
            set_x_input_state: set,
        }
    }
    fn get_x_input_state(
        &self,
        index: u32,
        state: *mut Win32::XINPUT_STATE,
    ) -> u32 {
        (self.get_x_input_state)(index, state)
    }

    #[allow(dead_code)]
    fn set_x_input_state(
        &self,
        index: u32,
        vibration: *mut Win32::XINPUT_VIBRATION,
    ) -> u32 {
        (self.set_x_input_state)(index, vibration)
    }
    fn load_x_input(&mut self, x_input: &str) {
        let library =
            unsafe { Win32::LoadLibraryW(Win32::c_str(x_input).as_ptr()) };
        if !library.is_null() {
            self.get_x_input_state = unsafe {
                let input_str =
                    std::ffi::CString::new("XInputGetState").unwrap();

                let get_state =
                    Win32::GetProcAddress(library, input_str.as_ptr());
                std::mem::transmute::<SomeXInputFunction, GetXInputState>(
                    get_state,
                )
            };
            self.set_x_input_state = unsafe {
                let output_str =
                    std::ffi::CString::new("XInputSetState").unwrap();
                let set_state =
                    Win32::GetProcAddress(library, output_str.as_ptr());
                std::mem::transmute::<SomeXInputFunction, SetXInputState>(
                    set_state,
                )
            }
        }
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
            pixel.write((green << 8) | blue);
        })
    })
}
unsafe extern "system" fn main_window_callback(
    window: Win32::HWND,
    message: Win32::UINT,
    w_param: Win32::WPARAM,
    l_param: Win32::LPARAM,
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
        Win32::WM_SYSKEYUP
        | Win32::WM_SYSKEYDOWN
        | Win32::WM_KEYUP
        | Win32::WM_KEYDOWN => {
            let vk_code = w_param;
            let _was_down = (l_param & (1 << 30)) != 0;
            let _is_down = (l_param & (1 << 31)) == 0;

            let diff = 20;

            match vk_code as i32 {
                Win32::VK_UP => {
                    Y_OFFSET += diff;
                }

                Win32::VK_DOWN => {
                    Y_OFFSET -= diff;
                }

                Win32::VK_LEFT => {
                    X_OFFSET -= diff;
                }

                Win32::VK_RIGHT => {
                    X_OFFSET += diff;
                }
                _ => {}
            }
        }
        _ => {
            result = Win32::DefWindowProcW(window, message, w_param, l_param);
        }
    };
    result
}

fn main() {
    let local_controller = unsafe {
        controller = Box::into_raw(Box::new(Controller::new(
            *GET_X_INPUT_THUNK,
            *SET_X_INPUT_THUNK,
        )));
        controller.as_mut().unwrap()
    };
    local_controller.load_x_input("xinput1_4.dll");

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
                    render_weird_gradient(buffer, X_OFFSET, Y_OFFSET);
                    for i in 0..Win32::XUSER_MAX_COUNT {
                        let state =
                            Box::into_raw(Box::new(std::mem::zeroed::<
                                Win32::XINPUT_STATE,
                            >(
                            )));

                        {
                            let state_result =
                                local_controller.get_x_input_state(i, state);
                            if state_result == Win32::ERROR_SUCCESS {
                                let pad = (*state).Gamepad;
                                let _up = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_UP;

                                let _down = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_DOWN;
                                let _left = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_LEFT;
                                let _right = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_RIGHT;
                                let _start =
                                    pad.wButtons & Win32::XINPUT_GAMEPAD_START;
                                let _back =
                                    pad.wButtons & Win32::XINPUT_GAMEPAD_BACK;
                                let _left_shoulder = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_LEFT_SHOULDER;
                                let _right_shoulder = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_LEFT_SHOULDER;
                                let a_button =
                                    pad.wButtons & Win32::XINPUT_GAMEPAD_A;
                                let _b_button =
                                    pad.wButtons & Win32::XINPUT_GAMEPAD_B;
                                let _x_button =
                                    pad.wButtons & Win32::XINPUT_GAMEPAD_X;
                                let _y_button =
                                    pad.wButtons & Win32::XINPUT_GAMEPAD_Y;
                                let _stick_x = pad.sThumbLX;
                                let _stick_y = pad.sThumbLY;
                                if a_button > 0 {
                                    dbg!("AAAAAA");
                                }
                            } else {
                                // Controller is not connected
                            }
                        }
                    }
                    let device_context = Win32::GetDC(window);
                    let window_dimensions = get_window_dimensions(window);
                    buffer.update_window(
                        device_context,
                        window_dimensions.width,
                        window_dimensions.height,
                    );
                    Win32::ReleaseDC(window, device_context);
                    X_OFFSET += 1;
                    Y_OFFSET += 2;
                }
            }
        } else {
            dbg!("WINDOW_IS_NULL"); //TODO:{Thendo} LOGGING
        }
    } else {
        dbg!("FAILED TO REGISTER CLASS"); //TODO:{Thendo} LOGGING
    }
}
