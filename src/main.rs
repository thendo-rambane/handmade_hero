mod controller_manager;
mod screen_buffer;
mod sound_buffer;
mod win32;

use controller_manager::*;
use screen_buffer::*;
use sound_buffer::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_rdtsc;

#[cfg(target_arch = "x86")]
use core::arch::x86::_rdtsc;

static mut Y_OFFSET: i32 = 0;
static mut X_OFFSET: i32 = 0;

#[allow(non_upper_case_globals)]
static mut running: bool = true;

#[allow(non_upper_case_globals)]
static mut global_buffer: *mut OffScreenBuffer = core::ptr::null_mut();

struct WindowDimentions {
    width: i32,
    height: i32,
}

fn get_window_dimensions(window: win32::HWND) -> WindowDimentions {
    let mut client_rect = win32::RECT::default();
    unsafe { win32::GetClientRect(window, &mut client_rect) };
    WindowDimentions {
        width: client_rect.right - client_rect.left,
        height: client_rect.bottom - client_rect.top,
    }
}

impl From<&mut OffScreenBuffer> for handmade_hero::GameScreenBuffer {
    fn from(screen_buffer: &mut OffScreenBuffer) -> Self {
        handmade_hero::GameScreenBuffer::new(
            screen_buffer.memory.cast(),
            screen_buffer.bytes_per_pixel,
            screen_buffer.height,
            screen_buffer.width,
        )
    }
}
impl From<&mut SoundOutput> for handmade_hero::GameAudioBuffer {
    fn from(sound_output: &mut SoundOutput) -> Self {
        handmade_hero::GameAudioBuffer::new(
            sound_output.buffer_size,
            sound_output.bytes_per_sample,
            sound_output.volume,
            sound_output.samples_per_second,
            sound_output.freq,
        )
    }
}

unsafe extern "system" fn main_window_callback(
    window: win32::HWND,
    message: win32::UINT,
    w_param: win32::WPARAM,
    l_param: win32::LPARAM,
) -> win32::LRESULT {
    let mut result: win32::LRESULT = 0;
    let buffer = if let Some(buffer) = global_buffer.as_mut() {
        buffer
    } else {
        panic!("COULD NOT UNWRAP BUFFER")
    };

    match message {
        win32::WM_ACTIVATEAPP => {
            dbg!("WM_ACTIVATEAPP");
        }
        win32::WM_CLOSE => {
            running = false;
            dbg!("WM_CLOSE");
        }
        win32::WM_DESTROY => {
            dbg!("WM_DESTROY");
        }
        win32::WM_PAINT => {
            let mut paint_struct = win32::PAINTSTRUCT::default();
            let device_context: win32::HDC =
                win32::BeginPaint(window, &mut paint_struct);
            let window_dimensions = get_window_dimensions(window);
            buffer.update_window(
                device_context,
                window_dimensions.width,
                window_dimensions.height,
            );
            win32::EndPaint(window, &paint_struct);
            dbg!("WM_PAINT");
        }
        win32::WM_SIZE => {
            dbg!("WM_SIZE");
        }
        win32::WM_SYSKEYUP
        | win32::WM_SYSKEYDOWN
        | win32::WM_KEYUP
        | win32::WM_KEYDOWN => {
            let vk_code = w_param;
            let _was_down = (l_param & (1 << 30)) != 0;
            let _is_down = (l_param & (1 << 31)) == 0;

            let diff = 20;
            let is_alt_key_down = (l_param & (1 << 29)) != 0;

            match vk_code as i32 {
                win32::VK_UP => {
                    Y_OFFSET += diff;
                }

                win32::VK_F4 if is_alt_key_down => running = false,

                win32::VK_DOWN => {
                    Y_OFFSET -= diff;
                }

                win32::VK_LEFT => {
                    X_OFFSET -= diff;
                }

                win32::VK_RIGHT => {
                    X_OFFSET += diff;
                }
                _ => {}
            }
        }
        _ => {
            result = win32::DefWindowProcA(window, message, w_param, l_param);
        }
    };
    result
}

