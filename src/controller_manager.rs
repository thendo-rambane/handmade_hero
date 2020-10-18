use crate::win32;
pub type GetXInputState = fn(u32, *mut win32::XINPUT_STATE) -> u32;
pub type SetXInputState = fn(u32, *mut win32::XINPUT_VIBRATION) -> u32;

type SomeFunction = win32::FARPROC;
pub struct ControllerManager {
    pub get_x_input_state: GetXInputState,
    pub set_x_input_state: SetXInputState,
}
impl Default for ControllerManager {
    fn default() -> Self {
        ControllerManager::new(
            |_, _| win32::ERROR_DEVICE_NOT_CONNECTED,
            |_, _| win32::ERROR_DEVICE_NOT_CONNECTED,
        )
    }
}

impl ControllerManager {
    pub fn new(get: GetXInputState, set: SetXInputState) -> Self {
        ControllerManager {
            get_x_input_state: get,
            set_x_input_state: set,
        }
    }
    pub fn get_x_input_state(
        &self,
        index: u32,
        state: *mut win32::XINPUT_STATE,
    ) -> u32 {
        (self.get_x_input_state)(index, state)
    }

    #[allow(dead_code)]
    pub fn set_x_input_state(
        &self,
        index: u32,
        vibration: *mut win32::XINPUT_VIBRATION,
    ) -> u32 {
        (self.set_x_input_state)(index, vibration)
    }

    pub fn load_x_input(&mut self, x_input: &str) {
        let x_input_lib =
            unsafe { win32::LoadLibraryA(win32::c_str_a(x_input).as_ptr()) };
        if !x_input_lib.is_null() {
            self.get_x_input_state = unsafe {
                let input_str = win32::c_str_a("XInputGetState");
                let get_state =
                    win32::GetProcAddress(x_input_lib, input_str.as_ptr());
                core::mem::transmute::<SomeFunction, GetXInputState>(get_state)
            };
            self.set_x_input_state = unsafe {
                let output_str = win32::c_str_a("XInputSetState");
                let set_state =
                    win32::GetProcAddress(x_input_lib, output_str.as_ptr());
                core::mem::transmute::<SomeFunction, SetXInputState>(set_state)
            }
        }
    }
}
