use crate::win32;

type DirectSoundCreate = fn(
    win32::LPCGUID,
    *mut win32::LPDIRECTSOUND,
    win32::LPUNKNOWN,
) -> win32::HRESULT;

type SomeFunction = win32::FARPROC;

pub struct SoundOutput {
    pub play_cursor: u32,
    pub write_cursor: u32,
    pub samples_per_second: u32,
    pub running_sample_index: u32,
    pub buffer_size: u32,
    pub bytes_per_sample: u32,
    pub time: f32,
    pub volume: i16,
    pub wave_period: u32,
    pub buffer: win32::LPDIRECTSOUNDBUFFER,
}

impl Default for SoundOutput {
    fn default() -> Self {
        SoundOutput::new(
            48000,
            (core::mem::size_of::<u16>() * 2) as u32,
            251,
            1500,
        )
    }
}

impl SoundOutput {
    pub fn new(
        samples_per_second: u32,
        bytes_per_sample: u32,
        freq: u32,
        volume: i16,
    ) -> Self {
        let buffer_size = samples_per_second * bytes_per_sample;
        let wave_period = samples_per_second.div_euclid(freq);
        Self {
            time: 0.0,
            samples_per_second,
            buffer_size,
            bytes_per_sample,
            volume,
            wave_period,
            running_sample_index: 0,
            buffer: core::ptr::null_mut(),
            play_cursor: 0,
            write_cursor: 0,
        }
    }
    pub fn init_sound(
        &mut self,
        window: win32::HWND,
    ) -> Result<win32::LPDIRECTSOUNDBUFFER, std::io::Error> {
        let direct_sound_lib = unsafe {
            win32::LoadLibraryA(win32::c_str_a("dsound.dll").as_ptr())
        };
        if !direct_sound_lib.is_null() {
            let create_direct_sound: DirectSoundCreate = unsafe {
                let function = win32::GetProcAddress(
                    direct_sound_lib,
                    win32::c_str_a("DirectSoundCreate").as_ptr(),
                );
                //TODO: Find an alternetive to mem::transmute it is extremely
                //unsafe
                core::mem::transmute::<SomeFunction, DirectSoundCreate>(
                    function,
                )
            };
            let mut direct_sound: win32::LPDIRECTSOUND = unsafe {
                Box::into_raw(Box::new(
                    core::mem::zeroed::<win32::IDirectSound>(),
                ))
            };
            if win32::SUCCEEDED(create_direct_sound(
                core::ptr::null(),
                &mut direct_sound,
                core::ptr::null_mut(),
            )) && win32::SUCCEEDED(unsafe {
                (*direct_sound)
                    .SetCooperativeLevel(window, win32::DSSCL_PRIORITY)
            }) {
                dbg!("DirectSoundCreate OK");
                dbg!("SetCooperativeLevel OK");
            } else {
                // TODO: logging
            }

            let mut wave_format = win32::WAVEFORMATEX::default();
            wave_format.wFormatTag = win32::WAVE_FORMAT_PCM;
            wave_format.nChannels = 2;
            wave_format.nSamplesPerSec = self.samples_per_second;
            wave_format.wBitsPerSample = 16;
            wave_format.nBlockAlign =
                wave_format.nChannels * wave_format.wBitsPerSample / 8;
            wave_format.nAvgBytesPerSec =
                wave_format.nSamplesPerSec * wave_format.nBlockAlign as u32;

            {
                let mut buffer_desc = win32::DSBUFFERDESC::default();
                buffer_desc.dwSize =
                    core::mem::size_of::<win32::DSBUFFERDESC>() as u32;
                buffer_desc.dwFlags = win32::DSBCAPS_PRIMARYBUFFER;

                let mut primary_buffer = Box::into_raw(Box::new(unsafe {
                    core::mem::zeroed::<win32::IDirectSoundBuffer>()
                }));

                if win32::SUCCEEDED(unsafe {
                    (*direct_sound).CreateSoundBuffer(
                        &buffer_desc,
                        &mut primary_buffer,
                        core::ptr::null_mut(),
                    )
                }) {
                    dbg!("Create primary buffer ok\n");
                    if win32::SUCCEEDED(unsafe {
                        (*primary_buffer).SetFormat(&wave_format)
                    }) {
                        dbg!("Primary buffer set format ok\n");
                    } else {
                        // TDOO: logging
                    }
                }
            }

            let mut buffer_desc = win32::DSBUFFERDESC::default();
            buffer_desc.dwSize =
                core::mem::size_of::<win32::DSBUFFERDESC>() as u32;
            buffer_desc.dwFlags = 0;
            buffer_desc.dwBufferBytes = self.buffer_size as u32;
            buffer_desc.lpwfxFormat = &mut wave_format;
            self.buffer = unsafe {
                Box::into_raw(Box::new(core::mem::zeroed::<
                    win32::IDirectSoundBuffer,
                >()))
            };
            if win32::SUCCEEDED(unsafe {
                (*direct_sound).CreateSoundBuffer(
                    &buffer_desc,
                    &mut self.buffer,
                    core::ptr::null_mut(),
                )
            }) {
                dbg!("Secondary buffer created\n");

                Ok(self.buffer)
            } else {
                Err(std::io::Error::last_os_error())
                // TODO: logging
            }
        } else {
            // TODO: logging
            Err(std::io::Error::last_os_error())
        }
    }
    pub fn fill_sound_buffer(&mut self) {
        let buffer = unsafe { self.buffer.as_mut().unwrap() };
        if win32::SUCCEEDED(unsafe {
            buffer.GetCurrentPosition(
                &mut self.play_cursor,
                &mut self.write_cursor,
            )
        }) {
            let mut region1: win32::LPVOID = core::ptr::null_mut();
            let mut region1_size = 0u32;
            let mut region2: win32::LPVOID = core::ptr::null_mut();
            let mut region2_size = 0u32;

            let lock_offset = (self.running_sample_index
                * self.bytes_per_sample)
                .rem_euclid(self.buffer_size);

            //TODO: need a more accurate check for play_cursor
            let bytes_to_lock: u32 = match lock_offset > self.play_cursor {
                true => self.buffer_size - lock_offset + self.play_cursor,
                _ => self.play_cursor - lock_offset,
            };
            if win32::SUCCEEDED(unsafe {
                buffer.Lock(
                    lock_offset,
                    bytes_to_lock,
                    &mut region1,
                    &mut region1_size,
                    &mut region2,
                    &mut region2_size,
                    0,
                )
            }) {
                let mut sample_out: *mut i16 = region1.cast();
                for _ in 0..region1_size.div_euclid(self.bytes_per_sample) {
                    let value = (self.time.sin() * self.volume as f32) as i16;
                    unsafe {
                        sample_out.write(value);
                        sample_out = sample_out.add(1);
                        sample_out.write(value);
                        sample_out = sample_out.add(1);
                    }
                    self.running_sample_index += 1;
                    self.time = 2.0
                        * std::f32::consts::PI
                        * (self.running_sample_index as f32
                            / self.wave_period as f32);
                }

                sample_out = region2.cast();
                for _ in 0..region2_size.div_euclid(self.bytes_per_sample) {
                    let value = (self.time.sin() * self.volume as f32) as i16;
                    unsafe {
                        sample_out.write(value);
                        sample_out = sample_out.add(1);
                        sample_out.write(value);
                        sample_out = sample_out.add(1);
                    }
                    self.running_sample_index += 1;
                    self.time = 2.0
                        * std::f32::consts::PI
                        * (self.running_sample_index as f32
                            / self.wave_period as f32);
                }

                unsafe {
                    buffer.Unlock(
                        region1,
                        region1_size,
                        region2,
                        region2_size,
                    );
                }
            }
        }
    }
}
