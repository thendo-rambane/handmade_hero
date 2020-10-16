//use std::convert::TryInto;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows as Win32;

static mut Y_OFFSET: i32 = 0;
static mut X_OFFSET: i32 = 0;

#[allow(non_upper_case_globals)]
static mut running: bool = true;
#[allow(non_upper_case_globals)]
static mut global_buffer: *mut OffScreenBuffer = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
static mut controller_manager: *mut ControllerManager = core::ptr::null_mut();

type GetXInputState = fn(u32, *mut Win32::XINPUT_STATE) -> u32;
type SetXInputState = fn(u32, *mut Win32::XINPUT_VIBRATION) -> u32;
type DirectSoundCreate = fn(
    Win32::LPCGUID,
    *mut Win32::LPDIRECTSOUND,
    Win32::LPUNKNOWN,
) -> Win32::HRESULT;
type SomeFunction = Win32::FARPROC;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref GET_X_INPUT_THUNK: GetXInputState =
        |_, _| Win32::ERROR_DEVICE_NOT_CONNECTED;
    static ref SET_X_INPUT_THUNK: SetXInputState =
        |_, _| Win32::ERROR_DEVICE_NOT_CONNECTED;
}

fn init_sound(
    window: Win32::HWND,
    samples_per_sec: u32,
    buffer_size: u32,
) -> Result<Win32::LPDIRECTSOUNDBUFFER, std::io::Error> {
    let direct_sound_lib =
        unsafe { Win32::LoadLibraryA(Win32::c_str_a("dsound.dll").as_ptr()) };
    if !direct_sound_lib.is_null() {
        let create_direct_sound: DirectSoundCreate = unsafe {
            let function = Win32::GetProcAddress(
                direct_sound_lib,
                Win32::c_str_a("DirectSoundCreate").as_ptr(),
            );
            //TODO: Find an alternetive to mem::transmute it is extremely
            //unsafe
            core::mem::transmute::<SomeFunction, DirectSoundCreate>(function)
        };
        let mut direct_sound: Win32::LPDIRECTSOUND = unsafe {
            Box::into_raw(Box::new(core::mem::zeroed::<Win32::IDirectSound>()))
        };
        if Win32::SUCCEEDED(create_direct_sound(
            core::ptr::null(),
            &mut direct_sound,
            core::ptr::null_mut(),
        )) && Win32::SUCCEEDED(unsafe {
            (*direct_sound).SetCooperativeLevel(window, Win32::DSSCL_PRIORITY)
        }) {
            dbg!("DirectSoundCreate OK");
            dbg!("SetCooperativeLevel OK");
        } else {
            // TODO: logging
        }
        let mut wave_format = Win32::WAVEFORMATEX::default();
        wave_format.wFormatTag = Win32::WAVE_FORMAT_PCM;
        wave_format.nChannels = 2;
        wave_format.nSamplesPerSec = samples_per_sec;
        wave_format.wBitsPerSample = 16;
        wave_format.nBlockAlign =
            wave_format.nChannels * wave_format.wBitsPerSample / 8;
        wave_format.nAvgBytesPerSec =
            wave_format.nSamplesPerSec * wave_format.nBlockAlign as u32;

        {
            let mut buffer_desc = Win32::DSBUFFERDESC::default();
            buffer_desc.dwSize =
                core::mem::size_of::<Win32::DSBUFFERDESC>() as u32;
            buffer_desc.dwFlags = Win32::DSBCAPS_PRIMARYBUFFER;

            let mut primary_buffer = Box::into_raw(Box::new(unsafe {
                core::mem::zeroed::<Win32::IDirectSoundBuffer>()
            }));

            if Win32::SUCCEEDED(unsafe {
                (*direct_sound).CreateSoundBuffer(
                    &buffer_desc,
                    &mut primary_buffer,
                    core::ptr::null_mut(),
                )
            }) {
                dbg!("Create primary buffer ok\n");
                if Win32::SUCCEEDED(unsafe {
                    (*primary_buffer).SetFormat(&wave_format)
                }) {
                    dbg!("Primary buffer set format ok\n");
                } else {
                    // TDOO: logging
                }
            }
        }

        let mut buffer_desc = Win32::DSBUFFERDESC::default();
        buffer_desc.dwSize =
            core::mem::size_of::<Win32::DSBUFFERDESC>() as u32;
        buffer_desc.dwFlags = 0;
        buffer_desc.dwBufferBytes = buffer_size as u32;
        buffer_desc.lpwfxFormat = &mut wave_format;
        let mut global_sound_buffer = unsafe {
            Box::into_raw(Box::new(core::mem::zeroed::<
                Win32::IDirectSoundBuffer,
            >()))
        };
        if Win32::SUCCEEDED(unsafe {
            (*direct_sound).CreateSoundBuffer(
                &buffer_desc,
                &mut global_sound_buffer,
                core::ptr::null_mut(),
            )
        }) {
            dbg!("Secondary buffer created\n");

            Ok(global_sound_buffer)
        } else {
            Err(std::io::Error::last_os_error())
            // TODO: logging
        }
    } else {
        // TODO: logging
        Err(std::io::Error::last_os_error())
    }
}

