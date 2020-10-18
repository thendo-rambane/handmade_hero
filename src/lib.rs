use core::ffi::c_void;

pub struct GameScreenBuffer {
    pub memory: *mut c_void,
    pub width: i32,
    pub height: i32,
    pub bytes_per_pixel: usize,
}

impl GameScreenBuffer {
    pub fn new(
        memory: *mut c_void,
        bytes_per_pixel: usize,
        height: i32,
        width: i32,
    ) -> Self {
        Self {
            memory,
            width,
            height,
            bytes_per_pixel,
        }
    }
}

pub struct GameAudioBuffer {
    pub samples: *mut c_void,
    pub buffer_size: u32,
    bytes_per_sample: u32,
}

fn render_weird_gradient(
    buffer: &mut GameScreenBuffer,
    x_offset: i32,
    y_offset: i32,
) {
    let pixel = buffer.memory.cast::<u32>();
    for x in 0..buffer.width {
        for y in 0..buffer.height {
            let index = (x + buffer.width * y) as usize;
            let red = unsafe { pixel.add(index).cast::<u8>().add(2) };
            let green = unsafe { pixel.add(index).cast::<u8>().add(1) };
            let blue = unsafe { pixel.add(index).cast::<u8>().add(0) };
            unsafe {
                blue.write((x + x_offset) as u8);
                green.write((y + y_offset) as u8);
                red.write(0);
            }
        }
    }
}
pub fn game_update_and_render(
    buffer: &mut GameScreenBuffer,
    x_offset: i32,
    y_offset: i32,
) {
    render_weird_gradient(buffer, x_offset, y_offset);
}
