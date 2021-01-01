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
    pub samples: *mut i16,
    pub sample_count: u32,
    pub buffer_size: u32,
    pub bytes_per_sample: u32,
    pub tone_volume: i16,
    pub wave_period: u32,
    pub time: f32,
    pub count: u32,
}

impl GameAudioBuffer {
    pub fn new(
        buffer_size: u32,
        bytes_per_sample: u32,
        tone_volume: i16,
        samples_per_second: u32,
        freq: u32,
    ) -> Self {
        Self {
            samples: core::ptr::null_mut(),
            buffer_size,
            sample_count: 0,
            bytes_per_sample,
            tone_volume,
            wave_period: samples_per_second / freq,
            time: 0.0,
            count: 0,
        }
    }
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

pub fn fill_audio_buffer(audio_buffer: &mut GameAudioBuffer) {
    let mut sample_out = audio_buffer.samples;
    let mut time = 0f32;
    for _ in 0..audio_buffer.sample_count {
        let value = (time.sin() * audio_buffer.tone_volume as f32) as i16;
        unsafe {
            sample_out.write(value);
            sample_out = sample_out.add(1);
            sample_out.write(value);
            sample_out = sample_out.add(1);
        }
        time = 2.0
            * std::f32::consts::PI
            * (audio_buffer.count as f32 / audio_buffer.wave_period as f32);
        audio_buffer.count += 1;
    }
}

pub fn game_update_and_render(
    video_buffer: &mut GameScreenBuffer,
    audio_buffer: &mut GameAudioBuffer,
    x_offset: i32,
    y_offset: i32,
) {
    render_weird_gradient(video_buffer, x_offset, y_offset);
    fill_audio_buffer(audio_buffer);
}