struct WindowDimentions {
    width: i32,
    height: i32,
}

fn get_window_dimensions(window: Win32::HWND) -> WindowDimentions {
    let mut client_rect = Win32::RECT::default();
    unsafe { Win32::GetClientRect(window, &mut client_rect) };
    WindowDimentions {
        width: client_rect.right - client_rect.left,
        height: client_rect.bottom - client_rect.top,
    }
}

struct ControllerManager {
    get_x_input_state: GetXInputState,
    set_x_input_state: SetXInputState,
}

impl ControllerManager {
    fn new(get: GetXInputState, set: SetXInputState) -> Self {
        ControllerManager {
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
        let x_input_lib =
            unsafe { Win32::LoadLibraryA(Win32::c_str_a(x_input).as_ptr()) };
        if !x_input_lib.is_null() {
            self.get_x_input_state = unsafe {
                let input_str = Win32::c_str_a("XInputGetState");
                let get_state =
                    Win32::GetProcAddress(x_input_lib, input_str.as_ptr());
                core::mem::transmute::<SomeFunction, GetXInputState>(get_state)
            };
            self.set_x_input_state = unsafe {
                let output_str = Win32::c_str_a("XInputSetState");
                let set_state =
                    Win32::GetProcAddress(x_input_lib, output_str.as_ptr());
                core::mem::transmute::<SomeFunction, SetXInputState>(set_state)
            }
        }
    }
}

struct OffScreenBuffer {
    memory: *mut Win32::c_void,
    info: Win32::BITMAPINFO,
    width: i32,
    height: i32,
    bytes_per_pixel: usize,
}

impl OffScreenBuffer {
    fn new() -> Self {
        OffScreenBuffer {
            memory: core::ptr::null_mut(),
            info: Win32::BITMAPINFO::default(),
            width: 0,
            height: 0,
            bytes_per_pixel: core::mem::size_of::<u32>(),
        }
    }
    fn create_window(
        &self,
        window_class_name: &str,
        window_name: &str,
    ) -> Result<Win32::HWND, std::io::Error> {
        let window_class_name = Win32::c_str_a(window_class_name);
        let window_name = Win32::c_str_a(window_name);
        let window_instance =
            unsafe { Win32::GetModuleHandleA(core::ptr::null()) };
        let mut window_class = Win32::WNDCLASSA::default();

        window_class.style = Win32::CS_VREDRAW | Win32::CS_HREDRAW;
        window_class.lpfnWndProc = Some(main_window_callback);
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
            core::mem::size_of::<Win32::BITMAPINFOHEADER>() as u32;
        self.info.bmiHeader.biWidth = self.width;
        self.info.bmiHeader.biHeight = -self.height;
        self.info.bmiHeader.biPlanes = 1;
        self.info.bmiHeader.biBitCount = 32;
        self.info.bmiHeader.biCompression = Win32::BI_RGB;

        let size = self.bytes_per_pixel * (self.width * self.height) as usize;
        self.memory = Win32::VirtualAlloc(
            core::ptr::null_mut::<Win32::c_void>(),
            size,
            Win32::MEM_COMMIT | Win32::MEM_RESERVE,
            Win32::PAGE_READWRITE,
        );
    }
}

unsafe fn render_weird_gradient(
    buffer: &mut OffScreenBuffer,
    x_offset: i32,
    y_offset: i32,
) {
    let pixel = buffer.memory.cast::<u32>();
    (0..buffer.width).for_each(|x| {
        (0..buffer.height).for_each(|y| {
            let index = (x + buffer.width * y) as usize;
            let red = pixel.add(index).cast::<u8>().add(2);
            let green = pixel.add(index).cast::<u8>().add(1);
            let blue = pixel.add(index).cast::<u8>().add(0);
            blue.write((x + x_offset) as u8);
            green.write((y + y_offset) as u8);
            red.write(0);
        })
    });
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
            let mut paint_struct = Win32::PAINTSTRUCT::default();
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
            let is_alt_key_down = (l_param & (1 << 29)) != 0;

            match vk_code as i32 {
                Win32::VK_UP => {
                    Y_OFFSET += diff;
                }

                Win32::VK_F4 if is_alt_key_down => running = false,

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
            result = Win32::DefWindowProcA(window, message, w_param, l_param);
        }
    };
    result
}

