# A Rust project following [Handmade Hero](https://handmadehero.org)
This will be a painfull journey of figuring out both the windows API,
at least at the begining, and rust C bindings through the winapi crate.


# Notes
## Day 1
Figured out the winapi crate by doing the example given in the 
[docs](https://docs.rs/winapi):
- Learn convertion from rust `&str` to c type `char` strings
	- Use a vector of u16 to represent wide strings and chain a	0 at the end to null terminate.
