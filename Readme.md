
# A Rust project following [Handmade Hero](https://handmadehero.org)

This will be a painful journey of figuring out both the windows API,
at least at the beginning, and rust C bindings through the winapi crate.

All functions and types from the windows API are defined in module Win32.

# Notes

## Day 1

Figured out the winapi crate by doing the example given in the
[docs](https://docs.rs/winapi):

- Learn conversion from rust `&str` to c type `char` strings
  - Use a vector of u16 to represent wide strings and chain a 0 at the end to
null terminate.

## Day 2 (Opening a window)

[`WNDCLASSW`](https://docs.rs/winapi/0.3.9/winapi/um/winuser/struct.WNDCLASSW.html)
A base template the system uses to create [windows](https://docs.microsoft.com/en-us/windows/win32/winmsg/about-window-classes).

- Takes the following parameters:
  
  - Style(`DWORD`): We use `CS_HREDRAW | CS_VREDRAW` to make window redraw on resize
  
  - `lpfnWinProc`: (`fn* WindowProc`): Function pointer to function of signature:

    ```rust
    extern "system" fn fn_name(
            _:Win32::HWND,
            _:Win32::UINT,
            _:Win32::WPARAM,
            _:Win32::LPARAM
            ) -> WIn32::LRESULT
    ```

    This function is used to define the behaviour of windows of the class

  - `hInstance`: (`HINSTANCE`): A handle to the process calling the window class
is attached to.

  - `lpszClassName`: (`Vec<u16>*`):The name of the window class used when
creating window of the class.

[`RegisterClassW`](https://docs.rs/winapi/0.3.9/winapi/um/winuser/fn.RegisterWindowMessageW.html)
A function that takes an `WNDCLASSW` instance and registers it to the current
execution thread.

[`CreateWindowExW`](https://docs.rs/winapi/0.3.9/winapi/um/winuser/fn.CreateWindowExW.html)
A function that creates a window of the registered class and returns its handle.

[`GetMessageW`](https://docs.rs/winapi/0.3.9/winapi/um/winuser/fn.GetMessageW.html)
Gets a `MSG` off a given window returns whether

[`TranslateMessage`](https://docs.rs/winapi/0.3.9/winapi/um/winuser/fn.TranslateMessage.html)
[`DispatchMessageW`](https://docs.rs/winapi/0.3.9/winapi/um/winuser/fn.DispatchMessageW.html)
Functions that are used to handle the messages retrieved by `GetMessageW`

## Day 3 (Allocating a back buffer)

[`StretchDIBits`](./target/doc/winapi/um/wingdi/fn.StretchDIBits.html)

```rust
int StretchDIBits(
    HDC hdc,
    int xDest,
    int yDest,
    int DestWidth,
    int DestHeight,
    int xSrc,
    int ySrc,
    int SrcWidth,
    int SrcHeight,
    const VOID *lpBits,
    const BITMAPINFO *lpbmi,
    UINT iUsage,
    DWORD rop
)
```

Copies colour data from `src` `RECT` to `Dest` `RECT` from `lpBits` using the info
supplied in `lpbmi`

[`CreateCompatibleDC`](./target/doc/winapi/um/wingdi/fn.CreateCompatibleDC.html)
`HDC CreateCompatibleDC(
    HDC hdc
)`
Creates a memory device context (DC) compatible with the specified device if
device is null a DC compatible with the current window is created.

[`CreateDIBSections`](`.target/doc/winapi/um/wingdi/fn.CreateDIBSection.html`)

```rust
HBITMAP CreateDIBSection(
    HDC hdc,
    const BITMAPINFO *pbmi,
    UINT usage,
    VOID **ppvBits,
    HANDLE hSection,
    DWORD offset
)
```

Create a DIB applications can write to directly.
If `hSection` is null then `ppvBits` is allocated as the DIB memory location.

## Day 4 (Animating The back buffer)

Bitmap memory is now no longer allocated by `CreateDIBSection`, now its
allocated by `VirtualAlloc` committing the memory as read/write.

`VirtualFree` releases the memory.

A custom function `render_weird_gradient` was defined to draw to bitmap memory
the memory is then used by `update_window` to draw to the screen

Animation is done in the main running loop with a `y_offset` and `x_offset`,
changing colours depending on those variables.

## Day 5 (Graphics Review)

Encapsulate window state in `struct OffScreenBuffer` this holds the memory
`BITMABITMAPINFO`, width, height, and `bytes_per_pixel`.

The `update_window` and `resize_dib_section` methods are now implemented for this
`OffScreenBuffer`.

## Day 6 (Gamepad and keyboard input)

### Using XInput to get gamepad

- To prevent crashes if XInput is not found on the system controller input is
    setup indirectly:

  - A Type of function pointer whose signature is the same as the x_input
    function needed from Xinput is defined in this case the types are

    ```rust
    Type GetXInputState = fn(u32, *mut Win32::XINPUT_STATE) -> u32;
    Type SetXInputState = fn(u32, *mut Win32::XINPUT_VIBRATION) -> u32;
    ```

  - Thunks (Functions of with the signatures defined above that return a
    default value in this case 0)

    ```rust
    lazy_static! {  
        static ref GET_X_INPUT_THUNK: GetXInputState = |_, _| 0;  
        static ref SET_X_INPUT_THUNK: SetXInputState = |_, _| 0;  
    }
    ```

  - Initially assign these thunks to variables that will later contain the true
    funtion pointers
  - Use `Win32:::LoadLibrary()` to get the xinput functions from the dll
  - If `Win32::LoadLibrary()` does not return null
    - Use `Win32::GetProcAddress()` get the respective funtions the functions
    are of type `Win32::FARPROC` they need to be cast to their respective types
    defined earlier `ptr::cast()` does not work so `mem::transmute()` was used
    instead
    `//NOTE:This is extremely unsafe (Type restricrions are not being
    considered only size)`
    - replace the thunks in the function pointer variables to call the function

### Getting keyboard input

- match these messages `Win32::WM_SYSKEYUP
   | Win32::WM_SYSKEYDOWN
   | Win32::WM_KEYUP
   | Win32::WM_KEYDOWN` and get the keycodes from `w_param` then match those
    codes and use each arm to define actions per key press

## Day 7 (Create the direct Sound buffer)

This process requires the dsound(direct sound) library to be dynamically loaded
in order to prevent crashes if the library does not exist.

- A Type of function pointer whose signature is the same as the dsound
    function needed from dsound is defined in this case the types are

    ```rust
    type DirectSoundCreate = fn(
        Win32::LPCGUID,
        *mut Win32::LPDIRECTSOUND,
        Win32::LPUNKNOWN,
    ) -> Win32::HRESULT;
    ```

- Initially assign these thunks to variables that will later contain the true
    funtion pointers
- Use `Win32:::LoadLibrary()` to get the dsound functions from the dll
- If `Win32::LoadLibrary()` does not return null
  - Use `Win32::GetProcAddress()` get the respective funtions the functions
    are of type `Win32::FARPROC` they need to be cast to their respective types
    defined earlier `ptr::cast()` does not work so `mem::transmute()` was used
    instead
    `//NOTE:This is extremely unsafe (Type restricrions are not being
    considered only size)`

### Initialise direct sound

   ```rust
    let mut direct_sound: Win32::LPDIRECTSOUND = unsafe {
        Box::into_raw(Box::new(core::mem::zeroed::<Win32::IDirectSound>()))
   ```

- `direct_sound` is a pointer to an `IDirectSound` object.

- This is required by the defined function of signature `DirectSoundCreate`,
  that will then mutate the object turning it into a pointer to the
  direct_sound object we will use as a handle to create and manage the sound buffers

- `create_direct_sound` is a function of type `DirectSoundCreate` that is called
   on the direct_sound pointer to initialize it after the initialization
   `SetCooperativeLevel` is called on a dereferenced `direct_sound`

- Create a `WAVEFORMATEX` object to store and define the format to use in the
  primary buffer setup

- Create a `DSBUFFERDESC` to store the description of a primary sound buffer.

- Create an `IDirectSoundBuffer` pointer and pass the pointer as and argument to
  a `CreateSoundBuffer` call with the description

- Call `SetFormat` on the buffer passing an instance of `WAVEFORMATEX`

- Create another `DSBUFFERDESC` object and another SoundBuffer this is the
  buffer the audio will be written to

## Day 8 Write a Square wave to the buffer

After initialising direct sound next we have to write to the secondary buffer we
defined, we do this by:

- Using the `GetCurrentPosition` function on the buffer, passing a mutable
  play_cursor and write_cursor which will store the position of those cursors.
  
- Work out the lock offset and bytes_to_lock then lock the buffer using the
  `Lock` method on the buffer pass 2 pointers to the cycle buffer lock region
  and each buffers length  
- write to the region pointers
- Use `Play` to play what in the buffer
-Unlock the buffer with `Unlock` method

## Day 9 Variable Pitch sine wave

- Encapsulate sound state in `SoundOutput` structure.
- Add a `fill_sound_buffer` method to the structure implementation this does the
process mention in Day 8,
- Change the play value so its generated by a sine function.

## Day 10 `QueryPerformanceCounter`

Display performance details:

- Use the win_32_api `LARGE_INTEGER` type to store `QueryPerformanceCounter`
output.
- use Intrinsic `_rdtsc` from `core::arch` a x86 feature that returns a copy of
the processors time stamp counter;