fn main() {
    let local_controller_manager = unsafe {
        controller_manager = Box::into_raw(Box::new(ControllerManager::new(
            *GET_X_INPUT_THUNK,
            *SET_X_INPUT_THUNK,
        )));
        controller_manager.as_mut().unwrap()
    };
    local_controller_manager.load_x_input("xinput1_4.dll");
    let buffer = unsafe {
        global_buffer = Box::into_raw(Box::new(OffScreenBuffer::new()));
        global_buffer.as_mut().unwrap()
    };

    let window_result =
        buffer.create_window("HandmadeHeroWindowClass", "Handmade Hero");
    if let Ok(window) = window_result {
        if !window.is_null() {
            dbg!("After init sound");
            unsafe {
                let samples_per_sec: u32 = 48000;
                let bytes_per_sample: u32 =
                    (core::mem::size_of::<u16>()) as u32 * 2;
                let sound_buffer_size = samples_per_sec * bytes_per_sample;
                let tone: u32 = 256;
                let tone_volume: i16 = 3000;
                let wave_period: u32 = samples_per_sec.div_euclid(tone);
                let half_wave_period = wave_period / 2;
                let sound_buffer =
                    &*(init_sound(window, samples_per_sec, sound_buffer_size)
                        .unwrap());
                sound_buffer.Play(0, 0, Win32::DSBPLAY_LOOPING);

                buffer.resize_dib_section(1280, 720);
                let mut running_sample_index = 0;
                let mut msg: Win32::MSG = core::mem::zeroed();
                while running {
                    while Win32::PeekMessageA(
                        &mut msg,
                        window,
                        0,
                        0,
                        Win32::PM_REMOVE,
                    ) != 0
                    {
                        Win32::TranslateMessage(&msg);
                        Win32::DispatchMessageA(&msg);
                    }
                    render_weird_gradient(buffer, X_OFFSET, Y_OFFSET);
                    let mut play_cursor: u32 = 0;
                    let mut write_cursor: u32 = 0;
                    if Win32::SUCCEEDED(sound_buffer.GetCurrentPosition(
                        &mut play_cursor,
                        &mut write_cursor,
                    )) {
                        let lock_offset = running_sample_index
                            * bytes_per_sample
                            % sound_buffer_size;
                        let mut region1: Win32::LPVOID = core::ptr::null_mut();
                        let mut region1_size = 0u32;
                        let mut region2: Win32::LPVOID = core::ptr::null_mut();
                        let mut region2_size = 0u32;

                        let bytes_to_lock = match lock_offset == play_cursor {
                            false if lock_offset > play_cursor => {
                                sound_buffer_size - lock_offset + play_cursor
                            }
                            true => sound_buffer_size,
                            _ => play_cursor - lock_offset,
                        };

                        if Win32::SUCCEEDED(sound_buffer.Lock(
                            lock_offset,
                            bytes_to_lock,
                            &mut region1,
                            &mut region1_size,
                            &mut region2,
                            &mut region2_size,
                            0,
                        )) {
                            let mut sample_out: *mut i16 = region1.cast();
                            for _ in 0..region1_size
                                .div_euclid(bytes_per_sample)
                                as usize
                            {
                                let value = if running_sample_index
                                    .div_euclid(half_wave_period)
                                    .rem_euclid(2)
                                    != 0
                                {
                                    tone_volume
                                } else {
                                    -tone_volume
                                };
                                sample_out.write(value);
                                sample_out = sample_out.add(1);
                                sample_out.write(value);
                                sample_out = sample_out.add(1);
                                running_sample_index += 1;
                            }

                            let mut sample_out: *mut i16 = region2.cast();
                            for _ in 0..region2_size
                                .div_euclid(bytes_per_sample)
                                as usize
                            {
                                let value = if running_sample_index
                                    .div_euclid(half_wave_period)
                                    .rem_euclid(2)
                                    != 0
                                {
                                    tone_volume
                                } else {
                                    -tone_volume
                                };
                                sample_out.write(value);
                                sample_out = sample_out.add(1);
                                sample_out.write(value);
                                sample_out = sample_out.add(1);
                                running_sample_index += 1;
                            }

                            sound_buffer.Unlock(
                                region1,
                                region1_size,
                                region2,
                                region2_size,
                            );
                        }
                    }
                    for i in 0..Win32::XUSER_MAX_COUNT {
                        let state = Box::into_raw(Box::new(
                            Win32::XINPUT_STATE::default(),
                        ));

                        {
                            let state_result = local_controller_manager
                                .get_x_input_state(i, state);
                            if state_result == Win32::ERROR_SUCCESS {
                                let pad = (*state).Gamepad;
                                let _up = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_UP
                                    != 0;

                                let _down = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_DOWN
                                    != 0;
                                let _left = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_LEFT
                                    != 0;
                                let _right = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_DPAD_RIGHT
                                    != 0;
                                let _start = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_START
                                    != 0;
                                let _back = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_BACK
                                    != 0;
                                let _left_shoulder = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_LEFT_SHOULDER
                                    != 0;
                                let _right_shoulder = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_LEFT_SHOULDER
                                    != 0;
                                let a_button = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_A
                                    != 0;
                                let b_button = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_B
                                    != 0;
                                let x_button = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_X
                                    != 0;
                                let y_button = pad.wButtons
                                    & Win32::XINPUT_GAMEPAD_Y
                                    != 0;
                                let _stick_x = pad.sThumbLX;
                                let _stick_y = pad.sThumbLY;
                                if a_button {
                                    dbg!("a");
                                }
                                if y_button {
                                    dbg!("y");
                                }
                                if b_button {
                                    dbg!("b");
                                }
                                if x_button {
                                    dbg!("x");
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
                }
            }
        } else {
            dbg!("WINDOW_IS_NULL"); //TODO:{Thendo} LOGGING
        }
    } else {
        dbg!("COULD NOT CREATE WINDOW"); //TODO:{Thendo} LOGGING
        dbg!(std::io::Error::last_os_error());
    }
}