fn main() {
    // Get get initial counter
    let mut counter_per_second = win32::LARGE_INTEGER::default();
    unsafe {
        win32::QueryPerformanceFrequency(&mut counter_per_second);
    }
    let mut local_controller_manager = ControllerManager::default();
    local_controller_manager.load_x_input("xinput1_4.dll");
    let buffer = unsafe {
        global_buffer = Box::into_raw(Box::new(OffScreenBuffer::new()));
        global_buffer.as_mut().unwrap()
    };
    let window_result = buffer.create_window(
        "HandmadeHeroWindowClass",
        "Handmade Hero",
        Some(main_window_callback),
    );
    if let Ok(window) = window_result {
        if !window.is_null() {
            dbg!("After init sound");
            unsafe {
                let mut sound_output = SoundOutput::default();
                let sound_buffer = &*sound_output.init_sound(window).unwrap();
                let sound_memory = win32::VirtualAlloc(
                    core::ptr::null_mut(),
                    sound_output.buffer_size as usize,
                    win32::MEM_COMMIT | win32::MEM_RESERVE,
                    win32::PAGE_READWRITE,
                );
                if sound_memory.is_null() {
                    panic!("Could Not Allocate Sound buffer");
                }
                buffer.resize_dib_section(1280, 720);
                // counter buffer
                let mut last_cycle_counter: u64 = _rdtsc();
                let mut last_counter = win32::LARGE_INTEGER::default();
                win32::QueryPerformanceCounter(&mut last_counter);
                sound_output.clear_sound_buffer();
                sound_buffer.Play(0, 0, win32::DSBPLAY_LOOPING);

                let mut msg: win32::MSG = win32::MSG::default();
                while running {
                    while win32::PeekMessageA(
                        &mut msg,
                        window,
                        0,
                        0,
                        win32::PM_REMOVE,
                    ) != 0
                    {
                        win32::TranslateMessage(&msg);
                        win32::DispatchMessageA(&msg);
                    }

                    let mut lock_offset = 0u32;
                    let mut bytes_to_lock = 0u32;
                    let mut sound_ready = false;
                    if win32::SUCCEEDED(sound_buffer.GetCurrentPosition(
                        &mut sound_output.play_cursor,
                        &mut sound_output.write_cursor,
                    )) {
                        let taget_cursor = sound_output.play_cursor;
                        //(sound_output.play_cursor
                        //+ sound_output.latency_sample_count
                        //* sound_output.bytes_per_sample)
                        //.rem_euclid(sound_output.buffer_size);
                        lock_offset = (sound_output.running_sample_index
                            * sound_output.bytes_per_sample)
                            .rem_euclid(sound_output.buffer_size);

                        //TODO: need a more accurate check for play_cursor
                        bytes_to_lock = match lock_offset > taget_cursor {
                            true => {
                                sound_output.buffer_size - lock_offset
                                    + taget_cursor
                            }
                            false => taget_cursor - lock_offset,
                        };
                        sound_ready = true;
                    }
                    let mut game_audio: handmade_hero::GameAudioBuffer =
                        (&mut sound_output).into();
                    game_audio.samples = sound_memory.cast();
                    game_audio.sample_count =
                        bytes_to_lock / sound_output.bytes_per_sample;
                    handmade_hero::game_update_and_render(
                        &mut buffer.into(),
                        &mut game_audio,
                        X_OFFSET,
                        Y_OFFSET,
                    );
                    if Y_OFFSET <= 1000 && Y_OFFSET >= -1000 {
                        let tmp = (Y_OFFSET / 1000) * 256 + 512;
                        sound_output.wave_period =
                            sound_output.samples_per_second / tmp as u32;
                    }

                    // Get input state
                    for i in 0..win32::XUSER_MAX_COUNT {
                        let state = Box::into_raw(Box::new(
                            win32::XINPUT_STATE::default(),
                        ));

                        {
                            let state_result = local_controller_manager
                                .get_x_input_state(i, state);
                            if state_result == win32::ERROR_SUCCESS {
                                let pad = (*state).Gamepad;
                                let _up = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_DPAD_UP
                                    != 0;

                                let _down = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_DPAD_DOWN
                                    != 0;
                                let _left = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_DPAD_LEFT
                                    != 0;
                                let _right = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_DPAD_RIGHT
                                    != 0;
                                let _start = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_START
                                    != 0;
                                let _back = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_BACK
                                    != 0;
                                let _left_shoulder = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_LEFT_SHOULDER
                                    != 0;
                                let _right_shoulder = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_LEFT_SHOULDER
                                    != 0;
                                let a_button = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_A
                                    != 0;
                                let b_button = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_B
                                    != 0;
                                let x_button = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_X
                                    != 0;
                                let y_button = pad.wButtons
                                    & win32::XINPUT_GAMEPAD_Y
                                    != 0;
                                let stick_x = pad.sThumbLX;
                                let stick_y = pad.sThumbLY;

                                Y_OFFSET += stick_y as i32;
                                X_OFFSET += stick_x as i32;

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
                    if sound_ready {
                        sound_output.fill_sound_buffer(
                            &game_audio,
                            lock_offset,
                            bytes_to_lock,
                        );
                    }
                    sound_buffer.Play(0, 0, win32::DSBPLAY_LOOPING);
                    let device_context = win32::GetDC(window);
                    let window_dimensions = get_window_dimensions(window);
                    buffer.update_window(
                        device_context,
                        window_dimensions.width,
                        window_dimensions.height,
                    );
                    win32::ReleaseDC(window, device_context);

                    let current_cycle_counter = _rdtsc();
                    let mut current_counter = win32::LARGE_INTEGER::default();
                    win32::QueryPerformanceCounter(&mut current_counter);

                    // Calculate counter elapsed
                    let counter_elapsed =
                        current_counter.QuadPart() - last_counter.QuadPart();

                    //Calculate cycle elapsed
                    let cycle_elapsed =
                        current_cycle_counter - last_cycle_counter;

                    // Calculate time elapsed per frame
                    let mili_sec_per_frame = 1000.0f32
                        * counter_elapsed as f32
                        / *counter_per_second.QuadPart() as f32;

                    // Calculate frame per second
                    let frames_per_sec = *counter_per_second.QuadPart() as f32
                        / counter_elapsed as f32;

                    //Calculate mega cycles per second
                    let mcpf = cycle_elapsed as f32 / (1000f32 * 1000f32);
                    println!(
                        "ms/f:{:.2},\t fps:{:.2},\t mc/f:{:.2},\t",
                        mili_sec_per_frame, frames_per_sec, mcpf
                    );

                    //Set counter and cycle counter for the next run
                    last_counter = current_counter;
                    last_cycle_counter = current_cycle_counter;
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
